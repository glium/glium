use buffer::{Content, Buffer, BufferAny, BufferType, BufferMode, BufferCreationError};
use buffer::{BufferSlice, BufferMutSlice};
use uniforms::{AsUniformValue, UniformBlock, UniformValue, LayoutMismatchError};
use program;

use gl;
use GlObject;

use std::ops::{Deref, DerefMut};

use backend::Facade;

/// Buffer that contains a uniform block.
///
/// For example, to use a `UniformBuffer<[u32; 8]>`, you must declare it as
///
///     uniform MyBlock {
///         uint array[8];
///     };
///
/// and pass it to `uniform!` like this:
///
///     uniform! {
///         MyBlock: &buffer,
///     }
#[derive(Debug)]
pub struct UniformBuffer<T: ?Sized> where T: Content {
    buffer: Buffer<T>,
}

/// Same as `UniformBuffer` but doesn't contain any information about the type.
#[derive(Debug)]
pub struct TypelessUniformBuffer {
    buffer: BufferAny,
}

impl<T: ?Sized + Content> GlObject for UniformBuffer<T> {
    type Id = gl::types::GLuint;

    #[inline]
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
    }
}

impl<T> UniformBuffer<T> where T: Copy {
    /// Uploads data in the uniforms buffer.
    #[inline]
    pub fn new<F: ?Sized>(facade: &F, data: T) -> Result<UniformBuffer<T>, BufferCreationError>
                  where F: Facade
    {
        UniformBuffer::new_impl(facade, data, BufferMode::Default)
    }

    /// Uploads data in the uniforms buffer.
    #[inline]
    pub fn dynamic<F: ?Sized>(facade: &F, data: T) -> Result<UniformBuffer<T>, BufferCreationError>
                      where F: Facade
    {
        UniformBuffer::new_impl(facade, data, BufferMode::Dynamic)
    }

    /// Uploads data in the uniforms buffer.
    #[inline]
    pub fn persistent<F: ?Sized>(facade: &F, data: T) -> Result<UniformBuffer<T>, BufferCreationError>
                  where F: Facade
    {
        UniformBuffer::new_impl(facade, data, BufferMode::Persistent)
    }

    /// Uploads data in the uniforms buffer.
    #[inline]
    pub fn immutable<F: ?Sized>(facade: &F, data: T) -> Result<UniformBuffer<T>, BufferCreationError>
                        where F: Facade
    {
        UniformBuffer::new_impl(facade, data, BufferMode::Immutable)
    }

    #[inline]
    fn new_impl<F: ?Sized>(facade: &F, data: T, mode: BufferMode)
                   -> Result<UniformBuffer<T>, BufferCreationError>
                   where F: Facade
    {
        let buffer = Buffer::new(facade, &data, BufferType::UniformBuffer, mode)?;

        Ok(UniformBuffer {
            buffer: buffer,
        })
    }

    /// Creates an empty buffer.
    #[inline]
    pub fn empty<F: ?Sized>(facade: &F) -> Result<UniformBuffer<T>, BufferCreationError> where F: Facade {
        UniformBuffer::empty_impl(facade, BufferMode::Default)
    }

    /// Creates an empty buffer.
    #[inline]
    pub fn empty_dynamic<F: ?Sized>(facade: &F) -> Result<UniformBuffer<T>, BufferCreationError>
                            where F: Facade
    {
        UniformBuffer::empty_impl(facade, BufferMode::Dynamic)
    }

    /// Creates an empty buffer.
    #[inline]
    pub fn empty_persistent<F: ?Sized>(facade: &F) -> Result<UniformBuffer<T>, BufferCreationError>
                               where F: Facade
    {
        UniformBuffer::empty_impl(facade, BufferMode::Persistent)
    }

    /// Creates an empty buffer.
    #[inline]
    pub fn empty_immutable<F: ?Sized>(facade: &F) -> Result<UniformBuffer<T>, BufferCreationError>
                              where F: Facade
    {
        UniformBuffer::empty_impl(facade, BufferMode::Immutable)
    }

    #[inline]
    fn empty_impl<F: ?Sized>(facade: &F, mode: BufferMode) -> Result<UniformBuffer<T>, BufferCreationError>
                     where F: Facade
    {
        let buffer = Buffer::empty(facade, BufferType::UniformBuffer, mode)?;

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
    /// Panics if the size passed as parameter is not suitable for the type of data.
    ///
    #[inline]
    pub fn empty_unsized<F: ?Sized>(facade: &F, size: usize)
                            -> Result<UniformBuffer<T>, BufferCreationError>
                            where F: Facade
    {
        UniformBuffer::empty_unsized_impl(facade, size, BufferMode::Default)
    }

    /// Creates an empty buffer.
    ///
    /// # Panic
    ///
    /// Panics if the size passed as parameter is not suitable for the type of data.
    ///
    #[inline]
    pub fn empty_unsized_dynamic<F: ?Sized>(facade: &F, size: usize)
                                    -> Result<UniformBuffer<T>, BufferCreationError>
                                    where F: Facade
    {
        UniformBuffer::empty_unsized_impl(facade, size, BufferMode::Dynamic)
    }

    /// Creates an empty buffer.
    ///
    /// # Panic
    ///
    /// Panics if the size passed as parameter is not suitable for the type of data.
    ///
    #[inline]
    pub fn empty_unsized_persistent<F: ?Sized>(facade: &F, size: usize)
                                       -> Result<UniformBuffer<T>, BufferCreationError>
                                       where F: Facade
    {
        UniformBuffer::empty_unsized_impl(facade, size, BufferMode::Persistent)
    }

    /// Creates an empty buffer.
    ///
    /// # Panic
    ///
    /// Panics if the size passed as parameter is not suitable for the type of data.
    ///
    #[inline]
    pub fn empty_unsized_immutable<F: ?Sized>(facade: &F, size: usize)
                                      -> Result<UniformBuffer<T>, BufferCreationError>
                                      where F: Facade
    {
        UniformBuffer::empty_unsized_impl(facade, size, BufferMode::Immutable)
    }

    #[inline]
    fn empty_unsized_impl<F: ?Sized>(facade: &F, size: usize, mode: BufferMode)
                             -> Result<UniformBuffer<T>, BufferCreationError>
                             where F: Facade
    {
        let buffer = Buffer::empty_unsized(facade, BufferType::UniformBuffer, size, mode)?;

        Ok(UniformBuffer {
            buffer: buffer,
        })
    }
}

impl<T: ?Sized> Deref for UniformBuffer<T> where T: Content {
    type Target = Buffer<T>;

    #[inline]
    fn deref(&self) -> &Buffer<T> {
        &self.buffer
    }
}

impl<T: ?Sized> DerefMut for UniformBuffer<T> where T: Content {
    #[inline]
    fn deref_mut(&mut self) -> &mut Buffer<T> {
        &mut self.buffer
    }
}

impl<'a, T: ?Sized> From<&'a UniformBuffer<T>> for BufferSlice<'a, T> where T: Content {
    #[inline]
    fn from(b: &'a UniformBuffer<T>) -> BufferSlice<'a, T> {
        b.buffer.as_slice()
    }
}

impl<'a, T: ?Sized> From<&'a mut UniformBuffer<T>> for BufferMutSlice<'a, T> where T: Content {
    #[inline]
    fn from(b: &'a mut UniformBuffer<T>) -> BufferMutSlice<'a, T> {
        b.buffer.as_mut_slice()
    }
}

impl<'a, T: ?Sized> AsUniformValue for &'a UniformBuffer<T> where T: UniformBlock + Content {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        #[inline]
        fn f<T: ?Sized>(block: &program::UniformBlock)
                        -> Result<(), LayoutMismatchError> where T: UniformBlock + Content
        {
            // TODO: more checks?
            T::matches(&block.layout, 0)
        }

        UniformValue::Block(self.buffer.as_slice_any(), f::<T>)
    }
}
