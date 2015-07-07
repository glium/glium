use buffer::{BufferView, BufferViewAny, BufferType, BufferCreationError};
use uniforms::{AsUniformValue, UniformValue, UniformBlock, UniformType};

use std::ops::{Deref, DerefMut};

use backend::Facade;

/// Buffer that contains a uniform block.
#[derive(Debug)]
pub struct UniformBuffer<T> where T: Copy {
    buffer: BufferView<T>,
}

/// Same as `UniformBuffer` but doesn't contain any information about the type.
#[derive(Debug)]
pub struct TypelessUniformBuffer {
    buffer: BufferViewAny,
}

impl<T> UniformBuffer<T> where T: Copy {
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
        let buffer = match BufferView::new(facade, &data, BufferType::UniformBuffer, true) {
            Ok(b) => b,
            Err(BufferCreationError::BufferTypeNotSupported) => return None,
            e @ Err(_) => e.unwrap(),
        };

        Some(UniformBuffer {
            buffer: buffer,
        })
    }

    /// Creates an empty buffer.
    ///
    /// # Features
    ///
    /// Only available if the `gl_uniform_blocks` feature is enabled.
    #[cfg(feature = "gl_uniform_blocks")]
    pub fn empty<F>(facade: &F) -> UniformBuffer<T> where F: Facade {
        UniformBuffer::empty_if_supported(facade).unwrap()
    }

    /// Creates an empty buffer.
    pub fn empty_if_supported<F>(facade: &F) -> Option<UniformBuffer<T>> where F: Facade {
        let buffer = match BufferView::empty(facade, BufferType::UniformBuffer, true) {
            Ok(b) => b,
            Err(BufferCreationError::BufferTypeNotSupported) => return None,
            e @ Err(_) => e.unwrap(),
        };

        Some(UniformBuffer {
            buffer: buffer,
        })
    }
}

impl<T> Deref for UniformBuffer<T> where T: Copy {
    type Target = BufferView<T>;

    fn deref(&self) -> &BufferView<T> {
        &self.buffer
    }
}

impl<T> DerefMut for UniformBuffer<T> where T: Copy {
    fn deref_mut(&mut self) -> &mut BufferView<T> {
        &mut self.buffer
    }
}

impl<'a, T> AsUniformValue for &'a UniformBuffer<T> where T: UniformBlock + Copy {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Block(self.buffer.as_slice_any(), <T as UniformBlock>::matches)
    }

    fn matches(_: &UniformType) -> bool {
        false
    }
}
