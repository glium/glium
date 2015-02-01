use Display;

use index::IndicesSource;
use index::ToIndicesSource;
use index::IntoIndexBuffer;
use index::IndexBuffer;
use index::Index;
use index::PrimitiveType;

/// A list of points stored in RAM.
#[derive(Clone)]
pub struct PointsList<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for PointsList<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &Display) -> IndexBuffer {
        IndexBuffer::from_raw(display, self.0, PrimitiveType::Points)
    }
}

impl<T> ToIndicesSource for PointsList<T> where T: Index + Send + Copy {
    type Data = T;

    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::Points,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of lines stored in RAM.
pub struct LinesList<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for LinesList<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &Display) -> IndexBuffer {
        IndexBuffer::from_raw(display, self.0, PrimitiveType::LinesList)
    }
}

impl<T> ToIndicesSource for LinesList<T> where T: Index + Send + Copy {
    type Data = T;

    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::LinesList,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of lines, with adjacency information, stored in RAM.
///
/// # Panic
///
/// OpenGL ES doesn't support adjacency information. Attempting to use this type while
/// drawing will thus panic.
pub struct LinesListAdjacency<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for LinesListAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &Display) -> IndexBuffer {
        IndexBuffer::from_raw(display, self.0, PrimitiveType::LinesListAdjacency)
    }
}

impl<T> ToIndicesSource for LinesListAdjacency<T> where T: Index + Send + Copy {
    type Data = T;

    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::LinesListAdjacency,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of lines connected together stored in RAM.
pub struct LineStrip<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for LineStrip<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &Display) -> IndexBuffer {
        IndexBuffer::from_raw(display, self.0, PrimitiveType::LineStrip)
    }
}

impl<T> ToIndicesSource for LineStrip<T> where T: Index + Send + Copy {
    type Data = T;

    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::LineStrip,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of lines connected together, with adjacency information, stored in RAM.
///
/// # Panic
///
/// OpenGL ES doesn't support adjacency information. Attempting to use this type while
/// drawing will thus panic.
pub struct LineStripAdjacency<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for LineStripAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &Display) -> IndexBuffer {
        IndexBuffer::from_raw(display, self.0, PrimitiveType::LineStripAdjacency)
    }
}

impl<T> ToIndicesSource for LineStripAdjacency<T> where T: Index + Send + Copy {
    type Data = T;

    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::LineStripAdjacency,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of triangles stored in RAM.
pub struct TrianglesList<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for TrianglesList<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &Display) -> IndexBuffer {
        IndexBuffer::from_raw(display, self.0, PrimitiveType::TrianglesList)
    }
}

impl<T> ToIndicesSource for TrianglesList<T> where T: Index + Send + Copy {
    type Data = T;

    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::TrianglesList,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of triangles, with adjacency information, stored in RAM.
///
/// # Panic
///
/// OpenGL ES doesn't support adjacency information. Attempting to use this type while
/// drawing will thus panic.
pub struct TrianglesListAdjacency<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for TrianglesListAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &Display) -> IndexBuffer {
        IndexBuffer::from_raw(display, self.0, PrimitiveType::TrianglesListAdjacency)
    }
}

impl<T> ToIndicesSource for TrianglesListAdjacency<T> where T: Index + Send + Copy {
    type Data = T;

    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::TrianglesListAdjacency,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of triangles connected together stored in RAM.
pub struct TriangleStrip<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for TriangleStrip<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &Display) -> IndexBuffer {
        IndexBuffer::from_raw(display, self.0, PrimitiveType::TriangleStrip)
    }
}

impl<T> ToIndicesSource for TriangleStrip<T> where T: Index + Send + Copy {
    type Data = T;

    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::TriangleStrip,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of triangles connected together, with adjacency information, stored in RAM.
///
/// # Panic
///
/// OpenGL ES doesn't support adjacency information. Attempting to use this type while
/// drawing will thus panic.
pub struct TriangleStripAdjacency<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for TriangleStripAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &Display) -> IndexBuffer {
        IndexBuffer::from_raw(display, self.0, PrimitiveType::TriangleStripAdjacency)
    }
}

impl<T> ToIndicesSource for TriangleStripAdjacency<T> where T: Index + Send + Copy {
    type Data = T;

    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::TriangleStripAdjacency,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of triangles stored in RAM.
pub struct TriangleFan<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for TriangleFan<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &Display) -> IndexBuffer {
        IndexBuffer::from_raw(display, self.0, PrimitiveType::TriangleFan)
    }
}

impl<T> ToIndicesSource for TriangleFan<T> where T: Index + Send + Copy {
    type Data = T;

    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::TriangleFan,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of patches stored in RAM.
///
/// The second parameter is the number of vertices per patch.
pub struct Patches<T>(pub Vec<T>, pub u16);

impl<T> IntoIndexBuffer for Patches<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &Display) -> IndexBuffer {
        IndexBuffer::from_raw(display, self.0, PrimitiveType::Patches { vertices_per_patch: self.1 })
    }
}

impl<T> ToIndicesSource for Patches<T> where T: Index + Send + Copy {
    type Data = T;

    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::Patches { vertices_per_patch: self.1 },
            offset: 0,
            length: self.0.len(),
        }
    }
}
