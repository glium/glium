/*!
In order to draw, you need to provide a source of indices which is used to link the vertices
together into *primitives*.

There are eleven types of primitives, each one with a corresponding struct:

 - `PointsList`
 - `LinesList`
 - `LinesListAdjacency`
 - `LineStrip`
 - `LineStripAdjacency`
 - `TrianglesList`
 - `TrianglesListAdjacency`
 - `TriangleStrip`
 - `TriangleStripAdjacency`
 - `TriangleFan`
 - `Patches`

These structs can be turned into an `IndexBuffer`, which uploads the data in video memory.

There are three ways to specify the indices that must be used:

 - Passing a reference to one of these structs.
 - Passing a reference to an `IndexBuffer`.
 - `NoIndices`, which is equivalent to `(0, 1, 2, 3, 4, 5, 6, 7, ..)`.

For performances it is highly recommended to use either an `IndexBuffer` or `NoIndices`, and to
avoid passing indices in RAM.

When you draw something, a draw command is sent to the GPU and the execution continues immediatly
after. But if you pass indices in RAM, the execution has to block until the GPU has finished
drawing in order to make sure that the indices are not free'd.

*/
use gl;
use ToGlEnum;

use sync::LinearSyncFence;
use std::sync::mpsc::Sender;

pub use self::buffer::IndexBuffer;
pub use self::local::{PointsList, LinesList, LinesListAdjacency, LineStrip, LineStripAdjacency};
pub use self::local::{TrianglesList, TrianglesListAdjacency, TriangleStrip, TriangleStripAdjacency};
pub use self::local::{TriangleFan, Patches};

mod buffer;
mod local;

/// Can be used as a source of indices when drawing.
pub trait ToIndicesSource {
    /// The type of data.
    type Data: Index;

    /// Builds the `IndicesSource`.
    fn to_indices_source<'a>(&'a self) -> IndicesSource<'a, Self::Data>;
}

/// Describes a source of indices used for drawing.
#[derive(Clone)]
pub enum IndicesSource<'a, T: 'a> {
    /// A buffer uploaded in video memory.
    IndexBuffer {
        /// The buffer.
        buffer: &'a IndexBuffer,
        /// Sender which must be used to send back a fence that is signaled when the buffer has
        /// finished being used.
        fence: Option<Sender<LinearSyncFence>>,
        /// Offset of the first element of the buffer to use.
        offset: usize,
        /// Number of elements in the buffer to use.
        length: usize,
    },

    /// A buffer in RAM.
    Buffer {
        /// Slice of data to use.
        pointer: &'a [T],
        /// Type of primitives contained in the buffer.
        primitives: PrimitiveType,
        /// Offset of the first element of the buffer to use.
        offset: usize,
        /// Number of elements in the buffer to use.
        length: usize,
    },

    /// Don't use indices. Assemble primitives by using the order in which the vertices are in
    /// the vertices source.
    NoIndices {
        /// Type of primitives contained in the vertex source.
        primitives: PrimitiveType,
    },
}

impl<'a, T> IndicesSource<'a, T> where T: Index {
    /// Returns the type of the primitives.
    pub fn get_primitives_type(&self) -> PrimitiveType {
        match self {
            &IndicesSource::IndexBuffer { ref buffer, .. } => buffer.get_primitives_type(),
            &IndicesSource::Buffer { primitives, .. } => primitives,
            &IndicesSource::NoIndices { primitives } => primitives,
        }
    }
}

/// List of available primitives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    ///
    Points,
    ///
    LinesList,
    ///
    LinesListAdjacency,
    ///
    LineStrip,
    ///
    LineStripAdjacency,
    ///
    TrianglesList,
    ///
    TrianglesListAdjacency,
    ///
    TriangleStrip,
    ///
    TriangleStripAdjacency,
    ///
    TriangleFan,
    ///
    Patches {
        /// Number of vertices per patch.
        vertices_per_patch: u16,
    },
}

impl ToGlEnum for PrimitiveType {
    fn to_glenum(&self) -> gl::types::GLenum {
        match self {
            &PrimitiveType::Points => gl::POINTS,
            &PrimitiveType::LinesList => gl::LINES,
            &PrimitiveType::LinesListAdjacency => gl::LINES_ADJACENCY,
            &PrimitiveType::LineStrip => gl::LINE_STRIP,
            &PrimitiveType::LineStripAdjacency => gl::LINE_STRIP_ADJACENCY,
            &PrimitiveType::TrianglesList => gl::TRIANGLES,
            &PrimitiveType::TrianglesListAdjacency => gl::TRIANGLES_ADJACENCY,
            &PrimitiveType::TriangleStrip => gl::TRIANGLE_STRIP,
            &PrimitiveType::TriangleStripAdjacency => gl::TRIANGLE_STRIP_ADJACENCY,
            &PrimitiveType::TriangleFan => gl::TRIANGLE_FAN,
            &PrimitiveType::Patches { .. } => gl::PATCHES,
        }
    }
}

/// Marker that can be used as an indices source when you don't need indices.
///
/// If you use this, then the primitives will be constructed using the order in which the
/// vertices are in the vertices sources.
#[derive(Copy, Clone, Debug)]
pub struct NoIndices(pub PrimitiveType);

impl ToIndicesSource for NoIndices {
    type Data = u16;      // TODO: u16?

    fn to_indices_source(&self) -> IndicesSource<u16> {     // TODO: u16?
        IndicesSource::NoIndices {
            primitives: self.0,
        }
    }
}

/// Type of the indices in an index source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]    // GLenum
pub enum IndexType {
    /// u8
    U8 = gl::UNSIGNED_BYTE,
    /// u16
    U16 = gl::UNSIGNED_SHORT,
    /// u32
    U32 = gl::UNSIGNED_INT,
}

impl ToGlEnum for IndexType {
    fn to_glenum(&self) -> gl::types::GLenum {
        *self as gl::types::GLenum
    }
}

/// An index from the index buffer.
pub unsafe trait Index: Copy + Send + 'static {
    /// Returns the `IndexType` corresponding to this type.
    fn get_type() -> IndexType;
}

unsafe impl Index for u8 {
    fn get_type() -> IndexType {
        IndexType::U8
    }
}

unsafe impl Index for u16 {
    fn get_type() -> IndexType {
        IndexType::U16
    }
}

unsafe impl Index for u32 {
    fn get_type() -> IndexType {
        IndexType::U32
    }
}

/// Object that is convertible to an index buffer.
pub trait IntoIndexBuffer {
    /// Creates a new `IndexBuffer` with the list of indices.
    fn into_index_buffer(self, &super::Display) -> IndexBuffer;
}
