//! Allows one to draw multiple geometry located in the same buffer.
//!
use libc;

use std::ops::Deref;
use std::ops::DerefMut;

use backend::Facade;
use buffer::{BufferCreationError, BufferType, BufferView};
use index::{IndicesSource, PrimitiveType, IndexBuffer, Index};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(missing_docs)]      // TODO: remove this
pub struct DrawCommandNoIndices {
    /// Number of vertices to draw.
    pub count: libc::c_uint,
    /// Number of instances to draw. If it's `0`, nothing will be drawn.
    pub instance_count: libc::c_uint,
    /// First vertex to draw in the vertices source.
    pub first_index: libc::c_uint,
    /// Numero of the first instance to draw.
    pub base_instance: libc::c_uint,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(missing_docs)]      // TODO: remove this
pub struct DrawCommandIndices {
    /// Number of indices to use in the index buffer.
    pub count: libc::c_uint,
    /// Number of instances to draw. If it's `0`, nothing will be drawn.
    pub instance_count: libc::c_uint,
    /// First index to draw in the index buffer.
    pub first_index: libc::c_uint,
    /// Value to add to each index.
    pub base_vertex: libc::c_uint,
    /// Numero of the first instance to draw.
    pub base_instance: libc::c_uint,
}

/// A buffer containing a list of draw commands.
pub struct DrawCommandsNoIndicesBuffer {
    buffer: BufferView<[DrawCommandNoIndices]>,
}

impl DrawCommandsNoIndicesBuffer {
    /// Builds an empty buffer.
    ///
    /// The parameter indicates the number of elements.
    pub fn empty_if_supported<F>(facade: &F, elements: usize)
                                 -> Option<DrawCommandsNoIndicesBuffer>
                                 where F: Facade
    {
        match BufferView::empty_array(facade, BufferType::DrawIndirectBuffer,
                                      elements, false)
        {
            Ok(buf) => Some(DrawCommandsNoIndicesBuffer { buffer: buf }),
            Err(BufferCreationError::BufferTypeNotSupported) => None,
            Err(_) => panic!()
        }
    }

    /// Builds an empty buffer.
    ///
    /// The parameter indicates the number of elements.
    pub fn empty_dynamic_if_supported<F>(facade: &F, elements: usize)
                                         -> Option<DrawCommandsNoIndicesBuffer>
                                         where F: Facade
    {
        match BufferView::empty_array(facade, BufferType::DrawIndirectBuffer,
                                      elements, true)
        {
            Ok(buf) => Some(DrawCommandsNoIndicesBuffer { buffer: buf }),
            Err(BufferCreationError::BufferTypeNotSupported) => None,
            Err(_) => panic!()
        }
    }

    /// Builds an indices source from this buffer and a primitives type. This indices source can
    /// be passed to the `draw()` function.
    pub fn with_primitive_type(&self, primitives: PrimitiveType) -> IndicesSource {
        IndicesSource::MultidrawArray {
            buffer: self.buffer.as_slice_any(),
            primitives: primitives,
        }
    }
}

impl Deref for DrawCommandsNoIndicesBuffer {
    type Target = BufferView<[DrawCommandNoIndices]>;

    fn deref(&self) -> &BufferView<[DrawCommandNoIndices]> {
        &self.buffer
    }
}

impl DerefMut for DrawCommandsNoIndicesBuffer {
    fn deref_mut(&mut self) -> &mut BufferView<[DrawCommandNoIndices]> {
        &mut self.buffer
    }
}

/// A buffer containing a list of draw commands.
pub struct DrawCommandsIndicesBuffer {
    buffer: BufferView<[DrawCommandIndices]>,
}

impl DrawCommandsIndicesBuffer {
    /// Builds an empty buffer.
    ///
    /// The parameter indicates the number of elements.
    pub fn empty_if_supported<F>(facade: &F, elements: usize)
                                 -> Option<DrawCommandsIndicesBuffer>
                                 where F: Facade
    {
        match BufferView::empty_array(facade, BufferType::DrawIndirectBuffer,
                                      elements, false)
        {
            Ok(buf) => Some(DrawCommandsIndicesBuffer { buffer: buf }),
            Err(BufferCreationError::BufferTypeNotSupported) => None,
            Err(_) => panic!()
        }
    }

    /// Builds an empty buffer.
    ///
    /// The parameter indicates the number of elements.
    pub fn empty_dynamic_if_supported<F>(facade: &F, elements: usize)
                                         -> Option<DrawCommandsIndicesBuffer>
                                         where F: Facade
    {
        match BufferView::empty_array(facade, BufferType::DrawIndirectBuffer,
                                      elements, true)
        {
            Ok(buf) => Some(DrawCommandsIndicesBuffer { buffer: buf }),
            Err(BufferCreationError::BufferTypeNotSupported) => None,
            Err(_) => panic!()
        }
    }

    /// Builds an indices source from this buffer and a primitives type. This indices source can
    /// be passed to the `draw()` function.
    pub fn with_index_buffer<'a, T>(&'a self, index_buffer: &'a IndexBuffer<T>)
                                    -> IndicesSource<'a> where T: Index
    {
        IndicesSource::MultidrawElement {
            commands: self.buffer.as_slice_any(),
            indices: index_buffer.as_slice_any(),
            data_type: index_buffer.get_indices_type(),
            primitives: index_buffer.get_primitives_type(),
        }
    }
}

impl Deref for DrawCommandsIndicesBuffer {
    type Target = BufferView<[DrawCommandIndices]>;

    fn deref(&self) -> &BufferView<[DrawCommandIndices]> {
        &self.buffer
    }
}

impl DerefMut for DrawCommandsIndicesBuffer {
    fn deref_mut(&mut self) -> &mut BufferView<[DrawCommandIndices]> {
        &mut self.buffer
    }
}
