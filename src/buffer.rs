use Display;
use context::{self, GlVersion};
use gl;
use libc;
use std::{fmt, mem, ptr, slice};
use std::marker::{MarkerTrait, PhantomData};
use std::sync::Mutex;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::ops::{Deref, DerefMut};
use GlObject;

use sync;
use version::Api;

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
pub trait BufferType: MarkerTrait {
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
                     -> Buffer where T: BufferType, D: Send + Copy + 'static
    {
        use std::mem;

        if persistent && display.context.context.get_version() < &context::GlVersion(Api::Gl, 4, 4) &&
           !display.context.context.get_extensions().gl_arb_buffer_storage
        {
            panic!("Persistent storage is not supported by the backend");
        }

        let elements_size = get_elements_size(&data);
        let elements_count = data.len();

        let buffer_size = match elements_count * elements_size as usize {
            0 => 1,     // use size 1 instead of 0, or nvidia drivers complain
            a => a
        };

        let (tx, rx) = channel();

        display.context.context.exec(move |mut ctxt| {
            let data = data;
            let data_ptr = if elements_count * elements_size as usize == 0 {
                ptr::null()
            } else {
                data.as_ptr()
            };

            unsafe {
                let mut id: gl::types::GLuint = mem::uninitialized();

                if ctxt.version >= &GlVersion(Api::Gl, 4, 5) || ctxt.extensions.gl_arb_direct_state_access {
                    ctxt.gl.CreateBuffers(1, &mut id);
                } else if ctxt.version >= &GlVersion(Api::Gl, 1, 5) {
                    ctxt.gl.GenBuffers(1, &mut id);
                } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                    ctxt.gl.GenBuffersARB(1, &mut id);
                } else {
                    unreachable!();
                }

                let bind = <T as BufferType>::get_bind_point();

                let mut flags = gl::DYNAMIC_STORAGE_BIT | gl::MAP_READ_BIT |
                                gl::MAP_WRITE_BIT;       // TODO: more specific flags

                if persistent {
                    flags = flags | gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT;
                }

                let mut obtained_size: gl::types::GLint = mem::uninitialized();

                if ctxt.version >= &GlVersion(Api::Gl, 4, 5) || ctxt.extensions.gl_arb_direct_state_access {
                    ctxt.gl.NamedBufferStorage(id, buffer_size as gl::types::GLsizei,
                                               data_ptr as *const libc::c_void,
                                               flags);
                    ctxt.gl.GetNamedBufferParameteriv(id, gl::BUFFER_SIZE, &mut obtained_size);

                } else if ctxt.extensions.gl_arb_buffer_storage &&
                          ctxt.extensions.gl_ext_direct_state_access
                {
                    ctxt.gl.NamedBufferStorageEXT(id, buffer_size as gl::types::GLsizeiptr,
                                                  data_ptr as *const libc::c_void,
                                                  flags);
                    ctxt.gl.GetNamedBufferParameterivEXT(id, gl::BUFFER_SIZE, &mut obtained_size);


                } else if ctxt.version >= &GlVersion(Api::Gl, 4, 4) ||
                          ctxt.extensions.gl_arb_buffer_storage
                {
                    ctxt.gl.BindBuffer(bind, id);
                    *<T as BufferType>::get_storage_point(ctxt.state) = id;
                    ctxt.gl.BufferStorage(bind, buffer_size as gl::types::GLsizeiptr,
                                          data_ptr as *const libc::c_void,
                                          flags);
                    ctxt.gl.GetBufferParameteriv(bind, gl::BUFFER_SIZE, &mut obtained_size);

                } else if ctxt.version >= &GlVersion(Api::Gl, 1, 5) {
                    debug_assert!(!persistent);
                    ctxt.gl.BindBuffer(bind, id);
                    *<T as BufferType>::get_storage_point(ctxt.state) = id;
                    ctxt.gl.BufferData(bind, buffer_size as gl::types::GLsizeiptr,
                                       data_ptr as *const libc::c_void, gl::STATIC_DRAW);      // TODO: better usage
                    ctxt.gl.GetBufferParameteriv(bind, gl::BUFFER_SIZE, &mut obtained_size);

                } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                    debug_assert!(!persistent);
                    ctxt.gl.BindBufferARB(bind, id);    // bind points are the same in the ext
                    *<T as BufferType>::get_storage_point(ctxt.state) = id;
                    ctxt.gl.BufferDataARB(bind, buffer_size as gl::types::GLsizeiptr,
                                          data_ptr as *const libc::c_void, gl::STATIC_DRAW);      // TODO: better usage
                    ctxt.gl.GetBufferParameterivARB(bind, gl::BUFFER_SIZE, &mut obtained_size);

                } else {
                    unreachable!()
                }

                if buffer_size != obtained_size as usize {
                    if ctxt.version >= &GlVersion(Api::Gl, 1, 5) {
                        ctxt.gl.DeleteBuffers(1, [id].as_ptr());
                    } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                        ctxt.gl.DeleteBuffersARB(1, [id].as_ptr());
                    } else {
                        unreachable!();
                    }
                    
                    panic!("Not enough available memory for buffer (required: {} bytes, \
                            obtained: {})", buffer_size, obtained_size);
                }

                let persistent_mapping = if persistent {
                    assert!(ctxt.version >= &GlVersion(Api::Gl, 3, 0) ||
                            ctxt.extensions.gl_arb_map_buffer_range);

                    // TODO: better handling of versions
                    let ptr = if ctxt.version >= &GlVersion(Api::Gl, 4, 5) {
                        ctxt.gl.MapNamedBufferRange(id, 0, buffer_size as gl::types::GLsizei,
                                                    gl::MAP_READ_BIT | gl::MAP_WRITE_BIT |
                                                    gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT)
                    } else {
                        if *<T as BufferType>::get_storage_point(ctxt.state) != id {
                            ctxt.gl.BindBuffer(bind, id);
                            *<T as BufferType>::get_storage_point(ctxt.state) = id;
                        }

                        ctxt.gl.MapBufferRange(bind, 0, buffer_size as gl::types::GLsizeiptr,
                                               gl::MAP_READ_BIT | gl::MAP_WRITE_BIT |
                                               gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT)
                    };

                    if ptr.is_null() {
                        let error = ::get_gl_error(&mut ctxt);
                        panic!("glMapBufferRange returned null (error: {:?})", error);
                    }

                    Some(MappedBufferWrapper(ptr))

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
            persistent_mapping: persistent_mapping.map(|p| p.0),
            fences: Mutex::new(Vec::new()),
        }
    }

    pub fn new_empty<T>(display: &Display, elements_size: usize, elements_count: usize,
                        usage: gl::types::GLenum) -> Buffer where T: BufferType
    {
        let buffer_size = elements_count * elements_size as usize;

        let (tx, rx) = channel();
        display.context.context.exec(move |ctxt| {
            unsafe {
                let mut id: gl::types::GLuint = mem::uninitialized();

                if ctxt.version >= &GlVersion(Api::Gl, 1, 5) {
                    ctxt.gl.GenBuffers(1, &mut id);
                } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                    ctxt.gl.GenBuffersARB(1, &mut id);
                } else {
                    unreachable!();
                }

                let storage = <T as BufferType>::get_storage_point(ctxt.state);
                let bind = <T as BufferType>::get_bind_point();

                if ctxt.version >= &GlVersion(Api::Gl, 1, 5) {
                    ctxt.gl.BindBuffer(bind, id);
                } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                    ctxt.gl.BindBufferARB(bind, id);    // bind points are the same in the ext
                } else {
                    unreachable!();
                }

                *storage = id;

                if ctxt.version >= &GlVersion(Api::Gl, 4, 4) || ctxt.extensions.gl_arb_buffer_storage {
                    ctxt.gl.BufferStorage(bind, buffer_size as gl::types::GLsizeiptr,
                                          ptr::null(),
                                          gl::DYNAMIC_STORAGE_BIT | gl::MAP_READ_BIT |
                                          gl::MAP_WRITE_BIT);       // TODO: more specific flags

                } else if ctxt.version >= &GlVersion(Api::Gl, 1, 5) {
                    ctxt.gl.BufferData(bind, buffer_size as gl::types::GLsizeiptr,
                                       ptr::null(), usage);

                } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                    ctxt.gl.BufferDataARB(bind, buffer_size as gl::types::GLsizeiptr,
                                          ptr::null(), usage);      // TODO: better usage

                } else {
                    unreachable!()
                }

                let mut obtained_size: gl::types::GLint = mem::uninitialized();

                if ctxt.version >= &GlVersion(Api::Gl, 1, 5) {
                    ctxt.gl.GetBufferParameteriv(bind, gl::BUFFER_SIZE, &mut obtained_size);
                } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                    ctxt.gl.GetBufferParameterivARB(bind, gl::BUFFER_SIZE, &mut obtained_size);
                } else {
                    unreachable!();
                }

                if buffer_size != obtained_size as usize {
                    if ctxt.version >= &GlVersion(Api::Gl, 1, 5) {
                        ctxt.gl.DeleteBuffers(1, [id].as_ptr());
                    } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                        ctxt.gl.DeleteBuffersARB(1, [id].as_ptr());
                    } else {
                        unreachable!();
                    }
                    
                    panic!("Not enough available memory for buffer (required: {} bytes, \
                            obtained: {})", buffer_size, obtained_size);
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
    pub fn upload<T, D>(&self, offset: usize, data: Vec<D>)
                        where D: Copy + Send + 'static, T: BufferType
    {
        let offset = offset * get_elements_size(&data);
        let buffer_size = get_elements_size(&data) * data.len();

        assert!(offset <= self.get_total_size());
        assert!(offset + buffer_size <= self.get_total_size());

        let invalidate_all = (offset == 0) && (buffer_size == self.get_total_size());

        let id = self.get_id();

        self.display.context.context.exec(move |ctxt| {
            let data = data;

            unsafe {
                if invalidate_all && (ctxt.version >= &GlVersion(Api::Gl, 4, 3) ||
                    ctxt.extensions.gl_arb_invalidate_subdata)
                {
                    ctxt.gl.InvalidateBufferData(id);
                }

                if ctxt.version >= &GlVersion(Api::Gl, 4, 5) {
                    ctxt.gl.NamedBufferSubData(id, offset as gl::types::GLintptr,
                                               buffer_size as gl::types::GLsizei,
                                               data.as_ptr() as *const libc::c_void)

                } else if ctxt.extensions.gl_ext_direct_state_access {
                    ctxt.gl.NamedBufferSubDataEXT(id, offset as gl::types::GLintptr,
                                                  buffer_size as gl::types::GLsizeiptr,
                                                  data.as_ptr() as *const libc::c_void)

                } else if ctxt.version >= &GlVersion(Api::Gl, 1, 5) {
                    let storage = <T as BufferType>::get_storage_point(ctxt.state);
                    let bind = <T as BufferType>::get_bind_point();

                    ctxt.gl.BindBuffer(bind, id);
                    *storage = id;
                    ctxt.gl.BufferSubData(bind, offset as gl::types::GLintptr,
                                          buffer_size as gl::types::GLsizeiptr,
                                          data.as_ptr() as *const libc::c_void);

                } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                    let storage = <T as BufferType>::get_storage_point(ctxt.state);
                    let bind = <T as BufferType>::get_bind_point();

                    ctxt.gl.BindBufferARB(bind, id);
                    *storage = id;
                    ctxt.gl.BufferSubDataARB(bind, offset as gl::types::GLintptr,
                                             buffer_size as gl::types::GLsizeiptr,
                                             data.as_ptr() as *const libc::c_void);

                } else {
                    unreachable!()
                }

                // TODO: fence in case of persistent mapping
            }
        });
    }

    /// Offset and size should be specified as number of elements
    pub fn map<'a, T, D>(&'a mut self, offset: usize, size: usize)
                         -> Mapping<'a, T, D> where T: BufferType, D: Send + 'static
    {
        if offset > self.elements_count || (offset + size) > self.elements_count {
            panic!("Trying to map out of range of buffer");
        }

        if let Some(existing_mapping) = self.persistent_mapping.clone() {
            // we have a `&mut self`, so there's no risk of deadlock when locking `fences`
            {
                let mut fences = self.fences.lock().unwrap();
                for fence in fences.drain() {
                    fence.recv().unwrap().into_sync_fence(&self.display).wait();
                }
            }

            return Mapping {
                buffer: self,
                data: unsafe { (existing_mapping as *mut D).offset(offset as isize) },
                len: size,
                marker: PhantomData,
            };
        }

        let (tx, rx) = channel();
        let id = self.id.clone();

        let offset_bytes = offset * self.elements_size;
        let size_bytes = size * self.elements_size;

        self.display.context.context.exec(move |ctxt| {
            let ptr = unsafe {
                if ctxt.version >= &GlVersion(Api::Gl, 4, 5) {
                    ctxt.gl.MapNamedBufferRange(id, offset_bytes as gl::types::GLintptr,
                                                size_bytes as gl::types::GLsizei,
                                                gl::MAP_READ_BIT | gl::MAP_WRITE_BIT)

                } else if ctxt.version >= &GlVersion(Api::Gl, 3, 0) ||
                    ctxt.extensions.gl_arb_map_buffer_range
                {
                    let storage = <T as BufferType>::get_storage_point(ctxt.state);
                    let bind = <T as BufferType>::get_bind_point();

                    ctxt.gl.BindBuffer(bind, id);
                    *storage = id;
                    ctxt.gl.MapBufferRange(bind, offset_bytes as gl::types::GLintptr,
                                           size_bytes as gl::types::GLsizeiptr,
                                           gl::MAP_READ_BIT | gl::MAP_WRITE_BIT)

                } else {
                    unimplemented!();       // FIXME: 
                }
            };

            tx.send(MappedBufferWrapper(ptr as *mut D)).unwrap();
        });

        Mapping {
            buffer: self,
            data: rx.recv().unwrap().0,
            len: size,
            marker: PhantomData,
        }
    }

    #[cfg(feature = "gl_read_buffer")]
    pub fn read<T, D>(&self) -> Vec<D> where T: BufferType, D: Send + 'static {
        self.read_if_supported::<T, D>().unwrap()
    }

    pub fn read_if_supported<T, D>(&self) -> Option<Vec<D>> where T: BufferType, D: Send + 'static {
        self.read_slice_if_supported::<T, D>(0, self.elements_count)
    }

    #[cfg(feature = "gl_read_buffer")]
    pub fn read_slice<T, D>(&self, offset: usize, size: usize)
                            -> Vec<D> where T: BufferType, D: Send + 'static
    {
        self.read_slice_if_supported::<T, D>(offset, size).unwrap()
    }

    pub fn read_slice_if_supported<T, D>(&self, offset: usize, size: usize)
                                         -> Option<Vec<D>> where T: BufferType, D: Send + 'static
    {
        assert!(offset + size <= self.elements_count);

        let id = self.id.clone();
        let elements_size = self.elements_size.clone();
        let (tx, rx) = channel();

        self.display.context.context.exec(move |ctxt| {
            unsafe {
                let mut data = Vec::with_capacity(size);
                data.set_len(size);

                if ctxt.version >= &GlVersion(Api::Gl, 4, 5) {
                    ctxt.gl.GetNamedBufferSubData(id, (offset * elements_size) as gl::types::GLintptr,
                        (size * elements_size) as gl::types::GLsizei,
                        data.as_mut_ptr() as *mut libc::c_void);

                } else if ctxt.version >= &GlVersion(Api::Gl, 1, 5) {
                    let storage = <T as BufferType>::get_storage_point(ctxt.state);
                    let bind = <T as BufferType>::get_bind_point();

                    ctxt.gl.BindBuffer(bind, id);
                    *storage = id;
                    ctxt.gl.GetBufferSubData(bind, (offset * elements_size) as gl::types::GLintptr,
                        (size * elements_size) as gl::types::GLsizeiptr,
                        data.as_mut_ptr() as *mut libc::c_void);

                } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                    let storage = <T as BufferType>::get_storage_point(ctxt.state);
                    let bind = <T as BufferType>::get_bind_point();

                    ctxt.gl.BindBufferARB(bind, id);
                    *storage = id;
                    ctxt.gl.GetBufferSubDataARB(bind, (offset * elements_size) as gl::types::GLintptr,
                        (size * elements_size) as gl::types::GLsizeiptr,
                        data.as_mut_ptr() as *mut libc::c_void);

                } else {
                    unreachable!()
                }

                tx.send(Some(data)).ok();
            }
        });

        rx.recv().unwrap()
    }
}

impl fmt::Debug for Buffer {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "Buffer #{}", self.id)
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let id = self.id.clone();
        self.display.context.context.exec(move |ctxt| {
            if ctxt.state.array_buffer_binding == id {
                ctxt.state.array_buffer_binding = 0;
            }

            if ctxt.state.pixel_pack_buffer_binding == id {
                ctxt.state.pixel_pack_buffer_binding = 0;
            }

            if ctxt.state.pixel_unpack_buffer_binding == id {
                ctxt.state.pixel_unpack_buffer_binding = 0;
            }

            unsafe {
                if ctxt.version >= &GlVersion(Api::Gl, 1, 5) {
                    ctxt.gl.DeleteBuffers(1, [id].as_ptr());
                } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                    ctxt.gl.DeleteBuffersARB(1, [id].as_ptr());
                } else {
                    unreachable!();
                }
            }
        });
    }
}

impl GlObject for Buffer {
    type Id = gl::types::GLuint;
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

/// A mapping of a buffer.
pub struct Mapping<'b, T, D> {
    buffer: &'b mut Buffer,
    data: *mut D,
    len: usize,
    marker: PhantomData<T>,
}

struct MappedBufferWrapper<D>(*mut D);
unsafe impl<D> Send for MappedBufferWrapper<D> {}
unsafe impl<D> Sync for MappedBufferWrapper<D> {}

#[unsafe_destructor]
impl<'a, T, D> Drop for Mapping<'a, T, D> where T: BufferType {
    fn drop(&mut self) {
        // don't unmap if the buffer is persistent
        if self.buffer.is_persistent() {
            return;
        }

        let id = self.buffer.id.clone();
        self.buffer.display.context.context.exec(move |ctxt| {
            unsafe {
                if ctxt.version >= &GlVersion(Api::Gl, 4, 5) {
                    ctxt.gl.UnmapNamedBuffer(id);

                } else if ctxt.version >= &GlVersion(Api::Gl, 1, 5) {
                    let storage = <T as BufferType>::get_storage_point(ctxt.state);
                    let bind = <T as BufferType>::get_bind_point();

                    if *storage != id {
                        ctxt.gl.BindBuffer(bind, id);
                        *storage = id;
                    }

                    ctxt.gl.UnmapBuffer(bind);

                } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                    let storage = <T as BufferType>::get_storage_point(ctxt.state);
                    let bind = <T as BufferType>::get_bind_point();

                    if *storage != id {
                        ctxt.gl.BindBufferARB(bind, id);
                        *storage = id;
                    }

                    ctxt.gl.UnmapBufferARB(bind);

                } else {
                    unreachable!();
                }
            }
        });
    }
}

impl<'a, T, D> Deref for Mapping<'a, T, D> {
    type Target = [D];
    fn deref<'b>(&'b self) -> &'b [D] {
        unsafe {
            slice::from_raw_parts_mut(self.data, self.len)
        }
    }
}

impl<'a, T, D> DerefMut for Mapping<'a, T, D> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut [D] {
        unsafe {
            slice::from_raw_parts_mut(self.data, self.len)
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
