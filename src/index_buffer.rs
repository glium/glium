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

impl IndicesSource for IndexBuffer {
    fn to_indices_source_helper(&self) -> IndicesSourceHelper {
        let id = self.buffer.get_id();
        let elems_count = self.buffer.get_elements_count();
        let datatype = self.data_type.clone();
        let primitives = self.primitives.clone();

        IndicesSourceHelper(proc(ctxt) {
            unsafe {
                use std::ptr;

                if ctxt.state.element_array_buffer_binding != Some(id) {
                    ctxt.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, id);
                    ctxt.state.element_array_buffer_binding = Some(id);
                }

                ctxt.gl.DrawElements(primitives, elems_count as i32, datatype, ptr::null());
            }
        })
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
            buffer: Buffer::new::<buffer::ElementArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::POINTS,
        }
    }
}

impl<T> IndicesSource for PointsList<T> where T: Index + Send + Copy {
    fn to_indices_source_helper(&self) -> IndicesSourceHelper {
        let elems_count = self.0.len();
        let data_type = Index::to_glenum(None::<T>);
        let ptr = self.0.as_ptr();

        IndicesSourceHelper(proc(ctxt) {
            unsafe {
                use libc;

                if ctxt.state.element_array_buffer_binding != None {
                    ctxt.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
                    ctxt.state.element_array_buffer_binding = None;
                }

                ctxt.gl.DrawElements(gl::POINTS, elems_count as i32, data_type,
                                ptr as *const libc::c_void);
            }
        })
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
            buffer: Buffer::new::<buffer::ElementArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::LINES,
        }
    }
}

impl<T> IndicesSource for LinesList<T> where T: Index + Send + Copy {
    fn to_indices_source_helper(&self) -> IndicesSourceHelper {
        let elems_count = self.0.len();
        let data_type = Index::to_glenum(None::<T>);
        let ptr = self.0.as_ptr();

        IndicesSourceHelper(proc(ctxt) {
            unsafe {
                use libc;

                if ctxt.state.element_array_buffer_binding != None {
                    ctxt.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
                    ctxt.state.element_array_buffer_binding = None;
                }

                ctxt.gl.DrawElements(gl::LINES, elems_count as i32, data_type,
                                ptr as *const libc::c_void);
            }
        })
    }
}

/// A list of lines with adjacency infos stored in RAM.
pub struct LinesListAdjacency<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for LinesListAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ElementArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::LINES_ADJACENCY,
        }
    }
}

impl<T> IndicesSource for LinesListAdjacency<T> where T: Index + Send + Copy {
    fn to_indices_source_helper(&self) -> IndicesSourceHelper {
        let elems_count = self.0.len();
        let data_type = Index::to_glenum(None::<T>);
        let ptr = self.0.as_ptr();

        IndicesSourceHelper(proc(ctxt) {
            unsafe {
                use libc;

                if ctxt.state.element_array_buffer_binding != None {
                    ctxt.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
                    ctxt.state.element_array_buffer_binding = None;
                }

                ctxt.gl.DrawElements(gl::LINES_ADJACENCY, elems_count as i32, data_type,
                                ptr as *const libc::c_void);
            }
        })
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
            buffer: Buffer::new::<buffer::ElementArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::LINE_STRIP,
        }
    }
}

impl<T> IndicesSource for LineStrip<T> where T: Index + Send + Copy {
    fn to_indices_source_helper(&self) -> IndicesSourceHelper {
        let elems_count = self.0.len();
        let data_type = Index::to_glenum(None::<T>);
        let ptr = self.0.as_ptr();

        IndicesSourceHelper(proc(ctxt) {
            unsafe {
                use libc;

                if ctxt.state.element_array_buffer_binding != None {
                    ctxt.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
                    ctxt.state.element_array_buffer_binding = None;
                }

                ctxt.gl.DrawElements(gl::LINE_STRIP, elems_count as i32, data_type,
                                ptr as *const libc::c_void);
            }
        })
    }
}

/// A list of lines connected together with adjacency infos stored in RAM.
pub struct LineStripAdjacency<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for LineStripAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ElementArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::LINE_STRIP_ADJACENCY,
        }
    }
}

impl<T> IndicesSource for LineStripAdjacency<T> where T: Index + Send + Copy {
    fn to_indices_source_helper(&self) -> IndicesSourceHelper {
        let elems_count = self.0.len();
        let data_type = Index::to_glenum(None::<T>);
        let ptr = self.0.as_ptr();

        IndicesSourceHelper(proc(ctxt) {
            unsafe {
                use libc;

                if ctxt.state.element_array_buffer_binding != None {
                    ctxt.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
                    ctxt.state.element_array_buffer_binding = None;
                }

                ctxt.gl.DrawElements(gl::LINE_STRIP_ADJACENCY, elems_count as i32, data_type,
                                ptr as *const libc::c_void);
            }
        })
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
            buffer: Buffer::new::<buffer::ElementArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::TRIANGLES,
        }
    }
}

impl<T> IndicesSource for TrianglesList<T> where T: Index + Send + Copy {
    fn to_indices_source_helper(&self) -> IndicesSourceHelper {
        let elems_count = self.0.len();
        let data_type = Index::to_glenum(None::<T>);
        let ptr = self.0.as_ptr();

        IndicesSourceHelper(proc(ctxt) {
            unsafe {
                use libc;

                if ctxt.state.element_array_buffer_binding != None {
                    ctxt.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
                    ctxt.state.element_array_buffer_binding = None;
                }

                ctxt.gl.DrawElements(gl::TRIANGLES, elems_count as i32, data_type,
                                ptr as *const libc::c_void);
            }
        })
    }
}

/// A list of triangles with adjacency infos stored in RAM.
pub struct TrianglesListAdjacency<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for TrianglesListAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ElementArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::TRIANGLES_ADJACENCY,
        }
    }
}

impl<T> IndicesSource for TrianglesListAdjacency<T> where T: Index + Send + Copy {
    fn to_indices_source_helper(&self) -> IndicesSourceHelper {
        let elems_count = self.0.len();
        let data_type = Index::to_glenum(None::<T>);
        let ptr = self.0.as_ptr();

        IndicesSourceHelper(proc(ctxt) {
            unsafe {
                use libc;

                if ctxt.state.element_array_buffer_binding != None {
                    ctxt.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
                    ctxt.state.element_array_buffer_binding = None;
                }

                ctxt.gl.DrawElements(gl::TRIANGLES_ADJACENCY, elems_count as i32, data_type,
                                ptr as *const libc::c_void);
            }
        })
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
            buffer: Buffer::new::<buffer::ElementArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::TRIANGLE_STRIP,
        }
    }
}

impl<T> IndicesSource for TriangleStrip<T> where T: Index + Send + Copy {
    fn to_indices_source_helper(&self) -> IndicesSourceHelper {
        let elems_count = self.0.len();
        let data_type = Index::to_glenum(None::<T>);
        let ptr = self.0.as_ptr();

        IndicesSourceHelper(proc(ctxt) {
            unsafe {
                use libc;

                if ctxt.state.element_array_buffer_binding != None {
                    ctxt.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
                    ctxt.state.element_array_buffer_binding = None;
                }

                ctxt.gl.DrawElements(gl::TRIANGLE_STRIP, elems_count as i32, data_type,
                                ptr as *const libc::c_void);
            }
        })
    }
}

/// A list of triangles connected together with adjacency infos stored in RAM.
pub struct TriangleStripAdjacency<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for TriangleStripAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ElementArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::TRIANGLE_STRIP_ADJACENCY,
        }
    }
}

impl<T> IndicesSource for TriangleStripAdjacency<T> where T: Index + Send + Copy {
    fn to_indices_source_helper(&self) -> IndicesSourceHelper {
        let elems_count = self.0.len();
        let data_type = Index::to_glenum(None::<T>);
        let ptr = self.0.as_ptr();

        IndicesSourceHelper(proc(ctxt) {
            unsafe {
                use libc;

                if ctxt.state.element_array_buffer_binding != None {
                    ctxt.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
                    ctxt.state.element_array_buffer_binding = None;
                }

                ctxt.gl.DrawElements(gl::TRIANGLE_STRIP_ADJACENCY, elems_count as i32, data_type,
                                ptr as *const libc::c_void);
            }
        })
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
            buffer: Buffer::new::<buffer::ElementArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::to_glenum(None::<T>),
            primitives: gl::TRIANGLE_FAN,
        }
    }
}

impl<T> IndicesSource for TriangleFan<T> where T: Index + Send + Copy {
    fn to_indices_source_helper(&self) -> IndicesSourceHelper {
        let elems_count = self.0.len();
        let data_type = Index::to_glenum(None::<T>);
        let ptr = self.0.as_ptr();

        IndicesSourceHelper(proc(ctxt) {
            unsafe {
                use libc;

                if ctxt.state.element_array_buffer_binding != None {
                    ctxt.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
                    ctxt.state.element_array_buffer_binding = None;
                }

                ctxt.gl.DrawElements(gl::TRIANGLE_FAN, elems_count as i32, data_type,
                                ptr as *const libc::c_void);
            }
        })
    }
}
