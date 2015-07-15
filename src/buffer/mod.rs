//! A buffer is a memory location accessible to the video card.
//!
//! The purpose of buffers is to serve as a space where the GPU can read from or write data to.
//! It can contain a list of vertices, indices, uniform data, etc.
//!
//! # Buffers management in glium
//!
//! There are three levels of abstraction in glium:
//!
//!  - A `Buffer` corresponds to an OpenGL buffer object. This type is not public.
//!  - A `BufferView` corresponds to a part of a `Buffer`. One buffer can contain one or multiple
//!    subbuffers.
//!  - The `VertexBuffer`, `IndexBuffer`, `UniformBuffer`, `PixelBuffer`, ... types are
//!    abstractions over a subbuffer indicating their specific purpose. They implement `Deref`
//!    for the subbuffer. These types are in the `vertex`, `index`, ... modules.
//!
pub use self::view::{BufferView, BufferViewAny, BufferViewMutSlice};
pub use self::view::{BufferViewSlice, BufferViewAnySlice};
pub use self::alloc::{Mapping, WriteMapping, ReadMapping, ReadError, is_buffer_read_supported};
pub use self::fences::Inserter;

use gl;
use std::mem;
use std::slice;

mod alloc;
mod fences;
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

    fn read<F, E>(size: usize, f: F) -> Result<T, E> where F: FnOnce(&mut T) -> Result<(), E> {
        assert!(size == mem::size_of::<T>());
        let mut value = unsafe { mem::uninitialized() };
        try!(f(&mut value));
        Ok(value)
    }

    fn get_elements_size() -> usize {
        mem::size_of::<T>()
    }

    fn to_void_ptr(&self) -> *const () {
        self as *const T as *const ()
    }

    fn ref_from_ptr<'a>(ptr: *mut (), size: usize) -> Option<*mut T> {
        if size != mem::size_of::<T>() {
            return None;
        }

        Some(ptr as *mut T)
    }

    fn is_size_suitable(size: usize) -> bool {
        size == mem::size_of::<T>()
    }
}

unsafe impl<T> Content for [T] where T: Copy {
    type Owned = Vec<T>;

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

    fn get_elements_size() -> usize {
        mem::size_of::<T>()
    }

    fn to_void_ptr(&self) -> *const () {
        &self[0] as *const T as *const ()
    }

    fn ref_from_ptr<'a>(ptr: *mut (), size: usize) -> Option<*mut [T]> {
        if size % mem::size_of::<T>() != 0 {
            return None;
        }

        let ptr = ptr as *mut T;
        let size = size / mem::size_of::<T>();
        Some(unsafe { slice::from_raw_parts_mut(&mut *ptr, size) as *mut [T] })
    }

    fn is_size_suitable(size: usize) -> bool {
        size % mem::size_of::<T>() == 0
    }
}

/// Error that can happen when creating a buffer.
#[derive(Debug)]
pub enum BufferCreationError {
    /// Not enough memory to create the buffer.
    OutOfMemory,

    /// This type of buffer is not supported.
    BufferTypeNotSupported,
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
