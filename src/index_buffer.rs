use buffer::{mod, Buffer};
use data_types;
use gl;
use PrimitiveType;

/// A list of indices loaded in the graphics card's memory.
#[deriving(Show)]
pub struct IndexBuffer {
    buffer: Buffer,
    data_type: gl::types::GLenum,
    primitives: gl::types::GLenum,
}

/// This public function is accessible from within `glium` but not for the user.
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
    /// let index_buffer = glium::IndexBuffer::new(&display, glium::TrianglesList,
    ///     &[0u8, 1, 2, 1, 3, 4, 2, 4, 3]);
    /// # }
    /// ```
    /// 
    pub fn new<T: data_types::GLDataType + Send + Clone>(display: &super::Display,
        prim: PrimitiveType, data: &[T]) -> IndexBuffer
    {
        IndexBuffer {
            buffer: Buffer::new::<buffer::ElementArrayBuffer, T>(display, data.to_vec(),
                gl::STATIC_DRAW),   // TODO: perf loss
            data_type: data_types::GLDataType::get_gl_type(None::<T>),
            primitives: prim.get_gl_enum()
        }
    }
}
