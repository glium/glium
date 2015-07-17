use buffer::{BufferView, BufferViewSlice, BufferViewAny, BufferType};
use buffer::{BufferMode, BufferCreationError};
use gl;
use BufferViewExt;
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
    fn from(err: BufferCreationError) -> CreationError {
        CreationError::BufferCreationError(err)
    }
}

/// A list of indices loaded in the graphics card's memory.
#[derive(Debug)]
pub struct IndexBuffer<T> where T: Index {
    buffer: BufferView<[T]>,
    primitives: PrimitiveType,
}

impl<T> IndexBuffer<T> where T: Index {
    /// Builds a new index buffer from a list of indices and a primitive type.
    pub fn new<F>(facade: &F, prim: PrimitiveType, data: &[T])
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
            buffer: try!(BufferView::new(facade, data, BufferType::ElementArrayBuffer,
                                         BufferMode::Default)).into(),
            primitives: prim,
        })
    }

    /// Builds a new index buffer from a list of indices and a primitive type.
    pub fn dynamic<F>(facade: &F, prim: PrimitiveType, data: &[T])
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
            buffer: try!(BufferView::new(facade, data, BufferType::ElementArrayBuffer,
                                         BufferMode::Dynamic)).into(),
            primitives: prim,
        })
    }

    /// Builds a new empty index buffer.
    pub fn empty<F>(facade: &F, prim: PrimitiveType, len: usize)
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
            buffer: try!(BufferView::empty_array(facade, BufferType::ElementArrayBuffer, len,
                                                 BufferMode::Default)).into(),
            primitives: prim,
        })
    }

    /// Builds a new empty index buffer.
    pub fn empty_dynamic<F>(facade: &F, prim: PrimitiveType, len: usize)
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
            buffer: try!(BufferView::empty_array(facade, BufferType::ElementArrayBuffer, len,
                                                 BufferMode::Dynamic)).into(),
            primitives: prim,
        })
    }

    /// Returns the type of primitives associated with this index buffer.
    pub fn get_primitives_type(&self) -> PrimitiveType {
        self.primitives
    }

    /// Returns the data type of the indices inside this index buffer.
    pub fn get_indices_type(&self) -> IndexType {
        <T as Index>::get_type()
    }

    /// Returns `None` if out of range.
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
    type Target = BufferView<[T]>;

    fn deref(&self) -> &BufferView<[T]> {
        &self.buffer
    }
}

impl<T> DerefMut for IndexBuffer<T> where T: Index {
    fn deref_mut(&mut self) -> &mut BufferView<[T]> {
        &mut self.buffer
    }
}

// TODO: remove this
impl<T> GlObject for IndexBuffer<T> where T: Index {
    type Id = gl::types::GLuint;

    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_buffer_id()
    }
}

impl<'a, T> From<&'a IndexBuffer<T>> for IndicesSource<'a> where T: Index {
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
pub struct IndexBufferSlice<'a, T: 'a> where T: Index {
    buffer: BufferViewSlice<'a, [T]>,
    primitives: PrimitiveType,
}

impl<'a, T: 'a> IndexBufferSlice<'a, T> where T: Index {
    /// Returns the type of primitives associated with this index buffer.
    pub fn get_primitives_type(&self) -> PrimitiveType {
        self.primitives
    }

    /// Returns the data type of the indices inside this index buffer.
    pub fn get_indices_type(&self) -> IndexType {
        <T as Index>::get_type()
    }

    /// Returns `None` if out of range.
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
    type Target = BufferViewSlice<'a, [T]>;

    fn deref(&self) -> &BufferViewSlice<'a, [T]> {
        &self.buffer
    }
}

impl<'a, T> DerefMut for IndexBufferSlice<'a, T> where T: Index {
    fn deref_mut(&mut self) -> &mut BufferViewSlice<'a, [T]> {
        &mut self.buffer
    }
}

impl<'a, T> From<IndexBufferSlice<'a, T>> for IndicesSource<'a> where T: Index {
    fn from(buf: IndexBufferSlice<'a, T>) -> IndicesSource<'a> {
        IndicesSource::IndexBuffer {
            buffer: buf.buffer.as_slice_any(),
            data_type: buf.get_indices_type(),
            primitives: buf.primitives,
        }
    }
}

impl<'a, 'r, T> From<&'r IndexBufferSlice<'a, T>> for IndicesSource<'a> where T: Index {
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
    buffer: BufferViewAny,
    primitives: PrimitiveType,
    data_type: IndexType,
}

impl IndexBufferAny {
    /// Returns the type of primitives associated with this index buffer.
    pub fn get_primitives_type(&self) -> PrimitiveType {
        self.primitives
    }

    /// Returns the data type of the indices inside this index buffer.
    pub fn get_indices_type(&self) -> IndexType {
        self.data_type
    }
}

impl Deref for IndexBufferAny {
    type Target = BufferViewAny;

    fn deref(&self) -> &BufferViewAny {
        &self.buffer
    }
}

impl DerefMut for IndexBufferAny {
    fn deref_mut(&mut self) -> &mut BufferViewAny {
        &mut self.buffer
    }
}

impl<T> From<IndexBuffer<T>> for IndexBufferAny where T: Index {
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
    fn from(buf: &'a IndexBufferAny) -> IndicesSource<'a> {
        IndicesSource::IndexBuffer {
            buffer: buf.buffer.as_slice_any(),
            data_type: buf.data_type,
            primitives: buf.primitives,
        }
    }
}
