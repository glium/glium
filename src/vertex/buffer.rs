use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use buffer::{self, Buffer};
use vertex::{Vertex, VerticesSource, IntoVerticesSource};
use vertex::format::VertexFormat;

use Display;
use GlObject;

use context;
use version::Api;
use gl;

/// A list of vertices loaded in the graphics card's memory.
#[derive(Debug)]
pub struct VertexBuffer<T> {
    buffer: VertexBufferAny,
    marker: PhantomData<T>,
}

/// Represents a slice of a `VertexBuffer`.
pub struct VertexBufferSlice<'b, T: 'b> {
    buffer: &'b VertexBuffer<T>,
    offset: usize,
    length: usize,
}

impl<T: Vertex + 'static + Send> VertexBuffer<T> {
    /// Builds a new vertex buffer.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[macro_use]
    /// # extern crate glium;
    /// # extern crate glutin;
    /// # fn main() {
    /// #[derive(Copy)]
    /// struct Vertex {
    ///     position: [f32; 3],
    ///     texcoords: [f32; 2],
    /// }
    ///
    /// implement_vertex!(Vertex, position, texcoords);
    ///
    /// # let display: glium::Display = unsafe { ::std::mem::uninitialized() };
    /// let vertex_buffer = glium::VertexBuffer::new(&display, vec![
    ///     Vertex { position: [0.0,  0.0, 0.0], texcoords: [0.0, 1.0] },
    ///     Vertex { position: [5.0, -3.0, 2.0], texcoords: [1.0, 0.0] },
    /// ]);
    /// # }
    /// ```
    ///
    pub fn new(display: &Display, data: Vec<T>) -> VertexBuffer<T> {
        let bindings = <T as Vertex>::build_bindings();

        let buffer = Buffer::new::<buffer::ArrayBuffer, T>(display, data, false);
        let elements_size = buffer.get_elements_size();

        VertexBuffer {
            buffer: VertexBufferAny {
                buffer: buffer,
                bindings: bindings,
                elements_size: elements_size,
            },
            marker: PhantomData,
        }
    }

    /// Builds a new vertex buffer.
    ///
    /// This function will create a buffer that has better performance when it is modified frequently.
    pub fn new_dynamic(display: &Display, data: Vec<T>) -> VertexBuffer<T> {
        let bindings = <T as Vertex>::build_bindings();

        let buffer = Buffer::new::<buffer::ArrayBuffer, T>(display, data, false);
        let elements_size = buffer.get_elements_size();

        VertexBuffer {
            buffer: VertexBufferAny {
                buffer: buffer,
                bindings: bindings,
                elements_size: elements_size,
            },
            marker: PhantomData,
        }
    }

    /// Builds a new vertex buffer with persistent mapping.
    ///
    /// ## Features
    ///
    /// Only available if the `gl_persistent_mapping` feature is enabled.
    #[cfg(feature = "gl_persistent_mapping")]
    pub fn new_persistent(display: &Display, data: Vec<T>) -> VertexBuffer<T> {
        VertexBuffer::new_persistent_if_supported(display, data).unwrap()
    }

    /// Builds a new vertex buffer with persistent mapping, or `None` if this is not supported.
    pub fn new_persistent_if_supported(display: &Display, data: Vec<T>)
                                       -> Option<VertexBuffer<T>>
    {
        if display.context.context.get_version() < &context::GlVersion(Api::Gl, 4, 4) &&
           !display.context.context.get_extensions().gl_arb_buffer_storage
        {
            return None;
        }

        let bindings = <T as Vertex>::build_bindings();

        let buffer = Buffer::new::<buffer::ArrayBuffer, T>(display, data, true);
        let elements_size = buffer.get_elements_size();

        Some(VertexBuffer {
            buffer: VertexBufferAny {
                buffer: buffer,
                bindings: bindings,
                elements_size: elements_size,
            },
            marker: PhantomData,
        })
    }
}

impl<T: Send + Copy + 'static> VertexBuffer<T> {
    /// Builds a new vertex buffer from an indeterminate data type and bindings.
    ///
    /// # Example
    ///
    /// ```no_run
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
    ///     glium::VertexBuffer::new_raw(&display, data, bindings, 3 * ::std::mem::size_of::<f32>())
    /// };
    /// # }
    /// ```
    ///
    pub unsafe fn new_raw(display: &Display, data: Vec<T>,
                          bindings: VertexFormat, elements_size: usize) -> VertexBuffer<T>
    {
        VertexBuffer {
            buffer: VertexBufferAny {
                buffer: Buffer::new::<buffer::ArrayBuffer, T>(display, data, false),
                bindings: bindings,
                elements_size: elements_size,
            },
            marker: PhantomData,
        }
    }

    /// Accesses a slice of the buffer.
    ///
    /// Returns `None` if the slice is out of range.
    pub fn slice(&self, offset: usize, len: usize) -> Option<VertexBufferSlice<T>> {
        if offset > self.len() || offset + len > self.len() {
            return None;
        }

        Some(VertexBufferSlice {
            buffer: self,
            offset: offset,
            length: len
        })
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

    /// Replaces the content of the buffer.
    ///
    /// ## Panic
    ///
    /// Panics if the length of `data` is different from the length of this buffer.
    pub fn write(&self, data: Vec<T>) {
        assert!(data.len() == self.len());
        self.buffer.buffer.upload::<buffer::ArrayBuffer, _>(0, data)
    }
}

impl<T> VertexBuffer<T> {
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

    /// Discard the type information and turn the vertex buffer into a `VertexBufferAny`.
    pub fn into_vertex_buffer_any(self) -> VertexBufferAny {
        self.buffer
    }

    /// Returns the number of elements in the buffer.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

impl<T> GlObject for VertexBuffer<T> {
    type Id = gl::types::GLuint;
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
    }
}

impl<'a, T> IntoVerticesSource<'a> for &'a VertexBuffer<T> {
    fn into_vertices_source(self) -> VerticesSource<'a> {
        (&self.buffer).into_vertices_source()
    }
}

impl<'b, T> VertexBufferSlice<'b, T> where T: Send + Copy + 'static {
    /// Reads the content of the slice.
    ///
    /// This function is usually better if are just doing one punctual read, while `map`
    /// is better if you want to have multiple small reads.
    ///
    /// # Features
    ///
    /// Only available if the `gl_read_buffer` feature is enabled.
    #[cfg(feature = "gl_read_buffer")]
    pub fn read(&self) -> Vec<T> {
        self.buffer.buffer.buffer.read_slice::<buffer::ArrayBuffer, T>(self.offset, self.length)
    }

    /// Reads the content of the buffer.
    ///
    /// This function is usually better if are just doing one punctual read, while `map`
    /// is better if you want to have multiple small reads.
    pub fn read_if_supported(&self) -> Option<Vec<T>> {
        self.buffer.buffer.buffer.read_slice_if_supported::<buffer::ArrayBuffer, T>(self.offset,
                                                                                    self.length)
    }

    /// Writes some vertices to the buffer.
    ///
    /// ## Panic
    ///
    /// Panics if the length of `data` is different from the length of this slice.
    pub fn write(&self, data: Vec<T>) {
        assert!(data.len() == self.length);
        self.buffer.buffer.buffer.upload::<buffer::ArrayBuffer, _>(self.offset, data)
    }
}

impl<'a, T> IntoVerticesSource<'a> for VertexBufferSlice<'a, T> {
    fn into_vertices_source(self) -> VerticesSource<'a> {
        let fence = if self.buffer.buffer.buffer.is_persistent() {
            Some(self.buffer.buffer.buffer.add_fence())
        } else {
            None
        };

        VerticesSource::VertexBuffer(&self.buffer.buffer, fence, self.offset, self.length)
    }
}

/// A list of vertices loaded in the graphics card's memory.
///
/// Contrary to `VertexBuffer`, this struct doesn't know about the type of data
/// inside the buffer. Therefore you can't map or read it.
///
/// This struct is provided for convenience, so that you can have a `Vec<VertexBufferAny>`,
/// or return a `VertexBufferAny` instead of a `VertexBuffer<MyPrivateVertexType>`.
#[derive(Debug)]
pub struct VertexBufferAny {
    buffer: Buffer,
    bindings: VertexFormat,
    elements_size: usize,
}

/// Represents a slice of a `VertexBufferAny`.
pub struct VertexBufferAnySlice<'b> {
    buffer: &'b VertexBufferAny,
    offset: usize,
    length: usize,
}

impl VertexBufferAny {
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

    /// Turns the vertex buffer into a `VertexBuffer` without checking the type.
    pub unsafe fn into_vertex_buffer<T>(self) -> VertexBuffer<T> {
        VertexBuffer {
            buffer: self,
            marker: PhantomData,
        }
    }

    /// Accesses a slice of the buffer.
    ///
    /// Returns `None` if the slice is out of range.
    pub fn slice(&self, offset: usize, len: usize) -> Option<VertexBufferAnySlice> {
        if offset >= self.len() || offset + len >= self.len() {
            return None;
        }

        Some(VertexBufferAnySlice {
            buffer: self,
            offset: offset,
            length: len
        })
    }
}

impl Drop for VertexBufferAny {
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

impl GlObject for VertexBufferAny {
    type Id = gl::types::GLuint;
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
    }
}

impl<'a> IntoVerticesSource<'a> for &'a VertexBufferAny {
    fn into_vertices_source(self) -> VerticesSource<'a> {
        let fence = if self.buffer.is_persistent() {
            Some(self.buffer.add_fence())
        } else {
            None
        };

        VerticesSource::VertexBuffer(self, fence, 0, self.len())
    }
}

impl<'a> IntoVerticesSource<'a> for VertexBufferAnySlice<'a> {
    fn into_vertices_source(self) -> VerticesSource<'a> {
        let fence = if self.buffer.buffer.is_persistent() {
            Some(self.buffer.buffer.add_fence())
        } else {
            None
        };

        VerticesSource::VertexBuffer(self.buffer, fence, self.offset, self.length)
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
