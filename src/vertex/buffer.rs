use std::ops::{Range, Deref, DerefMut};

use buffer::{BufferView, BufferViewSlice, BufferViewAny, BufferType};
use vertex::{Vertex, VerticesSource, IntoVerticesSource, PerInstance};
use vertex::format::VertexFormat;

use backend::Facade;
use version::{Api, Version};

use ContextExt;

/// A list of vertices loaded in the graphics card's memory.
#[derive(Debug)]
pub struct VertexBuffer<T> where T: Copy + Send + 'static {
    buffer: BufferView<T>,
    bindings: VertexFormat,
}

/// Represents a slice of a `VertexBuffer`.
pub struct VertexBufferSlice<'b, T: 'b> where T: Copy + Send + 'static {
    buffer: BufferViewSlice<'b, T>,
    bindings: &'b VertexFormat,
}

impl<T: Vertex + 'static + Send> VertexBuffer<T> {
    /// Builds a new vertex buffer.
    ///
    /// Note that operations such as `write` will be very slow. If you want to modify the buffer
    /// from time to time, you should use the `dynamic` function instead.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[macro_use]
    /// # extern crate glium;
    /// # extern crate glutin;
    /// # fn main() {
    /// #[derive(Copy, Clone)]
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
    pub fn new<F, D>(facade: &F, data: D) -> VertexBuffer<T> where F: Facade, D: AsRef<[T]> {
        let buffer = BufferView::new(facade, data.as_ref(), BufferType::ArrayBuffer,
                                    false).unwrap();
        buffer.into()
    }

    /// Builds a new vertex buffer.
    ///
    /// This function will create a buffer that is intended to be modified frequently.
    pub fn dynamic<F, D>(facade: &F, data: D) -> VertexBuffer<T> where F: Facade, D: AsRef<[T]> {
        let buffer = BufferView::new(facade, data.as_ref(), BufferType::ArrayBuffer,
                                    true).unwrap();
        buffer.into()
    }

    /// Builds an empty vertex buffer.
    ///
    /// The parameter indicates the number of elements.
    pub fn empty<F>(facade: &F, elements: usize) -> VertexBuffer<T> where F: Facade {
        let buffer = BufferView::empty(facade, BufferType::ArrayBuffer, elements, false).unwrap();
        buffer.into()
    }

    /// Builds an empty vertex buffer.
    ///
    /// The parameter indicates the number of elements.
    pub fn empty_dynamic<F>(facade: &F, elements: usize) -> VertexBuffer<T> where F: Facade {
        let buffer = BufferView::empty(facade, BufferType::ArrayBuffer, elements, true).unwrap();
        buffer.into()
    }
}

impl<T> VertexBuffer<T> where T: Send + Copy + 'static {
    /// Builds a new vertex buffer from an indeterminate data type and bindings.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # extern crate glium;
    /// # extern crate glutin;
    /// # fn main() {
    /// use std::borrow::Cow;
    ///
    /// let bindings = Cow::Owned(vec![(
    ///         Cow::Borrowed("position"), 0,
    ///         glium::vertex::AttributeType::F32F32,
    ///     ), (
    ///         Cow::Borrowed("color"), 2 * ::std::mem::size_of::<f32>(),
    ///         glium::vertex::AttributeType::F32,
    ///     ),
    /// ]);
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
    pub unsafe fn new_raw<F>(facade: &F, data: Vec<T>,
                             bindings: VertexFormat, elements_size: usize) -> VertexBuffer<T>
                             where F: Facade
    {
        VertexBuffer {
            buffer: BufferView::new(facade, &data, BufferType::ArrayBuffer,
                                   false).unwrap(),
            bindings: bindings,
        }
    }

    /// Dynamic version of `new_raw`.
    pub unsafe fn new_raw_dynamic<F>(facade: &F, data: Vec<T>,
                             bindings: VertexFormat, elements_size: usize) -> VertexBuffer<T>
                             where F: Facade
    {
        VertexBuffer {
            buffer: BufferView::new(facade, &data, BufferType::ArrayBuffer,
                                   true).unwrap(),
            bindings: bindings,
        }
    }

    /// Accesses a slice of the buffer.
    ///
    /// Returns `None` if the slice is out of range.
    pub fn slice(&self, range: Range<usize>) -> Option<VertexBufferSlice<T>> {
        let slice = match self.buffer.slice(range) {
            None => return None,
            Some(s) => s
        };

        Some(VertexBufferSlice {
            buffer: slice,
            bindings: &self.bindings,
        })
    }

    /// Returns the associated `VertexFormat`.
    pub fn get_bindings(&self) -> &VertexFormat {
        &self.bindings
    }

    /// DEPRECATED: use `.into()` instead.
    /// Discard the type information and turn the vertex buffer into a `VertexBufferAny`.
    pub fn into_vertex_buffer_any(self) -> VertexBufferAny {
        VertexBufferAny {
            buffer: self.buffer.into(),
            bindings: self.bindings,
        }
    }

    /// Creates a marker that instructs glium to use multiple instances.
    ///
    /// Instead of calling `surface.draw(&vertex_buffer, ...)` you can call
    /// `surface.draw(vertex_buffer.per_instance(), ...)`. This will draw one instance of the
    /// geometry for each element in this buffer. The attributes are still passed to the
    /// vertex shader, but each entry is passed for each different instance.
    ///
    /// Returns `None` if the backend doesn't support instancing.
    pub fn per_instance_if_supported(&self) -> Option<PerInstance> {
        // TODO: don't check this here
        if !(self.buffer.get_context().get_version() >= &Version(Api::Gl, 3, 3)) &&
            !self.buffer.get_context().get_extensions().gl_arb_instanced_arrays
        {
            return None;
        }

        Some(PerInstance(self.buffer.as_slice_any(), &self.bindings))
    }

    /// Creates a marker that instructs glium to use multiple instances.
    ///
    /// Instead of calling `surface.draw(&vertex_buffer, ...)` you can call
    /// `surface.draw(vertex_buffer.per_instance(), ...)`. This will draw one instance of the
    /// geometry for each element in this buffer. The attributes are still passed to the
    /// vertex shader, but each entry is passed for each different instance.
    ///
    /// # Features
    ///
    /// Only available if the `gl_instancing` feature is enabled.
    #[cfg(feature = "gl_instancing")]
    pub fn per_instance(&self) -> PerInstance {
        self.per_instance_if_supported().unwrap()
    }
}

impl<T> From<BufferView<T>> for VertexBuffer<T> where T: Vertex + Send + Copy + 'static {
    fn from(buffer: BufferView<T>) -> VertexBuffer<T> {
        let bindings = <T as Vertex>::build_bindings();

        VertexBuffer {
            buffer: buffer,
            bindings: bindings,
        }
    }
}

impl<T> Deref for VertexBuffer<T> where T: Send + Copy + 'static {
    type Target = BufferView<T>;

    fn deref(&self) -> &BufferView<T> {
        &self.buffer
    }
}

impl<T> DerefMut for VertexBuffer<T> where T: Send + Copy + 'static {
    fn deref_mut(&mut self) -> &mut BufferView<T> {
        &mut self.buffer
    }
}

impl<'a, T> IntoVerticesSource<'a> for &'a VertexBuffer<T> where T: Send + Copy + 'static {
    fn into_vertices_source(self) -> VerticesSource<'a> {
        VerticesSource::VertexBuffer(self.buffer.as_slice_any(), &self.bindings, false)
    }
}

impl<'a, T> Deref for VertexBufferSlice<'a, T> where T: Send + Copy + 'static {
    type Target = BufferViewSlice<'a, T>;

    fn deref(&self) -> &BufferViewSlice<'a, T> {
        &self.buffer
    }
}

impl<'a, T> DerefMut for VertexBufferSlice<'a, T> where T: Send + Copy + 'static {
    fn deref_mut(&mut self) -> &mut BufferViewSlice<'a, T> {
        &mut self.buffer
    }
}

impl<'a, T> IntoVerticesSource<'a> for VertexBufferSlice<'a, T> where T: Copy + Send + 'static {
    fn into_vertices_source(self) -> VerticesSource<'a> {
        VerticesSource::VertexBuffer(self.buffer.as_slice_any(), &self.bindings, false)
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
    buffer: BufferViewAny,
    bindings: VertexFormat,
}

impl VertexBufferAny {
    /// Returns the number of bytes between two consecutive elements in the buffer.
    pub fn get_elements_size(&self) -> usize {
        self.buffer.get_elements_size()
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
        unimplemented!();
    }

    /// Creates a marker that instructs glium to use multiple instances.
    ///
    /// Instead of calling `surface.draw(&vertex_buffer, ...)` you can call
    /// `surface.draw(vertex_buffer.per_instance(), ...)`. This will draw one instance of the
    /// geometry for each element in this buffer. The attributes are still passed to the
    /// vertex shader, but each entry is passed for each different instance.
    ///
    /// Returns `None` if the backend doesn't support instancing.
    pub fn per_instance_if_supported(&self) -> Option<PerInstance> {
        // TODO: don't check this here
        if !(self.buffer.get_context().get_version() >= &Version(Api::Gl, 3, 3)) &&
            !self.buffer.get_context().get_extensions().gl_arb_instanced_arrays
        {
            return None;
        }

        Some(PerInstance(self.buffer.as_slice_any(), &self.bindings))
    }

    /// Creates a marker that instructs glium to use multiple instances.
    ///
    /// Instead of calling `surface.draw(&vertex_buffer, ...)` you can call
    /// `surface.draw(vertex_buffer.per_instance(), ...)`. This will draw one instance of the
    /// geometry for each element in this buffer. The attributes are still passed to the
    /// vertex shader, but each entry is passed for each different instance.
    ///
    /// # Features
    ///
    /// Only available if the `gl_instancing` feature is enabled.
    #[cfg(feature = "gl_instancing")]
    pub fn per_instance(&self) -> PerInstance {
        self.per_instance_if_supported().unwrap()
    }
}

impl<T> From<VertexBuffer<T>> for VertexBufferAny where T: Send + Copy + 'static {
    fn from(buf: VertexBuffer<T>) -> VertexBufferAny {
        buf.into_vertex_buffer_any()
    }
}

impl<T> From<BufferView<T>> for VertexBufferAny where T: Vertex + Send + Copy + 'static {
    fn from(buf: BufferView<T>) -> VertexBufferAny {
        let buf: VertexBuffer<T> = buf.into();
        buf.into_vertex_buffer_any()
    }
}

impl<'a> IntoVerticesSource<'a> for &'a VertexBufferAny {
    fn into_vertices_source(self) -> VerticesSource<'a> {
        VerticesSource::VertexBuffer(self.buffer.as_slice_any(), &self.bindings, false)
    }
}
