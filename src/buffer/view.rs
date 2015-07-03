use std::fmt;
use std::mem;
use std::borrow::Cow;
use std::ops::Range;
use std::marker::PhantomData;

use texture::{PixelValue, Texture1dDataSink};
use gl;

use backend::Facade;
use BufferViewExt;
use BufferViewSliceExt;
use GlObject;

use context::Context;
use context::CommandContext;
use std::rc::Rc;
use ContextExt;

use buffer::BufferType;
use buffer::BufferCreationError;
use buffer::fences::Fences;
use buffer::fences::Inserter;
use buffer::alloc::Buffer;
use buffer::alloc::Mapping;
use buffer::alloc::ReadMapping;
use buffer::alloc::WriteMapping;

/// Represents a view of a buffer.
pub struct BufferView<T> where T: Copy {
    // TODO: this `Option` is here because we have a destructor and need to be able to move out
    alloc: Option<Buffer>,
    num_elements: usize,
    // TODO: this `Option` is here because we have a destructor and need to be able to move out
    fence: Option<Fences>,
    marker: PhantomData<T>,
}

impl<T> fmt::Debug for BufferView<T> where T: Copy {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self.alloc.as_ref().unwrap())
    }
}

impl<T> Drop for BufferView<T> where T: Copy {
    fn drop(&mut self) {
        if let (Some(alloc), Some(mut fence)) = (self.alloc.take(), self.fence.take()) {
            fence.clean(&mut alloc.get_context().make_current());
        }
    }
}

/// Represents a sub-part of a buffer.
#[derive(Copy, Clone)]
pub struct BufferViewSlice<'a, T> where T: Copy {
    alloc: &'a Buffer,
    offset_bytes: usize,
    num_elements: usize,
    fence: &'a Fences,
    marker: PhantomData<T>,
}

impl<'a, T> fmt::Debug for BufferViewSlice<'a, T> where T: Copy {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self.alloc)
    }
}

/// Represents a sub-part of a buffer.
pub struct BufferViewMutSlice<'a, T> where T: Copy {
    alloc: &'a mut Buffer,
    offset_bytes: usize,
    num_elements: usize,
    fence: &'a Fences,
    marker: PhantomData<T>,
}

impl<'a, T> fmt::Debug for BufferViewMutSlice<'a, T> where T: Copy {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self.alloc)
    }
}

/// Represents a sub-part of a buffer.
///
/// Doesn't contain any information about the content, contrary to `BufferView`.
pub struct BufferViewAny {
    alloc: Buffer,
    elements_size: usize,
    elements_count: usize,
    fence: Fences,
}

impl Drop for BufferViewAny {
    fn drop(&mut self) {
        self.fence.clean(&mut self.alloc.get_context().make_current());
    }
}

impl fmt::Debug for BufferViewAny {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self.alloc)
    }
}

/// Slice of a `BufferView` without any type info.
#[derive(Copy, Clone)]
pub struct BufferViewAnySlice<'a> {
    alloc: &'a Buffer,
    offset_bytes: usize,
    elements_size: usize,
    elements_count: usize,
    fence: &'a Fences,
}

impl<'a> fmt::Debug for BufferViewAnySlice<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self.alloc)
    }
}

impl<T> From<BufferView<T>> for BufferViewAny where T: Copy + Send + 'static {
    fn from(mut buffer: BufferView<T>) -> BufferViewAny {
        BufferViewAny {
            alloc: buffer.alloc.take().unwrap(),
            elements_size: mem::size_of::<T>(),
            elements_count: buffer.num_elements,
            fence: buffer.fence.take().unwrap(),
        }
    }
}

impl<T> BufferView<T> where T: Copy {
    /// Builds a new buffer containing the given data. The size of the buffer is equal to the size
    /// of the data.
    ///
    /// If `dynamic` is false, glium will attempt to use buffer storage to create an immutable
    /// buffer (without the `DYNAMIC_STORAGE_FLAG`).
    ///
    /// If `dynamic` is true, glium will attempt to use persistent mapping and manage
    /// synchronizations manually.
    pub fn new<F>(facade: &F, data: &[T], ty: BufferType, dynamic: bool)
                  -> Result<BufferView<T>, BufferCreationError>
                  where F: Facade
    {
        let len = data.len();

        Buffer::new(facade, data, ty, dynamic)
            .map(|buffer| {
                BufferView {
                    alloc: Some(buffer),
                    num_elements: len,
                    fence: Some(Fences::new()),
                    marker: PhantomData,
                }
            })
    }

    /// Builds a new buffer of the given size.
    ///
    /// If `dynamic` is false, glium will attempt to use buffer storage to create an immutable
    /// buffer (without the `DYNAMIC_STORAGE_FLAG`).
    ///
    /// If `dynamic` is true, glium will attempt to use persistent mapping and manage
    /// synchronizations manually.
    pub fn empty<F>(facade: &F, ty: BufferType, len: usize, dynamic: bool)
                    -> Result<BufferView<T>, BufferCreationError> where F: Facade
    {
        Buffer::empty(facade, ty, len * mem::size_of::<T>(), dynamic)
            .map(|buffer| {
                BufferView {
                    alloc: Some(buffer),
                    num_elements: len,
                    fence: Some(Fences::new()),
                    marker: PhantomData,
                }
            })
    }

    /// Returns the context corresponding to this buffer.
    pub fn get_context(&self) -> &Rc<Context> {
        self.alloc.as_ref().unwrap().get_context()
    }

    /// Returns the number of elements in this subbuffer.
    pub fn len(&self) -> usize {
        self.num_elements
    }

    /// Returns true if this buffer uses persistent mapping.
    pub fn is_persistent(&self) -> bool {
        self.alloc.as_ref().unwrap().uses_persistent_mapping()
    }

    /// Uploads some data in this buffer.
    ///
    /// ## Panic
    ///
    /// Panics if the length of `data` is different from the length of this buffer.
    pub fn write<P>(&self, data: P) where P: AsRef<[T]> {
        self.as_slice().write(data);
    }

    /// Invalidates the content of the buffer. The data becomes undefined.
    ///
    /// You should call this if you only use parts of a buffer. For example if you want to use
    /// the first half of the buffer, you invalidate the whole buffer then write the first half.
    ///
    /// This operation is a no-op if the backend doesn't support it.
    pub fn invalidate(&self) {
        self.as_slice().invalidate()
    }

    /// Reads the content of the buffer.
    ///
    /// # Features
    ///
    /// Only available if the `gl_read_buffer` feature is enabled.
    #[cfg(feature = "gl_read_buffer")]
    pub fn read(&self) -> Vec<T> {
        match self.read_if_supported() {
            Some(buf) => buf,
            None => unreachable!()
        }
    }

    /// Reads the content of the subbuffer. Returns `None` if this operation is not supported.
    pub fn read_if_supported(&self) -> Option<Vec<T>> {
        self.as_slice().read_if_supported()
    }

    /// Maps the buffer in memory for both reading and writing.
    pub fn map(&mut self) -> Mapping<T> {
        self.as_mut_slice().map()
    }

    /// Maps the buffer in memory for reading.
    pub fn map_read(&mut self) -> ReadMapping<T> {
        self.as_mut_slice().map_read()
    }

    /// Maps the buffer in memory for writing only.
    pub fn map_write(&mut self) -> WriteMapping<T> {
        self.as_mut_slice().map_write()
    }

    /// Builds a slice of this subbuffer. Returns `None` if out of range.
    pub fn slice(&self, range: Range<usize>) -> Option<BufferViewSlice<T>> {
        self.as_slice().slice(range)
    }

    /// Builds a slice of this subbuffer. Returns `None` if out of range.
    pub fn slice_mut(&mut self, range: Range<usize>) -> Option<BufferViewMutSlice<T>> {
        self.as_mut_slice().slice(range)
    }

    /// Builds a slice containing the whole subbuffer.
    pub fn as_slice(&self) -> BufferViewSlice<T> {
        BufferViewSlice {
            alloc: self.alloc.as_ref().unwrap(),
            offset_bytes: 0,
            num_elements: self.num_elements,
            fence: self.fence.as_ref().unwrap(),
            marker: PhantomData,
        }
    }

    /// Builds a slice containing the whole subbuffer.
    pub fn as_mut_slice(&mut self) -> BufferViewMutSlice<T> {
        BufferViewMutSlice {
            alloc: self.alloc.as_mut().unwrap(),
            offset_bytes: 0,
            num_elements: self.num_elements,
            fence: self.fence.as_ref().unwrap(),
            marker: PhantomData,
        }
    }

    /// Builds a slice-any containing the whole subbuffer.
    pub fn as_slice_any(&self) -> BufferViewAnySlice {
        BufferViewAnySlice {
            alloc: self.alloc.as_ref().unwrap(),
            offset_bytes: 0,
            elements_size: mem::size_of::<T>(),
            elements_count: self.num_elements,
            fence: self.fence.as_ref().unwrap(),
        }
    }
}

impl<T> BufferView<T> where T: PixelValue {
    /// Reads the content of the buffer.
    ///
    /// # Features
    ///
    /// Only available if the `gl_read_buffer` feature is enabled.
    #[cfg(feature = "gl_read_buffer")]
    pub fn read_as_texture_1d<S>(&self) -> S where S: Texture1dDataSink<T> {
        S::from_raw(Cow::Owned(self.read()), self.len() as u32)
    }

    /// Reads the content of the subbuffer. Returns `None` if this operation is not supported.
    pub fn read_as_texture_1d_if_supported<S>(&self) -> Option<S> where S: Texture1dDataSink<T> {
        self.read_if_supported().map(|data| {
            S::from_raw(Cow::Owned(data), self.len() as u32)
        })
    }
}

impl<'a, T> BufferViewSlice<'a, T> where T: Copy {
    /// Returns the number of elements in this slice.
    pub fn len(&self) -> usize {
        self.num_elements
    }

    /// Uploads some data in this buffer.
    ///
    /// ## Panic
    ///
    /// Panics if the length of `data` is different from the length of this buffer.
    pub fn write<P>(&self, data: P) where P: AsRef<[T]> {
        let data = data.as_ref();
        assert!(data.len() == self.num_elements);

        self.fence.wait(&mut self.alloc.get_context().make_current(),
                        self.offset_bytes .. self.offset_bytes + self.len() * mem::size_of::<T>());
        unsafe { self.alloc.upload(self.offset_bytes, data); }
    }

    /// Invalidates the content of the slice. The data becomes undefined.
    ///
    /// This operation is a no-op if the backend doesn't support it.
    pub fn invalidate(&self) {
        self.alloc.invalidate(self.offset_bytes, self.num_elements * mem::size_of::<T>());
    }

    /// Reads the content of the buffer.
    #[cfg(feature = "gl_read_buffer")]
    pub fn read(&self) -> Vec<T> {
        self.read_if_supported().unwrap()
    }

    /// Reads the content of the slice. Returns `None` if this operation is not supported.
    pub fn read_if_supported(&self) -> Option<Vec<T>> {
        self.fence.wait(&mut self.alloc.get_context().make_current(),
                        self.offset_bytes .. self.offset_bytes + self.len() * mem::size_of::<T>());

        unsafe {
            let mut data = Vec::with_capacity(self.num_elements);
            data.set_len(self.num_elements);        // TODO: is this safe?

            match self.alloc.read_if_supported(self.offset_bytes, &mut data) {
                Err(_) => return None,
                Ok(_) => ()
            };

            Some(data)
        }
    }

    /// Builds a subslice of this slice. Returns `None` if out of range.
    pub fn slice(&self, range: Range<usize>) -> Option<BufferViewSlice<'a, T>> {
        if range.start > self.num_elements || range.end > self.num_elements {
            return None;
        }

        Some(BufferViewSlice {
            alloc: self.alloc,
            offset_bytes: self.offset_bytes + range.start * mem::size_of::<T>(),
            num_elements: range.end - range.start,
            fence: self.fence,
            marker: PhantomData,
        })
    }

    /// Builds a slice-any containing the whole subbuffer.
    pub fn as_slice_any(&self) -> BufferViewAnySlice<'a> {
        BufferViewAnySlice {
            alloc: self.alloc,
            offset_bytes: self.offset_bytes,
            elements_size: mem::size_of::<T>(),
            elements_count: self.num_elements,
            fence: self.fence,
        }
    }
}

impl<'a, T> BufferViewSlice<'a, T> where T: PixelValue {
    /// Reads the content of the buffer.
    ///
    /// # Features
    ///
    /// Only available if the `gl_read_buffer` feature is enabled.
    #[cfg(feature = "gl_read_buffer")]
    pub fn read_as_texture_1d<S>(&self) -> S where S: Texture1dDataSink<T> {
        S::from_raw(Cow::Owned(self.read()), self.len() as u32)
    }

    /// Reads the content of the subbuffer. Returns `None` if this operation is not supported.
    pub fn read_as_texture_1d_if_supported<S>(&self) -> Option<S> where S: Texture1dDataSink<T> {
        self.read_if_supported().map(|data| {
            S::from_raw(Cow::Owned(data), self.len() as u32)
        })
    }
}

impl<'a, T> BufferViewMutSlice<'a, T> where T: Copy {
    /// Returns the number of elements in this slice.
    pub fn len(&self) -> usize {
        self.num_elements
    }

    /// Maps the buffer in memory for both reading and writing.
    pub fn map(self) -> Mapping<'a, T> {
        self.fence.wait(&mut self.alloc.get_context().make_current(),
                        self.offset_bytes .. self.offset_bytes + self.len() * mem::size_of::<T>());
        unsafe { self.alloc.map(self.offset_bytes, self.num_elements) }
    }

    /// Maps the buffer in memory for reading.
    pub fn map_read(self) -> ReadMapping<'a, T> {
        self.fence.wait(&mut self.alloc.get_context().make_current(),
                        self.offset_bytes .. self.offset_bytes + self.len() * mem::size_of::<T>());
        unsafe { self.alloc.map_read(self.offset_bytes, self.num_elements) }
    }

    /// Maps the buffer in memory for writing only.
    pub fn map_write(self) -> WriteMapping<'a, T> {
        self.fence.wait(&mut self.alloc.get_context().make_current(),
                        self.offset_bytes .. self.offset_bytes + self.len() * mem::size_of::<T>());
        unsafe { self.alloc.map_write(self.offset_bytes, self.num_elements) }
    }

    /// Uploads some data in this buffer.
    ///
    /// ## Panic
    ///
    /// Panics if the length of `data` is different from the length of this buffer.
    pub fn write<P>(&self, data: P) where P: AsRef<[T]> {
        let data = data.as_ref();
        assert!(data.len() == self.num_elements);

        self.fence.wait(&mut self.alloc.get_context().make_current(),
                        self.offset_bytes .. self.offset_bytes + self.len() * mem::size_of::<T>());
        unsafe { self.alloc.upload(self.offset_bytes, data); }
    }

    /// Invalidates the content of the slice. The data becomes undefined.
    ///
    /// This operation is a no-op if the backend doesn't support it.
    pub fn invalidate(&self) {
        self.alloc.invalidate(self.offset_bytes, self.num_elements * mem::size_of::<T>());
    }

    /// Reads the content of the buffer.
    #[cfg(feature = "gl_read_buffer")]
    pub fn read(&self) -> Vec<T> {
        self.read_if_supported().unwrap()
    }

    /// Reads the content of the slice. Returns `None` if this operation is not supported.
    pub fn read_if_supported(&self) -> Option<Vec<T>> {
        unsafe {
            let mut data = Vec::with_capacity(self.num_elements);
            data.set_len(self.num_elements);        // TODO: is this safe?

            match self.alloc.read_if_supported(self.offset_bytes, &mut data) {
                Err(_) => return None,
                Ok(_) => ()
            };

            Some(data)
        }
    }

    /// Builds a subslice of this slice. Returns `None` if out of range.
    pub fn slice(self, range: Range<usize>) -> Option<BufferViewMutSlice<'a, T>> {
        if range.start > self.num_elements || range.end > self.num_elements {
            return None;
        }

        Some(BufferViewMutSlice {
            alloc: self.alloc,
            offset_bytes: self.offset_bytes + range.start * mem::size_of::<T>(),
            num_elements: range.end - range.start,
            fence: self.fence,
            marker: PhantomData,
        })
    }

    /// Builds a slice-any containing the whole subbuffer.
    pub fn as_slice_any(self) -> BufferViewAnySlice<'a> {
        BufferViewAnySlice {
            alloc: self.alloc,
            offset_bytes: self.offset_bytes,
            elements_size: mem::size_of::<T>(),
            elements_count: self.num_elements,
            fence: self.fence,
        }
    }
}

impl<'a, T> BufferViewMutSlice<'a, T> where T: PixelValue {
    /// Reads the content of the buffer.
    ///
    /// # Features
    ///
    /// Only available if the `gl_read_buffer` feature is enabled.
    #[cfg(feature = "gl_read_buffer")]
    pub fn read_as_texture_1d<S>(&self) -> S where S: Texture1dDataSink<T> {
        S::from_raw(Cow::Owned(self.read()), self.len() as u32)
    }

    /// Reads the content of the subbuffer. Returns `None` if this operation is not supported.
    pub fn read_as_texture_1d_if_supported<S>(&self) -> Option<S> where S: Texture1dDataSink<T> {
        self.read_if_supported().map(|data| {
            S::from_raw(Cow::Owned(data), self.len() as u32)
        })
    }
}

impl BufferViewAny {
    /// Builds a slice-any containing the whole subbuffer.
    pub fn as_slice_any(&self) -> BufferViewAnySlice {
        BufferViewAnySlice {
            alloc: &self.alloc,
            offset_bytes: 0,
            elements_size: self.elements_size,
            elements_count: self.elements_count,
            fence: &self.fence,
        }
    }
    
    /// Returns the context corresponding to this buffer.
    pub fn get_context(&self) -> &Rc<Context> {
        self.alloc.get_context()
    }

    /// Returns the size of each element in this buffer.
    ///
    /// This information is taken from the original `BufferView` that was used to construct
    /// this object.
    pub fn get_elements_size(&self) -> usize {
        self.elements_size
    }

    /// Returns the number of elements in this buffer.
    ///
    /// This information is taken from the original `BufferView` that was used to construct
    /// this object.
    pub fn get_elements_count(&self) -> usize {
        self.elements_count
    }

    /// Returns the number of bytes in this subbuffer.
    pub fn get_size(&self) -> usize {
        self.elements_size * self.elements_count
    }

    /// Invalidates the content of the buffer. The data becomes undefined.
    ///
    /// This operation is a no-op if the backend doesn't support it.
    pub fn invalidate(&self) {
        self.alloc.invalidate(0, self.elements_count * self.elements_size);
    }

    /// UNSTABLE. This function can be removed at any moment without any further notice.
    ///
    /// Considers that the buffer is filled with elements of type `T` and reads them.
    ///
    /// # Panic
    ///
    /// Panicks if the size of the buffer is not a multiple of the size of the data.
    /// For example, trying to read some `(u8, u8, u8, u8)`s from a buffer of 7 bytes will panic.
    ///
    pub unsafe fn read_if_supported<T>(&self) -> Option<Vec<T>> where T: Copy {
        assert!(self.get_size() % mem::size_of::<T>() == 0);

        self.fence.wait(&mut self.alloc.get_context().make_current(), 0 .. self.get_size());

        let len = self.get_size() / mem::size_of::<T>();
        let mut data = Vec::with_capacity(len);
        data.set_len(len);        // TODO: is this safe?

        match self.alloc.read_if_supported(0, &mut data) {
            Err(_) => return None,
            Ok(_) => ()
        };

        Some(data)
    }
}

impl<'a> BufferViewAnySlice<'a> {
    /// Returns the size of each element in this buffer.
    ///
    /// This information is taken from the original `BufferView` that was used to construct
    /// this object.
    pub fn get_elements_size(&self) -> usize {
        self.elements_size
    }

    /// Returns the number of elements in this buffer.
    ///
    /// This information is taken from the original `BufferView` that was used to construct
    /// this object.
    pub fn get_elements_count(&self) -> usize {
        self.elements_count
    }

    /// Returns the number of bytes in this slice.
    pub fn get_size(&self) -> usize {
        self.elements_size * self.elements_count
    }

    /// Invalidates the content of the slice. The data becomes undefined.
    ///
    /// This operation is a no-op if the backend doesn't support it.
    pub fn invalidate(&self) {
        self.alloc.invalidate(self.offset_bytes, self.get_size());
    }
}

impl<T> BufferViewExt for BufferView<T> where T: Copy {
    fn get_offset_bytes(&self) -> usize {
        0
    }

    fn get_buffer_id(&self) -> gl::types::GLuint {
        let alloc = self.alloc.as_ref().unwrap();
        alloc.get_id()
    }

    fn prepare_for_vertex_attrib_array(&self, ctxt: &mut CommandContext) {
        let alloc = self.alloc.as_ref().unwrap();
        alloc.prepare_for_vertex_attrib_array(ctxt);
    }

    fn prepare_for_element_array(&self, ctxt: &mut CommandContext) {
        let alloc = self.alloc.as_ref().unwrap();
        alloc.prepare_for_element_array(ctxt);
    }

    fn bind_to_element_array(&self, ctxt: &mut CommandContext) {
        let alloc = self.alloc.as_ref().unwrap();
        alloc.bind_to_element_array(ctxt);
    }

    fn prepare_and_bind_for_pixel_pack(&self, ctxt: &mut CommandContext) {
        let alloc = self.alloc.as_ref().unwrap();
        alloc.prepare_and_bind_for_pixel_pack(ctxt);
    }

    fn unbind_pixel_pack(ctxt: &mut CommandContext) {
        Buffer::unbind_pixel_pack(ctxt)
    }

    fn prepare_and_bind_for_pixel_unpack(&self, ctxt: &mut CommandContext) {
        let alloc = self.alloc.as_ref().unwrap();
        alloc.prepare_and_bind_for_pixel_unpack(ctxt);
    }

    fn unbind_pixel_unpack(ctxt: &mut CommandContext) {
        Buffer::unbind_pixel_unpack(ctxt)
    }

    fn prepare_and_bind_for_draw_indirect(&self, ctxt: &mut CommandContext) {
        let alloc = self.alloc.as_ref().unwrap();
        alloc.prepare_and_bind_for_draw_indirect(ctxt);
    }

    fn prepare_and_bind_for_uniform(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        let alloc = self.alloc.as_ref().unwrap();
        alloc.prepare_and_bind_for_uniform(ctxt, index, 0 .. alloc.get_size());
    }

    fn prepare_and_bind_for_shared_storage(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        let alloc = self.alloc.as_ref().unwrap();
        alloc.prepare_and_bind_for_shared_storage(ctxt, index, 0 .. alloc.get_size());
    }

    fn bind_to_transform_feedback(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        let alloc = self.alloc.as_ref().unwrap();
        alloc.bind_to_transform_feedback(ctxt, index, 0 .. alloc.get_size());
    }
}

impl<'a, T> BufferViewSliceExt<'a> for BufferViewSlice<'a, T> where T: Copy {
    fn add_fence(&self) -> Option<Inserter<'a>> {
        if !self.alloc.uses_persistent_mapping() {
            return None;
        }

        Some(self.fence.inserter(self.offset_bytes .. self.offset_bytes +
                                                                  self.len() * mem::size_of::<T>()))
    }
}

impl<'a, T> BufferViewExt for BufferViewSlice<'a, T> where T: Copy {
    fn get_offset_bytes(&self) -> usize {
        self.offset_bytes
    }

    fn get_buffer_id(&self) -> gl::types::GLuint {
        self.alloc.get_id()
    }

    fn prepare_for_vertex_attrib_array(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_for_vertex_attrib_array(ctxt);
    }

    fn prepare_for_element_array(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_for_element_array(ctxt);
    }

    fn bind_to_element_array(&self, ctxt: &mut CommandContext) {
        self.alloc.bind_to_element_array(ctxt);
    }

    fn prepare_and_bind_for_pixel_pack(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_pixel_pack(ctxt);
    }

    fn unbind_pixel_pack(ctxt: &mut CommandContext) {
        Buffer::unbind_pixel_pack(ctxt)
    }

    fn prepare_and_bind_for_pixel_unpack(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_pixel_unpack(ctxt);
    }

    fn unbind_pixel_unpack(ctxt: &mut CommandContext) {
        Buffer::unbind_pixel_unpack(ctxt)
    }

    fn prepare_and_bind_for_draw_indirect(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_draw_indirect(ctxt);
    }

    fn prepare_and_bind_for_uniform(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        self.alloc.prepare_and_bind_for_uniform(ctxt, index, 0 .. self.alloc.get_size());
    }

    fn prepare_and_bind_for_shared_storage(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        self.alloc.prepare_and_bind_for_shared_storage(ctxt, index, 0 .. self.alloc.get_size());
    }

    fn bind_to_transform_feedback(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        self.alloc.bind_to_transform_feedback(ctxt, index, 0 .. self.alloc.get_size());
    }
}

impl BufferViewExt for BufferViewAny {
    fn get_offset_bytes(&self) -> usize {
        0
    }

    fn get_buffer_id(&self) -> gl::types::GLuint {
        self.alloc.get_id()
    }

    fn prepare_for_vertex_attrib_array(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_for_vertex_attrib_array(ctxt);
    }

    fn prepare_for_element_array(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_for_element_array(ctxt);
    }

    fn bind_to_element_array(&self, ctxt: &mut CommandContext) {
        self.alloc.bind_to_element_array(ctxt);
    }

    fn prepare_and_bind_for_pixel_pack(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_pixel_pack(ctxt);
    }

    fn unbind_pixel_pack(ctxt: &mut CommandContext) {
        Buffer::unbind_pixel_pack(ctxt)
    }

    fn prepare_and_bind_for_pixel_unpack(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_pixel_unpack(ctxt);
    }

    fn unbind_pixel_unpack(ctxt: &mut CommandContext) {
        Buffer::unbind_pixel_unpack(ctxt)
    }

    fn prepare_and_bind_for_draw_indirect(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_draw_indirect(ctxt);
    }

    fn prepare_and_bind_for_uniform(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        self.alloc.prepare_and_bind_for_uniform(ctxt, index, 0 .. self.alloc.get_size());
    }

    fn prepare_and_bind_for_shared_storage(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        self.alloc.prepare_and_bind_for_shared_storage(ctxt, index, 0 .. self.alloc.get_size());
    }

    fn bind_to_transform_feedback(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        self.alloc.bind_to_transform_feedback(ctxt, index, 0 .. self.alloc.get_size());
    }
}

impl<'a> BufferViewSliceExt<'a> for BufferViewAnySlice<'a> {
    fn add_fence(&self) -> Option<Inserter<'a>> {
        if !self.alloc.uses_persistent_mapping() {
            return None;
        }

        Some(self.fence.inserter(self.offset_bytes .. self.offset_bytes + self.get_size()))
    }
}

impl<'a> BufferViewExt for BufferViewAnySlice<'a> {
    fn get_offset_bytes(&self) -> usize {
        self.offset_bytes
    }

    fn get_buffer_id(&self) -> gl::types::GLuint {
        self.alloc.get_id()
    }

    fn prepare_for_vertex_attrib_array(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_for_vertex_attrib_array(ctxt);
    }

    fn prepare_for_element_array(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_for_element_array(ctxt);
    }

    fn bind_to_element_array(&self, ctxt: &mut CommandContext) {
        self.alloc.bind_to_element_array(ctxt);
    }

    fn prepare_and_bind_for_pixel_pack(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_pixel_pack(ctxt);
    }

    fn unbind_pixel_pack(ctxt: &mut CommandContext) {
        Buffer::unbind_pixel_pack(ctxt)
    }

    fn prepare_and_bind_for_pixel_unpack(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_pixel_unpack(ctxt);
    }

    fn unbind_pixel_unpack(ctxt: &mut CommandContext) {
        Buffer::unbind_pixel_unpack(ctxt)
    }

    fn prepare_and_bind_for_draw_indirect(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_draw_indirect(ctxt);
    }

    fn prepare_and_bind_for_uniform(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        self.alloc.prepare_and_bind_for_uniform(ctxt, index, 0 .. self.alloc.get_size());
    }

    fn prepare_and_bind_for_shared_storage(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        self.alloc.prepare_and_bind_for_shared_storage(ctxt, index, 0 .. self.alloc.get_size());
    }

    fn bind_to_transform_feedback(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        self.alloc.bind_to_transform_feedback(ctxt, index, 0 .. self.alloc.get_size());
    }
}
