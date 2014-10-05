use gl;
use libc;
use std::collections::HashMap;
use std::fmt;
use std::mem;
use std::sync::Arc;

/// A list of verices loaded in the graphics card's memory.
pub struct VertexBuffer<T> {
    display: Arc<super::DisplayImpl>,
    id: gl::types::GLuint,
    elements_size: uint,
    bindings: VertexBindings,
}

/// This public function is accessible from within `glium_core` but not for the user.
pub fn get_clone<T>(vb: &VertexBuffer<T>) -> (gl::types::GLuint, uint, VertexBindings) {
    (vb.id.clone(), vb.elements_size.clone(), vb.bindings.clone())
}

impl<T: VertexFormat + 'static + Send> VertexBuffer<T> {
    /// Builds a new vertex buffer.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #![feature(phase)]
    /// # #[phase(plugin)]
    /// # extern crate glium_core_macros;
    /// # extern crate glium_core;
    /// # fn main() {
    /// #[vertex_format]
    /// struct Vertex {
    ///     position: [f32, ..3],
    ///     texcoords: [f32, ..2],
    /// }
    ///
    /// # let display: glium_core::Display = unsafe { std::mem::uninitialized() };
    /// let vertex_buffer = glium_core::VertexBuffer::new(&display, vec![
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
    /// This function will create a buffer that has better performances when the it is modified
    ///  frequently.
    pub fn new_dynamic(display: &super::Display, data: Vec<T>) -> VertexBuffer<T> {
        VertexBuffer::new_impl(display, data, true)
    }

    fn new_impl(display: &super::Display, data: Vec<T>, dynamic: bool) -> VertexBuffer<T> {
        let bindings = VertexFormat::build_bindings(None::<T>);

        let elements_size = { use std::mem; mem::size_of::<T>() };
        let buffer_size = data.len() * elements_size as uint;

        let usage = if dynamic { gl::DYNAMIC_DRAW } else { gl::STATIC_DRAW };

        let (tx, rx) = channel();

        display.context.context.exec(proc(gl, state) {
            unsafe {
                let mut id: gl::types::GLuint = mem::uninitialized();
                gl.GenBuffers(1, &mut id);
                gl.BindBuffer(gl::ARRAY_BUFFER, id);
                state.array_buffer_binding = Some(id);
                gl.BufferData(gl::ARRAY_BUFFER, buffer_size as gl::types::GLsizeiptr,
                    data.as_ptr() as *const libc::c_void, usage);
                tx.send(id);
            }
        });

        VertexBuffer {
            display: display.context.clone(),
            id: rx.recv(),
            elements_size: elements_size,
            bindings: bindings
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

/// For each binding, the data type, number of elements, and offset.
/// Includes the total size.
#[doc(hidden)]
pub type VertexBindings = HashMap<String, (gl::types::GLenum, gl::types::GLint, uint)>;

/// Trait for structures that represent a vertex.
#[doc(hidden)]
pub trait VertexFormat: Copy {
    fn build_bindings(Option<Self>) -> VertexBindings;
}
