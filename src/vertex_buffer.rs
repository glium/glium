use buffer::{mod, Buffer};
use gl;
use GlObject;

/// A list of verices loaded in the graphics card's memory.
#[deriving(Show)]
pub struct VertexBuffer<T> {
    buffer: Buffer,
    bindings: VertexBindings,
    elements_size: uint,
}

/// Don't use this function outside of glium
#[doc(hidden)]
pub fn get_elements_size<T>(vb: &VertexBuffer<T>) -> uint {
    vb.elements_size
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
    /// ```no_run
    /// # #![feature(phase)]
    /// # #[phase(plugin)]
    /// # extern crate glium_macros;
    /// # extern crate glium;
    /// # extern crate glutin;
    /// # fn main() {
    /// #[vertex_format]
    /// #[deriving(Copy)]
    /// struct Vertex {
    ///     position: [f32, ..3],
    ///     texcoords: [f32, ..2],
    /// }
    ///
    /// # let display: glium::Display = unsafe { ::std::mem::uninitialized() };
    /// let vertex_buffer = glium::VertexBuffer::new(&display, vec![
    ///     Vertex { position: [0.0,  0.0, 0.0], texcoords: [0.0, 1.0] },
    ///     Vertex { position: [5.0, -3.0, 2.0], texcoords: [1.0, 0.0] },
    /// ]);
    /// # }
    /// ```
    /// 
    pub fn new(display: &super::Display, data: Vec<T>) -> VertexBuffer<T> {
        let bindings = VertexFormat::build_bindings(None::<T>);

        let buffer = Buffer::new::<buffer::ArrayBuffer, T>(display, data, gl::STATIC_DRAW);
        let elements_size = buffer.get_elements_size();

        VertexBuffer {
            buffer: buffer,
            bindings: bindings,
            elements_size: elements_size,
        }
    }

    /// Builds a new vertex buffer.
    ///
    /// This function will create a buffer that has better performances when it is modified
    ///  frequently.
    pub fn new_dynamic(display: &super::Display, data: Vec<T>) -> VertexBuffer<T> {
        let bindings = VertexFormat::build_bindings(None::<T>);

        let buffer = Buffer::new::<buffer::ArrayBuffer, T>(display, data, gl::DYNAMIC_DRAW);
        let elements_size = buffer.get_elements_size();

        VertexBuffer {
            buffer: buffer,
            bindings: bindings,
            elements_size: elements_size,
        }
    }
}

impl<T: Send + Copy> VertexBuffer<T> {
    /// Builds a new vertex buffer from an undeterminate data type and bindings.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #![feature(phase)]
    /// # #[phase(plugin)]
    /// # extern crate glium_macros;
    /// # extern crate glium;
    /// # extern crate glutin;
    /// # fn main() {
    /// let bindings = vec![
    ///     ("position".to_string(), glium::vertex_buffer::VertexAttrib {
    ///         offset: 0,
    ///         data_type: 0x1406,  // GL_FLOAT
    ///         elements_count: 2,
    ///     }),
    ///     ("color".to_string(), glium::vertex_buffer::VertexAttrib {
    ///         offset: 2 * ::std::mem::size_of::<f32>(),
    ///         data_type: 0x1406,  // GL_FLOAT
    ///         elements_count: 1,
    ///     }),
    /// ];
    ///
    /// # let display: glium::Display = unsafe { ::std::mem::uninitialized() };
    /// let data = vec![
    ///     1.0, -0.3, 409.0,
    ///     -0.4, 2.8, 715.0f32
    /// ];
    ///
    /// let vertex_buffer = unsafe {
    ///     glium::VertexBuffer::new_raw(&display, data, bindings, 3 * ::std::mem::size_of::<f32>())
    /// };
    /// # }
    /// ```
    ///
    #[experimental]
    pub unsafe fn new_raw(display: &super::Display, data: Vec<T>,
                          bindings: VertexBindings, elements_size: uint) -> VertexBuffer<T>
    {
        VertexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, T>(display, data, gl::STATIC_DRAW),
            bindings: bindings,
            elements_size: elements_size,
        }
    }

    /// Maps the buffer to allow write access to it.
    ///
    /// **Warning**: using this function can slow things down a lot because the function
    /// waits for all the previous commands to be executed before returning.
    ///
    /// # Panic
    ///
    /// OpenGL ES doesn't support mapping buffers. Using this function will thus panic.
    /// If you want to be compatible with all platforms, it is preferable to disable the
    /// `gl_extensions` feature.
    ///
    /// # Features
    ///
    /// Only available if the `gl_extensions` feature is enabled.
    #[cfg(feature = "gl_extensions")]
    pub fn map<'a>(&'a mut self) -> Mapping<'a, T> {
        Mapping(self.buffer.map::<buffer::ArrayBuffer, T>())
    }

    /// Reads the content of the buffer.
    ///
    /// This function is usually better if are just doing one punctual read, while `map` is better
    /// if you want to have multiple small reads.
    ///
    /// # Panic
    ///
    /// OpenGL ES doesn't support reading buffers. Using this function will thus panic.
    /// If you want to be compatible with all platforms, it is preferable to disable the
    /// `gl_extensions` feature.
    ///
    /// # Features
    ///
    /// Only available if the `gl_extensions` feature is enabled.
    #[cfg(feature = "gl_extensions")]
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
    ///
    /// OpenGL ES doesn't support reading buffers. Using this function will thus panic.
    /// If you want to be compatible with all platforms, it is preferable to disable the
    /// `gl_extensions` feature.
    ///
    /// # Features
    ///
    /// Only available if the `gl_extensions` feature is enabled.
    #[cfg(feature = "gl_extensions")]
    pub fn read_slice(&self, offset: uint, size: uint) -> Vec<T> {
        self.buffer.read_slice::<buffer::ArrayBuffer, T>(offset, size)
    }
}

#[unsafe_destructor]
impl<T> Drop for VertexBuffer<T> {
    fn drop(&mut self) {
        // removing VAOs which contain this vertex buffer
        let mut vaos = self.buffer.get_display().vertex_array_objects.lock();
        let to_delete = vaos.keys().filter(|&&(v, _, _)| v == self.buffer.get_id())
            .map(|k| k.clone()).collect::<Vec<_>>();
        for k in to_delete.into_iter() {
            vaos.remove(&k);
        }
    }
}

impl<T> GlObject for VertexBuffer<T> {
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
    }
}

/// Describes the attribute of a vertex.
///
/// When you create a vertex buffer, you need to pass some sort of array of data. In order for
/// OpenGL to use this data, we must tell it some informations about each field of each
/// element. This structure describes one such field.
#[deriving(Show, Clone, Copy)]
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
///
/// # Features
///
/// Only available if the `gl_extensions` feature is enabled.
#[cfg(feature = "gl_extensions")]
pub struct Mapping<'a, T>(buffer::Mapping<'a, buffer::ArrayBuffer, T>);

#[cfg(feature = "gl_extensions")]
impl<'a, T> Deref<[T]> for Mapping<'a, T> {
    fn deref<'b>(&'b self) -> &'b [T] {
        self.0.deref()
    }
}

#[cfg(feature = "gl_extensions")]
impl<'a, T> DerefMut<[T]> for Mapping<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut [T] {
        self.0.deref_mut()
    }
}
