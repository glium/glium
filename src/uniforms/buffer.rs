use buffer::{BufferView, BufferViewAny, BufferType, BufferCreationError};
use buffer::Mapping as BufferMapping;
use uniforms::{AsUniformValue, UniformValue, UniformBlock, UniformType};

use std::ops::{Deref, DerefMut};

use backend::Facade;

/// Buffer that contains a uniform block.
#[derive(Debug)]
pub struct UniformBuffer<T> where T: Copy + Send + 'static {
    buffer: BufferView<T>,
}

/// Mapping of a buffer in memory.
pub struct Mapping<'a, T> {
    mapping: BufferMapping<'a, T>,
}

/// Same as `UniformBuffer` but doesn't contain any information about the type.
#[derive(Debug)]
pub struct TypelessUniformBuffer {
    buffer: BufferViewAny,
}

impl<T> UniformBuffer<T> where T: Copy + Send + 'static {
    /// Uploads data in the uniforms buffer.
    ///
    /// # Features
    ///
    /// Only available if the `gl_uniform_blocks` feature is enabled.
    #[cfg(feature = "gl_uniform_blocks")]
    pub fn new<F>(facade: &F, data: T) -> UniformBuffer<T> where F: Facade {
        UniformBuffer::new_if_supported(facade, data).unwrap()
    }

    /// Uploads data in the uniforms buffer.
    pub fn new_if_supported<F>(facade: &F, data: T) -> Option<UniformBuffer<T>> where F: Facade {
        let buffer = match BufferView::new(facade, &[data], BufferType::UniformBuffer, true) {
            Ok(b) => b,
            Err(BufferCreationError::BufferTypeNotSupported) => return None,
            e @ Err(_) => e.unwrap(),
        };

        Some(UniformBuffer {
            buffer: buffer,
        })
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
    ///
    /// # Features
    ///
    /// Only available if the 'gl_read_buffer' feature is enabled.
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
    type Target = BufferView<T>;

    fn deref(&self) -> &BufferView<T> {
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
    fn deref_mut(&mut self) -> &mut BufferView<T> {
        &mut self.buffer
    }
}

impl<'a, T> AsUniformValue for &'a UniformBuffer<T> where T: UniformBlock + Send + Copy + 'static {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Block(self.buffer.as_slice_any(), <T as UniformBlock>::matches)
    }

    fn matches(_: &UniformType) -> bool {
        false
    }
}
