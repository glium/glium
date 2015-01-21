use std::ops::{Deref, DerefMut};

use buffer::{self, Buffer};
use vertex::{Vertex, VerticesSource, IntoVerticesSource};
use vertex::format::VertexFormat;

use Display;
use GlObject;

use context;
use gl;

/// A list of vertices loaded in the graphics card's memory.
#[derive(Show)]
pub struct PerInstanceAttributesBuffer<T> {
    buffer: PerInstanceAttributesBufferAny,
}

impl<T: Vertex + 'static + Send> PerInstanceAttributesBuffer<T> {
    /// Builds a new vertex buffer.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #![feature(plugin)]
    /// # #[plugin]
    /// # extern crate glium_macros;
    /// # extern crate glium;
    /// # extern crate glutin;
    /// # fn main() {
    /// #[vertex_format]
    /// #[derive(Copy)]
    /// struct Vertex {
    ///     position: [f32; 3],
    ///     texcoords: [f32; 2],
    /// }
    ///
    /// # let display: glium::Display = unsafe { ::std::mem::uninitialized() };
    /// let vertex_buffer = glium::vertex::PerInstanceAttributesBuffer::new(&display, vec![
    ///     Vertex { position: [0.0,  0.0, 0.0], texcoords: [0.0, 1.0] },
    ///     Vertex { position: [5.0, -3.0, 2.0], texcoords: [1.0, 0.0] },
    /// ]);
    /// # }
    /// ```
    ///
    #[cfg(feature = "gl_instancing")]
    pub fn new(display: &Display, data: Vec<T>) -> PerInstanceAttributesBuffer<T> {
        PerInstanceAttributesBuffer::new_if_supported(display, data).unwrap()
    }

    /// Builds a new buffer.
    pub fn new_if_supported(display: &Display, data: Vec<T>)
                            -> Option<PerInstanceAttributesBuffer<T>>
    {
        if display.context.context.get_version() < &context::GlVersion(3, 3) &&
            !display.context.context.get_extensions().gl_arb_instanced_arrays
        {
            return None;
        }

        let bindings = <T as Vertex>::build_bindings();

        let buffer = Buffer::new::<buffer::ArrayBuffer, T>(display, data, false);
        let elements_size = buffer.get_elements_size();

        Some(PerInstanceAttributesBuffer {
            buffer: PerInstanceAttributesBufferAny {
                buffer: buffer,
                bindings: bindings,
                elements_size: elements_size,
            }
        })
    }

    /// Builds a new vertex buffer.
    ///
    /// This function will create a buffer that has better performance when it is modified frequently.
    pub fn new_dynamic(display: &Display, data: Vec<T>) -> PerInstanceAttributesBuffer<T> {
        let bindings = <T as Vertex>::build_bindings();

        let buffer = Buffer::new::<buffer::ArrayBuffer, T>(display, data, false);
        let elements_size = buffer.get_elements_size();

        PerInstanceAttributesBuffer {
            buffer: PerInstanceAttributesBufferAny {
                buffer: buffer,
                bindings: bindings,
                elements_size: elements_size,
            }
        }
    }

    /// Builds a new vertex buffer with persistent mapping.
    ///
    /// ## Features
    ///
    /// Only available if the `gl_persistent_mapping` feature is enabled.
    #[cfg(all(feature = "gl_persistent_mapping", feature = "gl_instancing"))]
    pub fn new_persistent(display: &Display, data: Vec<T>) -> PerInstanceAttributesBuffer<T> {
        PerInstanceAttributesBuffer::new_persistent_if_supported(display, data).unwrap()
    }

    /// Builds a new vertex buffer with persistent mapping, or `None` if this is not supported.
    pub fn new_persistent_if_supported(display: &Display, data: Vec<T>)
                                       -> Option<PerInstanceAttributesBuffer<T>>
    {
        if display.context.context.get_version() < &context::GlVersion(3, 3) &&
            !display.context.context.get_extensions().gl_arb_instanced_arrays
        {
            return None;
        }

        if display.context.context.get_version() < &context::GlVersion(4, 4) &&
           !display.context.context.get_extensions().gl_arb_buffer_storage
        {
            return None;
        }

        let bindings = <T as Vertex>::build_bindings();

        let buffer = Buffer::new::<buffer::ArrayBuffer, T>(display, data, true);
        let elements_size = buffer.get_elements_size();

        Some(PerInstanceAttributesBuffer {
            buffer: PerInstanceAttributesBufferAny {
                buffer: buffer,
                bindings: bindings,
                elements_size: elements_size,
            }
        })
    }
}

impl<T: Send + Copy> PerInstanceAttributesBuffer<T> {
    /// Builds a new vertex buffer from an indeterminate data type and bindings.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #![feature(plugin)]
    /// # #[plugin]
    /// # extern crate glium_macros;
    /// # extern crate glium;
    /// # extern crate glutin;
    /// # fn main() {
    /// let bindings = vec![(
    ///         format!("position"), 0,
    ///         glium::vertex::AttributeType::F32F32,
    ///     ), (
    ///         format!("color"), 2 * ::std::mem::size_of::<f32>(),
    ///         glium::vertex::AttributeType::F32,
    ///     ),
    /// ];
    ///
    /// # let display: glium::Display = unsafe { ::std::mem::uninitialized() };
    /// let data = vec![
    ///     1.0, -0.3, 409.0,
    ///     -0.4, 2.8, 715.0f32
    /// ];
    ///
    /// let vertex_buffer = unsafe {
    ///     glium::vertex::PerInstanceAttributesBuffer::new_raw(&display, data, bindings, 3 * ::std::mem::size_of::<f32>())
    /// };
    /// # }
    /// ```
    ///
    #[experimental]
    pub unsafe fn new_raw(display: &Display, data: Vec<T>,
                          bindings: VertexFormat, elements_size: usize) -> PerInstanceAttributesBuffer<T>
    {
        PerInstanceAttributesBuffer {
            buffer: PerInstanceAttributesBufferAny {
                buffer: Buffer::new::<buffer::ArrayBuffer, T>(display, data, false),
                bindings: bindings,
                elements_size: elements_size,
            }
        }
    }

    /// Maps the buffer to allow write access to it.
    ///
    /// This function will block until the buffer stops being used by the backend.
    /// This operation is much faster if the buffer is persistent.
    pub fn map<'a>(&'a mut self) -> Mapping<'a, T> {
        let len = self.buffer.buffer.get_elements_count();
        let mapping = self.buffer.buffer.map::<buffer::ArrayBuffer, T>(0, len);
        Mapping(mapping)
    }

    /// Reads the content of the buffer.
    ///
    /// This function is usually better if are just doing one punctual read, while `map`
    /// is better if you want to have multiple small reads.
    ///
    /// # Features
    ///
    /// Only available if the `gl_read_buffer` feature is enabled.
    #[cfg(feature = "gl_read_buffer")]
    pub fn read(&self) -> Vec<T> {
        self.buffer.buffer.read::<buffer::ArrayBuffer, T>()
    }

    /// Reads the content of the buffer.
    ///
    /// This function is usually better if are just doing one punctual read, while `map`
    /// is better if you want to have multiple small reads.
    pub fn read_if_supported(&self) -> Option<Vec<T>> {
        self.buffer.buffer.read_if_supported::<buffer::ArrayBuffer, T>()
    }

    /// Reads the content of the buffer.
    ///
    /// This function is usually better if are just doing one punctual read, while `map`
    /// is better if you want to have multiple small reads.
    ///
    /// The offset and size are expressed in number of elements.
    ///
    /// ## Panic
    ///
    /// Panics if `offset` or `offset + size` are greated than the size of the buffer.
    ///
    /// # Features
    ///
    /// Only available if the `gl_read_buffer` feature is enabled.
    #[cfg(feature = "gl_read_buffer")]
    pub fn read_slice(&self, offset: usize, size: usize) -> Vec<T> {
        self.buffer.buffer.read_slice::<buffer::ArrayBuffer, T>(offset, size)
    }

    /// Reads the content of the buffer.
    ///
    /// This function is usually better if are just doing one punctual read, while `map`
    /// is better if you want to have multiple small reads.
    ///
    /// The offset and size are expressed in number of elements.
    ///
    /// ## Panic
    ///
    /// Panics if `offset` or `offset + size` are greated than the size of the buffer.
    pub fn read_slice_if_supported(&self, offset: usize, size: usize) -> Option<Vec<T>> {
        self.buffer.buffer.read_slice_if_supported::<buffer::ArrayBuffer, T>(offset, size)
    }

    /// Writes some vertices to the buffer.
    ///
    /// Replaces some vertices in the buffer with others.
    /// The `offset` represents a number of vertices, not a number of bytes.
    pub fn write(&mut self, offset: usize, data: Vec<T>) {
        self.buffer.buffer.upload::<buffer::ArrayBuffer, _>(offset, data)
    }
}

impl<T> PerInstanceAttributesBuffer<T> {
    /// Returns true if the buffer is mapped in a permanent way in memory.
    pub fn is_persistent(&self) -> bool {
        self.buffer.buffer.is_persistent()
    }

    /// Returns the number of bytes between two consecutive elements in the buffer.
    pub fn get_elements_size(&self) -> usize {
        self.buffer.elements_size
    }

    /// Returns the associated `VertexFormat`.
    pub fn get_bindings(&self) -> &VertexFormat {
        &self.buffer.bindings
    }

    /// Discard the type information and turn the vertex buffer into a `PerInstanceAttributesBufferAny`.
    pub fn into_vertex_buffer_any(self) -> PerInstanceAttributesBufferAny {
        self.buffer
    }
}

impl<T> GlObject for PerInstanceAttributesBuffer<T> {
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
    }
}

impl<'a, T> IntoVerticesSource<'a> for &'a PerInstanceAttributesBuffer<T> {
    fn into_vertices_source(self) -> VerticesSource<'a> {
        (&self.buffer).into_vertices_source()
    }
}

/// A list of vertices loaded in the graphics card's memory.
///
/// Contrary to `PerInstanceAttributesBuffer`, this struct doesn't know about the type of data
/// inside the buffer. Therefore you can't map or read it.
///
/// This struct is provided for convenience, so that you can have a `Vec<PerInstanceAttributesBufferAny>`,
/// or return a `PerInstanceAttributesBufferAny` instead of a `PerInstanceAttributesBuffer<MyPrivateVertexType>`.
#[derive(Show)]
pub struct PerInstanceAttributesBufferAny {
    buffer: Buffer,
    bindings: VertexFormat,
    elements_size: usize,
}

impl PerInstanceAttributesBufferAny {
    /// Returns the number of bytes between two consecutive elements in the buffer.
    pub fn get_elements_size(&self) -> usize {
        self.elements_size
    }

    /// Returns the number of elements in the buffer.
    pub fn len(&self) -> usize {
        self.buffer.get_elements_count()
    }

    /// Returns the associated `VertexFormat`.
    pub fn get_bindings(&self) -> &VertexFormat {
        &self.bindings
    }

    /// Turns the vertex buffer into a `PerInstanceAttributesBuffer` without checking the type.
    pub unsafe fn into_vertex_buffer<T>(self) -> PerInstanceAttributesBuffer<T> {
        PerInstanceAttributesBuffer {
            buffer: self,
        }
    }
}

impl Drop for PerInstanceAttributesBufferAny {
    fn drop(&mut self) {
        // removing VAOs which contain this vertex buffer
        let mut vaos = self.buffer.get_display().context.vertex_array_objects.lock().unwrap();
        let to_delete = vaos.keys()
                            .filter(|&&(ref v, _)| {
                                v.iter().find(|&&b| b == self.buffer.get_id()).is_some()
                            })
                            .map(|k| k.clone()).collect::<Vec<_>>();
        for k in to_delete.into_iter() {
            vaos.remove(&k);
        }
    }
}

impl GlObject for PerInstanceAttributesBufferAny {
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
    }
}

impl<'a> IntoVerticesSource<'a> for &'a PerInstanceAttributesBufferAny {
    fn into_vertices_source(self) -> VerticesSource<'a> {
        let fence = if self.buffer.is_persistent() {
            Some(self.buffer.add_fence())
        } else {
            None
        };

        VerticesSource::PerInstanceBuffer(self, fence)
    }
}

/// A mapping of a buffer.
pub struct Mapping<'a, T>(buffer::Mapping<'a, buffer::ArrayBuffer, T>);

impl<'a, T> Deref for Mapping<'a, T> {
    type Target = [T];
    fn deref<'b>(&'b self) -> &'b [T] {
        self.0.deref()
    }
}

impl<'a, T> DerefMut for Mapping<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut [T] {
        self.0.deref_mut()
    }
}
