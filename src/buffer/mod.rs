//! # Buffers management in glium
//!
//! There are three levels of abstraction in glium:
//!
//!  - A `Buffer` corresponds to an OpenGL buffer object. This type is not public.
//!  - A `SubBuffer` corresponds to a part of a `Buffer`. One buffer can contain one or multiple
//!    subbuffers.
//!  - The `VertexBuffer`, `IndexBuffer`, `UniformBuffer`, `PixelBuffer`, ... types are
//!    abstractions over a subbuffer indicating their specific purpose. They implement `Deref`
//!    for the subbuffer. These types are in the `vertex`, `index`, ... modules.
//!
pub use self::builder::Builder;
pub use self::sub::{SubBuffer, SubBufferAny, SubBufferMutSlice};
pub use self::sub::{SubBufferSlice, SubBufferAnySlice, Mapping};

use gl;

mod alloc;
mod builder;
mod sub;

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
#[derive(Debug, Copy, Clone)]
pub enum BufferType {
    ArrayBuffer,
    PixelPackBuffer,
    PixelUnpackBuffer,
    UniformBuffer,
    CopyReadBuffer,
    CopyWriteBuffer,
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
        }
    }
}
