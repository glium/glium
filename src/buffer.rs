use backend::Facade;
use context::{self, GlVersion};
use context::CommandContext;
use context::Context;
use ContextExt;
use gl;
use libc;
use std::{fmt, mem, ptr, slice};
use std::sync::Mutex;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::rc::Rc;
use std::ops::{Deref, DerefMut};
use GlObject;
use BufferExt;

use sync;
use version::Api;

/// A buffer in the graphics card's memory.
pub struct Buffer {
    context: Rc<Context>,
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

/// Flags to specify how the buffer should behave.
#[derive(Debug, Copy, Clone)]
pub struct BufferFlags {
    /// The contents of the data store may be updated after creation through calls to
    /// glBufferSubData. If this bit is not set, the buffer content may not be directly updated
    /// by the client. Regardless of the presence of this bit, buffers may always be updated with
    /// server-side calls such as glCopyBufferSubData and glClearBufferSubData.
    pub dynamic: bool,

    /// When all other criteria for the buffer storage allocation are met, this bit may be used by
    /// an implementation to determine whether to use storage that is local to the server or to
    /// the client to serve as the backing store for the buffer.
    pub client_storage: bool,

    /// Specifies how the buffer may be mapped.
    pub mapping: BufferFlagsMapping,
}

impl BufferFlags {
    /// Builds very tolerant flags.
    pub fn simple() -> BufferFlags {
        BufferFlags {
            dynamic: true,
            client_storage: false,
            mapping: BufferFlagsMapping::ReadWrite(BufferFlagsPersistent::None),
        }
    }

    /// Builds persistent flags.
    pub fn persistent() -> BufferFlags {
        BufferFlags {
            dynamic: true,
            client_storage: false,
            mapping: BufferFlagsMapping::ReadWrite(BufferFlagsPersistent::PersistentCoherent),
        }
    }

    /// Returns true if the flags request a persistent buffer.
    pub fn is_persistent(&self) -> bool {
        let persistent = match self.mapping {
            BufferFlagsMapping::None => return false,
            BufferFlagsMapping::Read(p) => p,
            BufferFlagsMapping::Write(p) => p,
            BufferFlagsMapping::ReadWrite(p) => p,
        };

        match persistent {
            BufferFlagsPersistent::None => false,
            BufferFlagsPersistent::Persistent => true,
            BufferFlagsPersistent::PersistentCoherent => true,
        }
    }
}

/// Flags specifying how the buffer may be mapped.
#[derive(Debug, Copy, Clone)]
pub enum BufferFlagsMapping {
    /// No mapping allowed.
    None,

    /// Read-only mapping. The mapped buffer may not be written to.
    Read(BufferFlagsPersistent),

    /// Write-only mapping. The mapped buffer may not be read from.
    Write(BufferFlagsPersistent),

    /// Read and write mapping.
    ReadWrite(BufferFlagsPersistent),
}

/// Flags specifying whether mapping is persistent.
#[derive(Debug, Copy, Clone)]
pub enum BufferFlagsPersistent {
    /// No persistent mapping.
    None,

    /// The client may request that the server read from or write to the buffer while it is mapped.
    /// The client's pointer to the data store remains valid so long as the data store is mapped,
    /// even during execution of drawing or dispatch commands.
    ///
    /// If the client performs a write followed by a call to the glMemoryBarrier command with the
    /// GL_CLIENT_MAPPED_BUFFER_BARRIER_BIT set, then in subsequent commands the server will see
    /// the writes.
    ///
    /// If the server performs a write, the application must call glMemoryBarrier with the
    /// GL_CLIENT_MAPPED_BUFFER_BARRIER_BIT set and then call glFenceSync with
    /// GL_SYNC_GPU_COMMANDS_COMPLETE (or glFinish). Then the CPU will see the writes after the
    /// sync is complete.
    Persistent,

    /// The client may request that the server read from or write to the buffer while it is mapped.
    /// The client's pointer to the data store remains valid so long as the data store is mapped,
    /// even during execution of drawing or dispatch commands.
    ///
    /// Shared access to buffers that are simultaneously mapped for client access and are used by
    /// the server will be coherent, so long as that mapping is performed using glMapBufferRange.
    /// That is, data written to the store by either the client or server will be immediately
    /// visible to the other with no further action taken by the application.
    ///
    /// If the client performs a write, then in subsequent commands the server will see the writes.
    ///
    /// If the server does a write, the app must call FenceSync with GL_SYNC_GPU_COMMANDS_COMPLETE
    /// (or glFinish). Then the CPU will see the writes after the sync is complete.
    PersistentCoherent,
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
    pub fn new<D, F>(facade: &F, data: &[D], ty: BufferType, flags: BufferFlags)
                     -> Result<Buffer, BufferCreationError>
                     where D: Send + Copy + 'static, F: Facade
    {
        let mut ctxt = facade.get_context().make_current();

        let elements_size = get_elements_size(data);
        let elements_count = data.len();

        let (id, persistent_mapping) = try!(unsafe {
            create_buffer(&mut ctxt, elements_size, elements_count, Some(&data), ty, flags)
        });

        Ok(Buffer {
            context: facade.get_context().clone(),
            id: id,
            ty: ty,
            elements_size: elements_size,
            elements_count: elements_count,
            persistent_mapping: persistent_mapping,
            fences: Mutex::new(Vec::new()),
        })
    }

    pub fn new_empty<F>(facade: &F, ty: BufferType, elements_size: usize,
                        elements_count: usize, flags: BufferFlags)
                        -> Result<Buffer, BufferCreationError> where F: Facade
    {
        let mut ctxt = facade.get_context().make_current();

        let (id, persistent_mapping) = try!(unsafe {
            create_buffer::<()>(&mut ctxt, elements_size, elements_count, None, ty, flags)
        });

        Ok(Buffer {
            context: facade.get_context().clone(),
            id: id,
            ty: ty,
            elements_size: elements_size,
            elements_count: elements_count,
            persistent_mapping: persistent_mapping,
            fences: Mutex::new(Vec::new()),
        })
    }

    pub fn get_context(&self) -> &Rc<Context> {
        &self.context
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

        let mut ctxt = self.context.make_current();

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
                    fence.recv().unwrap().into_sync_fence(&self.context).wait();
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
            let mut ctxt = self.context.make_current();

            // FIXME: incorrect flags
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

        let mut ctxt = self.context.make_current();

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
        let mut ctxt = self.context.make_current();

        self.context.vertex_array_objects.purge_buffer(&mut ctxt, self.id);

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

#[unsafe_destructor]
impl<'a, D> Drop for Mapping<'a, D> {
    fn drop(&mut self) {
        // don't unmap if the buffer is persistent
        if self.buffer.is_persistent() {
            return;
        }

        let mut ctxt = self.buffer.context.make_current();

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
fn get_elements_size<T>(data: &[T]) -> usize {
    if data.len() <= 1 {
        mem::size_of::<T>()
    } else {
        let d0: *const T = &data[0];
        let d1: *const T = &data[1];
        (d1 as usize) - (d0 as usize)
    }
}

/// Creates a new buffer.
unsafe fn create_buffer<D>(mut ctxt: &mut CommandContext, elements_size: usize,
                           elements_count: usize, data: Option<&[D]>, ty: BufferType,
                           flags: BufferFlags)
                           -> Result<(gl::types::GLuint, Option<*mut libc::c_void>),
                                     BufferCreationError>
                           where D: Send + Copy + 'static
{
    if flags.is_persistent() && !(ctxt.version >= &context::GlVersion(Api::Gl, 4, 4)) &&
       !ctxt.extensions.gl_arb_buffer_storage
    {
        return Err(BufferCreationError::PersistentMappingNotSupported);
    }

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

    let buffer_size = match elements_count * elements_size as usize {
        0 => 1,     // use size 1 instead of 0, or nvidia drivers complain
        a => a
    };

    let data_ptr = if let Some(data) = data {
        if elements_count * elements_size as usize == 0 {
            ptr::null()
        } else {
            data.as_ptr()
        }
    } else {
        ptr::null()
    };

    let mut obtained_size: gl::types::GLint = mem::uninitialized();

    if ctxt.version >= &GlVersion(Api::Gl, 4, 5) || ctxt.extensions.gl_arb_direct_state_access {
        let flags = immutable_storage_flags(flags);
        ctxt.gl.NamedBufferStorage(id, buffer_size as gl::types::GLsizei,
                                   data_ptr as *const libc::c_void,
                                   flags);
        ctxt.gl.GetNamedBufferParameteriv(id, gl::BUFFER_SIZE, &mut obtained_size);

    } else if ctxt.extensions.gl_arb_buffer_storage &&
              ctxt.extensions.gl_ext_direct_state_access
    {
        let flags = immutable_storage_flags(flags);
        ctxt.gl.NamedBufferStorageEXT(id, buffer_size as gl::types::GLsizeiptr,
                                      data_ptr as *const libc::c_void,
                                      flags);
        ctxt.gl.GetNamedBufferParameterivEXT(id, gl::BUFFER_SIZE, &mut obtained_size);


    } else if ctxt.version >= &GlVersion(Api::Gl, 4, 4) ||
              ctxt.extensions.gl_arb_buffer_storage
    {
        let bind = bind_buffer(&mut ctxt, id, ty);
        let flags = immutable_storage_flags(flags);
        ctxt.gl.BufferStorage(bind, buffer_size as gl::types::GLsizeiptr,
                              data_ptr as *const libc::c_void,
                              flags);
        ctxt.gl.GetBufferParameteriv(bind, gl::BUFFER_SIZE, &mut obtained_size);

    } else if ctxt.version >= &GlVersion(Api::Gl, 1, 5) ||
        ctxt.version >= &GlVersion(Api::GlEs, 2, 0)
    {
        let bind = bind_buffer(&mut ctxt, id, ty);
        let flags = try!(mutable_storage_flags(&mut ctxt, flags));
        ctxt.gl.BufferData(bind, buffer_size as gl::types::GLsizeiptr,
                           data_ptr as *const libc::c_void, flags);
        ctxt.gl.GetBufferParameteriv(bind, gl::BUFFER_SIZE, &mut obtained_size);

    } else if ctxt.extensions.gl_arb_vertex_buffer_object {
        let bind = bind_buffer(&mut ctxt, id, ty);
        let flags = try!(mutable_storage_flags(&mut ctxt, flags));
        ctxt.gl.BufferDataARB(bind, buffer_size as gl::types::GLsizeiptr,
                              data_ptr as *const libc::c_void, flags);
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

    let persistent_mapping = if flags.is_persistent() {
        assert!(ctxt.version >= &GlVersion(Api::Gl, 3, 0) ||
                ctxt.extensions.gl_arb_map_buffer_range);

        // TODO: better handling of versions
        // FIXME: incorrect flags
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

        Some(ptr)

    } else {
        None
    };

    Ok((id, persistent_mapping))
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

fn immutable_storage_flags(flags: BufferFlags) -> gl::types::GLenum {
    let mut output = 0;

    if flags.dynamic {
        output |= gl::DYNAMIC_STORAGE_BIT;
    }

    if flags.client_storage {
        output |= gl::CLIENT_STORAGE_BIT;
    }

    let persistent = match flags.mapping {
        BufferFlagsMapping::None => None,
        BufferFlagsMapping::Read(p) => {
            output |= gl::MAP_READ_BIT;
            Some(p)
        },
        BufferFlagsMapping::Write(p) => {
            output |= gl::MAP_WRITE_BIT;
            Some(p)
        },
        BufferFlagsMapping::ReadWrite(p) => {
            output |= gl::MAP_READ_BIT | gl::MAP_WRITE_BIT;
            Some(p)
        },
    };

    match persistent {
        None => (),
        Some(BufferFlagsPersistent::None) => (),
        Some(BufferFlagsPersistent::Persistent) => {
            output |= gl::MAP_PERSISTENT_BIT;
        },
        Some(BufferFlagsPersistent::PersistentCoherent) => {
            output |= gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT;
        },
    };

    output
}

fn mutable_storage_flags(ctxt: &mut CommandContext, flags: BufferFlags)
                         -> Result<gl::types::GLenum, BufferCreationError>
{
    // FIXME: do it properly
    // FIXME: detect persistent and return Err if not supported
    Ok(gl::STATIC_DRAW)
}
