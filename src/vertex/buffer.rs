use std::error::Error;
use std::fmt;
use std::ops::{Deref, DerefMut};
use crate::utils::range::RangeArgument;

use crate::buffer::{Buffer, BufferSlice, BufferMutSlice, BufferAny, BufferType, BufferMode, BufferCreationError, Content};
use crate::vertex::{Vertex, VerticesSource, PerInstance};
use crate::vertex::format::VertexFormat;

use crate::gl;
use crate::GlObject;

use crate::backend::Facade;
use crate::version::{Api, Version};
use crate::CapabilitiesSource;

/// Error that can happen when creating a vertex buffer.
#[derive(Copy, Clone, Debug)]
pub enum CreationError {
    /// The vertex format is not supported by the backend.
    ///
    /// Anything 64bits-related may not be supported.
    FormatNotSupported,

    /// Error while creating the vertex buffer.
    BufferCreationError(BufferCreationError),
}

impl From<BufferCreationError> for CreationError {
    #[inline]
    fn from(err: BufferCreationError) -> CreationError {
        CreationError::BufferCreationError(err)
    }
}

impl fmt::Display for CreationError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::CreationError::*;
        let desc = match self {
            FormatNotSupported => "The vertex format is not supported by the backend",
            BufferCreationError(_) => "Error while creating the vertex buffer",
        };
        fmt.write_str(desc)
    }
}

impl Error for CreationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::CreationError::*;
        match *self {
            BufferCreationError(ref error) => Some(error),
            FormatNotSupported => None,
        }
    }
}

impl<T: Copy> GlObject for VertexBuffer<T> {
    type Id = gl::types::GLuint;

    #[inline]
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
    }
}

/// A list of vertices loaded in the graphics card's memory.
#[derive(Debug)]
pub struct VertexBuffer<T> where T: Copy {
    buffer: Buffer<[T]>,
    bindings: VertexFormat,
}

/// Represents a slice of a `VertexBuffer`.
pub struct VertexBufferSlice<'b, T> where T: Copy {
    buffer: BufferSlice<'b, [T]>,
    bindings: VertexFormat,
}

impl<'b, T: 'b> VertexBufferSlice<'b, T> where T: Copy + Content {
    /// Creates a marker that instructs glium to use multiple instances.
    ///
    /// Instead of calling `surface.draw(&vertex_buffer.slice(...).unwrap(), ...)`
    /// you can call `surface.draw(vertex_buffer.slice(...).unwrap().per_instance(), ...)`.
    /// This will draw one instance of the geometry for each element in this buffer slice.
    /// The attributes are still passed to the vertex shader, but each entry is passed
    /// for each different instance.
    #[inline]
    pub fn per_instance(&'b self) -> Result<PerInstance<'b>, InstancingNotSupported> {
        // TODO: don't check this here
        if !(self.get_context().get_version() >= &Version(Api::Gl, 3, 3)) &&
            !(self.get_context().get_version() >= &Version(Api::GlEs, 3, 0)) &&
            !self.get_context().get_extensions().gl_arb_instanced_arrays
        {
            return Err(InstancingNotSupported);
        }

        Ok(PerInstance(self.buffer.as_slice_any(), self.bindings))
    }
}

impl<T> VertexBuffer<T> where T: Vertex {
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
    /// # fn main() {
    /// #[derive(Copy, Clone)]
    /// struct Vertex {
    ///     position: [f32; 3],
    ///     texcoords: [f32; 2],
    /// }
    ///
    /// implement_vertex!(Vertex, position, texcoords);
    /// # use glutin::surface::{ResizeableSurface, SurfaceTypeTrait};
    /// # fn example<T>(display: glium::Display<T>) where T: SurfaceTypeTrait + ResizeableSurface {
    /// let vertex_buffer = glium::VertexBuffer::new(&display, &[
    ///     Vertex { position: [0.0,  0.0, 0.0], texcoords: [0.0, 1.0] },
    ///     Vertex { position: [5.0, -3.0, 2.0], texcoords: [1.0, 0.0] },
    /// ]);
    /// # }
    /// # }
    /// ```
    ///
    #[inline]
    pub fn new<F: ?Sized>(facade: &F, data: &[T]) -> Result<VertexBuffer<T>, CreationError>
                  where F: Facade
    {
        VertexBuffer::new_impl(facade, data, BufferMode::Default)
    }

    /// Builds a new vertex buffer.
    ///
    /// This function will create a buffer that is intended to be modified frequently.
    #[inline]
    pub fn dynamic<F: ?Sized>(facade: &F, data: &[T]) -> Result<VertexBuffer<T>, CreationError>
                      where F: Facade
    {
        VertexBuffer::new_impl(facade, data, BufferMode::Dynamic)
    }

    /// Builds a new vertex buffer.
    #[inline]
    pub fn persistent<F: ?Sized>(facade: &F, data: &[T]) -> Result<VertexBuffer<T>, CreationError>
                         where F: Facade
    {
        VertexBuffer::new_impl(facade, data, BufferMode::Persistent)
    }

    /// Builds a new vertex buffer.
    #[inline]
    pub fn immutable<F: ?Sized>(facade: &F, data: &[T]) -> Result<VertexBuffer<T>, CreationError>
                        where F: Facade
    {
        VertexBuffer::new_impl(facade, data, BufferMode::Immutable)
    }

    #[inline]
    fn new_impl<F: ?Sized>(facade: &F, data: &[T], mode: BufferMode)
                   -> Result<VertexBuffer<T>, CreationError>
                   where F: Facade
    {
        if !T::is_supported(facade) {
            return Err(CreationError::FormatNotSupported);
        }

        let buffer = Buffer::new(facade, data, BufferType::ArrayBuffer, mode)?;
        Ok(buffer.into())
    }

    /// Builds an empty vertex buffer.
    ///
    /// The parameter indicates the number of elements.
    #[inline]
    pub fn empty<F: ?Sized>(facade: &F, elements: usize) -> Result<VertexBuffer<T>, CreationError>
                    where F: Facade
    {
        VertexBuffer::empty_impl(facade, elements, BufferMode::Default)
    }

    /// Builds an empty vertex buffer.
    ///
    /// The parameter indicates the number of elements.
    #[inline]
    pub fn empty_dynamic<F: ?Sized>(facade: &F, elements: usize) -> Result<VertexBuffer<T>, CreationError>
                            where F: Facade
    {
        VertexBuffer::empty_impl(facade, elements, BufferMode::Dynamic)
    }

    /// Builds an empty vertex buffer.
    ///
    /// The parameter indicates the number of elements.
    #[inline]
    pub fn empty_persistent<F: ?Sized>(facade: &F, elements: usize)
                               -> Result<VertexBuffer<T>, CreationError>
                               where F: Facade
    {
        VertexBuffer::empty_impl(facade, elements, BufferMode::Persistent)
    }

    /// Builds an empty vertex buffer.
    ///
    /// The parameter indicates the number of elements.
    #[inline]
    pub fn empty_immutable<F: ?Sized>(facade: &F, elements: usize) -> Result<VertexBuffer<T>, CreationError>
                              where F: Facade
    {
        VertexBuffer::empty_impl(facade, elements, BufferMode::Immutable)
    }

    #[inline]
    fn empty_impl<F: ?Sized>(facade: &F, elements: usize, mode: BufferMode)
                     -> Result<VertexBuffer<T>, CreationError>
                     where F: Facade
    {
        if !T::is_supported(facade) {
            return Err(CreationError::FormatNotSupported);
        }

        let buffer = Buffer::empty_array(facade, BufferType::ArrayBuffer, elements, mode)?;
        Ok(buffer.into())
    }
}

impl<T> VertexBuffer<T> where T: Copy {
    /// Builds a new vertex buffer from an indeterminate data type and bindings.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use glutin::surface::{ResizeableSurface, SurfaceTypeTrait};
    /// # fn example<T>(display: glium::Display<T>) where T: SurfaceTypeTrait + ResizeableSurface {
    /// use std::borrow::Cow;
    ///
    /// const BINDINGS: glium::vertex::VertexFormat = &[(
    ///         Cow::Borrowed("position"), 0, -1,
    ///         glium::vertex::AttributeType::F32F32,
    ///         false,
    ///     ), (
    ///         Cow::Borrowed("color"), 2 * ::std::mem::size_of::<f32>(), -1,
    ///         glium::vertex::AttributeType::F32,
    ///         false,
    ///     ),
    /// ];
    ///
    /// let data = vec![
    ///     1.0, -0.3, 409.0,
    ///     -0.4, 2.8, 715.0f32
    /// ];
    ///
    /// let vertex_buffer = unsafe {
    ///     glium::VertexBuffer::new_raw(&display, &data, BINDINGS, 3 * ::std::mem::size_of::<f32>())
    /// };
    /// # }
    /// ```
    ///
    #[inline]
    pub unsafe fn new_raw<F: ?Sized>(facade: &F, data: &[T],
                             bindings: VertexFormat, elements_size: usize)
                             -> Result<VertexBuffer<T>, CreationError>
                             where F: Facade
    {
        // FIXME: check that the format is supported

        Ok(VertexBuffer {
            buffer: Buffer::new(facade, data, BufferType::ArrayBuffer,
                                         BufferMode::Default)?,
            bindings,
        })
    }

    /// Dynamic version of `new_raw`.
    #[inline]
    pub unsafe fn new_raw_dynamic<F: ?Sized>(facade: &F, data: &[T],
                                     bindings: VertexFormat, elements_size: usize)
                                     -> Result<VertexBuffer<T>, CreationError>
                                     where F: Facade
    {
        // FIXME: check that the format is supported

        Ok(VertexBuffer {
            buffer: Buffer::new(facade, data, BufferType::ArrayBuffer,
                                         BufferMode::Dynamic)?,
            bindings,
        })
    }

    /// Accesses a slice of the buffer.
    ///
    /// Returns `None` if the slice is out of range.
    #[inline]
    pub fn slice<R: RangeArgument<usize>>(&self, range: R) -> Option<VertexBufferSlice<'_, T>> {
        let slice = match self.buffer.slice(range) {
            None => return None,
            Some(s) => s
        };

        Some(VertexBufferSlice {
            buffer: slice,
            bindings: self.bindings,
        })
    }

    /// Returns the associated `VertexFormat`.
    #[inline]
    pub fn get_bindings(&self) -> &VertexFormat {
        &self.bindings
    }

    /// Creates a marker that instructs glium to use multiple instances.
    ///
    /// Instead of calling `surface.draw(&vertex_buffer, ...)` you can call
    /// `surface.draw(vertex_buffer.per_instance(), ...)`. This will draw one instance of the
    /// geometry for each element in this buffer. The attributes are still passed to the
    /// vertex shader, but each entry is passed for each different instance.
    #[inline]
    pub fn per_instance(&self) -> Result<PerInstance<'_>, InstancingNotSupported> {
        // TODO: don't check this here
        if !(self.buffer.get_context().get_version() >= &Version(Api::Gl, 3, 3)) &&
            !(self.get_context().get_version() >= &Version(Api::GlEs, 3, 0)) &&
            !self.buffer.get_context().get_extensions().gl_arb_instanced_arrays
        {
            return Err(InstancingNotSupported);
        }

        Ok(PerInstance(self.buffer.as_slice_any(), self.bindings))
    }
}

impl<T> VertexBuffer<T> where T: Copy + Send + 'static {
    /// Discard the type information and turn the vertex buffer into a `VertexBufferAny`.
    #[inline]
    #[deprecated(note = "use .into() instead.")]
    pub fn into_vertex_buffer_any(self) -> VertexBufferAny {
        self.into_vertex_buffer_any_inner()
    }
    #[inline]
    fn into_vertex_buffer_any_inner(self) -> VertexBufferAny {
        VertexBufferAny {
            buffer: self.buffer.into(),
            bindings: self.bindings,
        }
    }
}

impl<T> From<Buffer<[T]>> for VertexBuffer<T> where T: Vertex + Copy {
    #[inline]
    fn from(buffer: Buffer<[T]>) -> VertexBuffer<T> {
        assert!(T::is_supported(buffer.get_context()));

        let bindings = <T as Vertex>::build_bindings();

        VertexBuffer {
            buffer,
            bindings,
        }
    }
}

impl<T> Deref for VertexBuffer<T> where T: Copy {
    type Target = Buffer<[T]>;

    #[inline]
    fn deref(&self) -> &Buffer<[T]> {
        &self.buffer
    }
}

impl<T> DerefMut for VertexBuffer<T> where T: Copy {
    #[inline]
    fn deref_mut(&mut self) -> &mut Buffer<[T]> {
        &mut self.buffer
    }
}

impl<'a, T> From<&'a VertexBuffer<T>> for BufferSlice<'a, [T]> where T: Copy {
    #[inline]
    fn from(b: &'a VertexBuffer<T>) -> BufferSlice<'a, [T]> {
        let b: &Buffer<[T]> = &*b;
        b.as_slice()
    }
}

impl<'a, T> From<&'a mut VertexBuffer<T>> for BufferMutSlice<'a, [T]> where T: Copy {
    #[inline]
    fn from(b: &'a mut VertexBuffer<T>) -> BufferMutSlice<'a, [T]> {
        let b: &mut Buffer<[T]> = &mut *b;
        b.as_mut_slice()
    }
}

impl<'a, T> From<&'a VertexBuffer<T>> for VerticesSource<'a> where T: Copy {
    #[inline]
    fn from(this: &VertexBuffer<T>) -> VerticesSource<'_> {
        VerticesSource::VertexBuffer(this.buffer.as_slice_any(), this.bindings, false)
    }
}

impl<'a, T> Deref for VertexBufferSlice<'a, T> where T: Copy {
    type Target = BufferSlice<'a, [T]>;

    #[inline]
    fn deref(&self) -> &BufferSlice<'a, [T]> {
        &self.buffer
    }
}

impl<'a, T> DerefMut for VertexBufferSlice<'a, T> where T: Copy {
    #[inline]
    fn deref_mut(&mut self) -> &mut BufferSlice<'a, [T]> {
        &mut self.buffer
    }
}

impl<'a, T> From<VertexBufferSlice<'a, T>> for BufferSlice<'a, [T]> where T: Copy {
    #[inline]
    fn from(b: VertexBufferSlice<'a, T>) -> BufferSlice<'a, [T]> {
        b.buffer
    }
}

impl<'a, T> From<VertexBufferSlice<'a, T>> for VerticesSource<'a> where T: Copy {
    #[inline]
    fn from(this: VertexBufferSlice<'a, T>) -> VerticesSource<'a> {
        VerticesSource::VertexBuffer(this.buffer.as_slice_any(), this.bindings, false)
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
    buffer: BufferAny,
    bindings: VertexFormat,
}

impl VertexBufferAny {
    /// Returns the number of bytes between two consecutive elements in the buffer.
    #[inline]
    pub fn get_elements_size(&self) -> usize {
        self.buffer.get_elements_size()
    }

    /// Returns the number of elements in the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.get_elements_count()
    }

    /// Returns the associated `VertexFormat`.
    #[inline]
    pub fn get_bindings(&self) -> &VertexFormat {
        &self.bindings
    }

    /// Turns the vertex buffer into a `VertexBuffer` without checking the type.
    #[inline]
    pub unsafe fn into_vertex_buffer<T: Copy>(self) -> VertexBuffer<T> {
        unimplemented!();
    }

    /// Creates a marker that instructs glium to use multiple instances.
    ///
    /// Instead of calling `surface.draw(&vertex_buffer, ...)` you can call
    /// `surface.draw(vertex_buffer.per_instance(), ...)`. This will draw one instance of the
    /// geometry for each element in this buffer. The attributes are still passed to the
    /// vertex shader, but each entry is passed for each different instance.
    #[inline]
    pub fn per_instance(&self) -> Result<PerInstance<'_>, InstancingNotSupported> {
        // TODO: don't check this here
        if !(self.buffer.get_context().get_version() >= &Version(Api::Gl, 3, 3)) &&
            !(self.get_context().get_version() >= &Version(Api::GlEs, 3, 0)) &&
            !self.buffer.get_context().get_extensions().gl_arb_instanced_arrays
        {
            return Err(InstancingNotSupported);
        }

        Ok(PerInstance(self.buffer.as_slice_any(), self.bindings))
    }
}

impl<T> From<VertexBuffer<T>> for VertexBufferAny where T: Copy + Send + 'static {
    #[inline]
    fn from(buf: VertexBuffer<T>) -> VertexBufferAny {
        buf.into_vertex_buffer_any_inner()
    }
}

impl<T> From<Buffer<[T]>> for VertexBufferAny where T: Vertex + Copy + Send + 'static {
    #[inline]
    fn from(buf: Buffer<[T]>) -> VertexBufferAny {
        let buf: VertexBuffer<T> = buf.into();
        buf.into_vertex_buffer_any_inner()
    }
}

impl Deref for VertexBufferAny {
    type Target = BufferAny;

    #[inline]
    fn deref(&self) -> &BufferAny {
        &self.buffer
    }
}

impl DerefMut for VertexBufferAny {
    #[inline]
    fn deref_mut(&mut self) -> &mut BufferAny {
        &mut self.buffer
    }
}

impl<'a> From<&'a VertexBufferAny> for VerticesSource<'a> {
    #[inline]
    fn from(this :&VertexBufferAny) -> VerticesSource<'_> {
        VerticesSource::VertexBuffer(this.buffer.as_slice_any(), this.bindings, false)
    }
}

/// Instancing is not supported by the backend.
#[derive(Debug, Copy, Clone)]
pub struct InstancingNotSupported;
