use Display;
use buffer::{self, Buffer, BufferType, BufferCreationError};
use uniforms::{IntoUniformValue, UniformValue, UniformBlock};

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::mpsc::Sender;

use GlObject;
use BufferExt;
use gl;
use context;
use sync;
use version::Api;

/// Buffer that contains a uniform block.
#[derive(Debug)]
pub struct UniformBuffer<T> {
    buffer: TypelessUniformBuffer,
    marker: PhantomData<T>,
}

/// Same as `UniformBuffer` but doesn't contain any information about the type.
#[derive(Debug)]
pub struct TypelessUniformBuffer {
    buffer: Buffer,
}

impl<T> UniformBuffer<T> where T: Copy + Send + 'static {
    /// Uploads data in the uniforms buffer.
    ///
    /// ## Features
    ///
    /// Only available if the `gl_uniform_blocks` feature is enabled.
    #[cfg(feature = "gl_uniform_blocks")]
    pub fn new(display: &Display, data: T) -> UniformBuffer<T> {
        let buffer = Buffer::new(display, vec![data], BufferType::UniformBuffer, false).unwrap();

        UniformBuffer {
            buffer: TypelessUniformBuffer {
                buffer: buffer,
            },
            marker: PhantomData,
        }
    }

    /// Uploads data in the uniforms buffer.
    pub fn new_if_supported(display: &Display, data: T) -> Option<UniformBuffer<T>> {
        UniformBuffer::new_impl(display, data, false)
    }

    /// Builds a new uniform buffer with persistent mapping.
    ///
    /// ## Features
    ///
    /// Only available if the `gl_uniform_blocks` and `gl_persistent_mapping` features are
    /// both enabled.
    #[cfg(all(feature = "gl_persistent_mapping", feature = "gl_uniform_blocks"))]
    pub fn new_persistent(display: &Display, data: T) -> UniformBuffer<T> {
        UniformBuffer::new_persistent_if_supported(display, data).unwrap()
    }

    /// Builds a new uniform buffer with persistent mapping, or `None` if this is not supported.
    pub fn new_persistent_if_supported(display: &Display, data: T) -> Option<UniformBuffer<T>> {
        UniformBuffer::new_impl(display, data, true)
    }

    /// Implementation of `new`.
    fn new_impl(display: &Display, data: T, persistent: bool) -> Option<UniformBuffer<T>> {
        if display.context.context.get_version() < &context::GlVersion(Api::Gl, 3, 1) &&
           !display.context.context.get_extensions().gl_arb_uniform_buffer_object
        {
            None

        } else {
            let buffer = match Buffer::new(display, vec![data], BufferType::UniformBuffer,
                                           persistent)
            {
                Err(BufferCreationError::PersistentMappingNotSupported) => return None,
                b => b.unwrap()
            };

            Some(UniformBuffer {
                buffer: TypelessUniformBuffer {
                    buffer: buffer,
                },
                marker: PhantomData,
            })
        }
    }

    /// Modifies the content of the buffer.
    pub fn upload(&mut self, data: T) {
        self.buffer.buffer.upload(0, vec![data])
    }

    /// Maps the buffer to allow write access to it.
    ///
    /// This function will block until the buffer stops being used by the backend.
    /// This operation is much faster if the buffer is persistent.
    pub fn map<'a>(&'a mut self) -> Mapping<'a, T> {
        Mapping(self.buffer.buffer.map(0, 1))
    }

    /// Reads the content of the buffer.
    ///
    /// This function is usually better if are just doing one punctual read, while `map`
    /// is better if you want to have multiple small reads.
    ///
    /// # Features
    ///
    /// Only available if the `gl_read_buffer` feature is enabled.
    #[cfg(feature = "gl_read_buffer")]
    pub fn read(&self) -> T {
        self.buffer.buffer.read().into_iter().next().unwrap()
    }

    /// Reads the content of the buffer.
    pub fn read_if_supported(&self) -> Option<T> {
        let res = self.buffer.buffer.read_if_supported();
        res.map(|res| res.into_iter().next().unwrap())
    }
}

impl<T> GlObject for UniformBuffer<T> {
    type Id = gl::types::GLuint;
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
    }
}

impl GlObject for TypelessUniformBuffer {
    type Id = gl::types::GLuint;
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
    }
}

impl<T> BufferExt for UniformBuffer<T> {
    fn add_fence(&self) -> Option<Sender<sync::LinearSyncFence>> {
        self.buffer.add_fence()
    }
}

impl BufferExt for TypelessUniformBuffer {
    fn add_fence(&self) -> Option<Sender<sync::LinearSyncFence>> {
        self.buffer.add_fence()
    }
}

/// A mapping of a uniform buffer.
pub struct Mapping<'a, T>(buffer::Mapping<'a, T>);

impl<'a, T> Deref for Mapping<'a, T> {
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T {
        self.0.deref().get(0).unwrap()
    }
}

impl<'a, T> DerefMut for Mapping<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
        self.0.deref_mut().get_mut(0).unwrap()
    }
}

impl<'a, T> IntoUniformValue<'a> for &'a UniformBuffer<T> where T: UniformBlock {
    fn into_uniform_value(self) -> UniformValue<'a> {
        UniformValue::Block(&self.buffer, <T as UniformBlock>::matches)
    }
}
