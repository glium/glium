//! A buffer is a memory location accessible to the video card.
//!
//! The purpose of buffers is to serve as a space where the GPU can read from or write data to.
//! It can contain a list of vertices, indices, uniform data, etc.
//!
//! # Buffers management in glium
//!
//! There are three levels of abstraction in glium:
//!
//!  - An `Alloc` corresponds to an OpenGL buffer object and is unsafe to use.
//!    This type is not public.
//!  - A `Buffer` wraps around an `Alloc` and provides safety by handling the data type and fences.
//!  - The `VertexBuffer`, `IndexBuffer`, `UniformBuffer`, `PixelBuffer`, etc. types are
//!    abstractions over a `Buffer` indicating their specific purpose. They implement `Deref`
//!    for the `Buffer`. These types are in the `vertex`, `index`, etc. modules.
//!
//! # Unsized types
//!
//! In order to put some data in a buffer, it must implement the `Content` trait. This trait is
//! automatically implemented on all `Sized` types and on slices (like `[u8]`). This means that
//! you can create a `Buffer<Foo>` (if `Foo` is sized) or a `Buffer<[u8]>` for example without
//! worrying about it.
//!
//! However unsized structs don't automatically implement this trait and you must call the
//! `implement_buffer_content!` macro on them. You must then use the `empty_unsized` constructor.
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
//!                                                               64, BufferMode::Default).unwrap();
//!
//! // you can then write to it like you normally would
//! buffer.map().data[4] = 2.1;
//! # }
//! ```
//!
pub use self::view::{Buffer, BufferAny, BufferMutSlice};
pub use self::view::{BufferSlice, BufferAnySlice};
pub use self::alloc::{Mapping, WriteMapping, ReadMapping, ReadError, CopyError};
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

    #[inline]
    fn read<F, E>(size: usize, f: F) -> Result<T, E> where F: FnOnce(&mut T) -> Result<(), E> {
        assert!(size == mem::size_of::<T>());
        // Note(Lokathor): This is brittle and dangerous if `T` isn't a type
        // that can be zeroed. However, it's a breaking change to adjust the API
        // here (eg: extra trait bound or something) so someone with more
        // authority than me needs to look at and fix this.
        let mut value = unsafe { mem::zeroed() };
        f(&mut value)?;
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
        f(&mut value)?;
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

    /// The mode to use when you modify a buffer multiple times per frame. Similar to `Default` in
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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
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
