use std::fmt;
use std::mem;
use std::ops::Range;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;

use sync::LinearSyncFence;
use gl;

use backend::Facade;
use SubBufferExt;
use SubBufferSliceExt;
use GlObject;

use context::Context;
use std::rc::Rc;

use buffer::BufferType;
use buffer::BufferCreationError;
use buffer::alloc::Buffer;
use buffer::alloc::Mapping as BufferMapping;

/// Represents a sub-part of a buffer.
pub struct SubBuffer<T> where T: Copy + Send + 'static {
    alloc: Buffer,
    offset_bytes: usize,
    num_elements: usize,
    fence: RefCell<Option<LinearSyncFence>>,
    marker: PhantomData<T>,
}

impl<T> fmt::Debug for SubBuffer<T> where T: Copy + Send + 'static {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self.alloc)
    }
}

/// Mapping of a buffer in memory.
pub struct Mapping<'a, T> {
    mapping: BufferMapping<'a, T>,
}

impl<'a, T> Deref for Mapping<'a, T> {
    type Target = [T];
    fn deref<'b>(&'b self) -> &'b [T] {
        self.mapping.deref()
    }
}

impl<'a, T> DerefMut for Mapping<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut [T] {
        self.mapping.deref_mut()
    }
}

/// Represents a sub-part of a buffer.
#[derive(Copy, Clone)]
pub struct SubBufferSlice<'a, T> where T: Copy + Send + 'static {
    alloc: &'a Buffer,
    offset_bytes: usize,
    num_elements: usize,
    fence: &'a RefCell<Option<LinearSyncFence>>,
    marker: PhantomData<T>,
}

impl<'a, T> fmt::Debug for SubBufferSlice<'a, T> where T: Copy + Send + 'static {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self.alloc)
    }
}

/// Represents a sub-part of a buffer.
#[derive(Copy, Clone)]
pub struct SubBufferMutSlice<'a, T> where T: Copy + Send + 'static {
    alloc: &'a Buffer,
    offset_bytes: usize,
    num_elements: usize,
    fence: &'a RefCell<Option<LinearSyncFence>>,
    marker: PhantomData<T>,
}

impl<'a, T> fmt::Debug for SubBufferMutSlice<'a, T> where T: Copy + Send + 'static {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self.alloc)
    }
}

/// Represents a sub-part of a buffer.
///
/// Doesn't contain any information about the content, contrary to `SubBuffer`.
pub struct SubBufferAny {
    alloc: Buffer,
    offset_bytes: usize,
    elements_size: usize,
    elements_count: usize,
    fence: RefCell<Option<LinearSyncFence>>,
}

impl fmt::Debug for SubBufferAny {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self.alloc)
    }
}

/// Slice of a `SubBuffer` without any type info.
#[derive(Copy, Clone)]
pub struct SubBufferAnySlice<'a> {
    alloc: &'a Buffer,
    offset_bytes: usize,
    elements_size: usize,
    elements_count: usize,
    fence: &'a RefCell<Option<LinearSyncFence>>,
}

impl<'a> fmt::Debug for SubBufferAnySlice<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self.alloc)
    }
}

impl<T> From<SubBuffer<T>> for SubBufferAny where T: Copy + Send + 'static {
    fn from(buffer: SubBuffer<T>) -> SubBufferAny {
        SubBufferAny {
            alloc: buffer.alloc,
            offset_bytes: buffer.offset_bytes,
            elements_size: mem::size_of::<T>(),
            elements_count: buffer.num_elements,
            fence: buffer.fence,
        }
    }
}

impl<T> SubBuffer<T> where T: Copy + Send + 'static {
    /// Builds a new buffer containing the given data. The size of the buffer is equal to the size
    /// of the data.
    ///
    /// If `dynamic` is false, glium will attempt to use buffer storage to create an immutable
    /// buffer (without the `DYNAMIC_STORAGE_FLAG`).
    ///
    /// If `dynamic` is true, glium will attempt to use persistent mapping and manage
    /// synchronizations manually.
    pub fn new<F>(facade: &F, data: &[T], ty: BufferType, dynamic: bool)
                  -> Result<SubBuffer<T>, BufferCreationError>
                  where F: Facade
    {
        let len = data.len();

        Buffer::new(facade, data, ty, dynamic)
            .map(|buffer| {
                SubBuffer {
                    alloc: buffer,
                    offset_bytes: 0,
                    num_elements: len,
                    fence: RefCell::new(None),
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
                    -> Result<SubBuffer<T>, BufferCreationError> where F: Facade
    {
        Buffer::empty(facade, ty, len * mem::size_of::<T>(), dynamic)
            .map(|buffer| {
                SubBuffer {
                    alloc: buffer,
                    offset_bytes: 0,
                    num_elements: len,
                    fence: RefCell::new(None),
                    marker: PhantomData,
                }
            })
    }

    /// Returns the context corresponding to this buffer.
    pub fn get_context(&self) -> &Rc<Context> {
        self.alloc.get_context()
    }

    /// Returns the number of elements in this subbuffer.
    pub fn len(&self) -> usize {
        self.num_elements
    }

    /// Returns true if this buffer uses persistent mapping.
    pub fn is_persistent(&self) -> bool {
        self.alloc.uses_persistent_mapping()
    }

    /// Uploads some data in this buffer.
    ///
    /// ## Panic
    ///
    /// Panics if the length of `data` is different from the length of this buffer.
    pub fn write<P>(&self, data: P) where P: AsRef<[T]> {
        self.as_slice().write(data);
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

    /// Maps the buffer in memory.
    pub fn map(&mut self) -> Mapping<T> {
        self.as_mut_slice().map()
    }

    /// Reads the content of the subbuffer. Returns `None` if this operation is not supported.
    pub fn read_if_supported(&self) -> Option<Vec<T>> {
        self.as_slice().read_if_supported()
    }

    /// Builds a slice of this subbuffer. Returns `None` if out of range.
    pub fn slice(&self, range: Range<usize>) -> Option<SubBufferSlice<T>> {
        self.as_slice().slice(range)
    }

    /// Builds a slice of this subbuffer. Returns `None` if out of range.
    pub fn slice_mut(&self, range: Range<usize>) -> Option<SubBufferMutSlice<T>> {
        self.as_mut_slice().slice(range)
    }

    /// Builds a slice containing the whole subbuffer.
    pub fn as_slice(&self) -> SubBufferSlice<T> {
        SubBufferSlice {
            alloc: &self.alloc,
            offset_bytes: self.offset_bytes,
            num_elements: self.num_elements,
            fence: &self.fence,
            marker: PhantomData,
        }
    }

    /// Builds a slice containing the whole subbuffer.
    pub fn as_mut_slice(&self) -> SubBufferMutSlice<T> {
        SubBufferMutSlice {
            alloc: &self.alloc,
            offset_bytes: self.offset_bytes,
            num_elements: self.num_elements,
            fence: &self.fence,
            marker: PhantomData,
        }
    }

    /// Builds a slice-any containing the whole subbuffer.
    pub fn as_slice_any(&self) -> SubBufferAnySlice {
        SubBufferAnySlice {
            alloc: &self.alloc,
            offset_bytes: self.offset_bytes,
            elements_size: mem::size_of::<T>(),
            elements_count: self.num_elements,
            fence: &self.fence,
        }
    }
}

impl SubBufferAny {
    /// Builds a slice-any containing the whole subbuffer.
    pub fn as_slice_any(&self) -> SubBufferAnySlice {
        SubBufferAnySlice {
            alloc: &self.alloc,
            offset_bytes: self.offset_bytes,
            elements_size: self.elements_size,
            elements_count: self.elements_count,
            fence: &self.fence,
        }
    }
    
    /// Returns the context corresponding to this buffer.
    pub fn get_context(&self) -> &Rc<Context> {
        self.alloc.get_context()
    }
}

impl<'a, T> SubBufferSlice<'a, T> where T: Copy + Send + 'static {
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
        consume_fence(self.alloc.get_context(), self.fence);
        unsafe { self.alloc.upload(self.offset_bytes, data); }
    }

    /// Reads the content of the slice. Returns `None` if this operation is not supported.
    pub fn read_if_supported(&self) -> Option<Vec<T>> {
        consume_fence(self.alloc.get_context(), self.fence);

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
    pub fn slice(&self, range: Range<usize>) -> Option<SubBufferSlice<'a, T>> {
        if range.start > self.num_elements || range.end > self.num_elements {
            return None;
        }

        Some(SubBufferSlice {
            alloc: self.alloc,
            offset_bytes: self.offset_bytes + range.start * mem::size_of::<T>(),
            num_elements: range.end - range.start,
            fence: self.fence,
            marker: PhantomData,
        })
    }

    /// Builds a slice-any containing the whole subbuffer.
    pub fn as_slice_any(&self) -> SubBufferAnySlice<'a> {
        SubBufferAnySlice {
            alloc: self.alloc,
            offset_bytes: self.offset_bytes,
            elements_size: mem::size_of::<T>(),
            elements_count: self.num_elements,
            fence: self.fence,
        }
    }
}

impl<'a, T> SubBufferMutSlice<'a, T> where T: Copy + Send + 'static {
    /// Returns the number of elements in this slice.
    pub fn len(&self) -> usize {
        self.num_elements
    }

    /// Maps the buffer in memory.
    pub fn map(&mut self) -> Mapping<'a, T> {
        consume_fence(self.alloc.get_context(), self.fence);
        unsafe {
            Mapping { mapping: self.alloc.map(self.offset_bytes, self.num_elements) }
        }
    }

    /// Uploads some data in this buffer.
    ///
    /// ## Panic
    ///
    /// Panics if the length of `data` is different from the length of this buffer.
    pub fn write<P>(&self, data: P) where P: AsRef<[T]> {
        let data = data.as_ref();
        assert!(data.len() == self.num_elements);
        consume_fence(self.alloc.get_context(), self.fence);
        unsafe { self.alloc.upload(self.offset_bytes, data); }
    }

    /// Reads the content of the buffer.
    #[cfg(feature = "gl_read_buffer")]
    pub fn read(&self) -> Vec<T> {
        self.read_if_supported().unwrap()
    }

    /// Reads the content of the slice. Returns `None` if this operation is not supported.
    pub fn read_if_supported(&self) -> Option<Vec<T>> {
        consume_fence(self.alloc.get_context(), self.fence);

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
    pub fn slice(self, range: Range<usize>) -> Option<SubBufferMutSlice<'a, T>> {
        if range.start > self.num_elements || range.end > self.num_elements {
            return None;
        }

        Some(SubBufferMutSlice {
            alloc: self.alloc,
            offset_bytes: self.offset_bytes + range.start * mem::size_of::<T>(),
            num_elements: range.end - range.start,
            fence: self.fence,
            marker: PhantomData,
        })
    }

    /// Builds a slice-any containing the whole subbuffer.
    pub fn as_slice_any(&self) -> SubBufferAnySlice<'a> {
        SubBufferAnySlice {
            alloc: self.alloc,
            offset_bytes: self.offset_bytes,
            elements_size: mem::size_of::<T>(),
            elements_count: self.num_elements,
            fence: self.fence,
        }
    }
}

impl SubBufferAny {
    /// Returns the size of each element in this buffer.
    ///
    /// This information is taken from the original `SubBuffer` that was used to construct
    /// this object.
    pub fn get_elements_size(&self) -> usize {
        self.elements_size
    }

    /// Returns the number of elements in this buffer.
    ///
    /// This information is taken from the original `SubBuffer` that was used to construct
    /// this object.
    pub fn get_elements_count(&self) -> usize {
        self.elements_count
    }

    /// Returns the number of bytes in this subbuffer.
    pub fn get_size(&self) -> usize {
        self.elements_size * self.elements_count
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
    pub unsafe fn read_if_supported<T>(&self) -> Option<Vec<T>> where T: Copy + Send + 'static {
        assert!(self.get_size() % mem::size_of::<T>() == 0);

        consume_fence(self.alloc.get_context(), &self.fence);

        let len = self.get_size() / mem::size_of::<T>();
        let mut data = Vec::with_capacity(len);
        data.set_len(len);        // TODO: is this safe?

        match self.alloc.read_if_supported(self.offset_bytes, &mut data) {
            Err(_) => return None,
            Ok(_) => ()
        };

        Some(data)
    }
}

impl<'a> SubBufferAnySlice<'a> {
    /// Returns the size of each element in this buffer.
    ///
    /// This information is taken from the original `SubBuffer` that was used to construct
    /// this object.
    pub fn get_elements_size(&self) -> usize {
        self.elements_size
    }

    /// Returns the number of elements in this buffer.
    ///
    /// This information is taken from the original `SubBuffer` that was used to construct
    /// this object.
    pub fn get_elements_count(&self) -> usize {
        self.elements_count
    }

    /// Returns the number of bytes in this slice.
    pub fn get_size(&self) -> usize {
        self.elements_size * self.elements_count
    }
}

impl<T> SubBufferExt for SubBuffer<T> where T: Copy + Send + 'static {
    fn get_offset_bytes(&self) -> usize {
        self.offset_bytes
    }

    fn get_buffer_id(&self) -> gl::types::GLuint {
        self.alloc.get_id()
    }
}

impl<'a, T> SubBufferSliceExt<'a> for SubBufferSlice<'a, T> where T: Copy + Send + 'static {
    fn add_fence(&self) -> Option<&'a RefCell<Option<LinearSyncFence>>> {
        if !self.alloc.uses_persistent_mapping() {
            return None;
        }

        Some(self.fence)
    }
}

impl<'a, T> SubBufferExt for SubBufferSlice<'a, T> where T: Copy + Send + 'static {
    fn get_offset_bytes(&self) -> usize {
        self.offset_bytes
    }

    fn get_buffer_id(&self) -> gl::types::GLuint {
        self.alloc.get_id()
    }
}

impl SubBufferExt for SubBufferAny {
    fn get_offset_bytes(&self) -> usize {
        self.offset_bytes
    }

    fn get_buffer_id(&self) -> gl::types::GLuint {
        self.alloc.get_id()
    }
}

impl<'a> SubBufferSliceExt<'a> for SubBufferAnySlice<'a> {
    fn add_fence(&self) -> Option<&'a RefCell<Option<LinearSyncFence>>> {
        if !self.alloc.uses_persistent_mapping() {
            return None;
        }

        Some(self.fence)
    }
}

impl<'a> SubBufferExt for SubBufferAnySlice<'a> {
    fn get_offset_bytes(&self) -> usize {
        self.offset_bytes
    }

    fn get_buffer_id(&self) -> gl::types::GLuint {
        self.alloc.get_id()
    }
}

/// Waits for the fence to be sync'ed.
fn consume_fence(context: &Rc<Context>, fence: &RefCell<Option<LinearSyncFence>>) {
    let fence = fence.borrow_mut().take();
    if let Some(fence) = fence {
        fence.into_sync_fence(context).wait();
    }
}
