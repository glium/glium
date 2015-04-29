use backend::Facade;
use context::CommandContext;
use context::Context;
use version::Version;
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

    /// OpenGL identifier ; can't be zero.
    id: gl::types::GLuint,

    /// Type of buffer.
    ty: BufferType,

    /// Size in bytes in each element of the buffer.
    elements_size: usize,

    /// Number of elements in the buffer.
    elements_count: usize,

    /// A pointer to the persistent mapping of this buffer in memory, if there is one.
    persistent_mapping: Option<*mut libc::c_void>,

    /// If true, then this buffer can only be modified by calls to `glCopyBufferSubData` or through
    /// the persistent mapping.
    immutable: bool,

    /// Fences that the buffer must wait on before locking the permanent mapping.
    fences: Mutex<Vec<Receiver<sync::LinearSyncFence>>>,
}

/// Error that can happen when creating a buffer.
#[derive(Debug)]
pub enum BufferCreationError {
    /// Not enough memory to create the buffer.
    OutOfMemory,

    /// This type of buffer is not supported.
    BufferTypeNotSupported,
}

/// Type of a buffer.
#[derive(Debug, Copy, Clone)]
pub enum BufferType {
    ArrayBuffer,
    PixelPackBuffer,
    PixelUnpackBuffer,
    UniformBuffer,
    CopyReadBuffer,
    CopyWriteBuffer,
}

/// A mapping of a buffer.
pub struct Mapping<'b, D> {
    buffer: &'b mut Buffer,
    temporary_buffer: Option<(gl::types::GLuint, usize)>,
    data: *mut D,
    len: usize,
}

impl BufferType {
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            BufferType::ArrayBuffer => gl::ARRAY_BUFFER,
            BufferType::PixelPackBuffer => gl::PIXEL_PACK_BUFFER,
            BufferType::PixelUnpackBuffer => gl::PIXEL_UNPACK_BUFFER,
            BufferType::UniformBuffer => gl::UNIFORM_BUFFER,
            BufferType::CopyReadBuffer => gl::COPY_READ_BUFFER,
            BufferType::CopyWriteBuffer => gl::COPY_WRITE_BUFFER,
        }
    }
}

impl Buffer {
    pub fn new<D, F>(facade: &F, data: &[D], ty: BufferType, dynamic: bool)
                     -> Result<Buffer, BufferCreationError>
                     where D: Send + Copy + 'static, F: Facade
    {
        let mut ctxt = facade.get_context().make_current();

        let elements_size = get_elements_size(data);
        let elements_count = data.len();

        let (id, immutable, persistent_mapping) = try!(unsafe {
            create_buffer(&mut ctxt, elements_size, elements_count, Some(&data), ty, dynamic, false)
        });

        Ok(Buffer {
            context: facade.get_context().clone(),
            id: id,
            ty: ty,
            elements_size: elements_size,
            elements_count: elements_count,
            persistent_mapping: persistent_mapping,
            immutable: immutable,
            fences: Mutex::new(Vec::new()),
        })
    }

    pub fn empty<F>(facade: &F, ty: BufferType, elements_size: usize,
                    elements_count: usize, dynamic: bool)
                    -> Result<Buffer, BufferCreationError> where F: Facade
    {
        let mut ctxt = facade.get_context().make_current();

        let (id, immutable, persistent_mapping) = try!(unsafe {
            create_buffer::<()>(&mut ctxt, elements_size, elements_count, None, ty, dynamic, false)
        });

        Ok(Buffer {
            context: facade.get_context().clone(),
            id: id,
            ty: ty,
            elements_size: elements_size,
            elements_count: elements_count,
            persistent_mapping: persistent_mapping,
            immutable: immutable,
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
    pub fn upload<D>(&self, offset: usize, data: &[D])
                     where D: Copy + Send + 'static
    {
        if self.persistent_mapping.is_some() {
            // FIXME: this code is safe, but ugly
            let me: &mut Buffer = unsafe { mem::transmute(self) };
            let mut mapping = me.map(offset, data.len());
            unsafe {
                ptr::copy_nonoverlapping(data.as_ptr(), mapping.deref_mut().as_mut_ptr(),
                                         data.len());
            }
            return;
        }

        let offset = offset * get_elements_size(data);
        let buffer_size = get_elements_size(data) * data.len();

        assert!(offset <= self.get_total_size());
        assert!(offset + buffer_size <= self.get_total_size());

        let invalidate_all = (offset == 0) && (buffer_size == self.get_total_size());

        let mut ctxt = self.context.make_current();

        unsafe {
            if invalidate_all && (ctxt.version >= &Version(Api::Gl, 4, 3) ||
                ctxt.extensions.gl_arb_invalidate_subdata)
            {
                ctxt.gl.InvalidateBufferData(self.id);
            }

            if self.immutable {
                let (tmp_buffer, _, _) = create_buffer(&mut ctxt, self.elements_size, data.len(),
                                                       Some(data), BufferType::CopyReadBuffer,
                                                       true, true).unwrap();
                copy_buffer(&mut ctxt, tmp_buffer, 0, self.id, offset, buffer_size);
                destroy_buffer(&mut ctxt, tmp_buffer);

            } else {
                if ctxt.version >= &Version(Api::Gl, 4, 5) {
                    ctxt.gl.NamedBufferSubData(self.id, offset as gl::types::GLintptr,
                                               buffer_size as gl::types::GLsizei,
                                               data.as_ptr() as *const libc::c_void)

                } else if ctxt.extensions.gl_ext_direct_state_access {
                    ctxt.gl.NamedBufferSubDataEXT(self.id, offset as gl::types::GLintptr,
                                                  buffer_size as gl::types::GLsizeiptr,
                                                  data.as_ptr() as *const libc::c_void)

                } else if ctxt.version >= &Version(Api::Gl, 1, 5) ||
                    ctxt.version >= &Version(Api::GlEs, 2, 0)
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
            }

            // TODO: fence in case of persistent mapping
        }
    }

    /// Offset and size should be specified as number of elements
    pub fn map<'a, D>(&'a mut self, offset: usize, size: usize)
                      -> Mapping<'a, D> where D: Copy + Send + 'static
    {
        if offset > self.elements_count || (offset + size) > self.elements_count {
            panic!("Trying to map out of range of buffer");
        }

        if let Some(existing_mapping) = self.persistent_mapping.clone() {
            // we have a `&mut self`, so there's no risk of deadlock when locking `fences`
            {
                let mut fences = self.fences.lock().unwrap();
                for fence in mem::replace(&mut *fences, Vec::with_capacity(0)) {
                    fence.recv().unwrap().into_sync_fence(&self.context).wait();
                }
            }

            Mapping {
                buffer: self,
                temporary_buffer: None,
                data: unsafe { (existing_mapping as *mut D).offset(offset as isize) },
                len: size,
            }

        } else if self.immutable {
            // we have to construct a temporary buffer that we will map in memory
            // then after the Mapping is destroyed, we will copy from the temporary buffer to the
            // real one
            let temporary_buffer = unsafe {
                let mut ctxt = self.context.make_current();
                let (temporary_buffer, _, _) = create_buffer::<D>(&mut ctxt, self.elements_size, size,
                                                                  None, BufferType::CopyWriteBuffer,
                                                                  true, true).unwrap();
                temporary_buffer
            };

            let offset_bytes = offset * self.elements_size;
            let size_bytes = size * self.elements_size;

            let ptr = unsafe {
                let mut ctxt = self.context.make_current();

                copy_buffer(&mut ctxt, self.id, offset_bytes, temporary_buffer, 0, size_bytes);

                if ctxt.version >= &Version(Api::Gl, 4, 5) {
                    ctxt.gl.MapNamedBufferRange(temporary_buffer, 0, size_bytes as gl::types::GLsizei,
                                                gl::MAP_READ_BIT | gl::MAP_WRITE_BIT)

                } else if ctxt.version >= &Version(Api::Gl, 3, 0) ||
                    ctxt.version >= &Version(Api::GlEs, 3, 0) ||
                    ctxt.extensions.gl_arb_map_buffer_range
                {
                    let bind = bind_buffer(&mut ctxt, temporary_buffer, self.ty);
                    ctxt.gl.MapBufferRange(bind, 0, size_bytes as gl::types::GLsizeiptr,
                                           gl::MAP_READ_BIT | gl::MAP_WRITE_BIT)

                } else {
                    unimplemented!();       // FIXME: 
                }
            };

            Mapping {
                buffer: self,
                temporary_buffer: Some((temporary_buffer, offset_bytes)),
                data: ptr as *mut D,
                len: size,
            }

        } else {
            let offset_bytes = offset * self.elements_size;
            let size_bytes = size * self.elements_size;

            let ptr = unsafe {
                let mut ctxt = self.context.make_current();

                if ctxt.version >= &Version(Api::Gl, 4, 5) {
                    ctxt.gl.MapNamedBufferRange(self.id, offset_bytes as gl::types::GLintptr,
                                                size_bytes as gl::types::GLsizei,
                                                gl::MAP_READ_BIT | gl::MAP_WRITE_BIT)

                } else if ctxt.version >= &Version(Api::Gl, 3, 0) ||
                    ctxt.version >= &Version(Api::GlEs, 3, 0) ||
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
                temporary_buffer: None,
                data: ptr as *mut D,
                len: size,
            }
        }
    }

    #[cfg(feature = "gl_read_buffer")]
    pub fn read<D>(&self) -> Vec<D> where D: Copy + Send + 'static {
        self.read_if_supported().unwrap()
    }

    pub fn read_if_supported<D>(&self) -> Option<Vec<D>> where D: Copy + Send + 'static {
        self.read_slice_if_supported(0, self.elements_count)
    }

    #[cfg(feature = "gl_read_buffer")]
    pub fn read_slice<D>(&self, offset: usize, size: usize)
                         -> Vec<D> where D: Copy + Send + 'static
    {
        self.read_slice_if_supported(offset, size).unwrap()
    }

    pub fn read_slice_if_supported<D>(&self, offset: usize, size: usize)
                                      -> Option<Vec<D>> where D: Copy + Send + 'static
    {
        assert!(offset + size <= self.elements_count);

        if self.persistent_mapping.is_some() {
            // FIXME: this code is safe, but ugly
            let me: &mut Buffer = unsafe { mem::transmute(self) };
            let mapping = me.map(offset, size);
            let mut result = Vec::with_capacity(size);
            unsafe {
                ptr::copy_nonoverlapping(mapping.as_ptr(), result.as_mut_ptr(), size);
                result.set_len(size);
            }
            return Some(result);
        }

        let mut ctxt = self.context.make_current();

        unsafe {
            let mut data = Vec::with_capacity(size);
            data.set_len(size);

            if ctxt.version >= &Version(Api::Gl, 4, 5) {
                ctxt.gl.GetNamedBufferSubData(self.id, (offset * self.elements_size) as gl::types::GLintptr,
                    (size * self.elements_size) as gl::types::GLsizei,
                    data.as_mut_ptr() as *mut libc::c_void);

            } else if ctxt.version >= &Version(Api::Gl, 1, 5) {
                let bind = bind_buffer(&mut ctxt, self.id, self.ty);
                ctxt.gl.GetBufferSubData(bind, (offset * self.elements_size) as gl::types::GLintptr,
                                         (size * self.elements_size) as gl::types::GLsizeiptr,
                                         data.as_mut_ptr() as *mut libc::c_void);

            } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                let bind = bind_buffer(&mut ctxt, self.id, self.ty);
                ctxt.gl.GetBufferSubDataARB(bind, (offset * self.elements_size) as gl::types::GLintptr,
                                            (size * self.elements_size) as gl::types::GLsizeiptr,
                                            data.as_mut_ptr() as *mut libc::c_void);

            } else if ctxt.version >= &Version(Api::GlEs, 1, 0) {
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
        if self.persistent_mapping.is_none() {
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
        unsafe {
            let mut ctxt = self.context.make_current();
            self.context.vertex_array_objects.purge_buffer(&mut ctxt, self.id);
            destroy_buffer(&mut ctxt, self.id);
        }
    }
}

impl GlObject for Buffer {
    type Id = gl::types::GLuint;
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

unsafe impl<'a, D> Sync for Mapping<'a, D> where D: Sync {}

impl<'a, D> Drop for Mapping<'a, D> {
    fn drop(&mut self) {
        // don't unmap if the buffer is persistent
        if self.buffer.is_persistent() {
            return;
        }

        let mut ctxt = self.buffer.context.make_current();

        if let Some((temporary_buffer, offset_bytes)) = self.temporary_buffer {
            unsafe {
                if ctxt.version >= &Version(Api::Gl, 4, 5) {
                    ctxt.gl.UnmapNamedBuffer(temporary_buffer);

                } else if ctxt.version >= &Version(Api::Gl, 1, 5) ||
                    ctxt.version >= &Version(Api::GlEs, 3, 0)
                {
                    let bind = bind_buffer(&mut ctxt, temporary_buffer, self.buffer.ty);
                    ctxt.gl.UnmapBuffer(bind);

                } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                    let bind = bind_buffer(&mut ctxt, temporary_buffer, self.buffer.ty);
                    ctxt.gl.UnmapBufferARB(bind);

                } else {
                    unreachable!();
                }

                copy_buffer(&mut ctxt, temporary_buffer, 0, self.buffer.id, offset_bytes,
                            self.len * self.buffer.elements_size);

                destroy_buffer(&mut ctxt, temporary_buffer);
            }

        } else {
            unsafe {
                if ctxt.version >= &Version(Api::Gl, 4, 5) {
                    ctxt.gl.UnmapNamedBuffer(self.buffer.id);

                } else if ctxt.version >= &Version(Api::Gl, 1, 5) ||
                    ctxt.version >= &Version(Api::GlEs, 3, 0)
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
                           dynamic: bool, avoid_persistent: bool)
                           -> Result<(gl::types::GLuint, bool, Option<*mut libc::c_void>),
                                     BufferCreationError>
                           where D: Send + Copy + 'static
{
    let mut id: gl::types::GLuint = mem::uninitialized();

    if ctxt.version >= &Version(Api::Gl, 4, 5) || ctxt.extensions.gl_arb_direct_state_access {
        ctxt.gl.CreateBuffers(1, &mut id);
    } else if ctxt.version >= &Version(Api::Gl, 1, 5) ||
        ctxt.version >= &Version(Api::GlEs, 2, 0)
    {
        ctxt.gl.GenBuffers(1, &mut id);
    } else if ctxt.extensions.gl_arb_vertex_buffer_object {
        ctxt.gl.GenBuffersARB(1, &mut id);
    } else {
        unreachable!();
    }

    let buffer_size = match elements_count * elements_size {
        0 => 1,     // use size 1 instead of 0, or nvidia drivers complain
        a => a
    };

    let data_ptr = if let Some(data) = data {
        if elements_count * elements_size == 0 {
            ptr::null()
        } else {
            data.as_ptr()
        }
    } else {
        ptr::null()
    };

    let mut obtained_size: gl::types::GLint = mem::uninitialized();
    let immutable: bool;

    let mutable_storage_flags = if dynamic {
        gl::DYNAMIC_DRAW
    } else {
        gl::STATIC_DRAW
    };

    let immutable_storage_flags = if dynamic && avoid_persistent {
        gl::DYNAMIC_STORAGE_BIT | gl::MAP_READ_BIT | gl::MAP_WRITE_BIT
    } else if dynamic {
        gl::MAP_PERSISTENT_BIT | gl::MAP_READ_BIT | gl::MAP_WRITE_BIT | gl::MAP_COHERENT_BIT
    } else {
        0
    };

    if ctxt.version >= &Version(Api::Gl, 4, 5) || ctxt.extensions.gl_arb_direct_state_access {
        ctxt.gl.NamedBufferStorage(id, buffer_size as gl::types::GLsizei,
                                   data_ptr as *const libc::c_void,
                                   immutable_storage_flags);
        ctxt.gl.GetNamedBufferParameteriv(id, gl::BUFFER_SIZE, &mut obtained_size);
        immutable = !avoid_persistent;

    } else if ctxt.extensions.gl_arb_buffer_storage &&
              ctxt.extensions.gl_ext_direct_state_access
    {
        ctxt.gl.NamedBufferStorageEXT(id, buffer_size as gl::types::GLsizeiptr,
                                      data_ptr as *const libc::c_void,
                                      immutable_storage_flags);
        ctxt.gl.GetNamedBufferParameterivEXT(id, gl::BUFFER_SIZE, &mut obtained_size);
        immutable = !avoid_persistent;

    } else if ctxt.version >= &Version(Api::Gl, 4, 4) ||
              ctxt.extensions.gl_arb_buffer_storage
    {
        let bind = bind_buffer(&mut ctxt, id, ty);
        ctxt.gl.BufferStorage(bind, buffer_size as gl::types::GLsizeiptr,
                              data_ptr as *const libc::c_void,
                              immutable_storage_flags);
        ctxt.gl.GetBufferParameteriv(bind, gl::BUFFER_SIZE, &mut obtained_size);
        immutable = !avoid_persistent;

    } else if ctxt.version >= &Version(Api::Gl, 1, 5) ||
        ctxt.version >= &Version(Api::GlEs, 2, 0)
    {
        let bind = bind_buffer(&mut ctxt, id, ty);
        ctxt.gl.BufferData(bind, buffer_size as gl::types::GLsizeiptr,
                           data_ptr as *const libc::c_void, mutable_storage_flags);
        ctxt.gl.GetBufferParameteriv(bind, gl::BUFFER_SIZE, &mut obtained_size);
        immutable = false;

    } else if ctxt.extensions.gl_arb_vertex_buffer_object {
        let bind = bind_buffer(&mut ctxt, id, ty);
        ctxt.gl.BufferDataARB(bind, buffer_size as gl::types::GLsizeiptr,
                              data_ptr as *const libc::c_void, mutable_storage_flags);
        ctxt.gl.GetBufferParameterivARB(bind, gl::BUFFER_SIZE, &mut obtained_size);
        immutable = false;

    } else {
        unreachable!();
    }

    if buffer_size != obtained_size as usize {
        if ctxt.version >= &Version(Api::Gl, 1, 5) ||
            ctxt.version >= &Version(Api::GlEs, 2, 0)
        {
            ctxt.gl.DeleteBuffers(1, [id].as_ptr());
        } else if ctxt.extensions.gl_arb_vertex_buffer_object {
            ctxt.gl.DeleteBuffersARB(1, [id].as_ptr());
        } else {
            unreachable!();
        }
        
        return Err(BufferCreationError::OutOfMemory);
    }

    let persistent_mapping = if immutable && dynamic && !avoid_persistent {
        let ptr = if ctxt.version >= &Version(Api::Gl, 4, 5) {
            ctxt.gl.MapNamedBufferRange(id, 0, buffer_size as gl::types::GLsizei,
                                        gl::MAP_READ_BIT | gl::MAP_WRITE_BIT |
                                        gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT)

        } else if ctxt.version >= &Version(Api::Gl, 3, 0) ||
                  ctxt.extensions.gl_arb_map_buffer_range
        {
            let bind = bind_buffer(&mut ctxt, id, ty);
            ctxt.gl.MapBufferRange(bind, 0, buffer_size as gl::types::GLsizeiptr,
                                   gl::MAP_READ_BIT | gl::MAP_WRITE_BIT |
                                   gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT)
        } else {
            unreachable!();
        };

        if ptr.is_null() {
            let error = ::get_gl_error(ctxt);
            panic!("glMapBufferRange returned null (error: {:?})", error);
        }

        Some(ptr)

    } else {
        None
    };

    Ok((id, immutable, persistent_mapping))
}

/// Binds a buffer of the given type, and returns the GLenum of the bind point.
unsafe fn bind_buffer(mut ctxt: &mut CommandContext, id: gl::types::GLuint, ty: BufferType)
                      -> gl::types::GLenum
{
    match ty {
        BufferType::ArrayBuffer => {
            if ctxt.state.array_buffer_binding != id {
                ctxt.state.array_buffer_binding = id;

                if ctxt.version >= &Version(Api::Gl, 1, 5) ||
                    ctxt.version >= &Version(Api::GlEs, 2, 0)
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

                if ctxt.version >= &Version(Api::Gl, 1, 5) ||
                    ctxt.version >= &Version(Api::GlEs, 2, 0)
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

                if ctxt.version >= &Version(Api::Gl, 1, 5) ||
                    ctxt.version >= &Version(Api::GlEs, 2, 0)
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

                if ctxt.version >= &Version(Api::Gl, 1, 5) ||
                    ctxt.version >= &Version(Api::GlEs, 2, 0)
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

        BufferType::CopyReadBuffer => {
            if ctxt.state.copy_read_buffer_binding != id {
                ctxt.state.copy_read_buffer_binding = id;

                if ctxt.version >= &Version(Api::Gl, 1, 5) ||
                    ctxt.version >= &Version(Api::GlEs, 2, 0)
                {
                    ctxt.gl.BindBuffer(gl::COPY_READ_BUFFER, id);
                } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                    ctxt.gl.BindBufferARB(gl::COPY_READ_BUFFER, id);    // bind points are the same in the ext
                } else {
                    unreachable!();
                }
            }

            gl::COPY_READ_BUFFER
        },

        BufferType::CopyWriteBuffer => {
            if ctxt.state.copy_write_buffer_binding != id {
                ctxt.state.copy_write_buffer_binding = id;

                if ctxt.version >= &Version(Api::Gl, 1, 5) ||
                    ctxt.version >= &Version(Api::GlEs, 2, 0)
                {
                    ctxt.gl.BindBuffer(gl::COPY_WRITE_BUFFER, id);
                } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                    ctxt.gl.BindBufferARB(gl::COPY_WRITE_BUFFER, id);    // bind points are the same in the ext
                } else {
                    unreachable!();
                }
            }

            gl::COPY_WRITE_BUFFER
        },
    }
}

/// Copies from a buffer to another.
unsafe fn copy_buffer(ctxt: &mut CommandContext, source: gl::types::GLuint,
                      source_offset: usize, dest: gl::types::GLuint, dest_offset: usize,
                      size: usize)
{
    if ctxt.version >= &Version(Api::Gl, 4, 5) || ctxt.extensions.gl_arb_direct_state_access {
        ctxt.gl.CopyNamedBufferSubData(source, dest, source_offset as gl::types::GLintptr,
                                       dest_offset as gl::types::GLintptr,
                                       size as gl::types::GLsizei);

    } else if ctxt.extensions.gl_ext_direct_state_access {
        ctxt.gl.NamedCopyBufferSubDataEXT(source, dest, source_offset as gl::types::GLintptr,
                                          dest_offset as gl::types::GLintptr,
                                          size as gl::types::GLsizeiptr);

    } else if ctxt.version >= &Version(Api::Gl, 3, 1) || ctxt.version >= &Version(Api::GlEs, 3, 0)
           || ctxt.extensions.gl_arb_copy_buffer || ctxt.extensions.gl_nv_copy_buffer
    {
        fn find_bind_point(ctxt: &mut CommandContext, id: gl::types::GLuint)
                           -> Option<gl::types::GLenum>
        {
            if ctxt.state.array_buffer_binding == id {
                Some(gl::ARRAY_BUFFER)
            } else if ctxt.state.pixel_pack_buffer_binding == id {
                Some(gl::PIXEL_PACK_BUFFER)
            } else if ctxt.state.pixel_unpack_buffer_binding == id {
                Some(gl::PIXEL_UNPACK_BUFFER)
            } else if ctxt.state.uniform_buffer_binding == id {
                Some(gl::UNIFORM_BUFFER)
            } else if ctxt.state.copy_read_buffer_binding == id {
                Some(gl::COPY_READ_BUFFER)
            } else if ctxt.state.copy_write_buffer_binding == id {
                Some(gl::COPY_WRITE_BUFFER)
            } else {
                None
            }
        }

        let source_bind_point = match find_bind_point(ctxt, source) {
            Some(p) => p,
            None => {
                // if the source is not binded and the destination is binded to COPY_READ,
                // we bind the source to COPY_WRITE instead, to avoid a state change
                if ctxt.state.copy_read_buffer_binding == dest {
                    bind_buffer(ctxt, source, BufferType::CopyWriteBuffer)
                } else {
                    bind_buffer(ctxt, source, BufferType::CopyReadBuffer)
                }
            }
        };

        let dest_bind_point = match find_bind_point(ctxt, dest) {
            Some(p) => p,
            None => bind_buffer(ctxt, dest, BufferType::CopyWriteBuffer)
        };

        if ctxt.version >= &Version(Api::Gl, 3, 1) || ctxt.version >= &Version(Api::GlEs, 3, 0)
            || ctxt.extensions.gl_arb_copy_buffer
        {
            ctxt.gl.CopyBufferSubData(source_bind_point, dest_bind_point,
                                      source_offset as gl::types::GLintptr,
                                      dest_offset as gl::types::GLintptr,
                                      size as gl::types::GLsizeiptr);
        } else if ctxt.extensions.gl_nv_copy_buffer {
            ctxt.gl.CopyBufferSubDataNV(source_bind_point, dest_bind_point,
                                        source_offset as gl::types::GLintptr,
                                        dest_offset as gl::types::GLintptr,
                                        size as gl::types::GLsizeiptr);
        } else {
            unreachable!();
        }

    } else {
        panic!("Buffers copy are not supported");
    }
}

unsafe fn destroy_buffer(mut ctxt: &mut CommandContext, id: gl::types::GLuint) {
    // FIXME: uncomment this and move it from Buffer's destructor
    //self.context.vertex_array_objects.purge_buffer(&mut ctxt, id);

    if ctxt.state.array_buffer_binding == id {
        ctxt.state.array_buffer_binding = 0;
    }

    if ctxt.state.pixel_pack_buffer_binding == id {
        ctxt.state.pixel_pack_buffer_binding = 0;
    }

    if ctxt.state.pixel_unpack_buffer_binding == id {
        ctxt.state.pixel_unpack_buffer_binding = 0;
    }

    if ctxt.state.uniform_buffer_binding == id {
        ctxt.state.uniform_buffer_binding = 0;
    }

    if ctxt.version >= &Version(Api::Gl, 1, 5) ||
        ctxt.version >= &Version(Api::GlEs, 2, 0)
    {
        ctxt.gl.DeleteBuffers(1, [id].as_ptr());
    } else if ctxt.extensions.gl_arb_vertex_buffer_object {
        ctxt.gl.DeleteBuffersARB(1, [id].as_ptr());
    } else {
        unreachable!();
    }
}
