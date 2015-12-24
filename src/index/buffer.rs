use buffer::{Buffer, BufferSlice, BufferMutSlice, BufferAny, BufferType};
use buffer::{BufferMode, BufferCreationError};

use buffer::BufferAnySlice;
use buffer::Create as BufferCreate;
use buffer::Storage as BufferStorage;
use buffer::Content as BufferContent;
use buffer::Invalidate as BufferInvalidate;
use buffer::ImmutableBuffer;
use buffer::ImmutableBufferSlice;
use buffer::ImmutableBufferMutSlice;
use buffer::PersistentBuffer;
use buffer::PersistentBufferSlice;
use buffer::PersistentBufferMutSlice;

use gl;
use BufferExt;
use GlObject;

use backend::Facade;

use index::IndicesSource;
use index::Index;
use index::IndexType;
use index::PrimitiveType;

use std::ops::{Deref, DerefMut};
use std::fmt;
use std::error::Error;
use utils::range::RangeArgument;

pub type IndexBuffer<T> = IndexStorage<Buffer<[T]>>;
pub type IndexBufferSlice<'a, T> = IndexStorage<BufferSlice<'a, [T]>>;
pub type IndexBufferMutSlice<'a, T> = IndexStorage<BufferMutSlice<'a, [T]>>;
pub type ImmutableIndexBuffer<T> = IndexStorage<ImmutableBuffer<[T]>>;
pub type ImmutableIndexBufferSlice<'a, T> = IndexStorage<ImmutableBufferSlice<'a, [T]>>;
pub type ImmutableIndexBufferMutSlice<'a, T> = IndexStorage<ImmutableBufferMutSlice<'a, [T]>>;
pub type PersistentIndexBuffer<T> = IndexStorage<PersistentBuffer<[T]>>;
pub type PersistentIndexBufferSlice<'a, T> = IndexStorage<PersistentBufferSlice<'a, [T]>>;
pub type PersistentIndexBufferMutSlice<'a, T> = IndexStorage<PersistentBufferMutSlice<'a, [T]>>;

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

impl fmt::Display for CreationError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

impl Error for CreationError {
    fn description(&self) -> &str {
        use self::CreationError::*;
        match *self {
            IndexTypeNotSupported =>
                "The type of index is not supported by the backend",
            PrimitiveTypeNotSupported =>
                "The type of primitives is not supported by the backend",
            BufferCreationError(_) =>
                "An error happened while creating the buffer",
        }
    }

    fn cause(&self) -> Option<&Error> {
        use self::CreationError::*;
        match *self {
            BufferCreationError(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<BufferCreationError> for CreationError {
    #[inline]
    fn from(err: BufferCreationError) -> CreationError {
        CreationError::BufferCreationError(err)
    }
}

/// A list of indices loaded in the graphics card's memory.
#[derive(Copy, Clone, Debug)]
pub struct IndexStorage<T> {
    buffer: T,
    primitives: PrimitiveType,
}

impl<T, I> IndexStorage<T> where T: BufferCreate<Content = [I]>, I: Index + BufferContent + Copy {
    /// Builds a new index buffer from a list of indices and a primitive type.
    #[inline]
    pub fn new<F>(facade: &F, prim: PrimitiveType, data: &[I])
                  -> Result<IndexStorage<T>, CreationError>
                  where F: Facade
    {
        if !prim.is_supported(facade) {
            return Err(CreationError::PrimitiveTypeNotSupported);
        }

        if !<I as Index>::is_supported(facade) {
            return Err(CreationError::IndexTypeNotSupported);
        }

        let buffer = try!(BufferCreate::new(facade, data, BufferType::ElementArrayBuffer));

        Ok(IndexStorage {
            buffer: buffer,
            primitives: prim,
        })
    }

    /// Builds a new index buffer from a list of indices and a primitive type.
    #[inline]
    pub fn empty<F>(facade: &F, prim: PrimitiveType, len: usize)
                    -> Result<IndexStorage<T>, CreationError>
                    where F: Facade
    {
        if !prim.is_supported(facade) {
            return Err(CreationError::PrimitiveTypeNotSupported);
        }

        if !<I as Index>::is_supported(facade) {
            return Err(CreationError::IndexTypeNotSupported);
        }

        let buffer = try!(BufferCreate::empty_array(facade, len, BufferType::ElementArrayBuffer));

        Ok(IndexStorage {
            buffer: buffer,
            primitives: prim,
        })
    }
}

impl<T, I> IndexStorage<T> where T: BufferStorage<Content = [I]>, [I]: BufferContent {
    #[inline]
    pub fn from_buffer(buffer: T, prim: PrimitiveType) -> IndexStorage<T> {
        IndexStorage {
            buffer: buffer,
            primitives: prim,
        }
    }
}

impl_buffer_wrapper!(IndexStorage, buffer);

impl<T> IndexStorage<T> {
    /// Returns the type of primitives associated with this index buffer.
    #[inline]
    pub fn get_primitives_type(&self) -> PrimitiveType {
        self.primitives
    }
}

impl<T, I> IndexStorage<T>
    where T: BufferStorage<Content = [I]>, I: Index
{
    /// Returns the data type of the indices inside this index buffer.
    #[inline]
    pub fn get_indices_type(&self) -> IndexType {
        <I as Index>::get_type()
    }
}

impl<'a, F, T> From<&'a IndexStorage<F>> for IndexStorage<T>
    where T: From<&'a F>
{
    #[inline]
    fn from(from: &'a IndexStorage<F>) -> IndexStorage<T> {
        IndexStorage {
            buffer: From::from(&from.buffer),
            primitives: from.primitives,
        }
    }
}

impl<'a, T, I> From<&'a IndexStorage<T>> for IndicesSource<'a>
    where T: BufferStorage<Content = [I]>, &'a T: Into<BufferAnySlice<'a>>, I: Index
{
    #[inline]
    fn from(buf: &'a IndexStorage<T>) -> IndicesSource<'a> {
        unsafe {
            IndicesSource::from_index_buffer((&buf.buffer).into(), buf.get_indices_type(),
                                             buf.primitives)
        }
    }
}

impl<'a, T, I> From<IndexStorage<T>> for IndicesSource<'a>
    where T: 'a + BufferStorage<Content = [I]>, T: Into<BufferAnySlice<'a>>, I: Index
{
    #[inline]
    fn from(buf: IndexStorage<T>) -> IndicesSource<'a> {
        unsafe {
            let indices_type = buf.get_indices_type();
            IndicesSource::from_index_buffer(buf.buffer.into(), indices_type, buf.primitives)
        }
    }
}
