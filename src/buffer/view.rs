use std::fmt;
use std::mem;
use std::borrow::Cow;
use utils::range::RangeArgument;
use std::marker::PhantomData;

use texture::{PixelValue, Texture1dDataSink};
use gl;

use backend::Facade;
use BufferExt;
use BufferSliceExt;
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
use buffer::alloc::Alloc;
use buffer::alloc::Mapping;
use buffer::alloc::ReadMapping;
use buffer::alloc::WriteMapping;
use buffer::alloc::ReadError;
use buffer::alloc::CopyError;

/// Represents a view of a buffer.
pub struct Buffer<T: ?Sized> where T: Content {
    // TODO: this `Option` is here because we have a destructor and need to be able to move out
    alloc: Option<Alloc>,
    // TODO: this `Option` is here because we have a destructor and need to be able to move out
    fence: Option<Fences>,
    marker: PhantomData<T>,
}

impl<T: ?Sized> GlObject for Buffer<T> where T: Content {
    type Id = gl::types::GLuint;

    #[inline]
    fn get_id(&self) -> gl::types::GLuint {
        self.alloc.as_ref().unwrap().get_id()
    }
}

impl<T: ?Sized> Buffer<T> where T: Content {
    /// Builds a new buffer containing the given data. The size of the buffer is equal to the size
    /// of the data.
    pub fn new<F: ?Sized>(facade: &F, data: &T, ty: BufferType, mode: BufferMode)
                  -> Result<Buffer<T>, BufferCreationError>
                  where F: Facade
    {
        Alloc::new(facade, data, ty, mode)
            .map(|buffer| {
                Buffer {
                    alloc: Some(buffer),
                    fence: Some(Fences::new()),
                    marker: PhantomData,
                }
            })
    }

    /// Builds a new buffer of the given size.
    pub fn empty_unsized<F: ?Sized>(facade: &F, ty: BufferType, size: usize, mode: BufferMode)
                            -> Result<Buffer<T>, BufferCreationError> where F: Facade
    {
        assert!(<T as Content>::is_size_suitable(size));

        Alloc::empty(facade, ty, size, mode)
            .map(|buffer| {
                Buffer {
                    alloc: Some(buffer),
                    fence: Some(Fences::new()),
                    marker: PhantomData,
                }
            })
    }

    /// Returns the context corresponding to this buffer.
    #[inline]
    pub fn get_context(&self) -> &Rc<Context> {
        self.alloc.as_ref().unwrap().get_context()
    }

    /// Returns the size in bytes of this buffer.
    #[inline]
    pub fn get_size(&self) -> usize {
        self.alloc.as_ref().unwrap().get_size()
    }

    /// Returns true if this buffer uses persistent mapping.
    #[inline]
    pub fn is_persistent(&self) -> bool {
        self.alloc.as_ref().unwrap().uses_persistent_mapping()
    }

    /// Uploads some data in this buffer.
    ///
    /// # Implementation
    ///
    /// - For persistent-mapped buffers, waits untils the data is no longer used by the GPU then
    ///   memcpies the data to the mapping.
    /// - For immutable buffers, creates a temporary buffer that contains the data then calls
    ///   `glCopyBufferSubData` to copy from the temporary buffer to the real one.
    /// - For other types, calls `glBufferSubData`.
    ///
    /// # Panic
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
    /// This operation is a no-op if the backend doesn't support it and for persistent-mapped
    /// buffers.
    ///
    /// # Implementation
    ///
    /// Calls `glInvalidateBufferData` if supported. Otherwise, calls `glBufferData` with a null
    /// pointer for data. If `glBufferStorage` has been used to create the buffer and
    /// `glInvalidateBufferData` is not supported, does nothing.
    ///
    #[inline]
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
    ///
    /// # Implementation
    ///
    /// - For persistent-mapped buffers, waits until the data is no longer accessed by the GPU then
    ///   returns a pointer to the existing mapping.
    /// - For immutable buffers, creates a temporary buffer containing the data of the buffer and
    ///   maps it. When the mapping object is destroyed, copies the content of the temporary buffer
    ///   to the real buffer.
    /// - For other types, calls `glMapBuffer` or `glMapSubBuffer`.
    ///
    pub fn map(&mut self) -> Mapping<T> {
        self.fence.as_ref().unwrap().wait(&mut self.alloc.as_ref().unwrap().get_context().make_current(),
                                          0 .. self.get_size());
        let size = self.get_size();
        unsafe { self.alloc.as_mut().unwrap().map(0 .. size) }
    }

    /// Maps the buffer in memory for reading.
    ///
    /// # Implementation
    ///
    /// - For persistent-mapped buffers, waits until the data is no longer accessed by the GPU then
    ///   returns a pointer to the existing mapping.
    /// - For immutable buffers, creates a temporary buffer containing the data of the buffer and
    ///   maps it.
    /// - For other types, calls `glMapBuffer` or `glMapSubBuffer`.
    ///
    pub fn map_read(&mut self) -> ReadMapping<T> {
        self.fence.as_ref().unwrap().wait(&mut self.alloc.as_ref().unwrap().get_context().make_current(),
                                          0 .. self.get_size());
        let size = self.get_size();
        unsafe { self.alloc.as_mut().unwrap().map_read(0 .. size) }
    }

    /// Maps the buffer in memory for writing only.
    ///
    /// # Implementation
    ///
    /// - For persistent-mapped buffers, waits until the data is no longer accessed by the GPU then
    ///   returns a pointer to the existing mapping.
    /// - For immutable buffers, creates a temporary buffer and
    ///   maps it. When the mapping object is destroyed, copies the content of the temporary buffer
    ///   to the real buffer.
    /// - For other types, calls `glMapBuffer` or `glMapSubBuffer`.
    ///
    pub fn map_write(&mut self) -> WriteMapping<T> {
        self.fence.as_ref().unwrap().wait(&mut self.alloc.as_ref().unwrap().get_context().make_current(),
                                          0 .. self.get_size());
        let size = self.get_size();
        unsafe { self.alloc.as_mut().unwrap().map_write(0 .. size) }
    }

    /// Copies the content of the buffer to another buffer.
    ///
    /// # Panic
    ///
    /// Panics if `T` is unsized and the other buffer is too small.
    ///
    pub fn copy_to<'a, S>(&self, target: S) -> Result<(), CopyError>
                          where S: Into<BufferSlice<'a, T>>, T: 'a
    {
        let target = target.into();
        let alloc = self.alloc.as_ref().unwrap();

        alloc.copy_to(0 .. self.get_size(), &target.alloc, target.get_offset_bytes())?;

        if let Some(inserter) = self.as_slice().add_fence() {
            let mut ctxt = alloc.get_context().make_current();
            inserter.insert(&mut ctxt);
        }

        if let Some(inserter) = target.add_fence() {
            let mut ctxt = alloc.get_context().make_current();
            inserter.insert(&mut ctxt);
        }

        Ok(())
    }

    /// Builds a slice that contains an element from inside the buffer.
    ///
    /// This method builds an object that represents a slice of the buffer. No actual operation
    /// OpenGL is performed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[derive(Copy, Clone)]
    /// struct BufferContent {
    ///     value1: u16,
    ///     value2: u16,
    /// }
    /// # let buffer: glium::buffer::BufferSlice<BufferContent> =
    /// #                                                   unsafe { std::mem::uninitialized() };
    /// let slice = unsafe { buffer.slice_custom(|content| &content.value2) };
    /// ```
    ///
    /// # Safety
    ///
    /// The object whose reference is passed to the closure is uninitialized. Therefore you
    /// **must not** access the content of the object.
    ///
    /// You **must** return a reference to an element from the parameter. The closure **must not**
    /// panic.
    #[inline]
    pub unsafe fn slice_custom<F, R: ?Sized>(&self, f: F) -> BufferSlice<R>
                                             where F: for<'r> FnOnce(&'r T) -> &'r R,
                                                    R: Content
    {
        self.as_slice().slice_custom(f)
    }

    /// Same as `slice_custom` but returns a mutable slice.
    ///
    /// This method builds an object that represents a slice of the buffer. No actual operation
    /// OpenGL is performed.
    #[inline]
    pub unsafe fn slice_custom_mut<F, R: ?Sized>(&mut self, f: F) -> BufferMutSlice<R>
                                                 where F: for<'r> FnOnce(&'r T) -> &'r R,
                                                        R: Content
    {
        self.as_mut_slice().slice_custom(f)
    }

    /// Builds a slice containing the whole subbuffer.
    ///
    /// This method builds an object that represents a slice of the buffer. No actual operation
    /// OpenGL is performed.
    #[inline]
    pub fn as_slice(&self) -> BufferSlice<T> {
        BufferSlice {
            alloc: self.alloc.as_ref().unwrap(),
            bytes_start: 0,
            bytes_end: self.get_size(),
            fence: self.fence.as_ref().unwrap(),
            marker: PhantomData,
        }
    }

    /// Builds a slice containing the whole subbuffer.
    ///
    /// This method builds an object that represents a slice of the buffer. No actual operation
    /// OpenGL is performed.
    #[inline]
    pub fn as_mut_slice(&mut self) -> BufferMutSlice<T> {
        let size = self.get_size();

        BufferMutSlice {
            alloc: self.alloc.as_mut().unwrap(),
            bytes_start: 0,
            bytes_end: size,
            fence: self.fence.as_ref().unwrap(),
            marker: PhantomData,
        }
    }

    /// Builds a slice-any containing the whole subbuffer.
    ///
    /// This method builds an object that represents a slice of the buffer. No actual operation
    /// OpenGL is performed.
    pub fn as_slice_any(&self) -> BufferAnySlice {
        let size = self.get_size();

        BufferAnySlice {
            alloc: self.alloc.as_ref().unwrap(),
            bytes_start: 0,
            bytes_end: self.get_size(),
            elements_size: <T as Content>::get_elements_size(),
            fence: self.fence.as_ref().unwrap(),
        }
    }
}

impl<T> Buffer<T> where T: Content + Copy {
    /// Builds a new buffer of the given size.
    pub fn empty<F: ?Sized>(facade: &F, ty: BufferType, mode: BufferMode)
                    -> Result<Buffer<T>, BufferCreationError> where F: Facade
    {
        Alloc::empty(facade, ty, mem::size_of::<T>(), mode)
            .map(|buffer| {
                Buffer {
                    alloc: Some(buffer),
                    fence: Some(Fences::new()),
                    marker: PhantomData,
                }
            })
    }
}

impl<T> Buffer<[T]> where [T]: Content, T: Copy {
    /// Builds a new buffer of the given size.
    pub fn empty_array<F: ?Sized>(facade: &F, ty: BufferType, len: usize, mode: BufferMode)
                          -> Result<Buffer<[T]>, BufferCreationError> where F: Facade
    {
        Alloc::empty(facade, ty, len * mem::size_of::<T>(), mode)
            .map(|buffer| {
                Buffer {
                    alloc: Some(buffer),
                    fence: Some(Fences::new()),
                    marker: PhantomData,
                }
            })
    }

    /// Returns the number of elements in this buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.alloc.as_ref().unwrap().get_size() / mem::size_of::<T>()
    }

    /// Builds a slice of this subbuffer. Returns `None` if out of range.
    ///
    /// This method builds an object that represents a slice of the buffer. No actual operation
    /// OpenGL is performed.
    #[inline]
    pub fn slice<R: RangeArgument<usize>>(&self, range: R) -> Option<BufferSlice<[T]>> {
        self.as_slice().slice(range)
    }

    /// Builds a slice of this subbuffer. Returns `None` if out of range.
    ///
    /// This method builds an object that represents a slice of the buffer. No actual operation
    /// OpenGL is performed.
    #[inline]
    pub fn slice_mut<R: RangeArgument<usize>>(&mut self, range: R) -> Option<BufferMutSlice<[T]>> {
        self.as_mut_slice().slice(range)
    }
}

impl<T> Buffer<[T]> where T: PixelValue {
    /// Reads the content of the buffer.
    #[inline]
    pub fn read_as_texture_1d<S>(&self) -> Result<S, ReadError> where S: Texture1dDataSink<T> {
        let data = self.read()?;
        Ok(S::from_raw(Cow::Owned(data), self.len() as u32))
    }
}

impl<T: ?Sized> fmt::Debug for Buffer<T> where T: Content {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self.alloc.as_ref().unwrap())
    }
}

impl<T: ?Sized> Drop for Buffer<T> where T: Content {
    #[inline]
    fn drop(&mut self) {
        if let (Some(alloc), Some(mut fence)) = (self.alloc.take(), self.fence.take()) {
            fence.clean(&mut alloc.get_context().make_current());
        }
    }
}

impl<T: ?Sized> BufferExt for Buffer<T> where T: Content {
    #[inline]
    fn get_offset_bytes(&self) -> usize {
        0
    }

    #[inline]
    fn prepare_for_vertex_attrib_array(&self, ctxt: &mut CommandContext) {
        let alloc = self.alloc.as_ref().unwrap();
        alloc.prepare_for_vertex_attrib_array(ctxt);
    }

    #[inline]
    fn prepare_for_element_array(&self, ctxt: &mut CommandContext) {
        let alloc = self.alloc.as_ref().unwrap();
        alloc.prepare_for_element_array(ctxt);
    }

    #[inline]
    fn bind_to_element_array(&self, ctxt: &mut CommandContext) {
        let alloc = self.alloc.as_ref().unwrap();
        alloc.bind_to_element_array(ctxt);
    }

    #[inline]
    fn prepare_and_bind_for_pixel_pack(&self, ctxt: &mut CommandContext) {
        let alloc = self.alloc.as_ref().unwrap();
        alloc.prepare_and_bind_for_pixel_pack(ctxt);
    }

    #[inline]
    fn unbind_pixel_pack(ctxt: &mut CommandContext) {
        Alloc::unbind_pixel_pack(ctxt)
    }

    #[inline]
    fn prepare_and_bind_for_pixel_unpack(&self, ctxt: &mut CommandContext) {
        let alloc = self.alloc.as_ref().unwrap();
        alloc.prepare_and_bind_for_pixel_unpack(ctxt);
    }

    #[inline]
    fn unbind_pixel_unpack(ctxt: &mut CommandContext) {
        Alloc::unbind_pixel_unpack(ctxt)
    }

    #[inline]
    fn prepare_and_bind_for_query(&self, ctxt: &mut CommandContext) {
        let alloc = self.alloc.as_ref().unwrap();
        alloc.prepare_and_bind_for_query(ctxt);
    }

    #[inline]
    fn unbind_query(ctxt: &mut CommandContext) {
        Alloc::unbind_query(ctxt)
    }

    #[inline]
    fn prepare_and_bind_for_draw_indirect(&self, ctxt: &mut CommandContext) {
        let alloc = self.alloc.as_ref().unwrap();
        alloc.prepare_and_bind_for_draw_indirect(ctxt);
    }

    #[inline]
    fn prepare_and_bind_for_dispatch_indirect(&self, ctxt: &mut CommandContext) {
        let alloc = self.alloc.as_ref().unwrap();
        alloc.prepare_and_bind_for_dispatch_indirect(ctxt);
    }

    #[inline]
    fn prepare_and_bind_for_uniform(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        let alloc = self.alloc.as_ref().unwrap();
        alloc.prepare_and_bind_for_uniform(ctxt, index, 0 .. alloc.get_size());
    }

    #[inline]
    fn prepare_and_bind_for_shared_storage(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        let alloc = self.alloc.as_ref().unwrap();
        alloc.prepare_and_bind_for_shared_storage(ctxt, index, 0 .. alloc.get_size());
    }

    #[inline]
    fn bind_to_transform_feedback(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        let alloc = self.alloc.as_ref().unwrap();
        alloc.bind_to_transform_feedback(ctxt, index, 0 .. alloc.get_size());
    }
}

/// Represents a sub-part of a buffer.
#[derive(Copy, Clone)]
pub struct BufferSlice<'a, T: ?Sized> where T: Content + 'a {
    alloc: &'a Alloc,
    bytes_start: usize,
    bytes_end: usize,
    fence: &'a Fences,
    marker: PhantomData<&'a T>,
}

impl<'a, T: ?Sized> BufferSlice<'a, T> where T: Content + 'a {
    /// Returns the size in bytes of this slice.
    #[inline]
    pub fn get_size(&self) -> usize {
        self.bytes_end - self.bytes_start
    }

    /// Returns the context corresponding to this buffer.
    #[inline]
    pub fn get_context(&self) -> &Rc<Context> {
        self.alloc.get_context()
    }

    /// Uploads some data in this buffer.
    ///
    /// # Implementation
    ///
    /// - For persistent-mapped buffers, waits untils the data is no longer used by the GPU then
    ///   memcpies the data to the mapping.
    /// - For immutable buffers, creates a temporary buffer that contains the data then calls
    ///   `glCopyBufferSubData` to copy from the temporary buffer to the real one.
    /// - For other types, calls `glBufferSubData`.
    ///
    /// # Panic
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
    /// This operation is a no-op if the backend doesn't support it and for persistent-mapped
    /// buffers.
    ///
    /// # Implementation
    ///
    /// Calls `glInvalidateBufferSubData` if supported.
    ///
    #[inline]
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

    /// Copies the content of this slice to another slice.
    ///
    /// # Panic
    ///
    /// Panics if `T` is unsized and the other buffer is too small.
    pub fn copy_to<S>(&self, target: S) -> Result<(), CopyError>
                      where S: Into<BufferSlice<'a, T>>
    {
        let target = target.into();

        self.alloc.copy_to(self.bytes_start .. self.bytes_end, &target.alloc,
                           target.get_offset_bytes())?;

        if let Some(inserter) = self.add_fence() {
            let mut ctxt = self.alloc.get_context().make_current();
            inserter.insert(&mut ctxt);
        }

        if let Some(inserter) = target.add_fence() {
            let mut ctxt = self.alloc.get_context().make_current();
            inserter.insert(&mut ctxt);
        }

        Ok(())
    }

    /// Builds a slice that contains an element from inside the buffer.
    ///
    /// This method builds an object that represents a slice of the buffer. No actual operation
    /// OpenGL is performed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[derive(Copy, Clone)]
    /// struct BufferContent {
    ///     value1: u16,
    ///     value2: u16,
    /// }
    /// # let buffer: glium::buffer::BufferSlice<BufferContent> =
    /// #                                                   unsafe { std::mem::uninitialized() };
    /// let slice = unsafe { buffer.slice_custom(|content| &content.value2) };
    /// ```
    ///
    /// # Safety
    ///
    /// The object whose reference is passed to the closure is uninitialized. Therefore you
    /// **must not** access the content of the object.
    ///
    /// You **must** return a reference to an element from the parameter. The closure **must not**
    /// panic.
    #[inline]
    pub unsafe fn slice_custom<F, R: ?Sized>(&self, f: F) -> BufferSlice<'a, R>
                                             where F: for<'r> FnOnce(&'r T) -> &'r R,
                                                   R: Content
    {
        let data: &T = mem::zeroed();
        let result = f(data);
        let size = mem::size_of_val(result);
        let result = result as *const R as *const () as usize;

        assert!(result <= self.get_size());
        assert!(result + size <= self.get_size());

        BufferSlice {
            alloc: self.alloc,
            bytes_start: self.bytes_start + result,
            bytes_end: self.bytes_start + result + size,
            fence: self.fence,
            marker: PhantomData,
        }
    }

    /// Builds a slice-any containing the whole subbuffer.
    ///
    /// This method builds an object that represents a slice of the buffer. No actual operation
    /// OpenGL is performed.
    #[inline]
    pub fn as_slice_any(&self) -> BufferAnySlice<'a> {
        BufferAnySlice {
            alloc: self.alloc,
            bytes_start: self.bytes_start,
            bytes_end: self.bytes_end,
            elements_size: <T as Content>::get_elements_size(),
            fence: self.fence,
        }
    }
}

impl<'a, T> BufferSlice<'a, [T]> where [T]: Content + 'a {
    /// Returns the number of elements in this slice.
    #[inline]
    pub fn len(&self) -> usize {
        (self.bytes_end - self.bytes_start) / mem::size_of::<T>()
    }

    /// Builds a subslice of this slice. Returns `None` if out of range.
    ///
    /// This method builds an object that represents a slice of the buffer. No actual operation
    /// OpenGL is performed.
    #[inline]
    pub fn slice<R: RangeArgument<usize>>(&self, range: R) -> Option<BufferSlice<'a, [T]>> {
        if range.start().map_or(0, |e| *e) > self.len() || range.end().map_or(0, |e| *e) > self.len() {
            return None;
        }

        Some(BufferSlice {
            alloc: self.alloc,
            bytes_start: self.bytes_start + range.start().map_or(0, |e| *e) * mem::size_of::<T>(),
            bytes_end: self.bytes_start + range.end().map_or(self.len(), |e| *e) * mem::size_of::<T>(),
            fence: self.fence,
            marker: PhantomData,
        })
    }
}

impl<'a, T> BufferSlice<'a, [T]> where T: PixelValue + 'a {
    /// Reads the content of the buffer.
    #[inline]
    pub fn read_as_texture_1d<S>(&self) -> Result<S, ReadError> where S: Texture1dDataSink<T> {
        let data = self.read()?;
        Ok(S::from_raw(Cow::Owned(data), self.len() as u32))
    }
}

impl<'a, T: ?Sized> fmt::Debug for BufferSlice<'a, T> where T: Content {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self.alloc)
    }
}

impl<'a, T: ?Sized> From<BufferMutSlice<'a, T>> for BufferSlice<'a, T> where T: Content + 'a {
    #[inline]
    fn from(s: BufferMutSlice<'a, T>) -> BufferSlice<'a, T> {
        BufferSlice {
            alloc: s.alloc,
            bytes_start: s.bytes_start,
            bytes_end: s.bytes_end,
            fence: s.fence,
            marker: PhantomData,
        }
    }
}

impl<'a, T: ?Sized> From<&'a Buffer<T>> for BufferSlice<'a, T> where T: Content + 'a {
    #[inline]
    fn from(b: &'a Buffer<T>) -> BufferSlice<'a, T> {
        b.as_slice()
    }
}

impl<'a, T: ?Sized> From<&'a mut Buffer<T>> for BufferSlice<'a, T> where T: Content + 'a {
    #[inline]
    fn from(b: &'a mut Buffer<T>) -> BufferSlice<'a, T> {
        b.as_slice()
    }
}

impl<'a, T: ?Sized> BufferSliceExt<'a> for BufferSlice<'a, T> where T: Content {
    #[inline]
    fn add_fence(&self) -> Option<Inserter<'a>> {
        if !self.alloc.uses_persistent_mapping() {
            return None;
        }

        Some(self.fence.inserter(self.bytes_start .. self.bytes_end))
    }
}

impl<'a, T: ?Sized> BufferExt for BufferSlice<'a, T> where T: Content {
    #[inline]
    fn get_offset_bytes(&self) -> usize {
        self.bytes_start
    }

    #[inline]
    fn prepare_for_vertex_attrib_array(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_for_vertex_attrib_array(ctxt);
    }

    #[inline]
    fn prepare_for_element_array(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_for_element_array(ctxt);
    }

    #[inline]
    fn bind_to_element_array(&self, ctxt: &mut CommandContext) {
        self.alloc.bind_to_element_array(ctxt);
    }

    #[inline]
    fn prepare_and_bind_for_pixel_pack(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_pixel_pack(ctxt);
    }

    #[inline]
    fn unbind_pixel_pack(ctxt: &mut CommandContext) {
        Alloc::unbind_pixel_pack(ctxt)
    }

    #[inline]
    fn prepare_and_bind_for_pixel_unpack(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_pixel_unpack(ctxt);
    }

    #[inline]
    fn unbind_pixel_unpack(ctxt: &mut CommandContext) {
        Alloc::unbind_pixel_unpack(ctxt)
    }

    #[inline]
    fn prepare_and_bind_for_query(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_query(ctxt);
    }

    #[inline]
    fn unbind_query(ctxt: &mut CommandContext) {
        Alloc::unbind_query(ctxt)
    }

    #[inline]
    fn prepare_and_bind_for_draw_indirect(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_draw_indirect(ctxt);
    }

    #[inline]
    fn prepare_and_bind_for_dispatch_indirect(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_dispatch_indirect(ctxt);
    }

    #[inline]
    fn prepare_and_bind_for_uniform(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        self.alloc.prepare_and_bind_for_uniform(ctxt, index, 0 .. self.alloc.get_size());
    }

    #[inline]
    fn prepare_and_bind_for_shared_storage(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        self.alloc.prepare_and_bind_for_shared_storage(ctxt, index, 0 .. self.alloc.get_size());
    }

    #[inline]
    fn bind_to_transform_feedback(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        self.alloc.bind_to_transform_feedback(ctxt, index, 0 .. self.alloc.get_size());
    }
}

/// Represents a sub-part of a buffer.
pub struct BufferMutSlice<'a, T: ?Sized> where T: Content {
    alloc: &'a mut Alloc,
    bytes_start: usize,
    bytes_end: usize,
    fence: &'a Fences,
    marker: PhantomData<T>,
}

impl<'a, T: ?Sized> BufferMutSlice<'a, T> where T: Content + 'a {
    /// Returns the size in bytes of this slice.
    #[inline]
    pub fn get_size(&self) -> usize {
        self.bytes_end - self.bytes_start
    }

    /// Maps the buffer in memory for both reading and writing.
    ///
    /// # Implementation
    ///
    /// - For persistent-mapped buffers, waits until the data is no longer accessed by the GPU then
    ///   returns a pointer to the existing mapping.
    /// - For immutable buffers, creates a temporary buffer containing the data of the buffer and
    ///   maps it. When the mapping object is destroyed, copies the content of the temporary buffer
    ///   to the real buffer.
    /// - For other types, calls `glMapBuffer` or `glMapSubBuffer`.
    ///
    #[inline]
    pub fn map(self) -> Mapping<'a, T> {
        self.fence.wait(&mut self.alloc.get_context().make_current(),
                        self.bytes_start .. self.bytes_end);
        unsafe { self.alloc.map(self.bytes_start .. self.bytes_end) }
    }

    /// Maps the buffer in memory for reading.
    ///
    /// # Implementation
    ///
    /// - For persistent-mapped buffers, waits until the data is no longer accessed by the GPU then
    ///   returns a pointer to the existing mapping.
    /// - For immutable buffers, creates a temporary buffer containing the data of the buffer and
    ///   maps it.
    /// - For other types, calls `glMapBuffer` or `glMapSubBuffer`.
    ///
    #[inline]
    pub fn map_read(self) -> ReadMapping<'a, T> {
        self.fence.wait(&mut self.alloc.get_context().make_current(),
                        self.bytes_start .. self.bytes_end);
        unsafe { self.alloc.map_read(self.bytes_start .. self.bytes_end) }
    }

    /// Maps the buffer in memory for writing only.
    ///
    /// # Implementation
    ///
    /// - For persistent-mapped buffers, waits until the data is no longer accessed by the GPU then
    ///   returns a pointer to the existing mapping.
    /// - For immutable buffers, creates a temporary buffer and maps it. When the mapping object
    ///   is destroyed, copies the content of the temporary buffer to the real buffer.
    /// - For other types, calls `glMapBuffer` or `glMapSubBuffer`.
    ///
    #[inline]
    pub fn map_write(self) -> WriteMapping<'a, T> {
        self.fence.wait(&mut self.alloc.get_context().make_current(),
                        self.bytes_start .. self.bytes_end);
        unsafe { self.alloc.map_write(self.bytes_start .. self.bytes_end) }
    }

    /// Uploads some data in this buffer.
    ///
    /// # Implementation
    ///
    /// - For persistent-mapped buffers, waits untils the data is no longer used by the GPU then
    ///   memcpies the data to the mapping.
    /// - For immutable buffers, creates a temporary buffer that contains the data then calls
    ///   `glCopyBufferSubData` to copy from the temporary buffer to the real one.
    /// - For other types, calls `glBufferSubData`.
    ///
    /// # Panic
    ///
    /// Panics if the length of `data` is different from the length of this buffer.
    #[inline]
    pub fn write(&self, data: &T) {
        self.fence.wait(&mut self.alloc.get_context().make_current(),
                        self.bytes_start .. self.bytes_end);
        unsafe { self.alloc.upload(self.bytes_start, data); }
    }

    /// Invalidates the content of the slice. The data becomes undefined.
    ///
    /// This operation is a no-op if the backend doesn't support it and for persistent-mapped
    /// buffers.
    ///
    /// # Implementation
    ///
    /// Calls `glInvalidateBufferSubData` if supported.
    ///
    #[inline]
    pub fn invalidate(&self) {
        self.alloc.invalidate(self.bytes_start, self.get_size());
    }

    /// Reads the content of the buffer.
    #[inline]
    pub fn read(&self) -> Result<T::Owned, ReadError> {
        unsafe {
            self.alloc.read::<T>(self.bytes_start .. self.bytes_end)
        }
    }

    /// Copies the content of this slice to another slice.
    ///
    /// # Panic
    ///
    /// Panics if `T` is unsized and the other buffer is too small.
    pub fn copy_to<S>(&self, target: S) -> Result<(), CopyError>
                      where S: Into<BufferSlice<'a, T>>
    {
        let target = target.into();

        self.alloc.copy_to(self.bytes_start .. self.bytes_end, &target.alloc,
                           target.get_offset_bytes())?;

        if let Some(inserter) = self.add_fence() {
            let mut ctxt = self.alloc.get_context().make_current();
            inserter.insert(&mut ctxt);
        }

        if let Some(inserter) = self.add_fence() {
            let mut ctxt = self.alloc.get_context().make_current();
            inserter.insert(&mut ctxt);
        }

        Ok(())
    }

    /// Builds a slice that contains an element from inside the buffer.
    ///
    /// This method builds an object that represents a slice of the buffer. No actual operation
    /// OpenGL is performed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[derive(Copy, Clone)]
    /// struct BufferContent {
    ///     value1: u16,
    ///     value2: u16,
    /// }
    /// # let buffer: glium::buffer::BufferSlice<BufferContent> =
    /// #                                                   unsafe { std::mem::uninitialized() };
    /// let slice = unsafe { buffer.slice_custom(|content| &content.value2) };
    /// ```
    ///
    /// # Safety
    ///
    /// The object whose reference is passed to the closure is uninitialized. Therefore you
    /// **must not** access the content of the object.
    ///
    /// You **must** return a reference to an element from the parameter. The closure **must not**
    /// panic.
    #[inline]
    pub unsafe fn slice_custom<F, R: ?Sized>(self, f: F) -> BufferMutSlice<'a, R>
                                             where F: for<'r> FnOnce(&'r T) -> &'r R,
                                                   R: Content
    {
        let data: &T = mem::zeroed();
        let result = f(data);
        let size = mem::size_of_val(result);
        let result = result as *const R as *const () as usize;

        assert!(result <= self.get_size());
        assert!(result + size <= self.get_size());

        BufferMutSlice {
            alloc: self.alloc,
            bytes_start: self.bytes_start + result,
            bytes_end: self.bytes_start + result + size,
            fence: self.fence,
            marker: PhantomData,
        }
    }

    /// Builds a slice-any containing the whole subbuffer.
    ///
    /// This method builds an object that represents a slice of the buffer. No actual operation
    /// OpenGL is performed.
    #[inline]
    pub fn as_slice_any(self) -> BufferAnySlice<'a> {
        BufferAnySlice {
            alloc: self.alloc,
            bytes_start: self.bytes_start,
            bytes_end: self.bytes_end,
            elements_size: <T as Content>::get_elements_size(),
            fence: self.fence,
        }
    }
}

impl<'a, T> BufferMutSlice<'a, [T]> where [T]: Content, T: Copy + 'a {
    /// Returns the number of elements in this slice.
    #[inline]
    pub fn len(&self) -> usize {
        (self.bytes_end - self.bytes_start) / mem::size_of::<T>()
    }

    /// Builds a subslice of this slice. Returns `None` if out of range.
    ///
    /// This method builds an object that represents a slice of the buffer. No actual operation
    /// OpenGL is performed.
    #[inline]
    pub fn slice<R: RangeArgument<usize>>(self, range: R) -> Option<BufferMutSlice<'a, [T]>> {
        if range.start().map_or(0, |e| *e) > self.len() || range.end().map_or(0, |e| *e) > self.len() {
            return None;
        }

        let len = self.len();
        Some(BufferMutSlice {
            alloc: self.alloc,
            bytes_start: self.bytes_start + range.start().map_or(0, |e| *e) * mem::size_of::<T>(),
            bytes_end: self.bytes_start + range.end().map_or(len, |e| *e) * mem::size_of::<T>(),
            fence: self.fence,
            marker: PhantomData,
        })
    }
}

impl<'a, T> BufferMutSlice<'a, [T]> where T: PixelValue + 'a {
    /// Reads the content of the buffer.
    #[inline]
    pub fn read_as_texture_1d<S>(&self) -> Result<S, ReadError> where S: Texture1dDataSink<T> {
        let data = self.read()?;
        Ok(S::from_raw(Cow::Owned(data), self.len() as u32))
    }
}

impl<'a, T: ?Sized> BufferSliceExt<'a> for BufferMutSlice<'a, T> where T: Content {
    #[inline]
    fn add_fence(&self) -> Option<Inserter<'a>> {
        if !self.alloc.uses_persistent_mapping() {
            return None;
        }

        Some(self.fence.inserter(self.bytes_start .. self.bytes_end))
    }
}

impl<'a, T: ?Sized> fmt::Debug for BufferMutSlice<'a, T> where T: Content {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self.alloc)
    }
}

impl<'a, T: ?Sized> From<&'a mut Buffer<T>> for BufferMutSlice<'a, T> where T: Content + 'a {
    #[inline]
    fn from(b: &'a mut Buffer<T>) -> BufferMutSlice<'a, T> {
        b.as_mut_slice()
    }
}

/// Represents a sub-part of a buffer.
///
/// Doesn't contain any information about the content, contrary to `Buffer`.
pub struct BufferAny {
    alloc: Alloc,
    size: usize,
    elements_size: usize,
    fence: Fences,
}

impl BufferAny {
    /// Builds a slice-any containing the whole subbuffer.
    #[inline]
    pub fn as_slice_any(&self) -> BufferAnySlice {
        BufferAnySlice {
            alloc: &self.alloc,
            bytes_start: 0,
            bytes_end: self.size,
            elements_size: self.elements_size,
            fence: &self.fence,
        }
    }

    /// Builds a mutable typed slice containing the whole subbuffer, without checking the type.
    #[inline]
    pub unsafe fn as_typed_slice_mut<T: ?Sized + Content>(&mut self) -> BufferMutSlice<T> {
        assert_eq!(<T as Content>::get_elements_size(), self.elements_size);
        BufferMutSlice {
            alloc: &mut self.alloc,
            bytes_start: 0,
            bytes_end: self.size,
            fence: &self.fence,
            marker: PhantomData,
        }
    }

    /// Builds a typed slice containing the whole subbuffer, without checking the type.
    #[inline]
    pub unsafe fn as_typed_slice<T: ?Sized + Content>(&self) -> BufferSlice<T> {
        assert_eq!(<T as Content>::get_elements_size(), self.elements_size);
        BufferSlice {
            alloc: &self.alloc,
            bytes_start: 0,
            bytes_end: self.size,
            fence: &self.fence,
            marker: PhantomData,
        }
    }

    /// Returns the size in bytes of each element in the buffer.
    // TODO: clumsy, remove this function
    #[inline]
    pub fn get_elements_size(&self) -> usize {
        self.elements_size
    }

    /// Returns the number of elements in the buffer.
    // TODO: clumsy, remove this function
    #[inline]
    pub fn get_elements_count(&self) -> usize {
        self.size / self.elements_size
    }

    /// Returns the context corresponding to this buffer.
    #[inline]
    pub fn get_context(&self) -> &Rc<Context> {
        self.alloc.get_context()
    }

    /// Returns the number of bytes in this subbuffer.
    #[inline]
    pub fn get_size(&self) -> usize {
        self.size
    }

    /// Invalidates the content of the buffer. The data becomes undefined.
    ///
    /// This operation is a no-op if the backend doesn't support it and for persistent-mapped
    /// buffers.
    #[inline]
    pub fn invalidate(&self) {
        self.alloc.invalidate(0, self.size);
    }

    /// UNSTABLE. This function can be removed at any moment without any further notice.
    ///
    /// Considers that the buffer is filled with elements of type `T` and reads them.
    ///
    /// # Panic
    ///
    /// Panics if the size of the buffer is not a multiple of the size of the data.
    /// For example, trying to read some `(u8, u8, u8, u8)`s from a buffer of 7 bytes will panic.
    ///
    #[inline]
    pub unsafe fn read<T>(&self) -> Result<T::Owned, ReadError> where T: Content {
        // TODO: add check
        self.fence.wait(&mut self.alloc.get_context().make_current(), 0 .. self.get_size());
        self.alloc.read::<T>(0 .. self.get_size())
    }
}

impl<T: ?Sized> From<Buffer<T>> for BufferAny where T: Content + Send + 'static {
    #[inline]
    fn from(mut buffer: Buffer<T>) -> BufferAny {
        let size = buffer.get_size();

        BufferAny {
            alloc: buffer.alloc.take().unwrap(),
            size: size,
            elements_size: <T as Content>::get_elements_size(),
            fence: buffer.fence.take().unwrap(),
        }
    }
}

impl Drop for BufferAny {
    #[inline]
    fn drop(&mut self) {
        self.fence.clean(&mut self.alloc.get_context().make_current());
    }
}

impl fmt::Debug for BufferAny {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self.alloc)
    }
}

impl BufferExt for BufferAny {
    #[inline]
    fn get_offset_bytes(&self) -> usize {
        0
    }

    #[inline]
    fn prepare_for_vertex_attrib_array(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_for_vertex_attrib_array(ctxt);
    }

    #[inline]
    fn prepare_for_element_array(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_for_element_array(ctxt);
    }

    #[inline]
    fn bind_to_element_array(&self, ctxt: &mut CommandContext) {
        self.alloc.bind_to_element_array(ctxt);
    }

    #[inline]
    fn prepare_and_bind_for_pixel_pack(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_pixel_pack(ctxt);
    }

    #[inline]
    fn unbind_pixel_pack(ctxt: &mut CommandContext) {
        Alloc::unbind_pixel_pack(ctxt)
    }

    #[inline]
    fn prepare_and_bind_for_pixel_unpack(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_pixel_unpack(ctxt);
    }

    #[inline]
    fn unbind_pixel_unpack(ctxt: &mut CommandContext) {
        Alloc::unbind_pixel_unpack(ctxt)
    }

    #[inline]
    fn prepare_and_bind_for_query(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_query(ctxt);
    }

    #[inline]
    fn unbind_query(ctxt: &mut CommandContext) {
        Alloc::unbind_query(ctxt)
    }

    #[inline]
    fn prepare_and_bind_for_draw_indirect(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_draw_indirect(ctxt);
    }

    #[inline]
    fn prepare_and_bind_for_dispatch_indirect(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_dispatch_indirect(ctxt);
    }

    #[inline]
    fn prepare_and_bind_for_uniform(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        self.alloc.prepare_and_bind_for_uniform(ctxt, index, 0 .. self.alloc.get_size());
    }

    #[inline]
    fn prepare_and_bind_for_shared_storage(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        self.alloc.prepare_and_bind_for_shared_storage(ctxt, index, 0 .. self.alloc.get_size());
    }

    #[inline]
    fn bind_to_transform_feedback(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        self.alloc.bind_to_transform_feedback(ctxt, index, 0 .. self.alloc.get_size());
    }
}

/// Slice of a `Buffer` without any type info.
#[derive(Copy, Clone)]
pub struct BufferAnySlice<'a> {
    alloc: &'a Alloc,
    bytes_start: usize,
    bytes_end: usize,
    elements_size: usize,
    fence: &'a Fences,
}

impl<'a> GlObject for BufferAnySlice<'a> {
    type Id = gl::types::GLuint;

    #[inline]
    fn get_id(&self) -> gl::types::GLuint {
        self.alloc.get_id()
    }
}

impl<'a> BufferAnySlice<'a> {
    /// Returns the number of bytes in this slice.
    #[inline]
    pub fn get_size(&self) -> usize {
        self.bytes_end - self.bytes_start
    }

    /// Returns the size in bytes of each element in the buffer.
    // TODO: clumsy, remove this function
    #[inline]
    pub fn get_elements_size(&self) -> usize {
        self.elements_size
    }

    /// Returns the number of elements in the buffer.
    // TODO: clumsy, remove this function
    #[inline]
    pub fn get_elements_count(&self) -> usize {
        self.get_size() / self.elements_size
    }

    /// Invalidates the content of the slice. The data becomes undefined.
    ///
    /// This operation is a no-op if the backend doesn't support it and for persistent-mapped
    /// buffers.
    #[inline]
    pub fn invalidate(&self) {
        self.alloc.invalidate(self.bytes_start, self.get_size());
    }

    /// Returns the context corresponding to this buffer.
    #[inline]
    pub fn get_context(&self) -> &Rc<Context> {
        self.alloc.get_context()
    }
}

impl<'a> fmt::Debug for BufferAnySlice<'a> {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self.alloc)
    }
}

impl<'a> BufferSliceExt<'a> for BufferAnySlice<'a> {
    #[inline]
    fn add_fence(&self) -> Option<Inserter<'a>> {
        if !self.alloc.uses_persistent_mapping() {
            return None;
        }

        Some(self.fence.inserter(self.bytes_start .. self.bytes_end))
    }
}

impl<'a> BufferExt for BufferAnySlice<'a> {
    #[inline]
    fn get_offset_bytes(&self) -> usize {
        self.bytes_start
    }

    #[inline]
    fn prepare_for_vertex_attrib_array(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_for_vertex_attrib_array(ctxt);
    }

    #[inline]
    fn prepare_for_element_array(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_for_element_array(ctxt);
    }

    #[inline]
    fn bind_to_element_array(&self, ctxt: &mut CommandContext) {
        self.alloc.bind_to_element_array(ctxt);
    }

    #[inline]
    fn prepare_and_bind_for_pixel_pack(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_pixel_pack(ctxt);
    }

    #[inline]
    fn unbind_pixel_pack(ctxt: &mut CommandContext) {
        Alloc::unbind_pixel_pack(ctxt)
    }

    #[inline]
    fn prepare_and_bind_for_pixel_unpack(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_pixel_unpack(ctxt);
    }

    #[inline]
    fn unbind_pixel_unpack(ctxt: &mut CommandContext) {
        Alloc::unbind_pixel_unpack(ctxt)
    }

    #[inline]
    fn prepare_and_bind_for_query(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_query(ctxt);
    }

    #[inline]
    fn unbind_query(ctxt: &mut CommandContext) {
        Alloc::unbind_query(ctxt)
    }

    #[inline]
    fn prepare_and_bind_for_draw_indirect(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_draw_indirect(ctxt);
    }

    #[inline]
    fn prepare_and_bind_for_dispatch_indirect(&self, ctxt: &mut CommandContext) {
        self.alloc.prepare_and_bind_for_dispatch_indirect(ctxt);
    }

    #[inline]
    fn prepare_and_bind_for_uniform(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        self.alloc.prepare_and_bind_for_uniform(ctxt, index, 0 .. self.alloc.get_size());
    }

    #[inline]
    fn prepare_and_bind_for_shared_storage(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        self.alloc.prepare_and_bind_for_shared_storage(ctxt, index, 0 .. self.alloc.get_size());
    }

    #[inline]
    fn bind_to_transform_feedback(&self, ctxt: &mut CommandContext, index: gl::types::GLuint) {
        self.alloc.bind_to_transform_feedback(ctxt, index, 0 .. self.alloc.get_size());
    }
}
