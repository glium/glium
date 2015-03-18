use Display;
use context::{self, GlVersion};
use context::CommandContext;
use gl;
use libc;
use std::{fmt, mem, ptr, slice};
use std::sync::Mutex;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::ops::{Deref, DerefMut};
use GlObject;
use BufferExt;

use sync;
use version::Api;

/// A buffer in the graphics card's memory.
pub struct Buffer {
    display: Display,
    id: gl::types::GLuint,
    ty: BufferType,
    elements_size: usize,
    elements_count: usize,
    persistent_mapping: Option<*mut libc::c_void>,

    /// Fences that the buffer must wait on before locking the permanent mapping.
    fences: Mutex<Vec<Receiver<sync::LinearSyncFence>>>,
}

/// Error that can happen when creating a buffer.
#[derive(Debug)]
pub enum BufferCreationError {
    /// Not enough memory to create the buffer.
    OutOfMemory,

    /// Persistent mapping is not supported.
    PersistentMappingNotSupported,

    /// This type of buffer is not supported.
    BufferTypeNotSupported,
}

/// Type of a buffer.
#[derive(Debug, Copy)]
pub enum BufferType {
    ArrayBuffer,
    PixelPackBuffer,
    PixelUnpackBuffer,
    UniformBuffer,
}

impl BufferType {
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            BufferType::ArrayBuffer => gl::ARRAY_BUFFER,
            BufferType::PixelPackBuffer => gl::PIXEL_PACK_BUFFER,
            BufferType::PixelUnpackBuffer => gl::PIXEL_UNPACK_BUFFER,
            BufferType::UniformBuffer => gl::UNIFORM_BUFFER,
        }
    }
}

impl Buffer {
    pub fn new<D>(display: &Display, data: Vec<D>, ty: BufferType, persistent: bool)
                  -> Result<Buffer, BufferCreationError>
                  where D: Send + Copy + 'static
    {
        let mut ctxt = display.context.context.make_current();

        let (id, persistent_mapping, elements_size, elements_count) = try!(unsafe {
            create_buffer(&mut ctxt, data, ty, persistent)
        });

        Ok(Buffer {
            display: display.clone(),
            id: id,
            ty: ty,
            elements_size: elements_size,
            elements_count: elements_count,
            persistent_mapping: persistent_mapping.map(|p| p.0),
            fences: Mutex::new(Vec::new()),
        })
    }

    pub fn new_empty(display: &Display, ty: BufferType, elements_size: usize, elements_count: usize,
                     usage: gl::types::GLenum) -> Buffer
    {
        let buffer_size = elements_count * elements_size as usize;

        let mut ctxt = display.context.context.make_current();

        let id = unsafe {
            let mut id: gl::types::GLuint = mem::uninitialized();

            if ctxt.version >= &GlVersion(Api::Gl, 1, 5) ||
                ctxt.version >= &GlVersion(Api::GlEs, 2, 0)
            {
                ctxt.gl.GenBuffers(1, &mut id);
            } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                ctxt.gl.GenBuffersARB(1, &mut id);
            } else {
                unreachable!();
            }

            let bind = bind_buffer(&mut ctxt, id, ty);

            if ctxt.version >= &GlVersion(Api::Gl, 4, 4) || ctxt.extensions.gl_arb_buffer_storage {
                ctxt.gl.BufferStorage(bind, buffer_size as gl::types::GLsizeiptr,
                                      ptr::null(),
                                      gl::DYNAMIC_STORAGE_BIT | gl::MAP_READ_BIT |
                                      gl::MAP_WRITE_BIT);       // TODO: more specific flags

            } else if ctxt.version >= &GlVersion(Api::Gl, 1, 5) ||
                ctxt.version >= &GlVersion(Api::GlEs, 2, 0)
            {
                ctxt.gl.BufferData(bind, buffer_size as gl::types::GLsizeiptr,
                                   ptr::null(), usage);

            } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                ctxt.gl.BufferDataARB(bind, buffer_size as gl::types::GLsizeiptr,
                                      ptr::null(), usage);      // TODO: better usage

            } else {
                unreachable!()
            }

            let mut obtained_size: gl::types::GLint = mem::uninitialized();

            if ctxt.version >= &GlVersion(Api::Gl, 1, 5) ||
                ctxt.version >= &GlVersion(Api::GlEs, 2, 0)
            {
                ctxt.gl.GetBufferParameteriv(bind, gl::BUFFER_SIZE, &mut obtained_size);
            } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                ctxt.gl.GetBufferParameterivARB(bind, gl::BUFFER_SIZE, &mut obtained_size);
            } else {
                unreachable!();
            }

            if buffer_size != obtained_size as usize {
                if ctxt.version >= &GlVersion(Api::Gl, 1, 5) ||
                    ctxt.version >= &GlVersion(Api::GlEs, 2, 0)
                {
                    ctxt.gl.DeleteBuffers(1, [id].as_ptr());
                } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                    ctxt.gl.DeleteBuffersARB(1, [id].as_ptr());
                } else {
                    unreachable!();
                }
                
                panic!("Not enough available memory for buffer (required: {} bytes, \
                        obtained: {})", buffer_size, obtained_size);
            }

            id
        };

        Buffer {
            display: display.clone(),
            id: id,
            ty: ty,
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

    /// Changes the type of the buffer. Returns `Err` if this is forbidden.
    pub fn set_type(mut self, ty: BufferType) -> Result<Buffer, Buffer> {
        // FIXME: return Err for GLES2
        self.ty = ty;
        Ok(self)
    }

    /// Uploads data in the buffer.
    ///
    /// This function considers that the buffer is filled of elements of type `D`. The offset
    /// is a number of elements, not a number of bytes.
    pub fn upload<D>(&self, offset: usize, data: Vec<D>)
                     where D: Copy + Send + 'static
    {
        let offset = offset * get_elements_size(&data);
        let buffer_size = get_elements_size(&data) * data.len();

        assert!(offset <= self.get_total_size());
        assert!(offset + buffer_size <= self.get_total_size());

        let invalidate_all = (offset == 0) && (buffer_size == self.get_total_size());

        let mut ctxt = self.display.context.context.make_current();

        unsafe {
            if invalidate_all && (ctxt.version >= &GlVersion(Api::Gl, 4, 3) ||
                ctxt.extensions.gl_arb_invalidate_subdata)
            {
                ctxt.gl.InvalidateBufferData(self.id);
            }

            if ctxt.version >= &GlVersion(Api::Gl, 4, 5) {
                ctxt.gl.NamedBufferSubData(self.id, offset as gl::types::GLintptr,
                                           buffer_size as gl::types::GLsizei,
                                           data.as_ptr() as *const libc::c_void)

            } else if ctxt.extensions.gl_ext_direct_state_access {
                ctxt.gl.NamedBufferSubDataEXT(self.id, offset as gl::types::GLintptr,
                                              buffer_size as gl::types::GLsizeiptr,
                                              data.as_ptr() as *const libc::c_void)

            } else if ctxt.version >= &GlVersion(Api::Gl, 1, 5) ||
                ctxt.version >= &GlVersion(Api::GlEs, 2, 0)
            {
                let bind = bind_buffer(&mut ctxt, self.id, self.ty);
                ctxt.gl.BufferSubData(bind, offset as gl::types::GLintptr,
                                      buffer_size as gl::types::GLsizeiptr,
                                      data.as_ptr() as *const libc::c_void);

            } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                let bind = bind_buffer(&mut ctxt, self.id, self.ty);
                ctxt.gl.BufferSubDataARB(bind, offset as gl::types::GLintptr,
                                         buffer_size as gl::types::GLsizeiptr,
                                         data.as_ptr() as *const libc::c_void);

            } else {
                unreachable!();
            }

            // TODO: fence in case of persistent mapping
        }
    }

    /// Offset and size should be specified as number of elements
    pub fn map<'a, D>(&'a mut self, offset: usize, size: usize)
                      -> Mapping<'a, D> where D: Send + 'static
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
            };
        }

        let offset_bytes = offset * self.elements_size;
        let size_bytes = size * self.elements_size;

        let ptr = unsafe {
            let mut ctxt = self.display.context.context.make_current();

            if ctxt.version >= &GlVersion(Api::Gl, 4, 5) {
                ctxt.gl.MapNamedBufferRange(self.id, offset_bytes as gl::types::GLintptr,
                                            size_bytes as gl::types::GLsizei,
                                            gl::MAP_READ_BIT | gl::MAP_WRITE_BIT)

            } else if ctxt.version >= &GlVersion(Api::Gl, 3, 0) ||
                ctxt.version >= &GlVersion(Api::GlEs, 3, 0) ||
                ctxt.extensions.gl_arb_map_buffer_range
            {
                let bind = bind_buffer(&mut ctxt, self.id, self.ty);
                ctxt.gl.MapBufferRange(bind, offset_bytes as gl::types::GLintptr,
                                       size_bytes as gl::types::GLsizeiptr,
                                       gl::MAP_READ_BIT | gl::MAP_WRITE_BIT)

            } else {
                unimplemented!();       // FIXME: 
            }
        };

        Mapping {
            buffer: self,
            data: ptr as *mut D,
            len: size,
        }
    }

    #[cfg(feature = "gl_read_buffer")]
    pub fn read<D>(&self) -> Vec<D> where D: Send + 'static {
        self.read_if_supported().unwrap()
    }

    pub fn read_if_supported<D>(&self) -> Option<Vec<D>> where D: Send + 'static {
        self.read_slice_if_supported(0, self.elements_count)
    }

    #[cfg(feature = "gl_read_buffer")]
    pub fn read_slice<D>(&self, offset: usize, size: usize)
                         -> Vec<D> where D: Send + 'static
    {
        self.read_slice_if_supported(offset, size).unwrap()
    }

    pub fn read_slice_if_supported<D>(&self, offset: usize, size: usize)
                                      -> Option<Vec<D>> where D: Send + 'static
    {
        assert!(offset + size <= self.elements_count);

        let mut ctxt = self.display.context.context.make_current();

        unsafe {
            let mut data = Vec::with_capacity(size);
            data.set_len(size);

            if ctxt.version >= &GlVersion(Api::Gl, 4, 5) {
                ctxt.gl.GetNamedBufferSubData(self.id, (offset * self.elements_size) as gl::types::GLintptr,
                    (size * self.elements_size) as gl::types::GLsizei,
                    data.as_mut_ptr() as *mut libc::c_void);

            } else if ctxt.version >= &GlVersion(Api::Gl, 1, 5) {
                let bind = bind_buffer(&mut ctxt, self.id, self.ty);
                ctxt.gl.GetBufferSubData(bind, (offset * self.elements_size) as gl::types::GLintptr,
                                         (size * self.elements_size) as gl::types::GLsizeiptr,
                                         data.as_mut_ptr() as *mut libc::c_void);

            } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                let bind = bind_buffer(&mut ctxt, self.id, self.ty);
                ctxt.gl.GetBufferSubDataARB(bind, (offset * self.elements_size) as gl::types::GLintptr,
                                            (size * self.elements_size) as gl::types::GLsizeiptr,
                                            data.as_mut_ptr() as *mut libc::c_void);

            } else if ctxt.version >= &GlVersion(Api::GlEs, 1, 0) {
                return None;

            } else {
                unreachable!()
            }

            Some(data)
        }
    }
}

impl BufferExt for Buffer {
    fn add_fence(&self) -> Option<Sender<sync::LinearSyncFence>> {
        if !self.is_persistent() {
            return None;
        }

        let (tx, rx) = channel();
        self.fences.lock().unwrap().push(rx);
        Some(tx)
    }
}

impl fmt::Debug for Buffer {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "Buffer #{} (size: {} bytes)", self.id, self.get_total_size())
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let mut ctxt = self.display.context.context.make_current();

        self.display.context.vertex_array_objects.purge_buffer(&mut ctxt, self.id);

        if ctxt.state.array_buffer_binding == self.id {
            ctxt.state.array_buffer_binding = 0;
        }

        if ctxt.state.pixel_pack_buffer_binding == self.id {
            ctxt.state.pixel_pack_buffer_binding = 0;
        }

        if ctxt.state.pixel_unpack_buffer_binding == self.id {
            ctxt.state.pixel_unpack_buffer_binding = 0;
        }

        if ctxt.state.uniform_buffer_binding == self.id {
            ctxt.state.uniform_buffer_binding = 0;
        }

        unsafe {
            if ctxt.version >= &GlVersion(Api::Gl, 1, 5) ||
                ctxt.version >= &GlVersion(Api::GlEs, 2, 0)
            {
                ctxt.gl.DeleteBuffers(1, [self.id].as_ptr());
            } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                ctxt.gl.DeleteBuffersARB(1, [self.id].as_ptr());
            } else {
                unreachable!();
            }
        }
    }
}

impl GlObject for Buffer {
    type Id = gl::types::GLuint;
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

/// A mapping of a buffer.
pub struct Mapping<'b, D> {
    buffer: &'b mut Buffer,
    data: *mut D,
    len: usize,
}

// TODO: remove
struct MappedBufferWrapper<D>(*mut D);
unsafe impl<D> Send for MappedBufferWrapper<D> {}
unsafe impl<D> Sync for MappedBufferWrapper<D> {}

#[unsafe_destructor]
impl<'a, D> Drop for Mapping<'a, D> {
    fn drop(&mut self) {
        // don't unmap if the buffer is persistent
        if self.buffer.is_persistent() {
            return;
        }

        let mut ctxt = self.buffer.display.context.context.make_current();

        unsafe {
            if ctxt.version >= &GlVersion(Api::Gl, 4, 5) {
                ctxt.gl.UnmapNamedBuffer(self.buffer.id);

            } else if ctxt.version >= &GlVersion(Api::Gl, 1, 5) ||
                ctxt.version >= &GlVersion(Api::GlEs, 3, 0)
            {
                let bind = bind_buffer(&mut ctxt, self.buffer.id, self.buffer.ty);
                ctxt.gl.UnmapBuffer(bind);

            } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                let bind = bind_buffer(&mut ctxt, self.buffer.id, self.buffer.ty);
                ctxt.gl.UnmapBufferARB(bind);

            } else {
                unreachable!();
            }
        }
    }
}

impl<'a, D> Deref for Mapping<'a, D> {
    type Target = [D];
    fn deref<'b>(&'b self) -> &'b [D] {
        unsafe {
            slice::from_raw_parts_mut(self.data, self.len)
        }
    }
}

impl<'a, D> DerefMut for Mapping<'a, D> {
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

/// Creates a new buffer.
unsafe fn create_buffer<D>(mut ctxt: &mut CommandContext, data: Vec<D>, ty: BufferType,
                           persistent: bool)
                           -> Result<(gl::types::GLuint,
                                      Option<MappedBufferWrapper<libc::c_void>>, usize, usize),
                                     BufferCreationError>
                           where D: Send + Copy + 'static
{
    if persistent && !(ctxt.version >= &context::GlVersion(Api::Gl, 4, 4)) &&
       !ctxt.extensions.gl_arb_buffer_storage
    {
        return Err(BufferCreationError::PersistentMappingNotSupported);
    }

    let elements_size = get_elements_size(&data);
    let elements_count = data.len();

    let buffer_size = match elements_count * elements_size as usize {
        0 => 1,     // use size 1 instead of 0, or nvidia drivers complain
        a => a
    };

    let data_ptr = if elements_count * elements_size as usize == 0 {
        ptr::null()
    } else {
        data.as_ptr()
    };

    let mut id: gl::types::GLuint = mem::uninitialized();

    if ctxt.version >= &GlVersion(Api::Gl, 4, 5) || ctxt.extensions.gl_arb_direct_state_access {
        ctxt.gl.CreateBuffers(1, &mut id);
    } else if ctxt.version >= &GlVersion(Api::Gl, 1, 5) ||
        ctxt.version >= &GlVersion(Api::GlEs, 2, 0)
    {
        ctxt.gl.GenBuffers(1, &mut id);
    } else if ctxt.extensions.gl_arb_vertex_buffer_object {
        ctxt.gl.GenBuffersARB(1, &mut id);
    } else {
        unreachable!();
    }

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
        let bind = bind_buffer(&mut ctxt, id, ty);
        ctxt.gl.BufferStorage(bind, buffer_size as gl::types::GLsizeiptr,
                              data_ptr as *const libc::c_void,
                              flags);
        ctxt.gl.GetBufferParameteriv(bind, gl::BUFFER_SIZE, &mut obtained_size);

    } else if ctxt.version >= &GlVersion(Api::Gl, 1, 5) ||
        ctxt.version >= &GlVersion(Api::GlEs, 2, 0)
    {
        debug_assert!(!persistent);
        let bind = bind_buffer(&mut ctxt, id, ty);
        ctxt.gl.BufferData(bind, buffer_size as gl::types::GLsizeiptr,
                           data_ptr as *const libc::c_void, gl::STATIC_DRAW);      // TODO: better usage
        ctxt.gl.GetBufferParameteriv(bind, gl::BUFFER_SIZE, &mut obtained_size);

    } else if ctxt.extensions.gl_arb_vertex_buffer_object {
        debug_assert!(!persistent);
        let bind = bind_buffer(&mut ctxt, id, ty);
        ctxt.gl.BufferDataARB(bind, buffer_size as gl::types::GLsizeiptr,
                              data_ptr as *const libc::c_void, gl::STATIC_DRAW);      // TODO: better usage
        ctxt.gl.GetBufferParameterivARB(bind, gl::BUFFER_SIZE, &mut obtained_size);

    } else {
        unreachable!();
    }

    if buffer_size != obtained_size as usize {
        if ctxt.version >= &GlVersion(Api::Gl, 1, 5) ||
            ctxt.version >= &GlVersion(Api::GlEs, 2, 0)
        {
            ctxt.gl.DeleteBuffers(1, [id].as_ptr());
        } else if ctxt.extensions.gl_arb_vertex_buffer_object {
            ctxt.gl.DeleteBuffersARB(1, [id].as_ptr());
        } else {
            unreachable!();
        }
        
        return Err(BufferCreationError::OutOfMemory);
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
            let bind = bind_buffer(&mut ctxt, id, ty);
            ctxt.gl.MapBufferRange(bind, 0, buffer_size as gl::types::GLsizeiptr,
                                   gl::MAP_READ_BIT | gl::MAP_WRITE_BIT |
                                   gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT)
        };

        if ptr.is_null() {
            let error = ::get_gl_error(ctxt);
            panic!("glMapBufferRange returned null (error: {:?})", error);
        }

        Some(MappedBufferWrapper(ptr))

    } else {
        None
    };

    Ok((id, persistent_mapping, elements_size, elements_count))
}

/// Binds a buffer of the given type, and returns the GLenum of the bind point.
unsafe fn bind_buffer(mut ctxt: &mut CommandContext, id: gl::types::GLuint, ty: BufferType)
                      -> gl::types::GLenum
{
    match ty {
        BufferType::ArrayBuffer => {
            if ctxt.state.array_buffer_binding != id {
                ctxt.state.array_buffer_binding = id;

                if ctxt.version >= &GlVersion(Api::Gl, 1, 5) ||
                    ctxt.version >= &GlVersion(Api::GlEs, 2, 0)
                {
                    ctxt.gl.BindBuffer(gl::ARRAY_BUFFER, id);
                } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                    ctxt.gl.BindBufferARB(gl::ARRAY_BUFFER, id);    // bind points are the same in the ext
                } else {
                    unreachable!();
                }
            }

            gl::ARRAY_BUFFER
        },

        BufferType::PixelPackBuffer => {
            if ctxt.state.pixel_pack_buffer_binding != id {
                ctxt.state.pixel_pack_buffer_binding = id;

                if ctxt.version >= &GlVersion(Api::Gl, 1, 5) ||
                    ctxt.version >= &GlVersion(Api::GlEs, 2, 0)
                {
                    ctxt.gl.BindBuffer(gl::PIXEL_PACK_BUFFER, id);
                } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                    ctxt.gl.BindBufferARB(gl::PIXEL_PACK_BUFFER, id);    // bind points are the same in the ext
                } else {
                    unreachable!();
                }
            }

            gl::PIXEL_PACK_BUFFER
        },

        BufferType::PixelUnpackBuffer => {
            if ctxt.state.pixel_unpack_buffer_binding != id {
                ctxt.state.pixel_unpack_buffer_binding = id;

                if ctxt.version >= &GlVersion(Api::Gl, 1, 5) ||
                    ctxt.version >= &GlVersion(Api::GlEs, 2, 0)
                {
                    ctxt.gl.BindBuffer(gl::PIXEL_UNPACK_BUFFER, id);
                } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                    ctxt.gl.BindBufferARB(gl::PIXEL_UNPACK_BUFFER, id);    // bind points are the same in the ext
                } else {
                    unreachable!();
                }
            }

            gl::PIXEL_UNPACK_BUFFER
        },

        BufferType::UniformBuffer => {
            if ctxt.state.uniform_buffer_binding != id {
                ctxt.state.uniform_buffer_binding = id;

                if ctxt.version >= &GlVersion(Api::Gl, 1, 5) ||
                    ctxt.version >= &GlVersion(Api::GlEs, 2, 0)
                {
                    ctxt.gl.BindBuffer(gl::UNIFORM_BUFFER, id);
                } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                    ctxt.gl.BindBufferARB(gl::UNIFORM_BUFFER, id);    // bind points are the same in the ext
                } else {
                    unreachable!();
                }
            }

            gl::UNIFORM_BUFFER
        },
    }
}
