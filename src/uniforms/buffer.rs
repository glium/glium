use buffer::{SubBuffer, SubBufferAny, BufferType};
use buffer::Mapping as BufferMapping;
use uniforms::{AsUniformValue, UniformValue, UniformBlock};

use std::ops::{Deref, DerefMut};

use backend::Facade;

use version::Version;
use version::Api;

/// Buffer that contains a uniform block.
#[derive(Debug)]
pub struct UniformBuffer<T> where T: Copy + Send + 'static {
    buffer: SubBuffer<T>,
}

/// Mapping of a buffer in memory.
pub struct Mapping<'a, T> {
    mapping: BufferMapping<'a, T>,
}

/// Same as `UniformBuffer` but doesn't contain any information about the type.
#[derive(Debug)]
pub struct TypelessUniformBuffer {
    buffer: SubBufferAny,
}

impl<T> UniformBuffer<T> where T: Copy + Send + 'static {
    /// Uploads data in the uniforms buffer.
    ///
    /// ## Features
    ///
    /// Only available if the `gl_uniform_blocks` feature is enabled.
    #[cfg(feature = "gl_uniform_blocks")]
    pub fn new<F>(facade: &F, data: T) -> UniformBuffer<T> where F: Facade {
        UniformBuffer::new_if_supported(facade, data).unwrap()
    }

    /// Uploads data in the uniforms buffer.
    pub fn new_if_supported<F>(facade: &F, data: T) -> Option<UniformBuffer<T>> where F: Facade {
        if facade.get_context().get_version() < &Version(Api::Gl, 3, 1) &&
           !facade.get_context().get_extensions().gl_arb_uniform_buffer_object
        {
            None

        } else {
            let buffer = SubBuffer::new(facade, &[data], BufferType::UniformBuffer, true).unwrap();

            Some(UniformBuffer {
                buffer: buffer,
            })
        }
    }

    /// Modifies the content of the buffer.
    pub fn upload(&mut self, data: T) {
        self.write(&[data]);
    }

    /// Reads the content of the buffer if supported.
    pub fn read_if_supported(&self) -> Option<T> {
        self.buffer.read_if_supported().and_then(|buf| buf.into_iter().next())
    }

    /// Reads the content of the buffer.
    #[cfg(feature = "gl_read_buffer")]
    pub fn read(&self) -> T {
        self.read_if_supported().unwrap()
    }

    /// Maps the buffer in memory.
    pub fn map(&mut self) -> Mapping<T> {
        Mapping { mapping: self.buffer.map() }
    }
}

impl<T> Deref for UniformBuffer<T> where T: Send + Copy + 'static {
    type Target = SubBuffer<T>;

    fn deref(&self) -> &SubBuffer<T> {
        &self.buffer
    }
}

impl<'a, D> Deref for Mapping<'a, D> {
    type Target = D;

    fn deref(&self) -> &D {
        &self.mapping.deref()[0]
    }
}

impl<'a, D> DerefMut for Mapping<'a, D> {
    fn deref_mut(&mut self) -> &mut D {
        &mut self.mapping.deref_mut()[0]
    }
}

impl<T> DerefMut for UniformBuffer<T> where T: Send + Copy + 'static {
    fn deref_mut(&mut self) -> &mut SubBuffer<T> {
        &mut self.buffer
    }
}

impl<'a, T> AsUniformValue for &'a UniformBuffer<T> where T: UniformBlock + Send + Copy + 'static {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Block(self.buffer.as_slice_any(), <T as UniformBlock>::matches)
    }
}
