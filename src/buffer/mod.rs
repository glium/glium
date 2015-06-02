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
pub use self::view::{BufferViewSlice, BufferViewAnySlice, Mapping};

use gl;

mod alloc;
mod view;

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
