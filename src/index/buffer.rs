use buffer::{SubBuffer, SubBufferAny, BufferType};
use gl;
use SubBufferExt;
use GlObject;

use backend::Facade;

use index::IndicesSource;
use index::ToIndicesSource;
use index::IntoIndexBuffer;
use index::Index;
use index::IndexType;
use index::PrimitiveType;

use std::mem;
use std::convert::AsRef;
use std::ops::Range;

/// A list of indices loaded in the graphics card's memory.
#[derive(Debug)]
pub struct IndexBuffer {
    buffer: SubBufferAny,
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
    pub fn new<T: IntoIndexBuffer, F>(facade: &F, data: T) -> IndexBuffer where F: Facade {
        data.into_index_buffer(facade)
    }

    /// Builds a new index buffer from raw data and a primitive type.
    pub fn from_raw<T, F, D>(facade: &F, data: D, prim: PrimitiveType) -> IndexBuffer
                       where T: Index, F: Facade, D: AsRef<[T]>
    {
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: SubBuffer::new(facade, data.as_ref(), BufferType::ArrayBuffer,
                                   false).unwrap().into(),    // FIXME: ElementArrayBuffer
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
    pub fn slice(&self, Range { start, end }: Range<usize>) -> Option<IndexBufferSlice> {
        let len = end - start;

        if start > self.buffer.get_elements_count() ||
            start + len > self.buffer.get_elements_count()
        {
            return None;
        }

        Some(IndexBufferSlice {
            buffer: self,
            offset: start,
            len: len,
        })
    }
}

impl SubBufferExt for IndexBuffer {
    fn get_offset_bytes(&self) -> usize {
        self.buffer.get_offset_bytes()
    }

    fn get_buffer_id(&self) -> gl::types::GLuint {
        self.buffer.get_buffer_id()
    }
}

// TODO: remove this
impl GlObject for IndexBuffer {
    type Id = gl::types::GLuint;

    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_buffer_id()
    }
}

impl ToIndicesSource for IndexBuffer {
    fn to_indices_source(&self) -> IndicesSource {
        IndicesSource::IndexBuffer {
            buffer: self,
            offset: 0,
            length: self.buffer.get_elements_count() as usize,
        }
    }
}

impl<'a> ToIndicesSource for IndexBufferSlice<'a> {
    fn to_indices_source(&self) -> IndicesSource {
        IndicesSource::IndexBuffer {
            buffer: self.buffer,
            offset: self.offset,
            length: self.len,
        }
    }
}
