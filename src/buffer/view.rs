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
use buffer::CopyError;
use buffer::CopyTo;
use buffer::Content;
use buffer::ArrayContent;
use buffer::Storage;
use buffer::Create;
use buffer::Invalidate;
use buffer::Read;
use buffer::Write;
use buffer::SliceMut;
use buffer::Slice;
use buffer::SliceCustomMut;
use buffer::SliceCustom;
use buffer::Map;
use buffer::fences::Fences;
use buffer::fences::Inserter;
use buffer::alloc::Alloc;
use buffer::alloc::Mapping;
use buffer::alloc::ReadMapping;
use buffer::alloc::WriteMapping;
use buffer::alloc::ReadError;

pub use self::DynamicBuffer as Buffer;
pub use self::DynamicBufferSlice as BufferSlice;
pub use self::DynamicBufferMutSlice as BufferMutSlice;

/// Represents a view of a buffer.
pub struct DynamicBuffer<T: ?Sized> where T: Content {
    // TODO: this `Option` is here because we have a destructor and need to be able to move out
    alloc: Option<Alloc>,
    marker: PhantomData<T>,
}

/// Represents a view of a buffer.
pub struct ImmutableBuffer<T: ?Sized> where T: Content {
    // TODO: this `Option` is here because we have a destructor and need to be able to move out
    alloc: Option<Alloc>,
    marker: PhantomData<T>,
}

/// Represents a view of a buffer.
pub struct PersistentBuffer<T: ?Sized> where T: Content {
    // TODO: this `Option` is here because we have a destructor and need to be able to move out
    alloc: Option<Alloc>,
    marker: PhantomData<T>,
}

macro_rules! impl_buffer_base {
    ($ty:ident, $slice_ty:ident, $slice_mut_ty:ident) => (
        impl<T: ?Sized> $ty<T> where T: Content {
            /// Builds a new buffer containing the given data. The size of the buffer is equal to the size
            /// of the data.
            pub fn new<F>(facade: &F, data: &T, ty: BufferType, mode: BufferMode)
                          -> Result<$ty<T>, BufferCreationError>
                          where F: Facade
            {
                Alloc::new(facade, data, ty, mode)
                    .map(|buffer| {
                        $ty {
                            alloc: Some(buffer),
                            marker: PhantomData,
                        }
                    })
            }

            /// Builds a new buffer of the given size.
            pub fn empty_unsized<F>(facade: &F, ty: BufferType, size: usize, mode: BufferMode)
                                    -> Result<$ty<T>, BufferCreationError> where F: Facade
            {
                assert!(<T as Content>::is_size_suitable(size));

                Alloc::empty(facade, ty, size, mode)
                    .map(|buffer| {
                        $ty {
                            alloc: Some(buffer),
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

            /// Copies the content of the buffer to another buffer.
            ///
            /// # Panic
            ///
            /// Panics if `T` is unsized and the other buffer is too small.
            ///
            pub fn copy_to<'a, S>(&self, target: S) -> Result<(), CopyError>
                                  where S: Into<$slice_ty<'a, T>>, T: 'a
            {
                let target = target.into();
                let alloc = self.alloc.as_ref().unwrap();

                try!(alloc.copy_to(0 .. self.get_size(), &target.alloc, target.get_offset_bytes()));

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
            pub unsafe fn slice_custom<F, R: ?Sized>(&self, f: F) -> $slice_ty<R>
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
            pub unsafe fn slice_custom_mut<F, R: ?Sized>(&mut self, f: F) -> $slice_mut_ty<R>
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
            pub fn as_slice(&self) -> $slice_ty<T> {
                $slice_ty {
                    alloc: self.alloc.as_ref().unwrap(),
                    bytes_start: 0,
                    bytes_end: self.get_size(),
                    marker: PhantomData,
                }
            }

            /// Builds a slice containing the whole subbuffer.
            ///
            /// This method builds an object that represents a slice of the buffer. No actual operation
            /// OpenGL is performed.
            #[inline]
            pub fn as_mut_slice(&mut self) -> $slice_mut_ty<T> {
                let size = self.get_size();

                $slice_mut_ty {
                    alloc: self.alloc.as_mut().unwrap(),
                    bytes_start: 0,
                    bytes_end: size,
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
                }
            }
        }

        impl<T: ?Sized> Storage for $ty<T> where T: Content {
            type Content = T;

            #[inline]
            fn size(&self) -> usize {
                self.get_size()
            }

            #[inline]
            fn as_slice_any(&self) -> BufferAnySlice {
                self.as_slice_any()
            }
        }

        impl<'a, T: ?Sized> Storage for &'a $ty<T> where T: Content {
            type Content = T;

            #[inline]
            fn size(&self) -> usize {
                self.get_size()
            }

            #[inline]
            fn as_slice_any(&self) -> BufferAnySlice {
                self.as_slice_any()
            }
        }

        impl<'a, T: ?Sized> Storage for &'a mut $ty<T> where T: Content {
            type Content = T;

            #[inline]
            fn size(&self) -> usize {
                self.get_size()
            }

            #[inline]
            fn as_slice_any(&self) -> BufferAnySlice {
                self.as_slice_any()
            }
        }

        impl<'a, T: ?Sized> Storage for $slice_ty<'a, T> where T: Content {
            type Content = T;

            #[inline]
            fn size(&self) -> usize {
                self.get_size()
            }

            #[inline]
            fn as_slice_any(&self) -> BufferAnySlice {
                self.as_slice_any()
            }
        }

        impl<'a, T: ?Sized> Storage for $slice_mut_ty<'a, T> where T: Content {
            type Content = T;

            #[inline]
            fn size(&self) -> usize {
                self.get_size()
            }

            #[inline]
            fn as_slice_any(&self) -> BufferAnySlice {
                self.as_slice_any()
            }
        }

        impl<T: ?Sized> Create for $ty<T> where T: Content {
            #[inline]
            fn new<F>(facade: &F, data: &Self::Content, ty: BufferType)
                      -> Result<Self, BufferCreationError>
                where F: Facade
            {
                $ty::new(facade, data, ty, BufferMode::Default)     // TODO: remove buffer mode
            }

            fn empty<F>(facade: &F, ty: BufferType)
                        -> Result<Self, BufferCreationError>
                where F: Facade, Self: Sized, Self::Content: Copy
            {
                $ty::empty(facade, ty, BufferMode::Default)     // TODO: remove buffer mode
            }

            fn empty_array<F>(facade: &F, len: usize, ty: BufferType)
                              -> Result<Self, BufferCreationError>
                where F: Facade, Self: Sized, Self::Content: ArrayContent
            {
                Alloc::empty(facade, ty, len * <Self::Content as ArrayContent>::element_size(), BufferMode::Default)
                    .map(|buffer| {
                        $ty {
                            alloc: Some(buffer),
                            marker: PhantomData,
                        }
                    })
            }

            fn empty_unsized<F>(facade: &F, size: usize, ty: BufferType)
                                -> Result<Self, BufferCreationError>
                where F: Facade, Self: Sized, Self::Content: Copy
            {
                $ty::empty_unsized(facade, ty, size, BufferMode::Default)     // TODO: remove buffer mode
            }
        }

        impl<T> CopyTo for $ty<T> where T: Content {
            fn copy_to<S>(&self, target: &S) -> Result<(), CopyError>
                where S: Storage
            {
                let target = target.as_slice_any();
                let alloc = self.alloc.as_ref().unwrap();

                try!(alloc.copy_to(0 .. self.get_size(), &target.alloc, target.get_offset_bytes()));

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
        }

        impl<'a, T> Slice for &'a $ty<T> where T: Content {
            type Slice = $slice_ty<'a, T>;

            fn as_slice(self) -> $slice_ty<'a, T> {
                self.as_slice()
            }

            fn slice<R: RangeArgument<usize>>(self, range: R) -> Option<$slice_ty<'a, T>>
                where T: ArrayContent
            {
                self.slice(range)
            }
        }

        impl<'a, T> Slice for &'a mut $ty<T> where T: Content {
            type Slice = $slice_ty<'a, T>;

            fn as_slice(self) -> $slice_ty<'a, T> {
                self.as_slice()
            }

            fn slice<R: RangeArgument<usize>>(self, range: R) -> Option<$slice_ty<'a, T>>
                where T: ArrayContent
            {
                self.slice(range)
            }
        }

        impl<'a, T> SliceMut for &'a mut $ty<T> where T: Content {
            type SliceMut = $slice_mut_ty<'a, T>;

            fn as_mut_slice(self) -> $slice_mut_ty<'a, T> {
                self.as_mut_slice()
            }

            fn slice_mut<R: RangeArgument<usize>>(self, range: R) -> Option<$slice_mut_ty<'a, T>>
                where T: ArrayContent
            {
                self.slice_mut(range)
            }
        }

        impl<'a, T, R: ?Sized> SliceCustom<R> for &'a $ty<T>
            where T: Content, R: Content
        {
            type Slice = $slice_ty<'a, R>;

            #[inline]
            unsafe fn slice_custom<F>(self, f: F) -> Self::Slice
                where F: for<'r> FnOnce(&'r Self::Content) -> &'r R, Self: Sized
            {
                self.slice_custom(f)
            }
        }

        impl<'a, T, R: ?Sized> SliceCustom<R> for &'a mut $ty<T>
            where T: Content, R: Content
        {
            type Slice = $slice_ty<'a, R>;

            #[inline]
            unsafe fn slice_custom<F>(self, f: F) -> Self::Slice
                where F: for<'r> FnOnce(&'r Self::Content) -> &'r R, Self: Sized
            {
                self.slice_custom(f)
            }
        }

        impl<'a, T, R: ?Sized> SliceCustomMut<R> for &'a mut $ty<T>
            where T: Content, R: Content
        {
            type SliceMut = $slice_mut_ty<'a, R>;

            #[inline]
            unsafe fn slice_custom_mut<F>(self, f: F) -> Self::SliceMut
                where F: for<'r> FnOnce(&'r Self::Content) -> &'r R, Self: Sized
            {
                self.slice_custom_mut(f)
            }
        }

        impl<'a, T: ?Sized> From<&'a $ty<T>> for BufferAnySlice<'a> where T: Content {
            #[inline]
            fn from(buf: &'a $ty<T>) -> BufferAnySlice<'a> {
                buf.as_slice_any()
            }
        }

        impl<'a, T: ?Sized> From<$slice_ty<'a, T>> for BufferAnySlice<'a> where T: Content {
            #[inline]
            fn from(buf: $slice_ty<'a, T>) -> BufferAnySlice<'a> {
                buf.as_slice_any()
            }
        }

        impl<T> $ty<T> where T: Content + Copy {
            /// Builds a new buffer of the given size.
            pub fn empty<F>(facade: &F, ty: BufferType, mode: BufferMode)
                            -> Result<$ty<T>, BufferCreationError> where F: Facade
            {
                Alloc::empty(facade, ty, mem::size_of::<T>(), mode)
                    .map(|buffer| {
                        $ty {
                            alloc: Some(buffer),
                            marker: PhantomData,
                        }
                    })
            }
        }

        impl<T> $ty<[T]> where [T]: Content, T: Copy {
            /// Builds a new buffer of the given size.
            pub fn empty_array<F>(facade: &F, ty: BufferType, len: usize, mode: BufferMode)
                                  -> Result<$ty<[T]>, BufferCreationError> where F: Facade
            {
                Alloc::empty(facade, ty, len * mem::size_of::<T>(), mode)
                    .map(|buffer| {
                        $ty {
                            alloc: Some(buffer),
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
            pub fn slice<R: RangeArgument<usize>>(&self, range: R) -> Option<$slice_ty<[T]>> {
                self.as_slice().slice(range)
            }

            /// Builds a slice of this subbuffer. Returns `None` if out of range.
            ///
            /// This method builds an object that represents a slice of the buffer. No actual operation
            /// OpenGL is performed.
            #[inline]
            pub fn slice_mut<R: RangeArgument<usize>>(&mut self, range: R) -> Option<$slice_mut_ty<[T]>> {
                self.as_mut_slice().slice(range)
            }
        }

        impl<T: ?Sized> fmt::Debug for $ty<T> where T: Content {
            #[inline]
            fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                write!(fmt, "{:?}", self.alloc.as_ref().unwrap())
            }
        }

        impl<T: ?Sized> BufferExt for $ty<T> where T: Content {
            #[inline]
            fn get_offset_bytes(&self) -> usize {
                0
            }

            #[inline]
            fn get_buffer_id(&self) -> gl::types::GLuint {
                let alloc = self.alloc.as_ref().unwrap();
                alloc.get_id()
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
        pub struct $slice_ty<'a, T: ?Sized> where T: Content + 'a {
            alloc: &'a Alloc,
            bytes_start: usize,
            bytes_end: usize,
            marker: PhantomData<&'a T>,
        }

        impl<'a, T: ?Sized> $slice_ty<'a, T> where T: Content + 'a {
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

            /// Copies the content of this slice to another slice.
            ///
            /// # Panic
            ///
            /// Panics if `T` is unsized and the other buffer is too small.
            pub fn copy_to<S>(&self, target: S) -> Result<(), CopyError>
                              where S: Into<$slice_ty<'a, T>>
            {
                let target = target.into();

                try!(self.alloc.copy_to(self.bytes_start .. self.bytes_end, &target.alloc,
                                        target.get_offset_bytes()));

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
            pub unsafe fn slice_custom<F, R: ?Sized>(&self, f: F) -> $slice_ty<'a, R>
                                                     where F: for<'r> FnOnce(&'r T) -> &'r R,
                                                           R: Content
            {
                let data: &T = mem::zeroed();
                let result = f(data);
                let size = mem::size_of_val(result);
                let result = result as *const R as *const () as usize;

                assert!(result <= self.get_size());
                assert!(result + size <= self.get_size());

                $slice_ty {
                    alloc: self.alloc,
                    bytes_start: self.bytes_start + result,
                    bytes_end: self.bytes_start + result + size,
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
                }
            }
        }

        impl<'a, T> $slice_ty<'a, [T]> where [T]: Content + 'a {
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
            pub fn slice<R: RangeArgument<usize>>(&self, range: R) -> Option<$slice_ty<'a, [T]>> {
                if range.start().map_or(0, |e| *e) > self.len() || range.end().map_or(0, |e| *e) > self.len() {
                    return None;
                }

                Some($slice_ty {
                    alloc: self.alloc,
                    bytes_start: self.bytes_start + range.start().map_or(0, |e| *e) * mem::size_of::<T>(),
                    bytes_end: self.bytes_start + range.end().map_or(self.len(), |e| *e) * mem::size_of::<T>(),
                    marker: PhantomData,
                })
            }
        }

        impl<'a, T: ?Sized> fmt::Debug for $slice_ty<'a, T> where T: Content {
            #[inline]
            fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                write!(fmt, "{:?}", self.alloc)
            }
        }

        impl<'a, T: ?Sized> From<$slice_mut_ty<'a, T>> for $slice_ty<'a, T> where T: Content + 'a {
            #[inline]
            fn from(s: $slice_mut_ty<'a, T>) -> $slice_ty<'a, T> {
                $slice_ty {
                    alloc: s.alloc,
                    bytes_start: s.bytes_start,
                    bytes_end: s.bytes_end,
                    marker: PhantomData,
                }
            }
        }

        impl<'a, T: ?Sized> From<&'a $ty<T>> for $slice_ty<'a, T> where T: Content + 'a {
            #[inline]
            fn from(b: &'a $ty<T>) -> $slice_ty<'a, T> {
                b.as_slice()
            }
        }

        impl<'a, T: ?Sized> From<&'a mut $ty<T>> for $slice_ty<'a, T> where T: Content + 'a {
            #[inline]
            fn from(b: &'a mut $ty<T>) -> $slice_ty<'a, T> {
                b.as_slice()
            }
        }

        impl<'a, T: ?Sized> BufferSliceExt<'a> for $slice_ty<'a, T> where T: Content {
            #[inline]
            fn add_fence(&self) -> Option<Inserter<'a>> {
                unimplemented!()
            }
        }

        impl<'a, T: ?Sized> BufferExt for $slice_ty<'a, T> where T: Content {
            #[inline]
            fn get_offset_bytes(&self) -> usize {
                self.bytes_start
            }

            #[inline]
            fn get_buffer_id(&self) -> gl::types::GLuint {
                self.alloc.get_id()
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
        pub struct $slice_mut_ty<'a, T: ?Sized> where T: Content {
            alloc: &'a mut Alloc,
            bytes_start: usize,
            bytes_end: usize,
            marker: PhantomData<T>,
        }

        impl<'a, T: ?Sized> $slice_mut_ty<'a, T> where T: Content + 'a {
            /// Returns the size in bytes of this slice.
            #[inline]
            pub fn get_size(&self) -> usize {
                self.bytes_end - self.bytes_start
            }

            /// Copies the content of this slice to another slice.
            ///
            /// # Panic
            ///
            /// Panics if `T` is unsized and the other buffer is too small.
            pub fn copy_to<S>(&self, target: S) -> Result<(), CopyError>
                              where S: Into<$slice_ty<'a, T>>
            {
                let target = target.into();

                try!(self.alloc.copy_to(self.bytes_start .. self.bytes_end, &target.alloc,
                                        target.get_offset_bytes()));

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
            pub unsafe fn slice_custom<F, R: ?Sized>(self, f: F) -> $slice_mut_ty<'a, R>
                                                     where F: for<'r> FnOnce(&'r T) -> &'r R,
                                                           R: Content
            {
                let data: &T = mem::zeroed();
                let result = f(data);
                let size = mem::size_of_val(result);
                let result = result as *const R as *const () as usize;

                assert!(result <= self.get_size());
                assert!(result + size <= self.get_size());

                $slice_mut_ty {
                    alloc: self.alloc,
                    bytes_start: self.bytes_start + result,
                    bytes_end: self.bytes_start + result + size,
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
                }
            }
        }

        impl<'a, T> $slice_mut_ty<'a, [T]> where [T]: Content, T: Copy + 'a {
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
            pub fn slice<R: RangeArgument<usize>>(self, range: R) -> Option<$slice_mut_ty<'a, [T]>> {
                if range.start().map_or(0, |e| *e) > self.len() || range.end().map_or(0, |e| *e) > self.len() {
                    return None;
                }

                let len = self.len();
                Some($slice_mut_ty {
                    alloc: self.alloc,
                    bytes_start: self.bytes_start + range.start().map_or(0, |e| *e) * mem::size_of::<T>(),
                    bytes_end: self.bytes_start + range.end().map_or(len, |e| *e) * mem::size_of::<T>(),
                    marker: PhantomData,
                })
            }
        }

        impl<'a, T: ?Sized> BufferSliceExt<'a> for $slice_mut_ty<'a, T> where T: Content {
            #[inline]
            fn add_fence(&self) -> Option<Inserter<'a>> {
                unimplemented!()
            }
        }

        impl<'a, T: ?Sized> fmt::Debug for $slice_mut_ty<'a, T> where T: Content {
            #[inline]
            fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                write!(fmt, "{:?}", self.alloc)
            }
        }

        impl<'a, T: ?Sized> From<&'a mut $ty<T>> for $slice_mut_ty<'a, T> where T: Content + 'a {
            #[inline]
            fn from(b: &'a mut $ty<T>) -> $slice_mut_ty<'a, T> {
                b.as_mut_slice()
            }
        }
    )
}

impl_buffer_base!(DynamicBuffer, DynamicBufferSlice, DynamicBufferMutSlice);
impl_buffer_base!(ImmutableBuffer, ImmutableBufferSlice, ImmutableBufferMutSlice);
impl_buffer_base!(PersistentBuffer, PersistentBufferSlice, PersistentBufferMutSlice);

impl<T: ?Sized> DynamicBuffer<T> where T: Content {
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
        unsafe { self.alloc.as_ref().unwrap().upload(0, data); }
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
        self.alloc.as_ref().unwrap().invalidate(0, self.get_size());
    }

    /// Reads the content of the buffer.
    pub fn read(&self) -> Result<T::Owned, ReadError> {
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
        let size = self.get_size();
        unsafe { self.alloc.as_mut().unwrap().map(0 .. size) }
    }
}

impl<T: ?Sized> DynamicBuffer<[T]> where T: Content + PixelValue {
    /// Reads the content of the buffer.
    #[inline]
    pub fn read_as_texture_1d<S>(&self) -> Result<S, ReadError> where S: Texture1dDataSink<T> {
        let data = try!(self.read());
        Ok(S::from_raw(Cow::Owned(data), self.len() as u32))
    }
}

impl<T: ?Sized> Invalidate for DynamicBuffer<T> where T: Content {
    #[inline]
    fn invalidate(&self) {
        self.invalidate()
    }
}

impl<T: ?Sized> Read for DynamicBuffer<T> where T: Content {
    #[inline]
    fn read(&self) -> Result<T::Owned, ReadError> {
        self.read()
    }
}

impl<T: ?Sized> Write for DynamicBuffer<T> where T: Content {
    #[inline]
    fn write(&self, data: &T) {
        self.write(data)
    }
}

impl<'a, T> Map for &'a mut DynamicBuffer<T> where T: Content {
    type Mapping = Mapping<'a, T>;

    fn map(self) -> Mapping<'a, T> {
        self.map()
    }
}

impl<'a, T: ?Sized> DynamicBufferSlice<'a, T> where T: Content {
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
        unsafe {
            self.alloc.read::<T>(self.bytes_start .. self.bytes_end)
        }
    }
}

impl<'a, T> DynamicBufferSlice<'a, [T]> where T: Content + PixelValue + 'a {
    /// Reads the content of the buffer.
    #[inline]
    pub fn read_as_texture_1d<S>(&self) -> Result<S, ReadError> where S: Texture1dDataSink<T> {
        let data = try!(self.read());
        Ok(S::from_raw(Cow::Owned(data), self.len() as u32))
    }
}

impl<'a, T: ?Sized> Invalidate for DynamicBufferSlice<'a, T> where T: Content {
    #[inline]
    fn invalidate(&self) {
        self.invalidate()
    }
}

impl<'a, T: ?Sized> Read for DynamicBufferSlice<'a, T> where T: Content {
    #[inline]
    fn read(&self) -> Result<T::Owned, ReadError> {
        self.read()
    }
}

impl<'a, T: ?Sized> Write for DynamicBufferSlice<'a, T> where T: Content {
    #[inline]
    fn write(&self, data: &T) {
        self.write(data)
    }
}

impl<'a, T: ?Sized> DynamicBufferMutSlice<'a, T> where T: Content {
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
        unsafe {
            self.alloc.read::<T>(self.bytes_start .. self.bytes_end)
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
        unsafe { self.alloc.map(self.bytes_start .. self.bytes_end) }
    }
}

impl<'a, T> DynamicBufferMutSlice<'a, [T]> where T: Content + PixelValue + 'a {
    /// Reads the content of the buffer.
    #[inline]
    pub fn read_as_texture_1d<S>(&self) -> Result<S, ReadError> where S: Texture1dDataSink<T> {
        let data = try!(self.read());
        Ok(S::from_raw(Cow::Owned(data), self.len() as u32))
    }
}

impl<'a, T: ?Sized> Invalidate for DynamicBufferMutSlice<'a, T> where T: Content {
    #[inline]
    fn invalidate(&self) {
        self.invalidate()
    }
}

impl<'a, T: ?Sized> Read for DynamicBufferMutSlice<'a, T> where T: Content {
    #[inline]
    fn read(&self) -> Result<T::Owned, ReadError> {
        self.read()
    }
}

impl<'a, T: ?Sized> Write for DynamicBufferMutSlice<'a, T> where T: Content {
    #[inline]
    fn write(&self, data: &T) {
        self.write(data)
    }
}

impl<'a, T> Map for DynamicBufferMutSlice<'a, T> where T: Content {
    type Mapping = Mapping<'a, T>;

    #[inline]
    fn map(self) -> Mapping<'a, T> {
        self.map()
    }
}

/// Represents a sub-part of a buffer.
///
/// Doesn't contain any information about the content, contrary to `Buffer`.
pub struct BufferAny {
    alloc: Alloc,
    size: usize,
    elements_size: usize,
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
        }
    }

    /// Returns the size in bytes of each element in the buffer.
    // TODO: clumbsy, remove this function
    #[inline]
    pub fn get_elements_size(&self) -> usize {
        self.elements_size
    }

    /// Returns the number of elements in the buffer.
    // TODO: clumbsy, remove this function
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
    /// Panicks if the size of the buffer is not a multiple of the size of the data.
    /// For example, trying to read some `(u8, u8, u8, u8)`s from a buffer of 7 bytes will panic.
    ///
    #[inline]
    pub unsafe fn read<T>(&self) -> Result<T::Owned, ReadError> where T: Content {
        // TODO: add check
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
        }
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
    fn get_buffer_id(&self) -> gl::types::GLuint {
        self.alloc.get_id()
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
}

impl<'a> BufferAnySlice<'a> {
    /// Returns the number of bytes between the start of the buffer and the start of this slice.
    #[inline]
    pub fn get_offset_bytes(&self) -> usize {
        self.bytes_start
    }

    /// Returns the number of bytes in this slice.
    #[inline]
    pub fn get_size(&self) -> usize {
        self.bytes_end - self.bytes_start
    }

    /// Returns the size in bytes of each element in the buffer.
    // TODO: clumbsy, remove this function
    #[inline]
    pub fn get_elements_size(&self) -> usize {
        self.elements_size
    }

    /// Returns the number of elements in the buffer.
    // TODO: clumbsy, remove this function
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
        unimplemented!()
    }
}

impl<'a> BufferExt for BufferAnySlice<'a> {
    #[inline]
    fn get_offset_bytes(&self) -> usize {
        self.bytes_start
    }

    #[inline]
    fn get_buffer_id(&self) -> gl::types::GLuint {
        self.alloc.get_id()
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
