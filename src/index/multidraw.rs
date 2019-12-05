//! Allows one to draw multiple geometry located in the same buffer.
//!
use std::ops::Deref;
use std::ops::DerefMut;
use std::os::raw;

use backend::Facade;
use buffer::{BufferCreationError, BufferType, BufferMode, Buffer};
use buffer::{BufferSlice, BufferMutSlice};
use index::{IndicesSource, PrimitiveType, IndexBuffer, Index};

/// Represents an element in a list of draw commands.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DrawCommandNoIndices {
    /// Number of vertices to draw.
    pub count: raw::c_uint,
    /// Number of instances to draw. If it's `0`, nothing will be drawn.
    pub instance_count: raw::c_uint,
    /// First vertex to draw in the vertices source.
    pub first_index: raw::c_uint,
    /// Numero of the first instance to draw.
    pub base_instance: raw::c_uint,
}

implement_uniform_block!(DrawCommandNoIndices, count, instance_count,
                         first_index, base_instance);

/// Represents an element in a list of draw commands.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DrawCommandIndices {
    /// Number of indices to use in the index buffer.
    pub count: raw::c_uint,
    /// Number of instances to draw. If it's `0`, nothing will be drawn.
    pub instance_count: raw::c_uint,
    /// First index to draw in the index buffer.
    pub first_index: raw::c_uint,
    /// Value to add to each index.
    pub base_vertex: raw::c_uint,
    /// Numero of the first instance to draw.
    pub base_instance: raw::c_uint,
}

implement_uniform_block!(DrawCommandIndices, count, instance_count, first_index,
                         base_vertex, base_instance);

/// A buffer containing a list of draw commands.
pub struct DrawCommandsNoIndicesBuffer {
    buffer: Buffer<[DrawCommandNoIndices]>,
}

impl DrawCommandsNoIndicesBuffer {
    /// Builds an empty buffer.
    ///
    /// The parameter indicates the number of elements.
    #[inline]
    pub fn empty<F: ?Sized>(facade: &F, elements: usize)
                    -> Result<DrawCommandsNoIndicesBuffer, BufferCreationError>
                    where F: Facade
    {
        let buf = Buffer::empty_array(facade, BufferType::DrawIndirectBuffer,
                                               elements, BufferMode::Default)?;
        Ok(DrawCommandsNoIndicesBuffer { buffer: buf })
    }

    /// Builds an empty buffer.
    ///
    /// The parameter indicates the number of elements.
    #[inline]
    pub fn empty_dynamic<F: ?Sized>(facade: &F, elements: usize)
                            -> Result<DrawCommandsNoIndicesBuffer, BufferCreationError>
                            where F: Facade
    {
        let buf = Buffer::empty_array(facade, BufferType::DrawIndirectBuffer,
                                               elements, BufferMode::Dynamic)?;
        Ok(DrawCommandsNoIndicesBuffer { buffer: buf })
    }

    /// Builds an empty buffer.
    ///
    /// The parameter indicates the number of elements.
    #[inline]
    pub fn empty_persistent<F: ?Sized>(facade: &F, elements: usize)
                               -> Result<DrawCommandsNoIndicesBuffer, BufferCreationError>
                               where F: Facade
    {
        let buf = Buffer::empty_array(facade, BufferType::DrawIndirectBuffer,
                                               elements, BufferMode::Persistent)?;
        Ok(DrawCommandsNoIndicesBuffer { buffer: buf })
    }

    /// Builds an empty buffer.
    ///
    /// The parameter indicates the number of elements.
    #[inline]
    pub fn empty_immutable<F: ?Sized>(facade: &F, elements: usize)
                              -> Result<DrawCommandsNoIndicesBuffer, BufferCreationError>
                              where F: Facade
    {
        let buf = Buffer::empty_array(facade, BufferType::DrawIndirectBuffer,
                                               elements, BufferMode::Immutable)?;
        Ok(DrawCommandsNoIndicesBuffer { buffer: buf })
    }

    /// Builds an indices source from this buffer and a primitives type. This indices source can
    /// be passed to the `draw()` function.
    #[inline]
    pub fn with_primitive_type(&self, primitives: PrimitiveType) -> IndicesSource {
        IndicesSource::MultidrawArray {
            buffer: self.buffer.as_slice_any(),
            primitives: primitives,
        }
    }
}

impl Deref for DrawCommandsNoIndicesBuffer {
    type Target = Buffer<[DrawCommandNoIndices]>;

    #[inline]
    fn deref(&self) -> &Buffer<[DrawCommandNoIndices]> {
        &self.buffer
    }
}

impl DerefMut for DrawCommandsNoIndicesBuffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut Buffer<[DrawCommandNoIndices]> {
        &mut self.buffer
    }
}

impl<'a> From<&'a DrawCommandsNoIndicesBuffer> for BufferSlice<'a, [DrawCommandNoIndices]> {
    #[inline]
    fn from(b: &'a DrawCommandsNoIndicesBuffer) -> BufferSlice<'a, [DrawCommandNoIndices]> {
        let b: &Buffer<[DrawCommandNoIndices]> = &*b;
        b.as_slice()
    }
}

impl<'a> From<&'a mut DrawCommandsNoIndicesBuffer> for BufferMutSlice<'a, [DrawCommandNoIndices]> {
    #[inline]
    fn from(b: &'a mut DrawCommandsNoIndicesBuffer) -> BufferMutSlice<'a, [DrawCommandNoIndices]> {
        let b: &mut Buffer<[DrawCommandNoIndices]> = &mut *b;
        b.as_mut_slice()
    }
}

/// A buffer containing a list of draw commands.
pub struct DrawCommandsIndicesBuffer {
    buffer: Buffer<[DrawCommandIndices]>,
}

impl DrawCommandsIndicesBuffer {
    /// Builds an empty buffer.
    ///
    /// The parameter indicates the number of elements.
    #[inline]
    pub fn empty<F: ?Sized>(facade: &F, elements: usize)
                    -> Result<DrawCommandsIndicesBuffer, BufferCreationError>
                    where F: Facade
    {
        let buf = Buffer::empty_array(facade, BufferType::DrawIndirectBuffer,
                                               elements, BufferMode::Default)?;
        Ok(DrawCommandsIndicesBuffer { buffer: buf })
    }

    /// Builds an empty buffer.
    ///
    /// The parameter indicates the number of elements.
    #[inline]
    pub fn empty_dynamic<F: ?Sized>(facade: &F, elements: usize)
                            -> Result<DrawCommandsIndicesBuffer, BufferCreationError>
                            where F: Facade
    {
        let buf = Buffer::empty_array(facade, BufferType::DrawIndirectBuffer,
                                               elements, BufferMode::Dynamic)?;
        Ok(DrawCommandsIndicesBuffer { buffer: buf })
    }

    /// Builds an empty buffer.
    ///
    /// The parameter indicates the number of elements.
    #[inline]
    pub fn empty_persistent<F: ?Sized>(facade: &F, elements: usize)
                               -> Result<DrawCommandsIndicesBuffer, BufferCreationError>
                               where F: Facade
    {
        let buf = Buffer::empty_array(facade, BufferType::DrawIndirectBuffer,
                                               elements, BufferMode::Persistent)?;
        Ok(DrawCommandsIndicesBuffer { buffer: buf })
    }

    /// Builds an empty buffer.
    ///
    /// The parameter indicates the number of elements.
    #[inline]
    pub fn empty_immutable<F: ?Sized>(facade: &F, elements: usize)
                              -> Result<DrawCommandsIndicesBuffer, BufferCreationError>
                              where F: Facade
    {
        let buf = Buffer::empty_array(facade, BufferType::DrawIndirectBuffer,
                                               elements, BufferMode::Immutable)?;
        Ok(DrawCommandsIndicesBuffer { buffer: buf })
    }

    /// Builds an indices source from this buffer and a primitives type. This indices source can
    /// be passed to the `draw()` function.
    #[inline]
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
    type Target = Buffer<[DrawCommandIndices]>;

    #[inline]
    fn deref(&self) -> &Buffer<[DrawCommandIndices]> {
        &self.buffer
    }
}

impl DerefMut for DrawCommandsIndicesBuffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut Buffer<[DrawCommandIndices]> {
        &mut self.buffer
    }
}

impl<'a> From<&'a DrawCommandsIndicesBuffer> for BufferSlice<'a, [DrawCommandIndices]> {
    #[inline]
    fn from(b: &'a DrawCommandsIndicesBuffer) -> BufferSlice<'a, [DrawCommandIndices]> {
        let b: &Buffer<[DrawCommandIndices]> = &*b;
        b.as_slice()
    }
}

impl<'a> From<&'a mut DrawCommandsIndicesBuffer> for BufferMutSlice<'a, [DrawCommandIndices]> {
    #[inline]
    fn from(b: &'a mut DrawCommandsIndicesBuffer) -> BufferMutSlice<'a, [DrawCommandIndices]> {
        let b: &mut Buffer<[DrawCommandIndices]> = &mut *b;
        b.as_mut_slice()
    }
}
