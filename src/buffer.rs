use context::{mod, GlVersion};
use gl;
use libc;
use std::c_vec::CVec;
use std::{fmt, mem, ptr};
use std::sync::Arc;
use GlObject;

/// A buffer in the graphics card's memory.
pub struct Buffer {
    display: Arc<super::DisplayImpl>,
    id: gl::types::GLuint,
    elements_size: uint,
    elements_count: uint,
}

/// Type of a buffer.
pub trait BufferType {
    /// Should return `&mut ctxt.state.something`.
    fn get_storage_point(Option<Self>, &mut context::GLState)
        -> &mut gl::types::GLuint;
    /// Should return `gl::SOMETHING_BUFFER`.
    fn get_bind_point(Option<Self>) -> gl::types::GLenum;
}

/// Used for vertex buffers.
pub struct ArrayBuffer;

impl BufferType for ArrayBuffer {
    fn get_storage_point(_: Option<ArrayBuffer>, state: &mut context::GLState)
        -> &mut gl::types::GLuint
    {
        &mut state.array_buffer_binding
    }

    fn get_bind_point(_: Option<ArrayBuffer>) -> gl::types::GLenum {
        gl::ARRAY_BUFFER
    }
}

/// Used for pixel buffers.
pub struct PixelPackBuffer;

impl BufferType for PixelPackBuffer {
    fn get_storage_point(_: Option<PixelPackBuffer>, state: &mut context::GLState)
        -> &mut gl::types::GLuint
    {
        &mut state.pixel_pack_buffer_binding
    }

    fn get_bind_point(_: Option<PixelPackBuffer>) -> gl::types::GLenum {
        gl::PIXEL_PACK_BUFFER
    }
}

/// Used for pixel buffers.
pub struct PixelUnpackBuffer;

impl BufferType for PixelUnpackBuffer {
    fn get_storage_point(_: Option<PixelUnpackBuffer>, state: &mut context::GLState)
        -> &mut gl::types::GLuint
    {
        &mut state.pixel_unpack_buffer_binding
    }

    fn get_bind_point(_: Option<PixelUnpackBuffer>) -> gl::types::GLenum {
        gl::PIXEL_UNPACK_BUFFER
    }
}

impl Buffer {
    pub fn new<T, D>(display: &super::Display, data: Vec<D>, usage: gl::types::GLenum)
        -> Buffer where T: BufferType, D: Send + Copy
    {
        use std::mem;

        let elements_size = {
            let d0: *const D = &data[0];
            let d1: *const D = &data[1];
            (d1 as uint) - (d0 as uint)
        };

        let elements_count = data.len();
        let buffer_size = elements_count * elements_size as uint;

        let (tx, rx) = channel();

        display.context.context.exec(move |: ctxt| {
            let data = data;

            unsafe {
                let mut id: gl::types::GLuint = mem::uninitialized();
                ctxt.gl.GenBuffers(1, &mut id);
                tx.send(id);

                let storage = BufferType::get_storage_point(None::<T>, ctxt.state);
                let bind = BufferType::get_bind_point(None::<T>);

                ctxt.gl.BindBuffer(bind, id);
                *storage = id;
                ctxt.gl.BufferData(bind, buffer_size as gl::types::GLsizeiptr,
                                   data.as_ptr() as *const libc::c_void, usage);

                let mut obtained_size: gl::types::GLint = mem::uninitialized();
                ctxt.gl.GetBufferParameteriv(bind, gl::BUFFER_SIZE, &mut obtained_size);
                if buffer_size != obtained_size as uint {
                    ctxt.gl.DeleteBuffers(1, [id].as_ptr());
                    panic!("Not enough available memory for buffer");
                }
            }
        });

        Buffer {
            display: display.context.clone(),
            id: rx.recv(),
            elements_size: elements_size,
            elements_count: elements_count,
        }
    }

    pub fn new_empty<T>(display: &super::Display, elements_size: uint, elements_count: uint,
                        usage: gl::types::GLenum) -> Buffer where T: BufferType
    {
        let buffer_size = elements_count * elements_size as uint;

        let (tx, rx) = channel();
        display.context.context.exec(move |: ctxt| {
            unsafe {
                let mut id: gl::types::GLuint = mem::uninitialized();
                ctxt.gl.GenBuffers(1, &mut id);

                let storage = BufferType::get_storage_point(None::<T>, ctxt.state);
                let bind = BufferType::get_bind_point(None::<T>);

                ctxt.gl.BindBuffer(bind, id);
                *storage = id;
                ctxt.gl.BufferData(bind, buffer_size as gl::types::GLsizeiptr, ptr::null(), usage);

                let mut obtained_size: gl::types::GLint = mem::uninitialized();
                ctxt.gl.GetBufferParameteriv(bind, gl::BUFFER_SIZE, &mut obtained_size);
                if buffer_size != obtained_size as uint {
                    ctxt.gl.DeleteBuffers(1, [id].as_ptr());
                    panic!("Not enough available memory for buffer");
                }

                tx.send(id);
            }
        });

        Buffer {
            display: display.context.clone(),
            id: rx.recv(),
            elements_size: elements_size,
            elements_count: elements_count,
        }
    }

    pub fn get_display(&self) -> &Arc<super::DisplayImpl> {
        &self.display
    }

    pub fn get_elements_size(&self) -> uint {
        self.elements_size
    }

    pub fn get_elements_count(&self) -> uint {
        self.elements_count
    }

    pub fn get_total_size(&self) -> uint {
        self.elements_count * self.elements_size
    }

    #[cfg(feature = "gl_extensions")]
    pub fn map<'a, T, D>(&'a mut self) -> Mapping<'a, T, D> where T: BufferType, D: Send {
        let (tx, rx) = channel();
        let id = self.id.clone();
        let elements_count = self.elements_count.clone();

        self.display.context.exec(move |: ctxt| {
            if ctxt.opengl_es {
                tx.send(Err("OpenGL ES doesn't support glMapBuffer"));
                return;
            }

            let ptr = unsafe {
                if ctxt.version >= &GlVersion(4, 5) {
                    ctxt.gl.MapNamedBuffer(id, gl::READ_WRITE)

                } else {
                    let storage = BufferType::get_storage_point(None::<T>, ctxt.state);
                    let bind = BufferType::get_bind_point(None::<T>);

                    ctxt.gl.BindBuffer(bind, id);
                    *storage = id;
                    ctxt.gl.MapBuffer(bind, gl::READ_WRITE)
                }
            };

            if ptr.is_null() {
                tx.send(Err("glMapBuffer returned null"));
            } else {
                tx.send(Ok(ptr as *mut D));
            }
        });

        Mapping {
            buffer: self,
            data: unsafe { CVec::new(rx.recv().unwrap(), elements_count) },
        }
    }

    #[cfg(feature = "gl_extensions")]
    pub fn read<T, D>(&self) -> Vec<D> where T: BufferType, D: Send {
        self.read_slice::<T, D>(0, self.elements_count)
    }

    #[cfg(feature = "gl_extensions")]
    pub fn read_slice<T, D>(&self, offset: uint, size: uint)
                            -> Vec<D> where T: BufferType, D: Send
    {
        assert!(offset + size <= self.elements_count);

        let id = self.id.clone();
        let elements_size = self.elements_size.clone();
        let (tx, rx) = channel();

        self.display.context.exec(move |: ctxt| {
            if ctxt.opengl_es {
                panic!("OpenGL ES doesn't support glGetBufferSubData");
            }

            unsafe {
                let mut data = Vec::with_capacity(size);
                data.set_len(size);

                if !ctxt.opengl_es && ctxt.version >= &GlVersion(4, 5) {
                    ctxt.gl.GetNamedBufferSubData(id, (offset * elements_size) as gl::types::GLintptr,
                        (size * elements_size) as gl::types::GLsizei,
                        data.as_mut_ptr() as *mut libc::c_void);

                } else {
                    let storage = BufferType::get_storage_point(None::<T>, ctxt.state);
                    let bind = BufferType::get_bind_point(None::<T>);

                    ctxt.gl.BindBuffer(bind, id);
                    *storage = id;
                    ctxt.gl.GetBufferSubData(bind, (offset * elements_size) as gl::types::GLintptr,
                        (size * elements_size) as gl::types::GLsizeiptr,
                        data.as_mut_ptr() as *mut libc::c_void);
                }

                tx.send_opt(data).ok();
            }
        });

        rx.recv()
    }
}

impl fmt::Show for Buffer {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        (format!("Buffer #{}", self.id)).fmt(formatter)
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let id = self.id.clone();
        self.display.context.exec(move |: ctxt| {
            if ctxt.state.array_buffer_binding == id {
                ctxt.state.array_buffer_binding = 0;
            }

            if ctxt.state.pixel_pack_buffer_binding == id {
                ctxt.state.pixel_pack_buffer_binding = 0;
            }

            if ctxt.state.pixel_unpack_buffer_binding == id {
                ctxt.state.pixel_unpack_buffer_binding = 0;
            }

            unsafe { ctxt.gl.DeleteBuffers(1, [ id ].as_ptr()); }
        });
    }
}

impl GlObject for Buffer {
    fn get_id(&self) -> gl::types::GLuint {
        self.id
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
#[cfg(feature = "gl_extensions")]
pub struct Mapping<'b, T, D> {
    buffer: &'b mut Buffer,
    data: CVec<D>,
}

#[cfg(feature = "gl_extensions")]
#[unsafe_destructor]
impl<'a, T, D> Drop for Mapping<'a, T, D> where T: BufferType {
    fn drop(&mut self) {
        let id = self.buffer.id.clone();
        self.buffer.display.context.exec(move |: ctxt| {
            unsafe {
                if ctxt.version >= &GlVersion(4, 5) {
                    ctxt.gl.UnmapNamedBuffer(id);

                } else {
                    let storage = BufferType::get_storage_point(None::<T>, ctxt.state);
                    let bind = BufferType::get_bind_point(None::<T>);

                    if *storage != id {
                        ctxt.gl.BindBuffer(bind, id);
                        *storage = id;
                    }

                    ctxt.gl.UnmapBuffer(bind);
                }
            }
        });
    }
}

#[cfg(feature = "gl_extensions")]
impl<'a, T, D> Deref<[D]> for Mapping<'a, T, D> {
    fn deref<'b>(&'b self) -> &'b [D] {
        self.data.as_slice()
    }
}

#[cfg(feature = "gl_extensions")]
impl<'a, T, D> DerefMut<[D]> for Mapping<'a, T, D> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut [D] {
        self.data.as_mut_slice()
    }
}
