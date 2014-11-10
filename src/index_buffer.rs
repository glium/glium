/*!

*/
use buffer::{mod, Buffer};
use gl;
use GlObject;

/// A list of indices loaded in the graphics card's memory.
#[deriving(Show)]
pub struct IndexBuffer {
    buffer: Buffer,
    data_type: gl::types::GLenum,
    primitives: gl::types::GLenum,
}

/// This public function is accessible from within `glium` but not for the user.
#[doc(hidden)]
pub fn get_clone(ib: &IndexBuffer) -> (gl::types::GLuint, uint, gl::types::GLenum,
    gl::types::GLenum)
{
    (ib.buffer.get_id(), ib.buffer.get_elements_count(), ib.data_type.clone(), ib.primitives.clone())
}

impl IndexBuffer {
    /// Builds a new index buffer.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate glium;
    /// # extern crate glutin;
    /// # use glium::DisplayBuild;
    /// # fn main() {
    /// # let display: glium::Display = glutin::HeadlessRendererBuilder::new(1024, 768)
    /// #  .build_glium().unwrap();
    /// let index_buffer = glium::IndexBuffer::new(&display,
    ///     glium::index_buffer::TrianglesList(vec![0u8, 1, 2, 1, 3, 4, 2, 4, 3]));
    /// # }
    /// ```
    /// 
    pub fn new<T: IntoIndexBuffer>(display: &super::Display, data: T) -> IndexBuffer {
        data.into_index_buffer(display)
    }
}

impl GlObject for IndexBuffer {
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
    }
}

/// An index from the vertex buffer.
pub trait Index {
    /// Returns the GL_ENUM corresponding to this type.
    fn to_glenum(Option<Self>) -> gl::types::GLenum;
}

impl Index for u8 {
    fn to_glenum(_: Option<u8>) -> gl::types::GLenum {
        gl::UNSIGNED_BYTE
    }
}

impl Index for u16 {
    fn to_glenum(_: Option<u16>) -> gl::types::GLenum {
        gl::UNSIGNED_SHORT
    }
}

impl Index for u32 {
    fn to_glenum(_: Option<u32>) -> gl::types::GLenum {
        gl::UNSIGNED_INT
    }
}

/// Object is convertible to an index buffer.
pub trait IntoIndexBuffer {
    /// Creates a new `IndexBuffer` with the list of indices.
    fn into_index_buffer(self, &super::Display) -> IndexBuffer;
}

/// A list of points stored in RAM.
#[deriving(Clone)]
pub struct PointsList<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for PointsList<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        IndexBuffer {
            buffer: Buffer::new::<buffer::ElementArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::POINTS,
        }
    }
}

/// A list of lines stored in RAM.
pub struct LinesList<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for LinesList<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        IndexBuffer {
            buffer: Buffer::new::<buffer::ElementArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::LINES,
        }
    }
}

/// A list of lines with adjacency infos stored in RAM.
pub struct LinesListAdjacency<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for LinesListAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        IndexBuffer {
            buffer: Buffer::new::<buffer::ElementArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::LINES_ADJACENCY,
        }
    }
}

/// A list of lines connected together stored in RAM.
pub struct LineStrip<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for LineStrip<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        IndexBuffer {
            buffer: Buffer::new::<buffer::ElementArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::LINE_STRIP,
        }
    }
}

/// A list of lines connected together with adjacency infos stored in RAM.
pub struct LineStripAdjacency<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for LineStripAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        IndexBuffer {
            buffer: Buffer::new::<buffer::ElementArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::LINE_STRIP_ADJACENCY,
        }
    }
}

/// A list of triangles stored in RAM.
pub struct TrianglesList<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for TrianglesList<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        IndexBuffer {
            buffer: Buffer::new::<buffer::ElementArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::TRIANGLES,
        }
    }
}

/// A list of triangles with adjacency infos stored in RAM.
pub struct TrianglesListAdjacency<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for TrianglesListAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        IndexBuffer {
            buffer: Buffer::new::<buffer::ElementArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::TRIANGLES_ADJACENCY,
        }
    }
}

/// A list of triangles connected together stored in RAM.
pub struct TriangleStrip<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for TriangleStrip<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        IndexBuffer {
            buffer: Buffer::new::<buffer::ElementArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::TRIANGLE_STRIP,
        }
    }
}

/// A list of triangles connected together with adjacency infos stored in RAM.
pub struct TriangleStripAdjacency<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for TriangleStripAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        IndexBuffer {
            buffer: Buffer::new::<buffer::ElementArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::TRIANGLE_STRIP_ADJACENCY,
        }
    }
}

/// A list of triangles stored in RAM.
pub struct TriangleFan<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for TriangleFan<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        IndexBuffer {
            buffer: Buffer::new::<buffer::ElementArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::TRIANGLE_FAN,
        }
    }
}
