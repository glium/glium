//! A buffer is a memory location accessible to the video card.
//!
//! The purpose of buffers is to serve as a space where the GPU can read from or write data to.
//! It can contain a list of vertices, indices, uniform data, etc.
//!
//! # Buffers management in glium
//!
//! There are three types of buffers in glium:
//! 
//! - Dynamic buffers, which are the buffers that you usually use by default. All OpenGL tutorials
//!   that you find on the web use these kinds of buffers.
//! - Immutable buffers, whose content is inaccessible from the CPU. They usually offer slightly
//!   better performances when the buffer's content never needs to be modified.
//! - Persistent buffers (short for "persistent-mapped buffers"). They are more difficult to use
//!   but offer the best control over what's happening and are ideal when you are often modifying
//!   the content of the buffer.
//!
//! # Unsized types
//!
//! In order to put some data in a buffer, it must implement the `Content` trait. This trait is
//! automatically implemented on all `Sized` types and on slices (like `[u8]`). This means that
//! you can for example create a `Buffer<Foo>` (if `Foo` is sized) or a `Buffer<[u8]>` without
//! having to worry about this.
//!
//! However if you create an unsized struct, it doesn't automatically implement this trait. You can
//! solve this by manually calling the `implement_buffer_content!` macro on them. You must then
//! use the `empty_unsized` constructor on buffers in order to store their content.
//!
//! ```no_run
//! # #[macro_use] extern crate glium; fn main() {
//! # use std::mem;
//! # use glium::buffer::{BufferType, BufferMode};
//! # let display: glium::Display = unsafe { mem::uninitialized() };
//! struct Data {
//!     data: [f32],        // `[f32]` is unsized, therefore `Data` is unsized too
//! }
//!
//! implement_buffer_content!(Data);    // without this, you can't put `Data` in a glium buffer
//!
//! // creates a buffer of 64 bytes, which thus holds 8 f32s
//! let mut buffer = glium::buffer::Buffer::<Data>::empty_unsized(&display, BufferType::UniformBuffer,
//!                                                               64).unwrap();
//!
//! // you can then write to it like you normally would
//! buffer.map().data[4] = 2.1;
//! # }
//! ```
//!
use std::borrow::Borrow;
use std::borrow::Cow;
use utils::range::RangeArgument;

use texture::PixelValue;
use texture::Texture1dDataSink;

pub use self::view::{Buffer, BufferSlice, BufferMutSlice};
pub use self::view::{DynamicBuffer, DynamicBufferSlice, DynamicBufferMutSlice};
pub use self::immutable::{ImmutableBuffer, ImmutableBufferSlice, ImmutableBufferMutSlice};
pub use self::persistent::{PersistentBuffer, PersistentBufferSlice, PersistentBufferMutSlice};
pub use self::view::{BufferAny, BufferAnySlice};
pub use self::alloc::{Mapping, WriteMapping, ReadMapping, ReadError};
pub use self::alloc::{is_buffer_read_supported};
pub use self::fences::Inserter;

/// DEPRECATED. Only here for backward compatibility.
pub use self::view::Buffer as BufferView;
/// DEPRECATED. Only here for backward compatibility.
pub use self::view::BufferSlice as BufferViewSlice;
/// DEPRECATED. Only here for backward compatibility.
pub use self::view::BufferMutSlice as BufferViewMutSlice;
/// DEPRECATED. Only here for backward compatibility.
pub use self::view::BufferAny as BufferViewAny;
/// DEPRECATED. Only here for backward compatibility.
pub use self::view::BufferAnySlice as BufferViewAnySlice;

use gl;
use std::error::Error;
use std::fmt;
use std::mem;
use std::slice;

use backend::Facade;

macro_rules! buffers_base {
    ($buffer:ident, $slice:ident, $mut_slice:ident) => (
        impl<T: ?Sized> $buffer<T> where T: Content {
            /// Returns the size in bytes of the buffer.
            #[inline]
            pub fn size(&self) -> usize {
                ::buffer::Storage::size(self)
            }

            /// Returns the number of elements in this buffer.
            ///
            /// This method is only available if the content is an array.
            #[inline]
            pub fn len(&self) -> usize where T: ArrayContent {
                self.size() / <T as ArrayContent>::element_size()
            }

            /// Builds a subslice of this slice. Returns `None` if out of range.
            ///
            /// This method builds an object that represents a slice of the buffer. No actual operation
            /// OpenGL is performed.
            ///
            /// # Example
            ///
            /// ```no_run
            /// # let buffer: glium::buffer::ImmutableBuffer<[u8]> = unsafe { ::std::mem::uninitialized() };
            /// // let buffer: glium::buffer::ImmutableBuffer<[u8]>;
            /// let slice = buffer.slice(0 .. 8).unwrap();
            /// slice.invalidate();     // invalidates only elements 0 to 8
            /// ```
            #[inline]
            pub fn slice<R>(&self, range: R) -> Option<$slice<T>>
                where T: ArrayContent,
                      R: ::utils::range::RangeArgument<usize>
            {
                self.as_slice().slice(range)
            }

            /// Builds a subslice of this slice. Returns `None` if out of range.
            ///
            /// This method builds an object that represents a slice of the buffer. No actual operation
            /// OpenGL is performed.
            #[inline]
            pub fn slice_mut<R>(&mut self, range: R) -> Option<$mut_slice<T>>
                where T: ArrayContent,
                      R: ::utils::range::RangeArgument<usize>
            {
                self.as_mut_slice().slice(range)
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
            pub unsafe fn slice_custom<F, R: ?Sized>(&self, f: F) -> $slice<R>
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
            pub unsafe fn slice_custom_mut<F, R: ?Sized>(&mut self, f: F) -> $mut_slice<R>
                                                         where F: for<'r> FnOnce(&'r T) -> &'r R,
                                                                R: Content
            {
                self.as_mut_slice().slice_custom(f)
            }

            /// Builds a slice representing the whole buffer.
            ///
            /// This method builds an object that represents a slice of the buffer. No actual operation
            /// OpenGL is performed.
            #[inline]
            pub fn as_slice(&self) -> $slice<T> {
                $slice {
                    context: &self.context,
                    buffer: self.id,
                    bytes_start: 0,
                    bytes_end: self.size(),
                    marker: ::std::marker::PhantomData,
                }
            }

            /// Builds a slice representing the whole buffer.
            ///
            /// This method builds an object that represents a slice of the buffer. No actual operation
            /// OpenGL is performed.
            #[inline]
            pub fn as_mut_slice(&mut self) -> $mut_slice<T> {
                let size = self.size();

                $mut_slice {
                    context: &mut self.context,
                    buffer: self.id,
                    bytes_start: 0,
                    bytes_end: size,
                    marker: ::std::marker::PhantomData,
                }
            }
        }

        impl<T: ?Sized> ::std::fmt::Debug for $buffer<T> where T: Content {
            #[inline]
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
                write!(fmt, "OpenGL buffer (len: {} bytes)", self.size())
            }
        }

        #[derive(Copy, Clone)]
        pub struct $slice<'a, T: ?Sized + 'a> where T: Content {
            context: &'a Rc<Context>,
            buffer: gl::types::GLuint,
            bytes_start: usize,
            bytes_end: usize,
            marker: ::std::marker::PhantomData<T>,
        }

        // TODO: we need this because PhantomData<T> requires T: Sized
        impl<'a, T: ?Sized> ::std::fmt::Debug for $slice<'a, T> where T: Content {
            #[inline]
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
                write!(fmt, "OpenGL buffer slice (len: {} bytes)", self.size())
            }
        }

        impl<'a, T: ?Sized> $slice<'a, T> where T: Content + 'a {
            /// Returns the size in bytes of this slice.
            #[inline]
            pub fn size(&self) -> usize {
                self.bytes_end - self.bytes_start
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
            pub unsafe fn slice_custom<F, R: ?Sized>(&self, f: F) -> $slice<'a, R>
                                                     where F: for<'r> FnOnce(&'r T) -> &'r R,
                                                           R: Content
            {
                let data: &T = mem::zeroed();
                let result = f(data);
                let size = mem::size_of_val(result);
                let result = result as *const R as *const () as usize;

                assert!(result <= self.size());
                assert!(result + size <= self.size());

                $slice {
                    context: self.context,
                    buffer: self.buffer,
                    bytes_start: self.bytes_start + result,
                    bytes_end: self.bytes_start + result + size,
                    marker: ::std::marker::PhantomData,
                }
            }

            /// Builds a slice-any containing the whole subbuffer.
            ///
            /// This method builds an object that represents a slice of the buffer. No actual operation
            /// OpenGL is performed.
            #[inline]
            pub fn as_slice_any(&self) -> BufferAnySlice<'a> {
                unimplemented!()
                /*BufferAnySlice {
                    alloc: self.alloc,
                    bytes_start: self.bytes_start,
                    bytes_end: self.bytes_end,
                    elements_size: <T as Content>::get_elements_size(),
                }*/
            }

            /// Returns the number of elements in this slice.
            #[inline]
            pub fn len(&self) -> usize where T: ArrayContent {
                (self.bytes_end - self.bytes_start) / <T as ArrayContent>::element_size()
            }

            /// Builds a subslice of this slice. Returns `None` if out of range.
            ///
            /// This method builds an object that represents a slice of the buffer. No actual operation
            /// OpenGL is performed.
            #[inline]
            pub fn slice<R>(&self, range: R) -> Option<$slice<'a, T>>
                where T: ArrayContent,
                      R: ::utils::range::RangeArgument<usize>
            {
                if range.start().map_or(0, |e| *e) > self.len() || range.end().map_or(0, |e| *e) > self.len() {
                    return None;
                }

                unimplemented!()
                /*Some($slice {
                    buffer: self.buffer,
                    bytes_start: self.bytes_start + range.start().map_or(0, |e| *e) * <T as ArrayContent>::element_size(),
                    bytes_end: self.bytes_start + range.end().map_or(self.len(), |e| *e) * <T as ArrayContent>::element_size(),
                })*/
            }
        }

        pub struct $mut_slice<'a, T: ?Sized + 'a> where T: Content {
            context: &'a mut Rc<Context>,
            buffer: gl::types::GLuint,
            bytes_start: usize,
            bytes_end: usize,
            marker: ::std::marker::PhantomData<T>,
        }

        // TODO: we need this because PhantomData<T> requires T: Sized
        impl<'a, T: ?Sized> ::std::fmt::Debug for $mut_slice<'a, T> where T: Content {
            #[inline]
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
                write!(fmt, "OpenGL buffer slice (len: {} bytes)", self.size())
            }
        }

        impl<'a, T: ?Sized> $mut_slice<'a, T> where T: Content + 'a {
            /// Returns the size in bytes of this slice.
            #[inline]
            pub fn size(&self) -> usize {
                self.bytes_end - self.bytes_start
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
            pub unsafe fn slice_custom<F, R: ?Sized>(self, f: F) -> $mut_slice<'a, R>
                                                     where F: for<'r> FnOnce(&'r T) -> &'r R,
                                                           R: Content
            {
                let data: &T = mem::zeroed();
                let result = f(data);
                let size = mem::size_of_val(result);
                let result = result as *const R as *const () as usize;

                assert!(result <= self.size());
                assert!(result + size <= self.size());

                unimplemented!()
                /*$mut_slice {
                    buffer: self.buffer,
                    bytes_start: self.bytes_start + result,
                    bytes_end: self.bytes_start + result + size,
                }*/
            }

            /// Builds a slice-any containing the whole subbuffer.
            ///
            /// This method builds an object that represents a slice of the buffer. No actual operation
            /// OpenGL is performed.
            #[inline]
            pub fn as_slice_any(self) -> BufferAnySlice<'a> {
                unimplemented!()
                /*BufferAnySlice {
                    alloc: self.alloc,
                    bytes_start: self.bytes_start,
                    bytes_end: self.bytes_end,
                    elements_size: <T as Content>::get_elements_size(),
                }*/
            }

            /// Returns the number of elements in this slice.
            #[inline]
            pub fn len(&self) -> usize where T: ArrayContent {
                (self.bytes_end - self.bytes_start) / <T as ArrayContent>::element_size()
            }

            /// Builds a subslice of this slice. Returns `None` if out of range.
            ///
            /// This method builds an object that represents a slice of the buffer. No actual operation
            /// OpenGL is performed.
            #[inline]
            pub fn slice<R>(self, range: R) -> Option<$mut_slice<'a, T>>
                where T: ArrayContent,
                      R: ::utils::range::RangeArgument<usize>
            {
                if range.start().map_or(0, |e| *e) > self.len() || range.end().map_or(0, |e| *e) > self.len() {
                    return None;
                }

                let new_start = self.bytes_start + range.start().map_or(0, |e| *e) * <T as ArrayContent>::element_size();
                let new_end = self.bytes_start + range.end().map_or(self.len(), |e| *e) * <T as ArrayContent>::element_size();

                Some($mut_slice {
                    context: self.context,
                    buffer: self.buffer,
                    bytes_start: new_start,
                    bytes_end: new_end,
                    marker: ::std::marker::PhantomData,
                })
            }
        }

        impl<'a, T> ::buffer::Slice for &'a $buffer<T> where T: Content {
            type Slice = $slice<'a, T>;

            fn as_slice(self) -> $slice<'a, T> {
                self.as_slice()
            }

            fn slice<R>(self, range: R) -> Option<$slice<'a, T>>
                where T: ::buffer::ArrayContent,
                      R: ::utils::range::RangeArgument<usize>
            {
                self.slice(range)
            }
        }

        impl<'a, T> ::buffer::Slice for &'a mut $buffer<T> where T: Content {
            type Slice = $slice<'a, T>;

            fn as_slice(self) -> $slice<'a, T> {
                self.as_slice()
            }

            fn slice<R>(self, range: R) -> Option<$slice<'a, T>>
                where T: ::buffer::ArrayContent,
                      R: ::utils::range::RangeArgument<usize>
            {
                self.slice(range)
            }
        }

        impl<'a, T> ::buffer::SliceMut for &'a mut $buffer<T> where T: Content {
            type SliceMut = $mut_slice<'a, T>;

            fn as_mut_slice(self) -> $mut_slice<'a, T> {
                self.as_mut_slice()
            }

            fn slice_mut<R>(self, range: R) -> Option<$mut_slice<'a, T>>
                where T: ::buffer::ArrayContent,
                      R: ::utils::range::RangeArgument<usize>
            {
                self.slice_mut(range)
            }
        }

        impl<'a, T, R: ?Sized> ::buffer::SliceCustom<R> for &'a $buffer<T>
            where T: Content, R: Content
        {
            type Slice = $slice<'a, R>;

            #[inline]
            unsafe fn slice_custom<F>(self, f: F) -> Self::Slice
                where F: for<'r> FnOnce(&'r Self::Content) -> &'r R, Self: Sized
            {
                self.slice_custom(f)
            }
        }

        impl<'a, T, R: ?Sized> ::buffer::SliceCustom<R> for &'a mut $buffer<T>
            where T: Content, R: Content
        {
            type Slice = $slice<'a, R>;

            #[inline]
            unsafe fn slice_custom<F>(self, f: F) -> Self::Slice
                where F: for<'r> FnOnce(&'r Self::Content) -> &'r R, Self: Sized
            {
                self.slice_custom(f)
            }
        }

        impl<'a, T, R: ?Sized> ::buffer::SliceCustomMut<R> for &'a mut $buffer<T>
            where T: Content, R: Content
        {
            type SliceMut = $mut_slice<'a, R>;

            #[inline]
            unsafe fn slice_custom_mut<F>(self, f: F) -> Self::SliceMut
                where F: for<'r> FnOnce(&'r Self::Content) -> &'r R, Self: Sized
            {
                self.slice_custom_mut(f)
            }
        }

        impl<'a, T: ?Sized> From<&'a $buffer<T>> for ::buffer::BufferAnySlice<'a>
            where T: ::buffer::Content
        {
            #[inline]
            fn from(buf: &'a $buffer<T>) -> ::buffer::BufferAnySlice<'a> {
                buf.as_slice_any()
            }
        }

        impl<'a, T: ?Sized> From<$slice<'a, T>> for ::buffer::BufferAnySlice<'a>
            where T: ::buffer::Content
        {
            #[inline]
            fn from(buf: $slice<'a, T>) -> ::buffer::BufferAnySlice<'a> {
                buf.as_slice_any()
            }
        }

        impl<'a, T: ?Sized> Storage for &'a $buffer<T> where T: Content {
            type Content = T;

            #[inline]
            fn size(&self) -> usize {
                (*self).size()
            }

            #[inline]
            fn as_slice_any(&self) -> BufferAnySlice {
                (*self).as_slice_any()
            }
        }

        impl<'a, T: ?Sized> Storage for &'a mut $buffer<T> where T: Content {
            type Content = T;

            #[inline]
            fn size(&self) -> usize {
                (*self).size()
            }

            #[inline]
            fn as_slice_any(&self) -> BufferAnySlice {
                (*self).as_slice_any()
            }
        }

        impl<'a, T: ?Sized> Storage for $slice<'a, T> where T: Content {
            type Content = T;

            #[inline]
            fn size(&self) -> usize {
                self.bytes_end - self.bytes_start
            }

            #[inline]
            fn as_slice_any(&self) -> BufferAnySlice {
                self.as_slice_any()
            }
        }

        impl<'a, T: ?Sized> Storage for $mut_slice<'a, T> where T: Content {
            type Content = T;

            #[inline]
            fn size(&self) -> usize {
                self.bytes_end - self.bytes_start
            }

            #[inline]
            fn as_slice_any(&self) -> BufferAnySlice {
                self.as_slice_any()
            }
        }
    );
}

mod alloc;
//mod dynamic;
mod fences;
mod immutable;
//mod lock;
mod persistent;
mod raw;
mod view;

/// Trait for types of data that can be put inside buffers.
pub unsafe trait Content {
    /// A type that holds a sized version of the content.
    type Owned;

    /// Prepares an output buffer, then turns this buffer into an `Owned`.
    fn read<F, E>(size: usize, F) -> Result<Self::Owned, E>
                  where F: FnOnce(&mut Self) -> Result<(), E>;

    /// Returns the size of each element.
    fn get_elements_size() -> usize;

    /// Produces a pointer to the data.
    fn to_void_ptr(&self) -> *const ();

    /// Builds a pointer to this type from a raw pointer.
    fn ref_from_ptr<'a>(ptr: *mut (), size: usize) -> Option<*mut Self>;

    /// Returns true if the size is suitable to store a type like this.
    fn is_size_suitable(usize) -> bool;
}

unsafe impl<T> Content for T where T: Copy {
    type Owned = T;

    #[inline]
    fn read<F, E>(size: usize, f: F) -> Result<T, E> where F: FnOnce(&mut T) -> Result<(), E> {
        assert!(size == mem::size_of::<T>());
        let mut value = unsafe { mem::uninitialized() };
        try!(f(&mut value));
        Ok(value)
    }

    #[inline]
    fn get_elements_size() -> usize {
        mem::size_of::<T>()
    }

    #[inline]
    fn to_void_ptr(&self) -> *const () {
        self as *const T as *const ()
    }

    #[inline]
    fn ref_from_ptr<'a>(ptr: *mut (), size: usize) -> Option<*mut T> {
        if size != mem::size_of::<T>() {
            return None;
        }

        Some(ptr as *mut T)
    }

    #[inline]
    fn is_size_suitable(size: usize) -> bool {
        size == mem::size_of::<T>()
    }
}

unsafe impl<T> Content for [T] where T: Copy {
    type Owned = Vec<T>;

    #[inline]
    fn read<F, E>(size: usize, f: F) -> Result<Vec<T>, E>
                  where F: FnOnce(&mut [T]) -> Result<(), E>
    {
        assert!(size % mem::size_of::<T>() == 0);
        let len = size / mem::size_of::<T>();
        let mut value = Vec::with_capacity(len);
        unsafe { value.set_len(len) };
        try!(f(&mut value));
        Ok(value)
    }

    #[inline]
    fn get_elements_size() -> usize {
        mem::size_of::<T>()
    }

    #[inline]
    fn to_void_ptr(&self) -> *const () {
        &self[0] as *const T as *const ()
    }

    #[inline]
    fn ref_from_ptr<'a>(ptr: *mut (), size: usize) -> Option<*mut [T]> {
        if size % mem::size_of::<T>() != 0 {
            return None;
        }

        let ptr = ptr as *mut T;
        let size = size / mem::size_of::<T>();
        Some(unsafe { slice::from_raw_parts_mut(&mut *ptr, size) as *mut [T] })
    }

    #[inline]
    fn is_size_suitable(size: usize) -> bool {
        size % mem::size_of::<T>() == 0
    }
}

pub unsafe trait ArrayContent: Content {
    type Element: Copy;

    /// Returns the size of each element.
    #[inline]
    fn element_size() -> usize {
        mem::size_of::<Self::Element>()
    }
}

unsafe impl<T> ArrayContent for [T] where T: Copy {
    type Element = T;
}

/// Error that can happen when creating a buffer.
#[derive(Debug, Copy, Clone)]
pub enum BufferCreationError {
    /// Not enough memory to create the buffer.
    OutOfMemory,

    /// This type of buffer is not supported.
    BufferTypeNotSupported,
}

impl fmt::Display for BufferCreationError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

impl Error for BufferCreationError {
    fn description(&self) -> &str {
        match self {
            &BufferCreationError::OutOfMemory => "Not enough memory to create the buffer",
            &BufferCreationError::BufferTypeNotSupported => "This type of buffer is not supported",
        }
    }
}

/// Error that can happen when copying data between buffers.
#[derive(Debug, Copy, Clone)]
pub enum CopyError {
    /// The backend doesn't support copying between buffers.
    NotSupported,
}

impl fmt::Display for CopyError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

impl Error for CopyError {
    fn description(&self) -> &str {
        use self::CopyError::*;
        match *self {
            NotSupported => "The backend doesn't support copying between buffers",
        }
    }
}

pub trait Storage {
    /// The data type that is contained in the buffer.
    ///
    /// Can be unsized. For example it can be `[u8]`.
    type Content: ?Sized + Content;

    /// Builds a slice-any containing the whole subbuffer. Intended to be used for operations
    /// internal to glium.
    ///
    /// This method builds an object that represents a slice of the buffer. No actual operation
    /// OpenGL is performed.
    fn as_slice_any(&self) -> BufferAnySlice;

    /// Returns the size in bytes of the buffer.
    fn size(&self) -> usize;

    /// Returns the offset, in bytes, between the start of the underlying buffer and the data
    /// described by this object.
    #[inline]
    fn offset_bytes(&self) -> usize {
        self.as_slice_any().get_offset_bytes()
    }

    /// Number of elements if the content is an array.
    #[inline]
    fn len(&self) -> usize
        where Self::Content: ArrayContent
    {
        self.size() / mem::size_of::<<Self::Content as ArrayContent>::Element>()
    }

    /// This function is called whenever the GPU needs to access this buffer.
    ///
    /// It indicates which kind of access is being done.
    ///
    /// If necessary the implementation can return an `Inserter` which will be used by the caller
    /// to indicate when the GPU is over with its operation.
    #[inline]
    fn gpu_access(&self, read: bool, write: bool) -> Option<Inserter> {
        None
    }
}

/// Trait for types whose content can be invalidated.
///
/// Invalidating the content of a buffer means that its content becomes undefined. This is used
/// as an optimization, as it informs OpenGL that the content of the buffer doesn't need to be
/// loaded.
pub trait Invalidate {
    /// Invalidates the content.
    fn invalidate(&self);
}

pub trait Read: Storage {
    /// Reads the content of the buffer.
    fn read(&self) -> Result<<Self::Content as Content>::Owned, ReadError>;

    /// Reads the content of the buffer.
    #[inline]
    fn read_as_texture_1d<S>(&self) -> Result<S, ReadError>
        where Self::Content: ArrayContent,
              S: Texture1dDataSink<<Self::Content as ArrayContent>::Element>,
              <Self::Content as ArrayContent>::Element: PixelValue,
              [<Self::Content as ArrayContent>::Element]: ToOwned<Owned = <Self::Content as Content>::Owned>
    {
        let data = try!(self.read());
        Ok(S::from_raw(Cow::Owned(data), self.len() as u32))
    }
}

pub trait Write: Storage {
    /// Uploads some data in this buffer.
    ///
    /// # Panic
    ///
    /// Panics if the length of `data` is different from the length of this buffer. This is only
    /// relevant for unsized content.
    ///
    /// For example if you try pass a `&[u8]` with 512 elements, but the buffer contains 256
    /// elements, you will get a panic.
    fn write(&self, data: &Self::Content);
}

pub trait Slice: Storage {
    type Slice: Storage;

    fn as_slice(self) -> Self::Slice;

    fn slice<R: RangeArgument<usize>>(self, range: R) -> Option<Self::Slice>
        where Self::Content: ArrayContent;
}

pub trait SliceMut: Slice {
    type SliceMut: Storage;

    fn as_mut_slice(self) -> Self::SliceMut;

    fn slice_mut<R: RangeArgument<usize>>(self, range: R) -> Option<Self::SliceMut>
        where Self::Content: ArrayContent;
}

pub trait SliceCustom<R: ?Sized>: Storage where R: Content {
    type Slice: Storage;

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
    unsafe fn slice_custom<F>(self, f: F) -> Self::Slice
        where F: for<'r> FnOnce(&'r Self::Content) -> &'r R, Self: Sized;
}

pub trait SliceCustomMut<R: ?Sized>: SliceCustom<R> where R: Content {
    type SliceMut: Storage;

    /// Same as `slice_custom` but returns a mutable slice.
    ///
    /// This method builds an object that represents a slice of the buffer. No actual operation
    /// OpenGL is performed.
    #[inline]
    unsafe fn slice_custom_mut<F>(self, f: F) -> Self::SliceMut
        where F: for<'r> FnOnce(&'r Self::Content) -> &'r R, Self: Sized;
}

pub trait Create: Storage {
    fn new<F>(facade: &F, data: &Self::Content, ty: BufferType)
              -> Result<Self, BufferCreationError>
        where F: Facade, Self: Sized;

    /// Builds a new buffer of the given size.
    fn empty<F>(facade: &F, ty: BufferType)
                -> Result<Self, BufferCreationError>
        where F: Facade, Self: Sized, Self::Content: Copy;

    /// Builds a new buffer of the given size.
    fn empty_array<F>(facade: &F, len: usize, ty: BufferType)
                      -> Result<Self, BufferCreationError>
        where F: Facade, Self: Sized, Self::Content: ArrayContent;

    /// Builds a new buffer of the given size.
    ///
    /// # Panic
    ///
    /// Panicks if the size passed as parameter isn't suitable for the content.
    ///
    fn empty_unsized<F>(facade: &F, size: usize, ty: BufferType)
                        -> Result<Self, BufferCreationError>
        where F: Facade, Self: Sized, Self::Content: Copy;
}

pub trait CopyTo: Storage {
    /// Copies the content of the buffer to another buffer.
    ///
    /// # Panic
    ///
    /// Panics if the content is unsized and the other buffer is too small.
    ///
    fn copy_to<S>(&self, target: &S) -> Result<(), CopyError>
        where S: Storage;
}

// TODO: chance this trait once HKTs are released
pub trait Map: Storage {
    type Mapping;

    fn map(self) -> Self::Mapping;
}

/// How the buffer is created.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BufferMode {
    /// This is the default mode suitable for any usage. Will never be slow, will never be fast
    /// either.
    ///
    /// Other modes should always be preferred, but you can use this one if you don't know what
    /// will happen to the buffer.
    ///
    /// # Implementation
    ///
    /// Tries to use `glBufferStorage` with the `GL_DYNAMIC_STORAGE_BIT` flag.
    ///
    /// If this function is not available, falls back to `glBufferData` with `GL_STATIC_DRAW`.
    ///
    Default,

    /// The mode to use when you modify a buffer multiple times per frame. Simiar to `Default` in
    /// that it is suitable for most usages.
    ///
    /// Use this if you do a quick succession of modify the buffer, draw, modify, draw, etc. This
    /// is something that you shouldn't do by the way.
    ///
    /// With this mode, the OpenGL driver automatically manages the buffer for us. It will try to
    /// find the most appropriate storage depending on how we use it. It is guaranteed to never be
    /// too slow, but it won't be too fast either.
    ///
    /// # Implementation
    ///
    /// Tries to use `glBufferStorage` with the `GL_DYNAMIC_STORAGE_BIT` and
    /// `GL_CLIENT_STORAGE_BIT` flags.
    ///
    /// If this function is not available, falls back to `glBufferData` with `GL_DYNAMIC_DRAW`.
    ///
    Dynamic,

    /// Optimized for when you modify a buffer exactly once per frame. You can modify it more than
    /// once per frame, but if you modify it too often things will slow down.
    ///
    /// With this mode, glium automatically handles synchronization to prevent the buffer from
    /// being access by both the GPU and the CPU simultaneously. If you try to modify the buffer,
    /// the execution will block until the GPU has finished using it. For this reason, a quick
    /// succession of modifying and drawing using the same buffer will be very slow.
    ///
    /// When using persistent mapping, it is recommended to use triple buffering. This is done by
    /// creating a buffer that has three times the capacity that it would normally have. You modify
    /// and draw the first third, then modify and draw the second third, then the last part, then
    /// go back to the first third, etc.
    ///
    /// # Implementation
    ///
    /// Tries to use `glBufferStorage` with `GL_MAP_PERSISTENT_BIT`. Sync fences are automatically
    /// managed by glium.
    ///
    /// If this function is not available, falls back to `glBufferData` with `GL_DYNAMIC_DRAW`.
    ///
    Persistent,

    /// Optimized when you will never touch the content of the buffer.
    ///
    /// Immutable buffers should be created once and never touched again. Modifying their content
    /// is permitted, but is very slow.
    ///
    /// # Implementation
    ///
    /// Tries to use `glBufferStorage` without any flag. Modifications are done by creating
    /// temporary buffers and making the GPU copy the data from the temporary buffer to the real
    /// one.
    ///
    /// If this function is not available, falls back to `glBufferData` with `GL_STATIC_DRAW`.
    ///
    Immutable,
}

impl Default for BufferMode {
    fn default() -> BufferMode {
        BufferMode::Default
    }
}

/// Type of a buffer.
#[doc(hidden)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BufferType {
    ArrayBuffer,
    PixelPackBuffer,
    PixelUnpackBuffer,
    UniformBuffer,
    CopyReadBuffer,
    CopyWriteBuffer,
    AtomicCounterBuffer,
    DispatchIndirectBuffer,
    DrawIndirectBuffer,
    QueryBuffer,
    ShaderStorageBuffer,
    TextureBuffer,
    TransformFeedbackBuffer,
    ElementArrayBuffer,
}

impl BufferType {
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            BufferType::ArrayBuffer => gl::ARRAY_BUFFER,
            BufferType::PixelPackBuffer => gl::PIXEL_PACK_BUFFER,
            BufferType::PixelUnpackBuffer => gl::PIXEL_UNPACK_BUFFER,
            BufferType::UniformBuffer => gl::UNIFORM_BUFFER,
            BufferType::CopyReadBuffer => gl::COPY_READ_BUFFER,
            BufferType::CopyWriteBuffer => gl::COPY_WRITE_BUFFER,
            BufferType::AtomicCounterBuffer => gl::ATOMIC_COUNTER_BUFFER,
            BufferType::DispatchIndirectBuffer => gl::DISPATCH_INDIRECT_BUFFER,
            BufferType::DrawIndirectBuffer => gl::DRAW_INDIRECT_BUFFER,
            BufferType::QueryBuffer => gl::QUERY_BUFFER,
            BufferType::ShaderStorageBuffer => gl::SHADER_STORAGE_BUFFER,
            BufferType::TextureBuffer => gl::TEXTURE_BUFFER,
            BufferType::TransformFeedbackBuffer => gl::TRANSFORM_FEEDBACK_BUFFER,
            BufferType::ElementArrayBuffer => gl::ELEMENT_ARRAY_BUFFER,
        }
    }
}

macro_rules! impl_buffer_wrapper {
    ($ty:ident, $inner:ident, [$($other_field:ident),+]) => (
        impl<T> $ty<T> where T: ::buffer::Storage {
            #[inline]
            pub fn size(&self) -> usize {
                self.$inner.size()
            }

            #[inline]
            pub fn len(&self) -> usize
                where T::Content: ::buffer::ArrayContent
            {
                self.$inner.len()
            }

            #[inline]
            pub fn map<'a>(&'a self) -> <&'a T as ::buffer::Map>::Mapping
                where &'a T: ::buffer::Map, T: ::buffer::Storage
            {
                ::buffer::Map::map(&self.$inner)
            }

            #[inline]
            pub fn invalidate(&self)
                where T: ::buffer::Invalidate
            {
                self.$inner.invalidate()
            }

            #[inline]
            pub fn read(&self) -> Result<<T::Content as ::buffer::Content>::Owned,
                                         ::buffer::ReadError>
                where T: ::buffer::Read {
                self.$inner.read()
            }

            // TODO: Add `read_as_texture_1d`

            #[inline]
            pub fn write(&self, data: &T::Content)
                where T: ::buffer::Write
            {
                self.$inner.write(data)
            }

            #[inline]
            pub fn copy_to<S>(&self, target: &S) -> Result<(), ::buffer::CopyError>
                where T: ::buffer::CopyTo, S: ::buffer::Storage
            {
                self.$inner.copy_to(target)
            }

            #[inline]
            pub fn as_slice<'a>(&'a self) -> $ty<<&'a T as ::buffer::Slice>::Slice>
                where &'a T: ::buffer::Slice
            {
                $ty {
                    $inner: ::buffer::Slice::as_slice(&self.$inner),
                    $(
                        $other_field: self.$other_field.clone()
                    ),+
                }
            }

            #[inline]
            pub fn slice<'a, R>(&'a self, range: R) -> Option<$ty<<&'a T as ::buffer::Slice>::Slice>>
                where <&'a T as ::buffer::Storage>::Content: ::buffer::ArrayContent,
                      &'a T: ::buffer::Slice, R: ::utils::range::RangeArgument<usize>
            {
                ::buffer::Slice::slice(&self.$inner, range).map(|slice| {
                    $ty {
                        $inner: slice,
                        $(
                            $other_field: self.$other_field.clone()
                        ),+
                    }
                })
            }

            #[inline]
            pub fn as_mut_slice<'a>(&'a mut self) -> $ty<<&'a mut T as ::buffer::SliceMut>::SliceMut>
                where &'a mut T: ::buffer::SliceMut
            {
                $ty {
                    $inner: ::buffer::SliceMut::as_mut_slice(&mut self.$inner),
                    $(
                        $other_field: self.$other_field.clone()
                    ),+
                }
            }

            #[inline]
            pub fn slice_mut<'a, R>(&'a mut self, range: R) -> Option<$ty<<&'a mut T as ::buffer::SliceMut>::SliceMut>>
                where <&'a mut T as ::buffer::Storage>::Content: ::buffer::ArrayContent,
                      &'a mut T: ::buffer::SliceMut, R: ::utils::range::RangeArgument<usize>
            {
                Some($ty {
                    $inner: match ::buffer::SliceMut::slice_mut(&mut self.$inner, range) {
                        Some(s) => s,
                        None => return None
                    },
                    $(
                        $other_field: self.$other_field.clone()
                    ),+
                })
            }

            #[inline]
            pub unsafe fn slice_custom<'a, F, R: ?Sized>(&'a self, f: F) -> $ty<<&'a T as ::buffer::SliceCustom<R>>::Slice>
                where &'a T: ::buffer::SliceCustom<R>, R: ::buffer::Content,
                      F: for<'r> FnOnce(&'r <&'a T as ::buffer::Storage>::Content) -> &'r R
            {
                $ty {
                    $inner: ::buffer::SliceCustom::slice_custom(&self.$inner, f),
                    $(
                        $other_field: self.$other_field.clone()
                    ),+
                }
            }

            #[inline]
            pub unsafe fn slice_custom_mut<'a, F, R: ?Sized>(&'a mut self, f: F) -> $ty<<&'a mut T as ::buffer::SliceCustomMut<R>>::SliceMut>
                where &'a mut T: ::buffer::SliceCustomMut<R>, R: ::buffer::Content,
                      F: for<'r> FnOnce(&'r <&'a mut T as ::buffer::Storage>::Content) -> &'r R
            {
                $ty {
                    $inner: ::buffer::SliceCustomMut::slice_custom_mut(&mut self.$inner, f),
                    $(
                        $other_field: self.$other_field.clone()
                    ),+
                }
            }
        }

        impl<T> ::buffer::Storage for $ty<T> where T: ::buffer::Storage {
            type Content = T::Content;

            #[inline]
            fn size(&self) -> usize {
                self.$inner.size()
            }

            #[inline]
            fn as_slice_any(&self) -> ::buffer::BufferAnySlice {
                self.$inner.as_slice_any()
            }
        }

        impl<'a, T> ::buffer::Storage for &'a $ty<T> where T: ::buffer::Storage {
            type Content = <T as ::buffer::Storage>::Content;

            #[inline]
            fn size(&self) -> usize {
                self.$inner.size()
            }

            #[inline]
            fn as_slice_any(&self) -> ::buffer::BufferAnySlice {
                self.$inner.as_slice_any()
            }
        }

        impl<'a, T> ::buffer::Storage for &'a mut $ty<T> where T: ::buffer::Storage {
            type Content = <T as ::buffer::Storage>::Content;

            #[inline]
            fn size(&self) -> usize {
                self.$inner.size()
            }

            #[inline]
            fn as_slice_any(&self) -> ::buffer::BufferAnySlice {
                self.$inner.as_slice_any()
            }
        }

        impl<'a, T> ::buffer::Map for &'a $ty<T> where &'a T: ::buffer::Map, T: ::buffer::Storage {
            type Mapping = <&'a T as ::buffer::Map>::Mapping;

            #[inline]
            fn map(self) -> <&'a T as ::buffer::Map>::Mapping {
                (&self.$inner).map()
            }
        }

        impl<'a, T> ::buffer::Map for &'a mut $ty<T>
            where &'a mut T: ::buffer::Map, T: ::buffer::Storage
        {
            type Mapping = <&'a mut T as ::buffer::Map>::Mapping;

            #[inline]
            fn map(self) -> <&'a mut T as ::buffer::Map>::Mapping {
                (&mut self.$inner).map()
            }
        }

        impl<T> ::buffer::Invalidate for $ty<T> where T: ::buffer::Invalidate {
            #[inline]
            fn invalidate(&self) {
                self.$inner.invalidate()
            }
        }

        impl<T> ::buffer::Read for $ty<T> where T: ::buffer::Read {
            #[inline]
            fn read(&self) -> Result<<T::Content as ::buffer::Content>::Owned, ::buffer::ReadError> {
                self.$inner.read()
            }
        }

        impl<T> ::buffer::Write for $ty<T> where T: ::buffer::Write {
            #[inline]
            fn write(&self, data: &T::Content) {
                self.$inner.write(data)
            }
        }

        impl<T> ::buffer::CopyTo for $ty<T> where T: ::buffer::CopyTo {
            #[inline]
            fn copy_to<S>(&self, target: &S) -> Result<(), ::buffer::CopyError>
                where T: ::buffer::CopyTo, S: ::buffer::Storage
            {
                self.$inner.copy_to(target)
            }
        }
    );
}
