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
    /// Should return `&mut state.something`.
    fn get_storage_point(Option<Self>, &mut context::GLState)
        -> &mut Option<gl::types::GLuint>;
    /// Should return `gl::SOMETHING_BUFFER`.
    fn get_bind_point(Option<Self>) -> gl::types::GLenum;
}

/// Used for vertex buffers.
pub struct ArrayBuffer;

impl BufferType for ArrayBuffer {
    fn get_storage_point(_: Option<ArrayBuffer>, state: &mut context::GLState)
        -> &mut Option<gl::types::GLuint>
    {
        &mut state.array_buffer_binding
    }

    fn get_bind_point(_: Option<ArrayBuffer>) -> gl::types::GLenum {
        gl::ARRAY_BUFFER
    }
}

/// Used for index buffers.
pub struct ElementArrayBuffer;

impl BufferType for ElementArrayBuffer {
    fn get_storage_point(_: Option<ElementArrayBuffer>, state: &mut context::GLState)
        -> &mut Option<gl::types::GLuint>
    {
        &mut state.element_array_buffer_binding
    }

    fn get_bind_point(_: Option<ElementArrayBuffer>) -> gl::types::GLenum {
        gl::ELEMENT_ARRAY_BUFFER
    }
}

/// Used for pixel buffers.
pub struct PixelPackBuffer;

impl BufferType for PixelPackBuffer {
    fn get_storage_point(_: Option<PixelPackBuffer>, state: &mut context::GLState)
        -> &mut Option<gl::types::GLuint>
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
        -> &mut Option<gl::types::GLuint>
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

        display.context.context.exec(proc(gl, state, version, extensions) {
            let data = data;

            unsafe {
                let mut id: gl::types::GLuint = mem::uninitialized();
                gl.GenBuffers(1, &mut id);
                tx.send(id);

                if version >= &GlVersion(4, 5) {
                    gl.NamedBufferData(id, buffer_size as gl::types::GLsizei,
                        data.as_ptr() as *const libc::c_void, usage);
                        
                } else if extensions.gl_ext_direct_state_access {
                    gl.NamedBufferDataEXT(id, buffer_size as gl::types::GLsizeiptr,
                        data.as_ptr() as *const libc::c_void, usage);

                } else {
                    let storage = BufferType::get_storage_point(None::<T>, state);
                    let bind = BufferType::get_bind_point(None::<T>);

                    gl.BindBuffer(bind, id);
                    *storage = Some(id);
                    gl.BufferData(bind, buffer_size as gl::types::GLsizeiptr,
                        data.as_ptr() as *const libc::c_void, usage);
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
        display.context.context.exec(proc(gl, state, version, extensions) {
            unsafe {
                let mut id: gl::types::GLuint = mem::uninitialized();
                gl.GenBuffers(1, &mut id);

                if version >= &GlVersion(4, 5) {
                    gl.NamedBufferData(id, buffer_size as gl::types::GLsizei, ptr::null(), usage);
                        
                } else if extensions.gl_ext_direct_state_access {
                    gl.NamedBufferDataEXT(id, buffer_size as gl::types::GLsizeiptr, ptr::null(),
                        usage);

                } else {
                    let storage = BufferType::get_storage_point(None::<T>, state);
                    let bind = BufferType::get_bind_point(None::<T>);

                    gl.BindBuffer(bind, id);
                    *storage = Some(id);
                    gl.BufferData(bind, buffer_size as gl::types::GLsizeiptr, ptr::null(), usage);
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

    pub fn map<'a, T, D>(&'a mut self) -> Mapping<'a, T, D> where T: BufferType, D: Send {
        let (tx, rx) = channel();
        let id = self.id.clone();
        let elements_count = self.elements_count.clone();

        self.display.context.exec(proc(gl, state, version, _) {
            let ptr = unsafe {
                if version >= &GlVersion(4, 5) {
                    gl.MapNamedBuffer(id, gl::READ_WRITE)

                } else {
                    let storage = BufferType::get_storage_point(None::<T>, state);
                    let bind = BufferType::get_bind_point(None::<T>);

                    gl.BindBuffer(bind, id);
                    *storage = Some(id);
                    gl.MapBuffer(bind, gl::READ_WRITE)
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

    pub fn read<T, D>(&self) -> Vec<D> where T: BufferType, D: Send {
        self.read_slice::<T, D>(0, self.elements_count)
    }

    pub fn read_slice<T, D>(&self, offset: uint, size: uint)
                            -> Vec<D> where T: BufferType, D: Send
    {
        assert!(offset + size <= self.elements_count);

        let id = self.id.clone();
        let elements_size = self.elements_size.clone();
        let (tx, rx) = channel();

        self.display.context.exec(proc(gl, state, version, _) {
            unsafe {
                let mut data = Vec::with_capacity(size);
                data.set_len(size);

                if version >= &GlVersion(4, 5) {
                    gl.GetNamedBufferSubData(id, (offset * elements_size) as gl::types::GLintptr,
                        (size * elements_size) as gl::types::GLsizei,
                        data.as_mut_ptr() as *mut libc::c_void);

                } else {
                    let storage = BufferType::get_storage_point(None::<T>, state);
                    let bind = BufferType::get_bind_point(None::<T>);

                    gl.BindBuffer(bind, id);
                    *storage = Some(id);
                    gl.GetBufferSubData(bind, (offset * elements_size) as gl::types::GLintptr,
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
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::FormatError> {
        (format!("Buffer #{}", self.id)).fmt(formatter)
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let id = self.id.clone();
        self.display.context.exec(proc(gl, state, _, _) {
            if state.array_buffer_binding == Some(id) {
                state.array_buffer_binding = None;
            }

            if state.element_array_buffer_binding == Some(id) {
                state.element_array_buffer_binding = None;
            }

            if state.pixel_pack_buffer_binding == Some(id) {
                state.pixel_pack_buffer_binding = None;
            }

            if state.pixel_unpack_buffer_binding == Some(id) {
                state.pixel_unpack_buffer_binding = None;
            }

            unsafe { gl.DeleteBuffers(1, [ id ].as_ptr()); }
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
pub struct Mapping<'b, T, D> {
    buffer: &'b mut Buffer,
    data: CVec<D>,
}

#[unsafe_destructor]
impl<'a, T, D> Drop for Mapping<'a, T, D> where T: BufferType {
    fn drop(&mut self) {
        let id = self.buffer.id.clone();
        self.buffer.display.context.exec(proc(gl, state, version, _) {
            unsafe {
                if version >= &GlVersion(4, 5) {
                    gl.UnmapNamedBuffer(id);

                } else {
                    let storage = BufferType::get_storage_point(None::<T>, state);
                    let bind = BufferType::get_bind_point(None::<T>);

                    if *storage != Some(id) {
                        gl.BindBuffer(bind, id);
                        *storage = Some(id);
                    }

                    gl.UnmapBuffer(bind);
                }
            }
        });
    }
}

impl<'a, T, D> Deref<[D]> for Mapping<'a, T, D> {
    fn deref<'b>(&'b self) -> &'b [D] {
        self.data.as_slice()
    }
}

impl<'a, T, D> DerefMut<[D]> for Mapping<'a, T, D> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut [D] {
        self.data.as_mut_slice()
    }
}
