use buffer::{Content, BufferView, BufferViewAny, BufferType, BufferCreationError};
use uniforms::{AsUniformValue, UniformBlock, UniformValue, LayoutMismatchError};
use program;

use std::ops::{Deref, DerefMut};

use backend::Facade;

/// Buffer that contains a uniform block.
#[derive(Debug)]
pub struct UniformBuffer<T: ?Sized> where T: Content {
    buffer: BufferView<T>,
}

/// Same as `UniformBuffer` but doesn't contain any information about the type.
#[derive(Debug)]
pub struct TypelessUniformBuffer {
    buffer: BufferViewAny,
}

impl<T> UniformBuffer<T> where T: Copy {
    /// Uploads data in the uniforms buffer.
    pub fn new<F>(facade: &F, data: T) -> Result<UniformBuffer<T>, BufferCreationError>
                  where F: Facade
    {
        let buffer = try!(BufferView::new(facade, &data, BufferType::UniformBuffer, true));

        Ok(UniformBuffer {
            buffer: buffer,
        })
    }

    /// Creates an empty buffer.
    pub fn empty<F>(facade: &F) -> Result<UniformBuffer<T>, BufferCreationError> where F: Facade {
        let buffer = try!(BufferView::empty(facade, BufferType::UniformBuffer, true));

        Ok(UniformBuffer {
            buffer: buffer,
        })
    }
}

impl<T: ?Sized> UniformBuffer<T> where T: Content {
    /// Creates an empty buffer.
    ///
    /// # Panic
    ///
    /// Panicks if the size passed as parameter is not suitable for the type of data.
    ///
    pub fn empty_unsized<F>(facade: &F, size: usize)
                            -> Result<UniformBuffer<T>, BufferCreationError>
                            where F: Facade
    {
        let buffer = try!(BufferView::empty_unsized(facade, BufferType::UniformBuffer, size, true));

        Ok(UniformBuffer {
            buffer: buffer,
        })
    }
}

impl<T: ?Sized> Deref for UniformBuffer<T> where T: Content {
    type Target = BufferView<T>;

    fn deref(&self) -> &BufferView<T> {
        &self.buffer
    }
}

impl<T: ?Sized> DerefMut for UniformBuffer<T> where T: Content {
    fn deref_mut(&mut self) -> &mut BufferView<T> {
        &mut self.buffer
    }
}

impl<'a, T: ?Sized> AsUniformValue for &'a UniformBuffer<T> where T: UniformBlock + Content {
    fn as_uniform_value(&self) -> UniformValue {
        fn f<T: ?Sized>(block: &program::UniformBlock)
                        -> Result<(), LayoutMismatchError> where T: UniformBlock + Content
        {
            // TODO: more checks?
            T::matches(&block.layout, 0)
        }

        UniformValue::Block(self.buffer.as_slice_any(), f::<T>)
    }
}
