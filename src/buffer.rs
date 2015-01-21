use Display;
use context::{self, GlVersion};
use gl;
use libc;
use std::{fmt, mem, ptr, slice};
use std::sync::Mutex;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::ops::{Deref, DerefMut};
use GlObject;

use sync;

/// A buffer in the graphics card's memory.
pub struct Buffer {
    display: Display,
    id: gl::types::GLuint,
    elements_size: usize,
    elements_count: usize,
    persistent_mapping: Option<*mut libc::c_void>,

    /// Fences that the buffer must wait on before locking the permanent mapping.
    fences: Mutex<Vec<Receiver<sync::LinearSyncFence>>>,
}

// we need to do this because `*mut libc::c_void*` is not Send
unsafe impl Send for Buffer {}

/// Type of a buffer.
pub trait BufferType {
    /// Should return `&mut ctxt.state.something`.
    fn get_storage_point(&mut context::GLState) -> &mut gl::types::GLuint;
    /// Should return `gl::SOMETHING_BUFFER`.
    fn get_bind_point() -> gl::types::GLenum;
}

/// Used for vertex buffers.
pub struct ArrayBuffer;

impl BufferType for ArrayBuffer {
    fn get_storage_point(state: &mut context::GLState) -> &mut gl::types::GLuint {
        &mut state.array_buffer_binding
    }

    fn get_bind_point() -> gl::types::GLenum {
        gl::ARRAY_BUFFER
    }
}

/// Used for pixel buffers.
pub struct PixelPackBuffer;

impl BufferType for PixelPackBuffer {
    fn get_storage_point(state: &mut context::GLState) -> &mut gl::types::GLuint {
        &mut state.pixel_pack_buffer_binding
    }

    fn get_bind_point() -> gl::types::GLenum {
        gl::PIXEL_PACK_BUFFER
    }
}

/// Used for pixel buffers.
pub struct PixelUnpackBuffer;

impl BufferType for PixelUnpackBuffer {
    fn get_storage_point(state: &mut context::GLState) -> &mut gl::types::GLuint {
        &mut state.pixel_unpack_buffer_binding
    }

    fn get_bind_point() -> gl::types::GLenum {
        gl::PIXEL_UNPACK_BUFFER
    }
}

/// Used for uniform buffers.
pub struct UniformBuffer;

impl BufferType for UniformBuffer {
    fn get_storage_point(state: &mut context::GLState) -> &mut gl::types::GLuint {
        &mut state.uniform_buffer_binding
    }

    fn get_bind_point() -> gl::types::GLenum {
        gl::UNIFORM_BUFFER
    }
}

impl Buffer {
    pub fn new<T, D>(display: &Display, data: Vec<D>, persistent: bool)
                     -> Buffer where T: BufferType, D: Send + Copy
    {
        use std::mem;

        if persistent && display.context.context.get_version() < &context::GlVersion(4, 4) &&
           !display.context.context.get_extensions().gl_arb_buffer_storage
        {
            panic!("Persistent storage is not supported by the backend");
        }

        let elements_size = get_elements_size(&data);
        let elements_count = data.len();
        let buffer_size = elements_count * elements_size as usize;

        let (tx, rx) = channel();

        display.context.context.exec(move |: mut ctxt| {
            let data = data;

            unsafe {
                let mut id: gl::types::GLuint = mem::uninitialized();
                ctxt.gl.GenBuffers(1, &mut id);;

                let bind = <T as BufferType>::get_bind_point();

                ctxt.gl.BindBuffer(bind, id);
                *<T as BufferType>::get_storage_point(ctxt.state) = id;

                if ctxt.version >= &GlVersion(4, 4) || ctxt.extensions.gl_arb_buffer_storage {
                    let mut flags = gl::DYNAMIC_STORAGE_BIT | gl::MAP_READ_BIT |
                                    gl::MAP_WRITE_BIT;       // TODO: more specific flags

                    if persistent {
                        flags = flags | gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT;
                    }

                    ctxt.gl.BufferStorage(bind, buffer_size as gl::types::GLsizeiptr,
                                          data.as_ptr() as *const libc::c_void,
                                          flags);

                } else {
                    debug_assert!(!persistent);
                    ctxt.gl.BufferData(bind, buffer_size as gl::types::GLsizeiptr,
                                       data.as_ptr() as *const libc::c_void, gl::STATIC_DRAW);      // TODO: better usage
                }

                let mut obtained_size: gl::types::GLint = mem::uninitialized();
                ctxt.gl.GetBufferParameteriv(bind, gl::BUFFER_SIZE, &mut obtained_size);
                if buffer_size != obtained_size as usize {
                    ctxt.gl.DeleteBuffers(1, [id].as_ptr());
                    panic!("Not enough available memory for buffer (required: {} bytes, \
                            obtained: {})", buffer_size, obtained_size);
                }

                let persistent_mapping = if persistent {
                    let ptr = ctxt.gl.MapBufferRange(bind, 0,
                                                     buffer_size as gl::types::GLsizeiptr,
                                                     gl::MAP_READ_BIT | gl::MAP_WRITE_BIT |
                                                     gl::MAP_PERSISTENT_BIT |
                                                     gl::MAP_COHERENT_BIT);
                    if ptr.is_null() {
                        let error = ::get_gl_error(&mut ctxt);
                        panic!("glMapBufferRange returned null (error: {:?})", error);
                    }

                    Some(ptr::Unique(ptr))

                } else {
                    None
                };

                tx.send((id, persistent_mapping)).unwrap();
            }
        });

        let (id, persistent_mapping) = rx.recv().unwrap();

        Buffer {
            display: display.clone(),
            id: id,
            elements_size: elements_size,
            elements_count: elements_count,
            persistent_mapping: persistent_mapping.map(|ptr::Unique(p)| p),
            fences: Mutex::new(Vec::new()),
        }
    }

    pub fn new_empty<T>(display: &Display, elements_size: usize, elements_count: usize,
                        usage: gl::types::GLenum) -> Buffer where T: BufferType
    {
        let buffer_size = elements_count * elements_size as usize;

        let (tx, rx) = channel();
        display.context.context.exec(move |: ctxt| {
            unsafe {
                let mut id: gl::types::GLuint = mem::uninitialized();
                ctxt.gl.GenBuffers(1, &mut id);

                let storage = <T as BufferType>::get_storage_point(ctxt.state);
                let bind = <T as BufferType>::get_bind_point();

                ctxt.gl.BindBuffer(bind, id);
                *storage = id;

                if ctxt.version >= &GlVersion(4, 4) || ctxt.extensions.gl_arb_buffer_storage {
                    ctxt.gl.BufferStorage(bind, buffer_size as gl::types::GLsizeiptr,
                                          ptr::null(),
                                          gl::DYNAMIC_STORAGE_BIT | gl::MAP_READ_BIT |
                                          gl::MAP_WRITE_BIT);       // TODO: more specific flags
                } else {
                    ctxt.gl.BufferData(bind, buffer_size as gl::types::GLsizeiptr,
                                       ptr::null(), usage);
                }

                let mut obtained_size: gl::types::GLint = mem::uninitialized();
                ctxt.gl.GetBufferParameteriv(bind, gl::BUFFER_SIZE, &mut obtained_size);
                if buffer_size != obtained_size as usize {
                    ctxt.gl.DeleteBuffers(1, [id].as_ptr());
                    panic!("Not enough available memory for buffer");
                }

                tx.send(id).unwrap();
            }
        });

        Buffer {
            display: display.clone(),
            id: rx.recv().unwrap(),
            elements_size: elements_size,
            elements_count: elements_count,
            persistent_mapping: None,
            fences: Mutex::new(Vec::new()),
        }
    }

    pub fn get_display(&self) -> &Display {
        &self.display
    }

    pub fn get_elements_size(&self) -> usize {
        self.elements_size
    }

    pub fn get_elements_count(&self) -> usize {
        self.elements_count
    }

    pub fn get_total_size(&self) -> usize {
        self.elements_count * self.elements_size
    }

    pub fn is_persistent(&self) -> bool {
        self.persistent_mapping.is_some()
    }

    /// Adds a new fence that must be waited on before being able to access the
    /// persistent mapping.
    ///
    /// Panics if not a persistent buffer.
    pub fn add_fence(&self) -> Sender<sync::LinearSyncFence> {
        assert!(self.is_persistent());
        let (tx, rx) = channel();
        self.fences.lock().unwrap().push(rx);
        tx
    }

    /// Uploads data in the buffer.
    ///
    /// This function considers that the buffer is filled of elements of type `D`. The offset
    /// is a number of elements, not a number of bytes.
    pub fn upload<T, D>(&mut self, offset: usize, data: Vec<D>)
                        where D: Copy + Send, T: BufferType
    {
        let offset = offset * get_elements_size(&data);
        let buffer_size = get_elements_size(&data) * data.len();

        assert!(offset <= self.get_total_size());
        assert!(offset + buffer_size <= self.get_total_size());

        let id = self.get_id();

        self.display.context.context.exec(move |: ctxt| {
            let data = data;

            unsafe {
                if ctxt.version >= &GlVersion(4, 5) {
                    ctxt.gl.NamedBufferSubData(id, offset as gl::types::GLintptr,
                                               buffer_size as gl::types::GLsizei,
                                               data.as_ptr() as *const libc::c_void)

                } else if ctxt.extensions.gl_ext_direct_state_access {
                    ctxt.gl.NamedBufferSubDataEXT(id, offset as gl::types::GLintptr,
                                                  buffer_size as gl::types::GLsizeiptr,
                                                  data.as_ptr() as *const libc::c_void)

                } else {
                    let storage = <T as BufferType>::get_storage_point(ctxt.state);
                    let bind = <T as BufferType>::get_bind_point();

                    ctxt.gl.BindBuffer(bind, id);
                    *storage = id;
                    ctxt.gl.BufferSubData(bind, offset as gl::types::GLintptr,
                                          buffer_size as gl::types::GLsizeiptr,
                                          data.as_ptr() as *const libc::c_void)
                }

                // TODO: fence in case of persistent mapping
            }
        });
    }

    /// Offset and size should be specified as number of elements
    pub fn map<'a, T, D>(&'a mut self, offset: usize, size: usize)
                         -> Mapping<'a, T, D> where T: BufferType, D: Send
    {
        if offset > self.elements_count || (offset + size) > self.elements_count {
            panic!("Trying to map out of range of buffer");
        }

        if let Some(existing_mapping) = self.persistent_mapping.clone() {
            // we have a `&mut self`, so there's no risk of deadlock when locking `fences`
            for fence in self.fences.lock().unwrap().drain() {
                fence.recv().unwrap().into_sync_fence(&self.display).wait();
            }

            return Mapping {
                buffer: self,
                data: unsafe { (existing_mapping as *mut D).offset(offset as isize) },
                len: size,
            };
        }

        let (tx, rx) = channel();
        let id = self.id.clone();

        let offset_bytes = offset * self.elements_size;
        let size_bytes = size * self.elements_size;

        self.display.context.context.exec(move |: ctxt| {
            let ptr = unsafe {
                if ctxt.version >= &GlVersion(4, 5) {
                    ctxt.gl.MapNamedBufferRange(id, offset_bytes as gl::types::GLintptr,
                                                size_bytes as gl::types::GLsizei,
                                                gl::MAP_READ_BIT | gl::MAP_WRITE_BIT)

                } else {
                    let storage = <T as BufferType>::get_storage_point(ctxt.state);
                    let bind = <T as BufferType>::get_bind_point();

                    ctxt.gl.BindBuffer(bind, id);
                    *storage = id;
                    ctxt.gl.MapBufferRange(bind, offset_bytes as gl::types::GLintptr,
                                           size_bytes as gl::types::GLsizeiptr,
                                           gl::MAP_READ_BIT | gl::MAP_WRITE_BIT)
                }
            };

            tx.send(ptr::Unique(ptr as *mut D)).unwrap();
        });

        Mapping {
            buffer: self,
            data: rx.recv().unwrap().0,
            len: size,
        }
    }

    #[cfg(feature = "gl_read_buffer")]
    pub fn read<T, D>(&self) -> Vec<D> where T: BufferType, D: Send {
        self.read_if_supported::<T, D>().unwrap()
    }

    pub fn read_if_supported<T, D>(&self) -> Option<Vec<D>> where T: BufferType, D: Send {
        self.read_slice_if_supported::<T, D>(0, self.elements_count)
    }

    #[cfg(feature = "gl_read_buffer")]
    pub fn read_slice<T, D>(&self, offset: usize, size: usize)
                            -> Vec<D> where T: BufferType, D: Send
    {
        self.read_slice_if_supported::<T, D>(offset, size).unwrap()
    }

    pub fn read_slice_if_supported<T, D>(&self, offset: usize, size: usize)
                                         -> Option<Vec<D>> where T: BufferType, D: Send
    {
        assert!(offset + size <= self.elements_count);

        let id = self.id.clone();
        let elements_size = self.elements_size.clone();
        let (tx, rx) = channel();

        self.display.context.context.exec(move |: ctxt| {
            if ctxt.opengl_es {
                tx.send(None).ok();
                return;
            }

            unsafe {
                let mut data = Vec::with_capacity(size);
                data.set_len(size);

                if !ctxt.opengl_es && ctxt.version >= &GlVersion(4, 5) {
                    ctxt.gl.GetNamedBufferSubData(id, (offset * elements_size) as gl::types::GLintptr,
                        (size * elements_size) as gl::types::GLsizei,
                        data.as_mut_ptr() as *mut libc::c_void);

                } else {
                    let storage = <T as BufferType>::get_storage_point(ctxt.state);
                    let bind = <T as BufferType>::get_bind_point();

                    ctxt.gl.BindBuffer(bind, id);
                    *storage = id;
                    ctxt.gl.GetBufferSubData(bind, (offset * elements_size) as gl::types::GLintptr,
                        (size * elements_size) as gl::types::GLsizeiptr,
                        data.as_mut_ptr() as *mut libc::c_void);
                }

                tx.send(Some(data)).ok();
            }
        });

        rx.recv().unwrap()
    }
}

impl fmt::Show for Buffer {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "Buffer #{}", self.id)
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let id = self.id.clone();
        self.display.context.context.exec(move |: ctxt| {
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

/// A mapping of a buffer.
pub struct Mapping<'b, T, D> {
    buffer: &'b mut Buffer,
    data: *mut D,
    len: usize,
}

#[unsafe_destructor]
impl<'a, T, D> Drop for Mapping<'a, T, D> where T: BufferType {
    fn drop(&mut self) {
        // don't unmap if the buffer is persistent
        if self.buffer.is_persistent() {
            return;
        }

        let id = self.buffer.id.clone();
        self.buffer.display.context.context.exec(move |: ctxt| {
            unsafe {
                if ctxt.version >= &GlVersion(4, 5) {
                    ctxt.gl.UnmapNamedBuffer(id);

                } else {
                    let storage = <T as BufferType>::get_storage_point(ctxt.state);
                    let bind = <T as BufferType>::get_bind_point();

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

impl<'a, T, D> Deref for Mapping<'a, T, D> {
    type Target = [D];
    fn deref<'b>(&'b self) -> &'b [D] {
        unsafe {
            slice::from_raw_mut_buf(&self.data, self.len)
        }
    }
}

impl<'a, T, D> DerefMut for Mapping<'a, T, D> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut [D] {
        unsafe {
            slice::from_raw_mut_buf(&self.data, self.len)
        }
    }
}

/// Returns the size of each element inside the vec.
fn get_elements_size<T>(data: &Vec<T>) -> usize {
    if data.len() <= 1 {
        mem::size_of::<T>()
    } else {
        let d0: *const T = &data[0];
        let d1: *const T = &data[1];
        (d1 as usize) - (d0 as usize)
    }
}
