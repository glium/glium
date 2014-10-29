use gl;
use libc;
use std::c_vec::CVec;
use std::fmt;
use std::mem;
use std::sync::Arc;

/// A list of verices loaded in the graphics card's memory.
pub struct VertexBuffer<T> {
    display: Arc<super::DisplayImpl>,
    id: gl::types::GLuint,
    elements_size: uint,
    bindings: VertexBindings,
    elements_count: uint,
}

/// This public function is accessible from within `glium` but not for the user.
#[doc(hidden)]
pub fn get_clone<T>(vb: &VertexBuffer<T>) -> (gl::types::GLuint, uint, VertexBindings) {
    (vb.id.clone(), vb.elements_size.clone(), vb.bindings.clone())
}

impl<T: VertexFormat + 'static + Send> VertexBuffer<T> {
    /// Builds a new vertex buffer.
    ///
    /// # Example
    ///
    /// ```
    /// # #![feature(phase)]
    /// # #[phase(plugin)]
    /// # extern crate glium_macros;
    /// # extern crate glium;
    /// # extern crate glutin;
    /// # use glium::DisplayBuild;
    /// # fn main() {
    /// #[vertex_format]
    /// struct Vertex {
    ///     position: [f32, ..3],
    ///     texcoords: [f32, ..2],
    /// }
    ///
    /// # let display: glium::Display = glutin::HeadlessRendererBuilder::new(1024, 768).build_glium().unwrap();
    /// let vertex_buffer = glium::VertexBuffer::new(&display, vec![
    ///     Vertex { position: [0.0,  0.0, 0.0], texcoords: [0.0, 1.0] },
    ///     Vertex { position: [5.0, -3.0, 2.0], texcoords: [1.0, 0.0] },
    /// ]);
    /// # }
    /// ```
    /// 
    pub fn new(display: &super::Display, data: Vec<T>) -> VertexBuffer<T> {
        VertexBuffer::new_impl(display, data, false)
    }

    /// Builds a new vertex buffer.
    ///
    /// This function will create a buffer that has better performances when it is modified
    ///  frequently.
    pub fn new_dynamic(display: &super::Display, data: Vec<T>) -> VertexBuffer<T> {
        VertexBuffer::new_impl(display, data, true)
    }

    fn new_impl(display: &super::Display, data: Vec<T>, dynamic: bool) -> VertexBuffer<T> {
        let bindings = VertexFormat::build_bindings(None::<T>);

        let elements_size = { use std::mem; mem::size_of::<T>() };
        let elements_count = data.len();
        let buffer_size = elements_count * elements_size as uint;

        let usage = if dynamic { gl::DYNAMIC_DRAW } else { gl::STATIC_DRAW };

        let (tx, rx) = channel();

        display.context.context.exec(proc(gl, state) {
            unsafe {
                let mut id: gl::types::GLuint = mem::uninitialized();
                gl.GenBuffers(1, &mut id);

                if gl.NamedBufferData.is_loaded() {
                    gl.NamedBufferData(id, buffer_size as gl::types::GLsizei,
                        data.as_ptr() as *const libc::c_void, usage);
                        
                } else if gl.NamedBufferDataEXT.is_loaded() {
                    gl.NamedBufferDataEXT(id, buffer_size as gl::types::GLsizeiptr,
                        data.as_ptr() as *const libc::c_void, usage);

                } else {
                    gl.BindBuffer(gl::ARRAY_BUFFER, id);
                    state.array_buffer_binding = Some(id);
                    gl.BufferData(gl::ARRAY_BUFFER, buffer_size as gl::types::GLsizeiptr,
                        data.as_ptr() as *const libc::c_void, usage);
                }

                tx.send(id);
            }
        });

        VertexBuffer {
            display: display.context.clone(),
            id: rx.recv(),
            elements_size: elements_size,
            bindings: bindings,
            elements_count: elements_count,
        }
    }

    /// Maps the buffer to allow write access to it.
    pub fn map<'a>(&'a mut self) -> Mapping<'a, T> {
        let (tx, rx) = channel();
        let id = self.id.clone();
        let elements_count = self.elements_count.clone();

        self.display.context.exec(proc(gl, state) {
            let ptr = {
                if gl.MapNamedBuffer.is_loaded() {
                    gl.MapNamedBuffer(id, gl::READ_WRITE)
                } else {
                    if state.array_buffer_binding != Some(id) {
                        gl.BindBuffer(gl::ARRAY_BUFFER, id);
                        state.array_buffer_binding = Some(id);
                    }

                    gl.MapBuffer(gl::ARRAY_BUFFER, gl::READ_WRITE)
                }
            };

            if ptr.is_null() {
                tx.send(Err("glMapBuffer returned null"));
            } else {
                tx.send(Ok(ptr as *mut T));
            }
        });

        Mapping {
            buffer: self,
            data: unsafe { CVec::new(rx.recv().unwrap(), elements_count) },
        }
    }
}

impl<T> fmt::Show for VertexBuffer<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::FormatError> {
        (format!("VertexBuffer #{}", self.id)).fmt(formatter)
    }
}

#[unsafe_destructor]
impl<T> Drop for VertexBuffer<T> {
    fn drop(&mut self) {
        let id = self.id.clone();
        self.display.context.exec(proc(gl, state) {
            if state.array_buffer_binding == Some(id) {
                state.array_buffer_binding = None;
            }

            if state.element_array_buffer_binding == Some(id) {
                state.element_array_buffer_binding = None;
            }

            unsafe { gl.DeleteBuffers(1, [ id ].as_ptr()); }
        });
    }
}

/// Describes the attribute of a vertex.
///
/// When you create a vertex buffer, you need to pass some sort of array of data. In order for
/// OpenGL to use this data, we must tell it some informations about each field of each
/// element. This structure describes one such field.
#[deriving(Show, Clone)]
pub struct VertexAttrib {
    /// The offset, in bytes, between the start of each vertex and the attribute.
    pub offset: uint,

    /// Type of the field.
    pub data_type: gl::types::GLenum,

    /// Number of invidual elements in the attribute.
    ///
    /// For example if `data_type` is a f32 and `elements_count` is 2, then you have a `vec2`.
    pub elements_count: u32,
}

/// Describes the layout of each vertex in a vertex buffer.
pub type VertexBindings = Vec<(String, VertexAttrib)>;

/// Trait for structures that represent a vertex.
pub trait VertexFormat: Copy {
    /// Builds the `VertexBindings` representing the layout of this element.
    fn build_bindings(Option<Self>) -> VertexBindings;
}

/// A mapping of a buffer.
pub struct Mapping<'b, T> {
    buffer: &'b mut VertexBuffer<T>,
    data: CVec<T>,
}

#[unsafe_destructor]
impl<'a, T> Drop for Mapping<'a, T> {
    fn drop(&mut self) {
        let id = self.buffer.id.clone();
        self.buffer.display.context.exec(proc(gl, state) {
            if gl.UnmapNamedBuffer.is_loaded() {
                gl.UnmapNamedBuffer(id);

            } else {
                if state.array_buffer_binding != Some(id) {
                    gl.BindBuffer(gl::ARRAY_BUFFER, id);
                    state.array_buffer_binding = Some(id);
                }

                gl.UnmapBuffer(gl::ARRAY_BUFFER);
            }
        });
    }
}

impl<'a, T> Deref<[T]> for Mapping<'a, T> {
    fn deref<'b>(&'b self) -> &'b [T] {
        self.data.as_slice()
    }
}

impl<'a, T> DerefMut<[T]> for Mapping<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut [T] {
        self.data.as_mut_slice()
    }
}
