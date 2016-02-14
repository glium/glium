use std::error::Error;
use std::fmt;
use std::ops::{Deref, DerefMut};
use utils::range::RangeArgument;

use buffer::{Buffer, BufferSlice, BufferMutSlice, BufferAny, BufferType, BufferMode, BufferCreationError, Content};
use buffer::{ImmutableBuffer, ImmutableBufferSlice, ImmutableBufferMutSlice};
use buffer::{PersistentBuffer, PersistentBufferSlice, PersistentBufferMutSlice};
use buffer::Storage as BufferStorage;
use buffer::Create as BufferCreate;
use vertex::{Vertex, IntoVerticesSource, PerInstance};
use vertex::format::VertexFormat;
use ops::VerticesSource;

use backend::Facade;
use version::{Api, Version};
use CapabilitiesSource;

pub type VertexBuffer<T> = VertexStorage<Buffer<[T]>>;
pub type VertexBufferSlice<'a, T> = VertexStorage<BufferSlice<'a, [T]>>;
pub type VertexBufferMutSlice<'a, T> = VertexStorage<BufferMutSlice<'a, [T]>>;
pub type ImmutableVertexBuffer<T> = VertexStorage<ImmutableBuffer<[T]>>;
pub type ImmutableVertexBufferSlice<'a, T> = VertexStorage<ImmutableBufferSlice<'a, [T]>>;
pub type ImmutableVertexBufferMutSlice<'a, T> = VertexStorage<ImmutableBufferMutSlice<'a, [T]>>;
pub type PersistentVertexBuffer<T> = VertexStorage<PersistentBuffer<[T]>>;
pub type PersistentVertexBufferSlice<'a, T> = VertexStorage<PersistentBufferSlice<'a, [T]>>;
pub type PersistentVertexBufferMutSlice<'a, T> = VertexStorage<PersistentBufferMutSlice<'a, [T]>>;

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
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

impl Error for CreationError {
    fn description(&self) -> &str {
        use self::CreationError::*;
        match *self {
            FormatNotSupported => "The vertex format is not supported by the backend",
            BufferCreationError(_) => "Error while creating the vertex buffer",
        }
    }

    fn cause(&self) -> Option<&Error> {
        use self::CreationError::*;
        match *self {
            BufferCreationError(ref error) => Some(error),
            FormatNotSupported => None,
        }
    }
}

/// A list of vertices loaded in the graphics card's memory.
#[derive(Debug)]
pub struct VertexStorage<T> where T: BufferStorage {
    buffer: T,
    bindings: VertexFormat,
}

impl<T, V> VertexStorage<T> where T: BufferCreate<Content = [V]>, V: Vertex {
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
    /// let vertex_buffer = glium::VertexStorage::new(&display, &[
    ///     Vertex { position: [0.0,  0.0, 0.0], texcoords: [0.0, 1.0] },
    ///     Vertex { position: [5.0, -3.0, 2.0], texcoords: [1.0, 0.0] },
    /// ]);
    /// # }
    /// ```
    ///
    #[inline]
    pub fn new<F>(facade: &F, data: &[V]) -> Result<VertexStorage<T>, CreationError>
                  where F: Facade
    {
        if !V::is_supported(facade) {
            return Err(CreationError::FormatNotSupported);
        }

        let buffer: T = try!(BufferCreate::new(facade, data, BufferType::ArrayBuffer));
        let bindings = <V as Vertex>::build_bindings();

        Ok(VertexStorage {
            buffer: buffer,
            bindings: bindings,
        })
    }

    /// Builds an empty vertex buffer.
    ///
    /// The parameter indicates the number of elements.
    #[inline]
    pub fn empty<F>(facade: &F, elements: usize) -> Result<VertexStorage<T>, CreationError>
                    where F: Facade
    {
        if !V::is_supported(facade) {
            return Err(CreationError::FormatNotSupported);
        }

        let buffer: T = try!(BufferCreate::empty_array(facade, elements, BufferType::ArrayBuffer));
        let bindings = <V as Vertex>::build_bindings();

        Ok(VertexStorage {
            buffer: buffer,
            bindings: bindings,
        })
    }
}

impl_buffer_wrapper!(VertexStorage, buffer, [bindings]);
/*
impl<T> VertexStorage<T> where T: Copy {
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
    ///     glium::VertexStorage::new_raw(&display, &data, bindings, 3 * ::std::mem::size_of::<f32>())
    /// };
    /// # }
    /// ```
    ///
    #[inline]
    pub unsafe fn new_raw<F>(facade: &F, data: &[T],
                             bindings: VertexFormat, elements_size: usize)
                             -> Result<VertexStorage<T>, CreationError>
                             where F: Facade
    {
        // FIXME: check that the format is supported

        Ok(VertexStorage {
            buffer: try!(Buffer::new(facade, data, BufferType::ArrayBuffer,
                                         BufferMode::Default)),
            bindings: bindings,
        })
    }

    /// Dynamic version of `new_raw`.
    #[inline]
    pub unsafe fn new_raw_dynamic<F>(facade: &F, data: &[T],
                                     bindings: VertexFormat, elements_size: usize)
                                     -> Result<VertexStorage<T>, CreationError>
                                     where F: Facade
    {
        // FIXME: check that the format is supported

        Ok(VertexStorage {
            buffer: try!(Buffer::new(facade, data, BufferType::ArrayBuffer,
                                         BufferMode::Dynamic)),
            bindings: bindings,
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
    pub fn per_instance(&self) -> Result<PerInstance, InstancingNotSupported> {
        // TODO: don't check this here
        if !(self.buffer.get_context().get_version() >= &Version(Api::Gl, 3, 3)) &&
            !self.buffer.get_context().get_extensions().gl_arb_instanced_arrays
        {
            return Err(InstancingNotSupported);
        }

        Ok(PerInstance(self.buffer.as_slice_any(), &self.bindings))
    }
}
*/
impl<T, V> From<T> for VertexStorage<T> where T: BufferStorage<Content = [V]>, V: Vertex + Copy {
    #[inline]
    fn from(buffer: T) -> VertexStorage<T> {
        // FIXME: important, restore
        //assert!(V::is_supported(buffer.get_context()));
        let bindings = <V as Vertex>::build_bindings();

        VertexStorage {
            buffer: buffer,
            bindings: bindings,
        }
    }
}

impl<'a, T> IntoVerticesSource<'a> for &'a VertexStorage<T> where T: BufferStorage {
    #[inline]
    fn into_vertices_source(self) -> VerticesSource<'a> {
        unsafe { VerticesSource::from_buffer(self.buffer.as_slice_any(), &self.bindings, false) }
    }
}

/// Instancing is not supported by the backend.
#[derive(Debug, Copy, Clone)]
pub struct InstancingNotSupported;
