use buffer::{self, Buffer};
use gl;
use BufferExt;
use Display;
use GlObject;

use sync;

use index::IndicesSource;
use index::ToIndicesSource;
use index::IntoIndexBuffer;
use index::Index;
use index::IndexType;
use index::PrimitiveType;

use std::mem;
use std::sync::mpsc::Sender;

/// A list of indices loaded in the graphics card's memory.
#[derive(Debug)]
pub struct IndexBuffer {
    buffer: Buffer,
    data_type: IndexType,
    primitives: PrimitiveType,
}

pub struct IndexBufferSlice<'a> {
    buffer: &'a IndexBuffer,
    offset: usize,  // in number of elements
    len: usize,     // in number of elements
}

impl IndexBuffer {
    /// Builds a new index buffer.
    pub fn new<T: IntoIndexBuffer>(display: &Display, data: T) -> IndexBuffer {
        data.into_index_buffer(display)
    }

    /// Builds a new index buffer from raw data and a primitive type.
    pub fn from_raw<T>(display: &Display, data: Vec<T>, prim: PrimitiveType) -> IndexBuffer
                       where T: Index
    {
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, data, false).unwrap(),
            data_type: <T as Index>::get_type(),
            primitives: prim,
        }
    }

    /// Returns the type of primitives associated with this index buffer.
    pub fn get_primitives_type(&self) -> PrimitiveType {
        self.primitives
    }

    /// Returns the data type of the indices inside this index buffer.
    pub fn get_indices_type(&self) -> IndexType {
        self.data_type
    }

    /// Returns `None` if out of range.
    pub fn slice(&self, offset: usize, len: usize) -> Option<IndexBufferSlice> {
        if offset > self.buffer.get_elements_count() ||
            offset + len > self.buffer.get_elements_count()
        {
            return None;
        }

        Some(IndexBufferSlice {
            buffer: self,
            offset: offset,
            len: len,
        })
    }
}

impl BufferExt for IndexBuffer {
    fn add_fence(&self) -> Option<Sender<sync::LinearSyncFence>> {
        self.buffer.add_fence()
    }
}

impl GlObject for IndexBuffer {
    type Id = gl::types::GLuint;
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
    }
}

impl ToIndicesSource for IndexBuffer {
    type Data = u16;      // TODO: u16?

    fn to_indices_source(&self) -> IndicesSource<u16> {     // TODO: u16?
        IndicesSource::IndexBuffer {
            buffer: self,
            offset: 0,
            length: self.buffer.get_elements_count() as usize,
        }
    }
}

impl<'a> ToIndicesSource for IndexBufferSlice<'a> {
    type Data = u16;      // TODO: u16?

    fn to_indices_source(&self) -> IndicesSource<u16> {     // TODO: u16?
        IndicesSource::IndexBuffer {
            buffer: self.buffer,
            offset: self.offset,
            length: self.len,
        }
    }
}
