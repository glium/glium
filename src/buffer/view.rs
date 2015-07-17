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
use buffer::BufferMode;
use buffer::BufferCreationError;
use buffer::Content;
use buffer::fences::Fences;
use buffer::fences::Inserter;
use buffer::alloc::Buffer;
use buffer::alloc::Mapping;
use buffer::alloc::ReadMapping;
use buffer::alloc::WriteMapping;
use buffer::alloc::ReadError;

/// Represents a view of a buffer.
pub struct BufferView<T: ?Sized> where T: Content {
    // TODO: this `Option` is here because we have a destructor and need to be able to move out
    alloc: Option<Buffer>,
    // TODO: this `Option` is here because we have a destructor and need to be able to move out
    fence: Option<Fences>,
    marker: PhantomData<T>,
}

impl<T: ?Sized> BufferView<T> where T: Content {
    /// Builds a new buffer containing the given data. The size of the buffer is equal to the size
    /// of the data.
    pub fn new<F>(facade: &F, data: &T, ty: BufferType, mode: BufferMode)
                  -> Result<BufferView<T>, BufferCreationError>
                  where F: Facade
    {
        Buffer::new(facade, data, ty, mode)
            .map(|buffer| {
                BufferView {
                    alloc: Some(buffer),
                    fence: Some(Fences::new()),
                    marker: PhantomData,
                }
            })
    }

    /// Builds a new buffer of the given size.
    pub fn empty_unsized<F>(facade: &F, ty: BufferType, size: usize, mode: BufferMode)
                            -> Result<BufferView<T>, BufferCreationError> where F: Facade
    {
        assert!(<T as Content>::is_size_suitable(size));

        Buffer::empty(facade, ty, size, mode)
            .map(|buffer| {
                BufferView {
                    alloc: Some(buffer),
                    fence: Some(Fences::new()),
                    marker: PhantomData,
                }
            })
    }

    /// Returns the context corresponding to this buffer.
    pub fn get_context(&self) -> &Rc<Context> {
        self.alloc.as_ref().unwrap().get_context()
    }

    /// Returns the size in bytes of this buffer.
    pub fn get_size(&self) -> usize {
        self.alloc.as_ref().unwrap().get_size()
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
    pub fn write(&self, data: &T) {
        assert!(mem::size_of_val(data) == self.get_size());

        self.fence.as_ref().unwrap().wait(&mut self.alloc.as_ref().unwrap().get_context().make_current(),
                                          0 .. self.get_size());
        unsafe { self.alloc.as_ref().unwrap().upload(0, data); }
    }

    /// Invalidates the content of the buffer. The data becomes undefined.
    ///
    /// You should call this if you only use parts of a buffer. For example if you want to use
    /// the first half of the buffer, you invalidate the whole buffer then write the first half.
    ///
    /// This operation is a no-op if the backend doesn't support it.
    pub fn invalidate(&self) {
        self.alloc.as_ref().unwrap().invalidate(0, self.get_size());
    }

    /// Reads the content of the buffer.
    pub fn read(&self) -> Result<T::Owned, ReadError> {
        self.fence.as_ref().unwrap().wait(&mut self.alloc.as_ref().unwrap().get_context().make_current(),
                                          0 .. self.get_size());

        unsafe {
            self.alloc.as_ref().unwrap().read::<T>(0 .. self.get_size())
        }
    }

    /// Maps the buffer in memory for both reading and writing.
    pub fn map(&mut self) -> Mapping<T> {
        self.fence.as_ref().unwrap().wait(&mut self.alloc.as_ref().unwrap().get_context().make_current(),
                                          0 .. self.get_size());
        let size = self.get_size();
        unsafe { self.alloc.as_mut().unwrap().map(0 .. size) }
    }

    /// Maps the buffer in memory for reading.
    pub fn map_read(&mut self) -> ReadMapping<T> {
        self.fence.as_ref().unwrap().wait(&mut self.alloc.as_ref().unwrap().get_context().make_current(),
                                          0 .. self.get_size());
        let size = self.get_size();
        unsafe { self.alloc.as_mut().unwrap().map_read(0 .. size) }
    }

    /// Maps the buffer in memory for writing only.
    pub fn map_write(&mut self) -> WriteMapping<T> {
        self.fence.as_ref().unwrap().wait(&mut self.alloc.as_ref().unwrap().get_context().make_current(),
                                          0 .. self.get_size());
        let size = self.get_size();
        unsafe { self.alloc.as_mut().unwrap().map_write(0 .. size) }
    }

    /// Builds a slice-any containing the whole subbuffer.
    pub fn as_slice_any(&self) -> BufferViewAnySlice {
        let size = self.get_size();

        BufferViewAnySlice {
            alloc: self.alloc.as_ref().unwrap(),
            bytes_start: 0,
            bytes_end: self.get_size(),
            elements_size: <T as Content>::get_elements_size(),
            fence: self.fence.as_ref().unwrap(),
        }
    }
}

impl<T> BufferView<T> where T: Content + Copy {
    /// Builds a new buffer of the given size.
    pub fn empty<F>(facade: &F, ty: BufferType, mode: BufferMode)
                    -> Result<BufferView<T>, BufferCreationError> where F: Facade
    {
        Buffer::empty(facade, ty, mem::size_of::<T>(), mode)
            .map(|buffer| {
                BufferView {
                    alloc: Some(buffer),
                    fence: Some(Fences::new()),
                    marker: PhantomData,
                }
            })
    }
}

impl<T> BufferView<[T]> where [T]: Content, T: Copy {
    /// Builds a new buffer of the given size.
    pub fn empty_array<F>(facade: &F, ty: BufferType, len: usize, mode: BufferMode)
                          -> Result<BufferView<[T]>, BufferCreationError> where F: Facade
    {
        Buffer::empty(facade, ty, len * mem::size_of::<T>(), mode)
            .map(|buffer| {
                BufferView {
                    alloc: Some(buffer),
                    fence: Some(Fences::new()),
                    marker: PhantomData,
                }
            })
    }

    /// Returns the number of elements in this buffer.
    pub fn len(&self) -> usize {
        self.alloc.as_ref().unwrap().get_size() / mem::size_of::<T>()
    }

    /// Builds a slice of this subbuffer. Returns `None` if out of range.
    pub fn slice(&self, range: Range<usize>) -> Option<BufferViewSlice<[T]>> {
        self.as_slice().slice(range)
    }

    /// Builds a slice of this subbuffer. Returns `None` if out of range.
    pub fn slice_mut(&mut self, range: Range<usize>) -> Option<BufferViewMutSlice<[T]>> {
        self.as_mut_slice().slice(range)
    }

    /// Builds a slice containing the whole subbuffer.
    pub fn as_slice(&self) -> BufferViewSlice<[T]> {
        BufferViewSlice {
            alloc: self.alloc.as_ref().unwrap(),
            bytes_start: 0,
            bytes_end: self.get_size(),
            fence: self.fence.as_ref().unwrap(),
            marker: PhantomData,
        }
    }

    /// Builds a slice containing the whole subbuffer.
    pub fn as_mut_slice(&mut self) -> BufferViewMutSlice<[T]> {
        let size = self.get_size();

        BufferViewMutSlice {
            alloc: self.alloc.as_mut().unwrap(),
            bytes_start: 0,
            bytes_end: size,
            fence: self.fence.as_ref().unwrap(),
            marker: PhantomData,
        }
    }
}

impl<T> BufferView<[T]> where T: PixelValue {
    /// Reads the content of the buffer.
    pub fn read_as_texture_1d<S>(&self) -> Result<S, ReadError> where S: Texture1dDataSink<T> {
        let data = try!(self.read());
        Ok(S::from_raw(Cow::Owned(data), self.len() as u32))
    }
}

impl<T: ?Sized> fmt::Debug for BufferView<T> where T: Content {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self.alloc.as_ref().unwrap())
    }
}

impl<T: ?Sized> Drop for BufferView<T> where T: Content {
    fn drop(&mut self) {
        if let (Some(alloc), Some(mut fence)) = (self.alloc.take(), self.fence.take()) {
            fence.clean(&mut alloc.get_context().make_current());
        }
    }
}

impl<T: ?Sized> BufferViewExt for BufferView<T> where T: Content {
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

/// Represents a sub-part of a buffer.
#[derive(Copy, Clone)]
pub struct BufferViewSlice<'a, T: ?Sized> where T: Content + 'a {
    alloc: &'a Buffer,
    bytes_start: usize,
    bytes_end: usize,
    fence: &'a Fences,
    marker: PhantomData<&'a T>,
}

impl<'a, T: ?Sized> BufferViewSlice<'a, T> where T: Content + 'a {
    /// Returns the size in bytes of this slice.
    pub fn get_size(&self) -> usize {
        self.bytes_end - self.bytes_start
    }

    /// Uploads some data in this buffer.
    ///
    /// ## Panic
    ///
    /// Panics if the length of `data` is different from the length of this buffer.
    pub fn write(&self, data: &T) {
        assert_eq!(mem::size_of_val(data), self.get_size());

        self.fence.wait(&mut self.alloc.get_context().make_current(),
                        self.bytes_start .. self.bytes_end);
        unsafe { self.alloc.upload(self.bytes_start, data); }
    }

    /// Invalidates the content of the slice. The data becomes undefined.
    ///
    /// This operation is a no-op if the backend doesn't support it.
    pub fn invalidate(&self) {
        self.alloc.invalidate(self.bytes_start, self.get_size());
    }

    /// Reads the content of the buffer.
    pub fn read(&self) -> Result<T::Owned, ReadError> {
        self.fence.wait(&mut self.alloc.get_context().make_current(),
                        self.bytes_start .. self.bytes_end);

        unsafe {
            self.alloc.read::<T>(self.bytes_start .. self.bytes_end)
        }
    }

    /// Builds a slice-any containing the whole subbuffer.
    pub fn as_slice_any(&self) -> BufferViewAnySlice<'a> {
        BufferViewAnySlice {
            alloc: self.alloc,
            bytes_start: self.bytes_start,
            bytes_end: self.bytes_end,
            elements_size: <T as Content>::get_elements_size(),
            fence: self.fence,
        }
    }
}

impl<'a, T> BufferViewSlice<'a, [T]> where [T]: Content + 'a {
    /// Returns the number of elements in this slice.
    pub fn len(&self) -> usize {
        (self.bytes_end - self.bytes_start) / mem::size_of::<T>()
    }

    /// Builds a subslice of this slice. Returns `None` if out of range.
    pub fn slice(&self, range: Range<usize>) -> Option<BufferViewSlice<'a, [T]>> {
        if range.start > self.len() || range.end > self.len() {
            return None;
        }

        Some(BufferViewSlice {
            alloc: self.alloc,
            bytes_start: self.bytes_start + range.start * mem::size_of::<T>(),
            bytes_end: self.bytes_start + range.end * mem::size_of::<T>(),
            fence: self.fence,
            marker: PhantomData,
        })
    }
}

impl<'a, T> BufferViewSlice<'a, [T]> where T: PixelValue + 'a {
    /// Reads the content of the buffer.
    pub fn read_as_texture_1d<S>(&self) -> Result<S, ReadError> where S: Texture1dDataSink<T> {
        let data = try!(self.read());
        Ok(S::from_raw(Cow::Owned(data), self.len() as u32))
    }
}

impl<'a, T: ?Sized> fmt::Debug for BufferViewSlice<'a, T> where T: Content {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self.alloc)
    }
}

impl<'a, T: ?Sized> BufferViewSliceExt<'a> for BufferViewSlice<'a, T> where T: Content {
    fn add_fence(&self) -> Option<Inserter<'a>> {
        if !self.alloc.uses_persistent_mapping() {
            return None;
        }

        Some(self.fence.inserter(self.bytes_start .. self.bytes_end))
    }
}

impl<'a, T: ?Sized> BufferViewExt for BufferViewSlice<'a, T> where T: Content {
    fn get_offset_bytes(&self) -> usize {
        self.bytes_start
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

/// Represents a sub-part of a buffer.
pub struct BufferViewMutSlice<'a, T: ?Sized> where T: Content {
    alloc: &'a mut Buffer,
    bytes_start: usize,
    bytes_end: usize,
    fence: &'a Fences,
    marker: PhantomData<T>,
}

impl<'a, T: ?Sized> BufferViewMutSlice<'a, T> where T: Content {
    /// Returns the size in bytes of this slice.
    pub fn get_size(&self) -> usize {
        self.bytes_end - self.bytes_start
    }

    /// Maps the buffer in memory for both reading and writing.
    pub fn map(self) -> Mapping<'a, T> {
        self.fence.wait(&mut self.alloc.get_context().make_current(),
                        self.bytes_start .. self.bytes_end);
        unsafe { self.alloc.map(self.bytes_start .. self.bytes_end) }
    }

    /// Maps the buffer in memory for reading.
    pub fn map_read(self) -> ReadMapping<'a, T> {
        self.fence.wait(&mut self.alloc.get_context().make_current(),
                        self.bytes_start .. self.bytes_end);
        unsafe { self.alloc.map_read(self.bytes_start .. self.bytes_end) }
    }

    /// Maps the buffer in memory for writing only.
    pub fn map_write(self) -> WriteMapping<'a, T> {
        self.fence.wait(&mut self.alloc.get_context().make_current(),
                        self.bytes_start .. self.bytes_end);
        unsafe { self.alloc.map_write(self.bytes_start .. self.bytes_end) }
    }

    /// Uploads some data in this buffer.
    ///
    /// ## Panic
    ///
    /// Panics if the length of `data` is different from the length of this buffer.
    pub fn write(&self, data: &T) {
        self.fence.wait(&mut self.alloc.get_context().make_current(),
                        self.bytes_start .. self.bytes_end);
        unsafe { self.alloc.upload(self.bytes_start, data); }
    }

    /// Invalidates the content of the slice. The data becomes undefined.
    ///
    /// This operation is a no-op if the backend doesn't support it.
    pub fn invalidate(&self) {
        self.alloc.invalidate(self.bytes_start, self.get_size());
    }

    /// Reads the content of the buffer.
    pub fn read(&self) -> Result<T::Owned, ReadError> {
        unsafe {
            self.alloc.read::<T>(self.bytes_start .. self.bytes_end)
        }
    }

    /// Builds a slice-any containing the whole subbuffer.
    pub fn as_slice_any(self) -> BufferViewAnySlice<'a> {
        BufferViewAnySlice {
            alloc: self.alloc,
            bytes_start: self.bytes_start,
            bytes_end: self.bytes_end,
            elements_size: <T as Content>::get_elements_size(),
            fence: self.fence,
        }
    }
}

impl<'a, T> BufferViewMutSlice<'a, [T]> where [T]: Content, T: Copy + 'a {
    /// Returns the number of elements in this slice.
    pub fn len(&self) -> usize {
        (self.bytes_end - self.bytes_start) / mem::size_of::<T>()
    }

    /// Builds a subslice of this slice. Returns `None` if out of range.
    pub fn slice(self, range: Range<usize>) -> Option<BufferViewMutSlice<'a, [T]>> {
        if range.start > self.len() || range.end > self.len() {
            return None;
        }

        Some(BufferViewMutSlice {
            alloc: self.alloc,
            bytes_start: self.bytes_start + range.start * mem::size_of::<T>(),
            bytes_end: self.bytes_start + range.end * mem::size_of::<T>(),
            fence: self.fence,
            marker: PhantomData,
        })
    }
}

impl<'a, T> BufferViewMutSlice<'a, [T]> where T: PixelValue + 'a {
    /// Reads the content of the buffer.
    pub fn read_as_texture_1d<S>(&self) -> Result<S, ReadError> where S: Texture1dDataSink<T> {
        let data = try!(self.read());
        Ok(S::from_raw(Cow::Owned(data), self.len() as u32))
    }
}

impl<'a, T: ?Sized> fmt::Debug for BufferViewMutSlice<'a, T> where T: Content {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self.alloc)
    }
}

/// Represents a sub-part of a buffer.
///
/// Doesn't contain any information about the content, contrary to `BufferView`.
pub struct BufferViewAny {
    alloc: Buffer,
    size: usize,
    elements_size: usize,
    fence: Fences,
}

impl BufferViewAny {
    /// Builds a slice-any containing the whole subbuffer.
    pub fn as_slice_any(&self) -> BufferViewAnySlice {
        BufferViewAnySlice {
            alloc: &self.alloc,
            bytes_start: 0,
            bytes_end: self.size,
            elements_size: self.elements_size,
            fence: &self.fence,
        }
    }

    /// Returns the size in bytes of each element in the buffer.
    // TODO: clumbsy, remove this function
    pub fn get_elements_size(&self) -> usize {
        self.elements_size
    }

    /// Returns the number of elements in the buffer.
    // TODO: clumbsy, remove this function
    pub fn get_elements_count(&self) -> usize {
        self.size / self.elements_size
    }
    
    /// Returns the context corresponding to this buffer.
    pub fn get_context(&self) -> &Rc<Context> {
        self.alloc.get_context()
    }

    /// Returns the number of bytes in this subbuffer.
    pub fn get_size(&self) -> usize {
        self.size
    }

    /// Invalidates the content of the buffer. The data becomes undefined.
    ///
    /// This operation is a no-op if the backend doesn't support it.
    pub fn invalidate(&self) {
        self.alloc.invalidate(0, self.size);
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
    pub unsafe fn read<T>(&self) -> Result<T::Owned, ReadError> where T: Content {
        // TODO: add check
        self.fence.wait(&mut self.alloc.get_context().make_current(), 0 .. self.get_size());
        self.alloc.read::<T>(0 .. self.get_size())
    }
}

impl<T: ?Sized> From<BufferView<T>> for BufferViewAny where T: Content + Send + 'static {
    fn from(mut buffer: BufferView<T>) -> BufferViewAny {
        let size = buffer.get_size();

        BufferViewAny {
            alloc: buffer.alloc.take().unwrap(),
            size: size,
            elements_size: <T as Content>::get_elements_size(),
            fence: buffer.fence.take().unwrap(),
        }
    }
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

/// Slice of a `BufferView` without any type info.
#[derive(Copy, Clone)]
pub struct BufferViewAnySlice<'a> {
    alloc: &'a Buffer,
    bytes_start: usize,
    bytes_end: usize,
    elements_size: usize,
    fence: &'a Fences,
}

impl<'a> BufferViewAnySlice<'a> {
    /// Returns the number of bytes in this slice.
    pub fn get_size(&self) -> usize {
        self.bytes_end - self.bytes_start
    }

    /// Returns the size in bytes of each element in the buffer.
    // TODO: clumbsy, remove this function
    pub fn get_elements_size(&self) -> usize {
        self.elements_size
    }

    /// Returns the number of elements in the buffer.
    // TODO: clumbsy, remove this function
    pub fn get_elements_count(&self) -> usize {
        self.get_size() / self.elements_size
    }

    /// Invalidates the content of the slice. The data becomes undefined.
    ///
    /// This operation is a no-op if the backend doesn't support it.
    pub fn invalidate(&self) {
        self.alloc.invalidate(self.bytes_start, self.get_size());
    }
}

impl<'a> fmt::Debug for BufferViewAnySlice<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self.alloc)
    }
}

impl<'a> BufferViewSliceExt<'a> for BufferViewAnySlice<'a> {
    fn add_fence(&self) -> Option<Inserter<'a>> {
        if !self.alloc.uses_persistent_mapping() {
            return None;
        }

        Some(self.fence.inserter(self.bytes_start .. self.bytes_end))
    }
}

impl<'a> BufferViewExt for BufferViewAnySlice<'a> {
    fn get_offset_bytes(&self) -> usize {
        self.bytes_start
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
