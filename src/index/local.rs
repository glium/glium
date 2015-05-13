use backend::Facade;

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
    fn into_index_buffer<F>(self, facade: &F) -> IndexBuffer where F: Facade {
        IndexBuffer::from_raw(facade, self.0, PrimitiveType::Points)
    }
}

/// A list of lines stored in RAM.
pub struct LinesList<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for LinesList<T> where T: Index + Send + Copy {
    fn into_index_buffer<F>(self, facade: &F) -> IndexBuffer where F: Facade {
        IndexBuffer::from_raw(facade, self.0, PrimitiveType::LinesList)
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
    fn into_index_buffer<F>(self, facade: &F) -> IndexBuffer where F: Facade {
        IndexBuffer::from_raw(facade, self.0, PrimitiveType::LinesListAdjacency)
    }
}

/// A list of lines connected together stored in RAM.
pub struct LineStrip<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for LineStrip<T> where T: Index + Send + Copy {
    fn into_index_buffer<F>(self, facade: &F) -> IndexBuffer where F: Facade {
        IndexBuffer::from_raw(facade, self.0, PrimitiveType::LineStrip)
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
    fn into_index_buffer<F>(self, facade: &F) -> IndexBuffer where F: Facade {
        IndexBuffer::from_raw(facade, self.0, PrimitiveType::LineStripAdjacency)
    }
}

/// A list of triangles stored in RAM.
pub struct TrianglesList<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for TrianglesList<T> where T: Index + Send + Copy {
    fn into_index_buffer<F>(self, facade: &F) -> IndexBuffer where F: Facade {
        IndexBuffer::from_raw(facade, self.0, PrimitiveType::TrianglesList)
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
    fn into_index_buffer<F>(self, facade: &F) -> IndexBuffer where F: Facade {
        IndexBuffer::from_raw(facade, self.0, PrimitiveType::TrianglesListAdjacency)
    }
}

/// A list of triangles connected together stored in RAM.
pub struct TriangleStrip<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for TriangleStrip<T> where T: Index + Send + Copy {
    fn into_index_buffer<F>(self, facade: &F) -> IndexBuffer where F: Facade {
        IndexBuffer::from_raw(facade, self.0, PrimitiveType::TriangleStrip)
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
    fn into_index_buffer<F>(self, facade: &F) -> IndexBuffer where F: Facade {
        IndexBuffer::from_raw(facade, self.0, PrimitiveType::TriangleStripAdjacency)
    }
}

/// A list of triangles stored in RAM.
pub struct TriangleFan<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for TriangleFan<T> where T: Index + Send + Copy {
    fn into_index_buffer<F>(self, facade: &F) -> IndexBuffer where F: Facade {
        IndexBuffer::from_raw(facade, self.0, PrimitiveType::TriangleFan)
    }
}

/// A list of patches stored in RAM.
///
/// The second parameter is the number of vertices per patch.
pub struct Patches<T>(pub Vec<T>, pub u16);

impl<T> IntoIndexBuffer for Patches<T> where T: Index + Send + Copy {
    fn into_index_buffer<F>(self, facade: &F) -> IndexBuffer where F: Facade {
        IndexBuffer::from_raw(facade, self.0, PrimitiveType::Patches { vertices_per_patch: self.1 })
    }
}
