use buffer::{Buffer, BufferSlice, BufferMutSlice, BufferAny, BufferType};
use buffer::{BufferMode, BufferCreationError};
use gl;
use BufferExt;
use GlObject;

use backend::Facade;

use index::IndicesSource;
use index::Index;
use index::IndexType;
use index::PrimitiveType;

use std::ops::{Deref, DerefMut, Range};

/// Error that can happen while creating an index buffer.
#[derive(Debug, Copy, Clone)]
pub enum CreationError {
    /// The type of index is not supported by the backend.
    IndexTypeNotSupported,

    /// The type of primitives is not supported by the backend.
    PrimitiveTypeNotSupported,

    /// An error happened while creating the buffer.
    BufferCreationError(BufferCreationError),
}

impl From<BufferCreationError> for CreationError {
    #[inline]
    fn from(err: BufferCreationError) -> CreationError {
        CreationError::BufferCreationError(err)
    }
}

/// A list of indices loaded in the graphics card's memory.
#[derive(Debug)]
pub struct IndexBuffer<T>
    where T: Index
{
    buffer: Buffer<[T]>,
    primitives: PrimitiveType,
}

impl<T> IndexBuffer<T> where T: Index {
    /// Builds a new index buffer from a list of indices and a primitive type.
    #[inline]
    pub fn new<F>(facade: &F,
                  prim: PrimitiveType,
                  data: &[T])
                  -> Result<IndexBuffer<T>, CreationError>
        where F: Facade
    {
        IndexBuffer::new_impl(facade, prim, data, BufferMode::Default)
    }

    /// Builds a new index buffer from a list of indices and a primitive type.
    #[inline]
    pub fn dynamic<F>(facade: &F,
                      prim: PrimitiveType,
                      data: &[T])
                      -> Result<IndexBuffer<T>, CreationError>
        where F: Facade
    {
        IndexBuffer::new_impl(facade, prim, data, BufferMode::Dynamic)
    }

    /// Builds a new index buffer from a list of indices and a primitive type.
    #[inline]
    pub fn persistent<F>(facade: &F,
                         prim: PrimitiveType,
                         data: &[T])
                         -> Result<IndexBuffer<T>, CreationError>
        where F: Facade
    {
        IndexBuffer::new_impl(facade, prim, data, BufferMode::Persistent)
    }

    /// Builds a new index buffer from a list of indices and a primitive type.
    #[inline]
    pub fn immutable<F>(facade: &F,
                        prim: PrimitiveType,
                        data: &[T])
                        -> Result<IndexBuffer<T>, CreationError>
        where F: Facade
    {
        IndexBuffer::new_impl(facade, prim, data, BufferMode::Immutable)
    }

    #[inline]
    fn new_impl<F>(facade: &F,
                   prim: PrimitiveType,
                   data: &[T],
                   mode: BufferMode)
                   -> Result<IndexBuffer<T>, CreationError>
        where F: Facade
    {
        if !prim.is_supported(facade) {
            return Err(CreationError::PrimitiveTypeNotSupported);
        }

        if !T::is_supported(facade) {
            return Err(CreationError::IndexTypeNotSupported);
        }

        Ok(IndexBuffer {
            buffer: try!(Buffer::new(facade, data, BufferType::ElementArrayBuffer, mode)).into(),
            primitives: prim,
        })
    }

    /// Builds a new empty index buffer.
    #[inline]
    pub fn empty<F>(facade: &F,
                    prim: PrimitiveType,
                    len: usize)
                    -> Result<IndexBuffer<T>, CreationError>
        where F: Facade
    {
        IndexBuffer::empty_impl(facade, prim, len, BufferMode::Default)
    }

    /// Builds a new empty index buffer.
    #[inline]
    pub fn empty_dynamic<F>(facade: &F,
                            prim: PrimitiveType,
                            len: usize)
                            -> Result<IndexBuffer<T>, CreationError>
        where F: Facade
    {
        IndexBuffer::empty_impl(facade, prim, len, BufferMode::Dynamic)
    }

    /// Builds a new empty index buffer.
    #[inline]
    pub fn empty_persistent<F>(facade: &F,
                               prim: PrimitiveType,
                               len: usize)
                               -> Result<IndexBuffer<T>, CreationError>
        where F: Facade
    {
        IndexBuffer::empty_impl(facade, prim, len, BufferMode::Persistent)
    }

    /// Builds a new empty index buffer.
    #[inline]
    pub fn empty_immutable<F>(facade: &F,
                              prim: PrimitiveType,
                              len: usize)
                              -> Result<IndexBuffer<T>, CreationError>
        where F: Facade
    {
        IndexBuffer::empty_impl(facade, prim, len, BufferMode::Immutable)
    }

    #[inline]
    fn empty_impl<F>(facade: &F,
                     prim: PrimitiveType,
                     len: usize,
                     mode: BufferMode)
                     -> Result<IndexBuffer<T>, CreationError>
        where F: Facade
    {
        if !prim.is_supported(facade) {
            return Err(CreationError::PrimitiveTypeNotSupported);
        }

        if !T::is_supported(facade) {
            return Err(CreationError::IndexTypeNotSupported);
        }

        Ok(IndexBuffer {
            buffer: try!(Buffer::empty_array(facade, BufferType::ElementArrayBuffer, len, mode))
                        .into(),
            primitives: prim,
        })
    }

    /// Returns the type of primitives associated with this index buffer.
    #[inline]
    pub fn get_primitives_type(&self) -> PrimitiveType {
        self.primitives
    }

    /// Returns the data type of the indices inside this index buffer.
    #[inline]
    pub fn get_indices_type(&self) -> IndexType {
        <T as Index>::get_type()
    }

    /// Returns `None` if out of range.
    #[inline]
    pub fn slice(&self, range: Range<usize>) -> Option<IndexBufferSlice<T>> {
        self.buffer.slice(range).map(|b| {
            IndexBufferSlice {
                buffer: b,
                primitives: self.primitives,
            }
        })
    }
}

impl<T> Deref for IndexBuffer<T> where T: Index {
    type Target = Buffer<[T]>;

    #[inline]
    fn deref(&self) -> &Buffer<[T]> {
        &self.buffer
    }
}

impl<T> DerefMut for IndexBuffer<T> where T: Index {
    #[inline]
    fn deref_mut(&mut self) -> &mut Buffer<[T]> {
        &mut self.buffer
    }
}

impl<'a, T> From<&'a IndexBuffer<T>> for BufferSlice<'a, [T]> where T: Index {
    #[inline]
    fn from(b: &'a IndexBuffer<T>) -> BufferSlice<'a, [T]> {
        let b: &Buffer<[T]> = &*b;
        b.as_slice()
    }
}

impl<'a, T> From<&'a mut IndexBuffer<T>> for BufferMutSlice<'a, [T]> where T: Index {
    #[inline]
    fn from(b: &'a mut IndexBuffer<T>) -> BufferMutSlice<'a, [T]> {
        let b: &mut Buffer<[T]> = &mut *b;
        b.as_mut_slice()
    }
}

// TODO: remove this
impl<T> GlObject for IndexBuffer<T> where T: Index {
    type Id = gl::types::GLuint;

    #[inline]
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_buffer_id()
    }
}

impl<'a, T> From<&'a IndexBuffer<T>> for IndicesSource<'a> where T: Index {
    #[inline]
    fn from(buf: &'a IndexBuffer<T>) -> IndicesSource<'a> {
        IndicesSource::IndexBuffer {
            buffer: buf.buffer.as_slice_any(),
            data_type: buf.get_indices_type(),
            primitives: buf.primitives,
        }
    }
}

/// Slice of an `IndexBuffer`.
#[derive(Debug)]
pub struct IndexBufferSlice<'a, T: 'a>
    where T: Index
{
    buffer: BufferSlice<'a, [T]>,
    primitives: PrimitiveType,
}

impl<'a, T: 'a> IndexBufferSlice<'a, T> where T: Index {
    /// Returns the type of primitives associated with this index buffer.
    #[inline]
    pub fn get_primitives_type(&self) -> PrimitiveType {
        self.primitives
    }

    /// Returns the data type of the indices inside this index buffer.
    #[inline]
    pub fn get_indices_type(&self) -> IndexType {
        <T as Index>::get_type()
    }

    /// Returns `None` if out of range.
    #[inline]
    pub fn slice(&self, range: Range<usize>) -> Option<IndexBufferSlice<'a, T>> {
        self.buffer.slice(range).map(|b| {
            IndexBufferSlice {
                buffer: b,
                primitives: self.primitives,
            }
        })
    }
}

impl<'a, T> Deref for IndexBufferSlice<'a, T> where T: Index {
    type Target = BufferSlice<'a, [T]>;

    #[inline]
    fn deref(&self) -> &BufferSlice<'a, [T]> {
        &self.buffer
    }
}

impl<'a, T> DerefMut for IndexBufferSlice<'a, T> where T: Index {
    #[inline]
    fn deref_mut(&mut self) -> &mut BufferSlice<'a, [T]> {
        &mut self.buffer
    }
}

impl<'a, T> From<IndexBufferSlice<'a, T>> for BufferSlice<'a, [T]> where T: Index {
    #[inline]
    fn from(b: IndexBufferSlice<'a, T>) -> BufferSlice<'a, [T]> {
        b.buffer
    }
}

impl<'a, T> From<IndexBufferSlice<'a, T>> for IndicesSource<'a> where T: Index {
    #[inline]
    fn from(buf: IndexBufferSlice<'a, T>) -> IndicesSource<'a> {
        IndicesSource::IndexBuffer {
            buffer: buf.buffer.as_slice_any(),
            data_type: buf.get_indices_type(),
            primitives: buf.primitives,
        }
    }
}

impl<'a, 'r, T> From<&'r IndexBufferSlice<'a, T>> for IndicesSource<'a> where T: Index {
    #[inline]
    fn from(buf: &'r IndexBufferSlice<'a, T>) -> IndicesSource<'a> {
        IndicesSource::IndexBuffer {
            buffer: buf.buffer.as_slice_any(),
            data_type: buf.get_indices_type(),
            primitives: buf.primitives,
        }
    }
}

/// An `IndexBuffer` without any type information.
///
/// Makes it easier to store in a `Vec` or return from a function, for example.
#[derive(Debug)]
pub struct IndexBufferAny {
    buffer: BufferAny,
    primitives: PrimitiveType,
    data_type: IndexType,
}

impl IndexBufferAny {
    /// Returns the type of primitives associated with this index buffer.
    #[inline]
    pub fn get_primitives_type(&self) -> PrimitiveType {
        self.primitives
    }

    /// Returns the data type of the indices inside this index buffer.
    #[inline]
    pub fn get_indices_type(&self) -> IndexType {
        self.data_type
    }
}

impl Deref for IndexBufferAny {
    type Target = BufferAny;

    #[inline]
    fn deref(&self) -> &BufferAny {
        &self.buffer
    }
}

impl DerefMut for IndexBufferAny {
    #[inline]
    fn deref_mut(&mut self) -> &mut BufferAny {
        &mut self.buffer
    }
}

impl<T> From<IndexBuffer<T>> for IndexBufferAny where T: Index {
    #[inline]
    fn from(buffer: IndexBuffer<T>) -> IndexBufferAny {
        let ty = buffer.get_indices_type();

        IndexBufferAny {
            buffer: buffer.buffer.into(),
            data_type: ty,
            primitives: buffer.primitives,
        }
    }
}

impl<'a> From<&'a IndexBufferAny> for IndicesSource<'a> {
    #[inline]
    fn from(buf: &'a IndexBufferAny) -> IndicesSource<'a> {
        IndicesSource::IndexBuffer {
            buffer: buf.buffer.as_slice_any(),
            data_type: buf.data_type,
            primitives: buf.primitives,
        }
    }
}
