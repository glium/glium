use buffer::{mod, Buffer};
use gl;

/// A list of verices loaded in the graphics card's memory.
#[deriving(Show)]
pub struct VertexBuffer<T> {
    buffer: Buffer,
    bindings: VertexBindings,
}

/// Don't use this function outside of glium
#[doc(hidden)]
pub fn get_id<T>(vb: &VertexBuffer<T>) -> gl::types::GLuint {
    vb.buffer.get_id()
}

/// Don't use this function outside of glium
#[doc(hidden)]
pub fn get_elements_size<T>(vb: &VertexBuffer<T>) -> uint {
    vb.buffer.get_elements_size()
}

/// Don't use this function outside of glium
#[doc(hidden)]
pub fn get_bindings<T>(vb: &VertexBuffer<T>) -> &VertexBindings {
    &vb.bindings
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
    /// # let display: glium::Display = glutin::HeadlessRendererBuilder::new(1024, 768)
    /// #   .build_glium().unwrap();
    /// let vertex_buffer = glium::VertexBuffer::new(&display, vec![
    ///     Vertex { position: [0.0,  0.0, 0.0], texcoords: [0.0, 1.0] },
    ///     Vertex { position: [5.0, -3.0, 2.0], texcoords: [1.0, 0.0] },
    /// ]);
    /// # }
    /// ```
    /// 
    pub fn new(display: &super::Display, data: Vec<T>) -> VertexBuffer<T> {
        let bindings = VertexFormat::build_bindings(None::<T>);

        VertexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, T>(display, data, gl::STATIC_DRAW),
            bindings: bindings,
        }
    }

    /// Builds a new vertex buffer.
    ///
    /// This function will create a buffer that has better performances when it is modified
    ///  frequently.
    pub fn new_dynamic(display: &super::Display, data: Vec<T>) -> VertexBuffer<T> {
        let bindings = VertexFormat::build_bindings(None::<T>);

        VertexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, T>(display, data, gl::DYNAMIC_DRAW),
            bindings: bindings,
        }
    }

    /// Maps the buffer to allow write access to it.
    ///
    /// **Warning**: using this function can slow things down a lot because the function
    /// waits for all the previous commands to be executed before returning.
    pub fn map<'a>(&'a mut self) -> Mapping<'a, T> {
        Mapping(self.buffer.map::<buffer::ArrayBuffer, T>())
    }

    /// Reads the content of the buffer.
    ///
    /// This function is usually better if are just doing one punctual read, while `map` is better
    /// if you want to have multiple small reads.
    pub fn read(&self) -> Vec<T> {
        self.buffer.read::<buffer::ArrayBuffer, T>()
    }

    /// Reads the content of the buffer.
    ///
    /// This function is usually better if are just doing one punctual read, while `map` is better
    /// if you want to have multiple small reads.
    ///
    /// The offset and size are expressed in number of elements.
    ///
    /// ## Panic
    ///
    /// Panics if `offset` or `offset + size` are greated than the size of the buffer.
    pub fn read_slice(&self, offset: uint, size: uint) -> Vec<T> {
        self.buffer.read_slice::<buffer::ArrayBuffer, T>(offset, size)
    }
}

#[unsafe_destructor]
impl<T> Drop for VertexBuffer<T> {
    fn drop(&mut self) {
        // removing VAOs which contain this vertex buffer
        let mut vaos = self.buffer.get_display().vertex_array_objects.lock();
        let to_delete = vaos.keys().filter(|&&(v, _)| v == self.buffer.get_id())
            .map(|k| k.clone()).collect::<Vec<_>>();
        for k in to_delete.into_iter() {
            vaos.remove(&k);
        }
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
pub struct Mapping<'a, T>(buffer::Mapping<'a, buffer::ArrayBuffer, T>);

impl<'a, T> Deref<[T]> for Mapping<'a, T> {
    fn deref<'b>(&'b self) -> &'b [T] {
        self.0.deref()
    }
}

impl<'a, T> DerefMut<[T]> for Mapping<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut [T] {
        self.0.deref_mut()
    }
}
