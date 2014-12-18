/*!
In order to draw, you need to provide a source of indices which is used to link the vertices
together into *primitives*.

There are height types of primitives, each one with a corresponding struct:
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

Each struct contains a vector with the indices and can be used as an `IndicesSource`.

However the most optimal way to draw something is to load the data in the video memory by
creating an `IndexBuffer`.

*/
use buffer::{mod, Buffer};
use gl;
use libc;
use GlObject;
use {IndicesSource, IndicesSourceHelper};

/// A list of indices loaded in the graphics card's memory.
#[deriving(Show)]
pub struct IndexBuffer {
    buffer: Buffer,
    data_type: gl::types::GLenum,
    primitives: gl::types::GLenum,
}

impl IndexBuffer {
    /// Builds a new index buffer.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # extern crate glium;
    /// # extern crate glutin;
    /// # fn main() {
    /// # let display: glium::Display = unsafe { ::std::mem::uninitialized() };
    /// let index_buffer = glium::IndexBuffer::new(&display,
    ///     glium::index_buffer::TrianglesList(vec![0u8, 1, 2, 1, 3, 4, 2, 4, 3]));
    /// # }
    /// ```
    ///
    /// # Panic
    ///
    /// Attempting to draw with an index buffer that uses an indices format with adjacency infos
    /// on OpenGL ES will trigger a panic.
    ///
    /// If you want to be compatible with all platforms, it is preferable to disable the
    /// `gl_extensions` feature, which prevents you from accidentally using them.
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

impl IndicesSource for IndexBuffer {
    fn to_indices_source_helper(&self) -> IndicesSourceHelper {
        IndicesSourceHelper {
            index_buffer: Some(&self.buffer),
            pointer: None,
            primitives: self.primitives,
            data_type: self.data_type,
            indices_count: self.buffer.get_elements_count() as u32,
        }
    }
}

impl Drop for IndexBuffer {
    fn drop(&mut self) {
        // removing VAOs which contain this index buffer
        let mut vaos = self.buffer.get_display().vertex_array_objects.lock();
        let to_delete = vaos.keys().filter(|&&(_, i, _)| i == self.buffer.get_id())
            .map(|k| k.clone()).collect::<Vec<_>>();
        for k in to_delete.into_iter() {
            vaos.remove(&k);
        }
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
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");

        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::POINTS,
        }
    }
}

impl<T> IndicesSource for PointsList<T> where T: Index + Send + Copy {
    fn to_indices_source_helper(&self) -> IndicesSourceHelper {
        IndicesSourceHelper {
            index_buffer: None,
            pointer: Some(self.0.as_ptr() as *const libc::c_void),
            primitives: gl::POINTS,
            data_type: Index::to_glenum(None::<T>),
            indices_count: self.0.len() as u32,
        }
    }
}

/// A list of lines stored in RAM.
pub struct LinesList<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for LinesList<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::LINES,
        }
    }
}

impl<T> IndicesSource for LinesList<T> where T: Index + Send + Copy {
    fn to_indices_source_helper(&self) -> IndicesSourceHelper {
        IndicesSourceHelper {
            index_buffer: None,
            pointer: Some(self.0.as_ptr() as *const libc::c_void),
            primitives: gl::LINES,
            data_type: Index::to_glenum(None::<T>),
            indices_count: self.0.len() as u32,
        }
    }
}

/// A list of lines with adjacency infos stored in RAM.
///
/// # Panic
///
/// OpenGL ES doesn't support adjacency infos. Attempting to use this type while
/// drawing will thus panic.
/// If you want to be compatible with all platforms, it is preferable to disable the
/// `gl_extensions` feature.
///
/// # Features
///
/// Only available if the `gl_extensions` feature is enabled.
#[cfg(feature = "gl_extensions")]
pub struct LinesListAdjacency<T>(pub Vec<T>);

#[cfg(feature = "gl_extensions")]
impl<T> IntoIndexBuffer for LinesListAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::LINES_ADJACENCY,
        }
    }
}

#[cfg(feature = "gl_extensions")]
impl<T> IndicesSource for LinesListAdjacency<T> where T: Index + Send + Copy {
    fn to_indices_source_helper(&self) -> IndicesSourceHelper {
        IndicesSourceHelper {
            index_buffer: None,
            pointer: Some(self.0.as_ptr() as *const libc::c_void),
            primitives: gl::LINES_ADJACENCY,
            data_type: Index::to_glenum(None::<T>),
            indices_count: self.0.len() as u32,
        }
    }
}

/// A list of lines connected together stored in RAM.
pub struct LineStrip<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for LineStrip<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::LINE_STRIP,
        }
    }
}

impl<T> IndicesSource for LineStrip<T> where T: Index + Send + Copy {
    fn to_indices_source_helper(&self) -> IndicesSourceHelper {
        IndicesSourceHelper {
            index_buffer: None,
            pointer: Some(self.0.as_ptr() as *const libc::c_void),
            primitives: gl::LINE_STRIP,
            data_type: Index::to_glenum(None::<T>),
            indices_count: self.0.len() as u32,
        }
    }
}

/// A list of lines connected together with adjacency infos stored in RAM.
///
/// # Panic
///
/// OpenGL ES doesn't support adjacency infos. Attempting to use this type while
/// drawing will thus panic.
/// If you want to be compatible with all platforms, it is preferable to disable the
/// `gl_extensions` feature.
///
/// # Features
///
/// Only available if the `gl_extensions` feature is enabled.
#[cfg(feature = "gl_extensions")]
pub struct LineStripAdjacency<T>(pub Vec<T>);

#[cfg(feature = "gl_extensions")]
impl<T> IntoIndexBuffer for LineStripAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::LINE_STRIP_ADJACENCY,
        }
    }
}

#[cfg(feature = "gl_extensions")]
impl<T> IndicesSource for LineStripAdjacency<T> where T: Index + Send + Copy {
    fn to_indices_source_helper(&self) -> IndicesSourceHelper {
        IndicesSourceHelper {
            index_buffer: None,
            pointer: Some(self.0.as_ptr() as *const libc::c_void),
            primitives: gl::LINE_STRIP_ADJACENCY,
            data_type: Index::to_glenum(None::<T>),
            indices_count: self.0.len() as u32,
        }
    }
}

/// A list of triangles stored in RAM.
pub struct TrianglesList<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for TrianglesList<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::TRIANGLES,
        }
    }
}

impl<T> IndicesSource for TrianglesList<T> where T: Index + Send + Copy {
    fn to_indices_source_helper(&self) -> IndicesSourceHelper {
        IndicesSourceHelper {
            index_buffer: None,
            pointer: Some(self.0.as_ptr() as *const libc::c_void),
            primitives: gl::TRIANGLES,
            data_type: Index::to_glenum(None::<T>),
            indices_count: self.0.len() as u32,
        }
    }
}

/// A list of triangles with adjacency infos stored in RAM.
///
/// # Panic
///
/// OpenGL ES doesn't support adjacency infos. Attempting to use this type while
/// drawing will thus panic.
/// If you want to be compatible with all platforms, it is preferable to disable the
/// `gl_extensions` feature.
///
/// # Features
///
/// Only available if the `gl_extensions` feature is enabled.
#[cfg(feature = "gl_extensions")]
pub struct TrianglesListAdjacency<T>(pub Vec<T>);

#[cfg(feature = "gl_extensions")]
impl<T> IntoIndexBuffer for TrianglesListAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::TRIANGLES_ADJACENCY,
        }
    }
}

#[cfg(feature = "gl_extensions")]
impl<T> IndicesSource for TrianglesListAdjacency<T> where T: Index + Send + Copy {
    fn to_indices_source_helper(&self) -> IndicesSourceHelper {
        IndicesSourceHelper {
            index_buffer: None,
            pointer: Some(self.0.as_ptr() as *const libc::c_void),
            primitives: gl::TRIANGLES_ADJACENCY,
            data_type: Index::to_glenum(None::<T>),
            indices_count: self.0.len() as u32,
        }
    }
}

/// A list of triangles connected together stored in RAM.
pub struct TriangleStrip<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for TriangleStrip<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::TRIANGLE_STRIP,
        }
    }
}

impl<T> IndicesSource for TriangleStrip<T> where T: Index + Send + Copy {
    fn to_indices_source_helper(&self) -> IndicesSourceHelper {
        IndicesSourceHelper {
            index_buffer: None,
            pointer: Some(self.0.as_ptr() as *const libc::c_void),
            primitives: gl::TRIANGLE_STRIP,
            data_type: Index::to_glenum(None::<T>),
            indices_count: self.0.len() as u32,
        }
    }
}

/// A list of triangles connected together with adjacency infos stored in RAM.
///
/// # Panic
///
/// OpenGL ES doesn't support adjacency infos. Attempting to use this type while
/// drawing will thus panic.
/// If you want to be compatible with all platforms, it is preferable to disable the
/// `gl_extensions` feature.
///
/// # Features
///
/// Only available if the `gl_extensions` feature is enabled.
#[cfg(feature = "gl_extensions")]
pub struct TriangleStripAdjacency<T>(pub Vec<T>);

#[cfg(feature = "gl_extensions")]
impl<T> IntoIndexBuffer for TriangleStripAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::TRIANGLE_STRIP_ADJACENCY,
        }
    }
}

#[cfg(feature = "gl_extensions")]
impl<T> IndicesSource for TriangleStripAdjacency<T> where T: Index + Send + Copy {
    fn to_indices_source_helper(&self) -> IndicesSourceHelper {
        IndicesSourceHelper {
            index_buffer: None,
            pointer: Some(self.0.as_ptr() as *const libc::c_void),
            primitives: gl::TRIANGLE_STRIP_ADJACENCY,
            data_type: Index::to_glenum(None::<T>),
            indices_count: self.0.len() as u32,
        }
    }
}

/// A list of triangles stored in RAM.
pub struct TriangleFan<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for TriangleFan<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::TRIANGLE_FAN,
        }
    }
}

impl<T> IndicesSource for TriangleFan<T> where T: Index + Send + Copy {
    fn to_indices_source_helper(&self) -> IndicesSourceHelper {
        IndicesSourceHelper {
            index_buffer: None,
            pointer: Some(self.0.as_ptr() as *const libc::c_void),
            primitives: gl::TRIANGLE_FAN,
            data_type: Index::to_glenum(None::<T>),
            indices_count: self.0.len() as u32,
        }
    }
}
