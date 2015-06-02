use backend::Facade;
use context::CommandContext;
use context::Context;
use version::Version;
use ContextExt;
use gl;
use libc;
use std::{fmt, mem, ptr, slice};
use std::cell::Cell;
use std::rc::Rc;
use std::ops::{Deref, DerefMut, Range};
use GlObject;
use TransformFeedbackSessionExt;

use buffer::{BufferType, BufferCreationError};
use vertex::TransformFeedbackSession;
use vertex_array_object::VertexAttributesSystem;

use version::Api;

/// A buffer in the graphics card's memory.
pub struct Buffer {
    context: Rc<Context>,

    /// OpenGL identifier ; can't be zero.
    id: gl::types::GLuint,

    /// Type of buffer.
    ty: BufferType,

    /// Size in bytes of the buffer.
    size: usize,

    /// A pointer to the persistent mapping of this buffer in memory, if there is one.
    persistent_mapping: Option<*mut libc::c_void>,

    /// If true, then this buffer can only be modified by calls to `glCopyBufferSubData` or through
    /// the persistent mapping.
    immutable: bool,

    /// If true, the buffer was created with the "dynamic" flag.
    dynamic_creation_flag: bool,

    /// True if the buffer is currently mapped with something else than persistent mapping.
    ///
    /// The purpose of this flag is to detect if the user mem::forgets the `Mapping` object.
    mapped: Cell<bool>,
}

/// A mapping of a buffer.
pub struct Mapping<'b, D> {
    mapping: MappingImpl<'b, D>,
}

/// A mapping of a buffer. Private object.
enum MappingImpl<'b, D> {
    PersistentMapping {
        buffer: &'b Buffer,
        offset_bytes: usize,
        data: *mut D,
        len: usize,
    },

    TemporaryBuffer {
        original_buffer: &'b Buffer,
        original_buffer_offset: usize,
        temporary_buffer: gl::types::GLuint,
        temporary_buffer_data: *mut D,
        temporary_buffer_len: usize,
    },

    RegularMapping {
        buffer: &'b mut Buffer,
        data: *mut D,
        len: usize,
    },
}

impl Buffer {
    /// Builds a new buffer containing the given data. The size of the buffer is equal to the
    /// size of the data.
    pub fn new<D, F>(facade: &F, data: &[D], ty: BufferType, dynamic: bool)
                     -> Result<Buffer, BufferCreationError>
                     where D: Send + Copy + 'static, F: Facade
    {
        let mut ctxt = facade.get_context().make_current();

        let size = data.len() * mem::size_of::<D>();

        let (id, immutable, persistent_mapping) = try!(unsafe {
            create_buffer(&mut ctxt, size, Some(&data), ty, dynamic, false)
        });

        Ok(Buffer {
            context: facade.get_context().clone(),
            id: id,
            ty: ty,
            size: size,
            persistent_mapping: persistent_mapping,
            immutable: immutable,
            dynamic_creation_flag: dynamic,
            mapped: Cell::new(false),
        })
    }

    /// Builds a new empty buffer of the given size.
    pub fn empty<F>(facade: &F, ty: BufferType, size: usize, dynamic: bool)
                    -> Result<Buffer, BufferCreationError> where F: Facade
    {
        let mut ctxt = facade.get_context().make_current();

        let (id, immutable, persistent_mapping) = try!(unsafe {
            create_buffer::<()>(&mut ctxt, size, None, ty, dynamic, false)
        });

        Ok(Buffer {
            context: facade.get_context().clone(),
            id: id,
            ty: ty,
            size: size,
            persistent_mapping: persistent_mapping,
            immutable: immutable,
            dynamic_creation_flag: dynamic,
            mapped: Cell::new(false),
        })
    }

    /// Returns the context corresponding to this buffer.
    pub fn get_context(&self) -> &Rc<Context> {
        &self.context
    }

    /// Returns the total size in bytes of this buffer.
    pub fn get_size(&self) -> usize {
        self.size
    }

    /// Returns true if the buffer is persistently mapped in memory.
    pub fn uses_persistent_mapping(&self) -> bool {
        self.persistent_mapping.is_some()
    }

    /// Changes the type of the buffer. Returns `Err` if this is forbidden.
    pub fn set_type(mut self, ty: BufferType) -> Result<Buffer, Buffer> {
        // FIXME: return Err for GLES2
        self.ty = ty;
        Ok(self)
    }

    /// Asserts that the buffer is not mapped and available for operations.
    /// No-op for persistent mapping.
    pub fn assert_unmapped(&self, ctxt: &mut CommandContext) {
        if self.mapped.get() {
            unsafe { unmap_buffer(ctxt, self.id, self.ty) };
            self.mapped.set(false);
        }
    }

    /// Ensures that the buffer isn't used by the transform feedback process.
    pub fn assert_not_transform_feedback(&self, ctxt: &mut CommandContext) {
        TransformFeedbackSession::ensure_buffer_out_of_transform_feedback(ctxt, self.id);
    }

    /// Makes sure that the buffer is binded to a specific bind point.
    ///
    /// The bind point is the value passed to `ty`.
    ///
    /// # Panic
    ///
    /// Panicks if the backend doesn't allow binding this buffer to the specified point.
    pub fn bind(&self, mut ctxt: &mut CommandContext, ty: BufferType) {
        self.assert_unmapped(ctxt);
        unsafe { bind_buffer(ctxt, self.id, ty); }
    }

    /// Makes sure that the buffer is binded to a specific indexed bind point.
    ///
    /// The bind point is the value passed to `ty`.
    ///
    /// # Panic
    ///
    /// - Panicks if `range` is out of range.
    /// - Panicks if the backend doesn't allow binding this buffer to the specified point.
    /// - Panicks if the bind point is not an indexed bind point.
    /// - Panicks if the bind point is over the maximum value.
    pub fn indexed_bind(&self, mut ctxt: &mut CommandContext, ty: BufferType,
                        index: gl::types::GLuint, range: Range<usize>)
    {
        self.assert_unmapped(ctxt);
        unsafe { indexed_bind_buffer(ctxt, self.id, ty, index, range); }
    }

    /// Uploads data in the buffer.
    ///
    /// The data must fit inside the buffer.
    ///
    /// # Panic
    ///
    /// Panics if `offset_bytes` is out of range or the data is too large to fit in the buffer.
    ///
    /// # Unsafety
    ///
    /// If the buffer uses persistent mapping, the caller of this function must handle
    /// synchronization.
    ///
    pub unsafe fn upload<D>(&self, offset_bytes: usize, data: &[D])
                            where D: Copy + Send + 'static
    {
        let to_upload = mem::size_of::<D>() * data.len();
        assert!(offset_bytes + to_upload <= self.size);

        if self.persistent_mapping.is_some() {
            let mut mapping = self.map(offset_bytes, data.len());
            ptr::copy_nonoverlapping(data.as_ptr(), mapping.deref_mut().as_mut_ptr(), data.len());

        } else if self.immutable {
            let mut ctxt = self.context.make_current();

            self.assert_unmapped(&mut ctxt);
            self.assert_not_transform_feedback(&mut ctxt);

            let (tmp_buffer, _, _) = create_buffer(&mut ctxt, to_upload, Some(data),
                                                   BufferType::CopyReadBuffer,
                                                   true, true).unwrap();
            copy_buffer(&mut ctxt, tmp_buffer, 0, self.id, offset_bytes, to_upload);
            destroy_buffer(&mut ctxt, tmp_buffer);

        } else {
            assert!(offset_bytes < self.size);

            let invalidate_all = offset_bytes == 0 && to_upload == self.size;

            let mut ctxt = self.context.make_current();

            self.assert_unmapped(&mut ctxt);
            self.assert_not_transform_feedback(&mut ctxt);

            if invalidate_all && (ctxt.version >= &Version(Api::Gl, 4, 3) ||
                ctxt.extensions.gl_arb_invalidate_subdata)
            {
                ctxt.gl.InvalidateBufferData(self.id);
            }

            if ctxt.version >= &Version(Api::Gl, 4, 5) {
                ctxt.gl.NamedBufferSubData(self.id, offset_bytes as gl::types::GLintptr,
                                           to_upload as gl::types::GLsizei,
                                           data.as_ptr() as *const libc::c_void)

            } else if ctxt.extensions.gl_ext_direct_state_access {
                ctxt.gl.NamedBufferSubDataEXT(self.id, offset_bytes as gl::types::GLintptr,
                                              to_upload as gl::types::GLsizeiptr,
                                              data.as_ptr() as *const libc::c_void)

            } else if ctxt.version >= &Version(Api::Gl, 1, 5) ||
                ctxt.version >= &Version(Api::GlEs, 2, 0)
            {
                let bind = bind_buffer(&mut ctxt, self.id, self.ty);
                ctxt.gl.BufferSubData(bind, offset_bytes as gl::types::GLintptr,
                                      to_upload as gl::types::GLsizeiptr,
                                      data.as_ptr() as *const libc::c_void);

            } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                let bind = bind_buffer(&mut ctxt, self.id, self.ty);
                ctxt.gl.BufferSubDataARB(bind, offset_bytes as gl::types::GLintptr,
                                         to_upload as gl::types::GLsizeiptr,
                                         data.as_ptr() as *const libc::c_void);

            } else {
                unreachable!();
            }
        }
    }

    /// Invalidates the content of the buffer. The data becomes undefined.
    ///
    /// `offset` and `size` are both in bytes.
    ///
    /// # Panic
    ///
    /// Panics if out of range.
    ///
    pub fn invalidate(&self, offset: usize, size: usize) {
        assert!(offset + size <= self.size);

        let is_whole_buffer = offset == 0 && size == self.size;

        let mut ctxt = self.context.make_current();
        self.assert_unmapped(&mut ctxt);
        self.assert_not_transform_feedback(&mut ctxt);

        if ctxt.version >= &Version(Api::Gl, 4, 3) || ctxt.extensions.gl_arb_invalidate_subdata {
            if is_whole_buffer {
                unsafe { ctxt.gl.InvalidateBufferData(self.id) };
            } else {
                unsafe { ctxt.gl.InvalidateBufferSubData(self.id, offset as gl::types::GLintptr,
                                                         size as gl::types::GLsizeiptr) };
            }

        } else if !self.immutable {
            if is_whole_buffer {
                let flags = if self.dynamic_creation_flag {
                    gl::DYNAMIC_DRAW
                } else {
                    gl::STATIC_DRAW
                };

                if ctxt.version >= &Version(Api::Gl, 1, 5) ||
                    ctxt.version >= &Version(Api::GlEs, 2, 0)
                {
                    unsafe {
                        let bind = bind_buffer(&mut ctxt, self.id, self.ty);
                        ctxt.gl.BufferData(bind, size as gl::types::GLsizeiptr,
                                           ptr::null(), flags);
                    }

                } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                    unsafe {
                        let bind = bind_buffer(&mut ctxt, self.id, self.ty);
                        ctxt.gl.BufferDataARB(bind, size as gl::types::GLsizeiptr,
                                              ptr::null(), flags);
                    }

                } else {
                    unreachable!();
                }
            }
        }
    }

    /// Returns a mapping in memory of the content of the buffer.
    ///
    /// There are two possibilities:
    ///
    ///  - If the buffer uses persistent mapping, it will simply return a wrapper around the
    ///    pointer to the existing mapping.
    ///  - If the buffer doesn't use persistent mapping, it will create a temporary buffer which
    ///    will be mapped. After the mapping is released, the temporary buffer will be copied
    ///    to the real buffer.
    ///
    /// In the second case, the changes will only be written when the mapping is released.
    /// Therefore this API is error-prone and shouldn't be exposed directly to the user. Instead
    /// `map` public functions should take a `&mut self` instead of a `&self` to prevent users
    /// from manipulating the buffer while it is "mapped".
    ///
    /// Contrary to `map_mut`, this function only requires a `&self` and can thus be used even
    /// with a `Rc<Buffer>` for example.
    ///
    /// # Unsafety
    ///
    /// If the buffer uses persistent mapping, the caller of this function must handle
    /// synchronization.
    ///
    pub unsafe fn map<D>(&self, offset_bytes: usize, elements: usize)
                         -> Mapping<D> where D: Copy + Send + 'static
    {
        assert!(offset_bytes % mem::size_of::<D>() == 0);
        assert!(offset_bytes <= self.size);
        assert!(offset_bytes + elements * mem::size_of::<D>() <= self.size);

        if let Some(existing_mapping) = self.persistent_mapping.clone() {
            Mapping {
                mapping: MappingImpl::PersistentMapping {
                    buffer: self,
                    offset_bytes: offset_bytes,
                    data: (existing_mapping as *mut u8).offset(offset_bytes as isize) as *mut D,
                    len: elements,
                },
            }

        } else {
            let size_bytes = elements * mem::size_of::<D>();

            // we have to construct a temporary buffer that we will map in memory
            // then after the Mapping is destroyed, we will copy from the temporary buffer to the
            // real one
            let temporary_buffer = {
                let mut ctxt = self.context.make_current();
                let (temporary_buffer, _, _) = create_buffer::<D>(&mut ctxt, size_bytes,
                                                                  None, BufferType::CopyWriteBuffer,
                                                                  true, true).unwrap();
                temporary_buffer
            };

            let ptr = {
                let mut ctxt = self.context.make_current();

                self.assert_unmapped(&mut ctxt);
                self.assert_not_transform_feedback(&mut ctxt);
                copy_buffer(&mut ctxt, self.id, offset_bytes, temporary_buffer, 0, size_bytes);
                map_buffer(&mut ctxt, temporary_buffer, self.ty, 0 .. size_bytes, true, true)
                                    .expect("Buffer mapping is not supported by the backend")
            };

            Mapping {
                mapping: MappingImpl::TemporaryBuffer {
                    original_buffer: self,
                    original_buffer_offset: offset_bytes,
                    temporary_buffer: temporary_buffer,
                    temporary_buffer_data: ptr as *mut D,
                    temporary_buffer_len: elements,
                }
            }
        }
    }

    /// Returns a mapping in memory of the content of the buffer.
    ///
    /// There are two possibilities:
    ///
    ///  - If the buffer uses persistent mapping, it will simply return a wrapper around the
    ///    pointer to the existing mapping.
    ///  - If the buffer doesn't use persistent mapping, it will map the buffer.
    ///
    /// Contrary to `map`, this function requires a `&mut self`. It can only be used if you
    /// have exclusive access to the buffer.
    ///
    /// # Unsafety
    ///
    /// If the buffer uses persistent mapping, the caller of this function must handle
    /// synchronization.
    ///
    pub unsafe fn map_mut<D>(&mut self, offset_bytes: usize, elements: usize)
                             -> Mapping<D> where D: Copy + Send + 'static
    {
        if self.persistent_mapping.is_some() || self.immutable {
            self.map(offset_bytes, elements)

        } else {
            assert!(offset_bytes % mem::size_of::<D>() == 0);
            assert!(offset_bytes <= self.size);
            assert!(offset_bytes + elements * mem::size_of::<D>() <= self.size);

            let size_bytes = elements * mem::size_of::<D>();

            let ptr = {
                let mut ctxt = self.context.make_current();
                self.assert_unmapped(&mut ctxt);
                self.assert_not_transform_feedback(&mut ctxt);
                self.mapped.set(true);
                map_buffer(&mut ctxt, self.id, self.ty,
                           offset_bytes .. offset_bytes + size_bytes, true, true)
                             .expect("Buffer mapping is not supported by the backend")
            };

            Mapping {
                mapping: MappingImpl::RegularMapping {
                    buffer: self,
                    data: ptr as *mut D,
                    len: elements,
                }
            }
        }
    }

    /// Reads the content of the buffer. Returns `None` if this operation is not supported.
    ///
    /// # Panic
    ///
    /// Panicks if out of range.
    ///
    /// # Unsafety
    ///
    /// If the buffer uses persistent mapping, the caller of this function must handle
    /// synchronization.
    ///
    pub unsafe fn read_if_supported<D>(&self, offset_bytes: usize, output: &mut [D])
                                       -> Result<(), ()> where D: Copy + Send + 'static
    {
        assert!(offset_bytes <= self.size);
        assert!(offset_bytes + output.len() * mem::size_of::<D>() <= self.size);

        if self.persistent_mapping.is_some() {
            let mapping = self.map(offset_bytes, output.len());
            ptr::copy_nonoverlapping(mapping.as_ptr(), output.as_mut_ptr(),
                                     output.len() * mem::size_of::<D>());
            Ok(())

        } else {
            let mut ctxt = self.context.make_current();
            self.assert_unmapped(&mut ctxt);

            if ctxt.version >= &Version(Api::Gl, 4, 5) {
                ctxt.gl.GetNamedBufferSubData(self.id, offset_bytes as gl::types::GLintptr,
                                              (output.len() * mem::size_of::<D>())
                                              as gl::types::GLsizei, output.as_mut_ptr()
                                              as *mut libc::c_void);

            } else if ctxt.version >= &Version(Api::Gl, 1, 5) {
                let bind = bind_buffer(&mut ctxt, self.id, self.ty);
                ctxt.gl.GetBufferSubData(bind, offset_bytes as gl::types::GLintptr,
                                         (output.len() * mem::size_of::<D>())
                                         as gl::types::GLsizeiptr, output.as_mut_ptr()
                                         as *mut libc::c_void);

            } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                let bind = bind_buffer(&mut ctxt, self.id, self.ty);
                ctxt.gl.GetBufferSubDataARB(bind, offset_bytes as gl::types::GLintptr,
                                            (output.len() * mem::size_of::<D>())
                                            as gl::types::GLsizeiptr, output.as_mut_ptr()
                                            as *mut libc::c_void);

            } else if ctxt.version >= &Version(Api::GlEs, 1, 0) {
                return Err(());

            } else {
                unreachable!()
            }

            Ok(())
        }
    }
}

impl fmt::Debug for Buffer {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "Buffer #{} (size: {} bytes)", self.id, self.size)
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            let mut ctxt = self.context.make_current();
            self.assert_unmapped(&mut ctxt);
            self.assert_not_transform_feedback(&mut ctxt);
            VertexAttributesSystem::purge_buffer(&mut ctxt, self.id);
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
        match self.mapping {
            MappingImpl::PersistentMapping { buffer, offset_bytes, data, len } => {
                let mut ctxt = buffer.context.make_current();
                unsafe {
                    flush_range(&mut ctxt, buffer.id, buffer.ty,
                                offset_bytes .. offset_bytes + len * mem::size_of::<D>());
                }
            },

            MappingImpl::TemporaryBuffer { original_buffer, original_buffer_offset,
                                           temporary_buffer, temporary_buffer_data,
                                           temporary_buffer_len } =>
            {
                let mut ctxt = original_buffer.context.make_current();

                unsafe {
                    flush_range(&mut ctxt, temporary_buffer, original_buffer.ty,
                                0 .. temporary_buffer_len * mem::size_of::<D>());
                    unmap_buffer(&mut ctxt, temporary_buffer, original_buffer.ty);
                    copy_buffer(&mut ctxt, temporary_buffer, 0, original_buffer.id,
                                original_buffer_offset, temporary_buffer_len * mem::size_of::<D>());

                    destroy_buffer(&mut ctxt, temporary_buffer);
                }
            },

            MappingImpl::RegularMapping { ref mut buffer, data, len } => {
                let mut ctxt = buffer.context.make_current();

                unsafe {
                    flush_range(&mut ctxt, buffer.id, buffer.ty, 0 .. len * mem::size_of::<D>());
                    unmap_buffer(&mut ctxt, buffer.id, buffer.ty);
                }

                buffer.mapped.set(false);
            },
        }
    }
}

impl<'a, D> Deref for Mapping<'a, D> {
    type Target = [D];
    fn deref(&self) -> &[D] {
        match self.mapping {
            MappingImpl::PersistentMapping { data, len, .. } => {
                unsafe { slice::from_raw_parts_mut(data, len) }
            },

            MappingImpl::TemporaryBuffer { temporary_buffer_data, temporary_buffer_len, .. } => {
                unsafe { slice::from_raw_parts_mut(temporary_buffer_data, temporary_buffer_len) }
            },

            MappingImpl::RegularMapping { data, len, .. } => {
                unsafe { slice::from_raw_parts_mut(data, len) }
            },
        }
    }
}

impl<'a, D> DerefMut for Mapping<'a, D> {
    fn deref_mut(&mut self) -> &mut [D] {
        match self.mapping {
            MappingImpl::PersistentMapping { data, len, .. } => {
                unsafe { slice::from_raw_parts_mut(data, len) }
            },

            MappingImpl::TemporaryBuffer { temporary_buffer_data, temporary_buffer_len, .. } => {
                unsafe { slice::from_raw_parts_mut(temporary_buffer_data, temporary_buffer_len) }
            },

            MappingImpl::RegularMapping { data, len, .. } => {
                unsafe { slice::from_raw_parts_mut(data, len) }
            },
        }
    }
}

/// Creates a new buffer.
///
/// # Panic
///
/// Panics if `data.len() * size_of::<D>() < size` or if `size % size_of::<D>() != 0`.
unsafe fn create_buffer<D>(mut ctxt: &mut CommandContext, size: usize, data: Option<&[D]>,
                           ty: BufferType, dynamic: bool, avoid_persistent: bool)
                           -> Result<(gl::types::GLuint, bool, Option<*mut libc::c_void>),
                                     BufferCreationError>
                           where D: Send + Copy + 'static
{
    if !is_buffer_type_supported(ctxt, ty) {
        return Err(BufferCreationError::BufferTypeNotSupported);
    }

    if let Some(ref data) = data {
        assert!(data.len() * mem::size_of::<D>() >= size);
        assert!(size % mem::size_of::<D>() == 0);
    }

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

    let data_ptr = if let Some(data) = data {
        if size == 0 {
            ptr::null()
        } else {
            data.as_ptr()
        }
    } else {
        ptr::null()
    };

    let size = match size {
        0 => 1,     // use size 1 instead of 0, or nvidia drivers complain
        a => a
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
        gl::MAP_PERSISTENT_BIT | gl::MAP_READ_BIT | gl::MAP_WRITE_BIT
    } else {
        0
    };

    if ctxt.version >= &Version(Api::Gl, 4, 5) || ctxt.extensions.gl_arb_direct_state_access {
        ctxt.gl.NamedBufferStorage(id, size as gl::types::GLsizei,
                                   data_ptr as *const libc::c_void,
                                   immutable_storage_flags);
        ctxt.gl.GetNamedBufferParameteriv(id, gl::BUFFER_SIZE, &mut obtained_size);
        immutable = !avoid_persistent;

    } else if ctxt.extensions.gl_arb_buffer_storage &&
              ctxt.extensions.gl_ext_direct_state_access
    {
        ctxt.gl.NamedBufferStorageEXT(id, size as gl::types::GLsizeiptr,
                                      data_ptr as *const libc::c_void,
                                      immutable_storage_flags);
        ctxt.gl.GetNamedBufferParameterivEXT(id, gl::BUFFER_SIZE, &mut obtained_size);
        immutable = !avoid_persistent;

    } else if ctxt.version >= &Version(Api::Gl, 4, 4) ||
              ctxt.extensions.gl_arb_buffer_storage
    {
        let bind = bind_buffer(&mut ctxt, id, ty);
        ctxt.gl.BufferStorage(bind, size as gl::types::GLsizeiptr,
                              data_ptr as *const libc::c_void,
                              immutable_storage_flags);
        ctxt.gl.GetBufferParameteriv(bind, gl::BUFFER_SIZE, &mut obtained_size);
        immutable = !avoid_persistent;

    } else if ctxt.version >= &Version(Api::Gl, 1, 5) ||
        ctxt.version >= &Version(Api::GlEs, 2, 0)
    {
        let bind = bind_buffer(&mut ctxt, id, ty);
        ctxt.gl.BufferData(bind, size as gl::types::GLsizeiptr,
                           data_ptr as *const libc::c_void, mutable_storage_flags);
        ctxt.gl.GetBufferParameteriv(bind, gl::BUFFER_SIZE, &mut obtained_size);
        immutable = false;

    } else if ctxt.extensions.gl_arb_vertex_buffer_object {
        let bind = bind_buffer(&mut ctxt, id, ty);
        ctxt.gl.BufferDataARB(bind, size as gl::types::GLsizeiptr,
                              data_ptr as *const libc::c_void, mutable_storage_flags);
        ctxt.gl.GetBufferParameterivARB(bind, gl::BUFFER_SIZE, &mut obtained_size);
        immutable = false;

    } else {
        unreachable!();
    }

    if size != obtained_size as usize {
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
            ctxt.gl.MapNamedBufferRange(id, 0, size as gl::types::GLsizei,
                                        gl::MAP_READ_BIT | gl::MAP_WRITE_BIT |
                                        gl::MAP_PERSISTENT_BIT | gl::MAP_FLUSH_EXPLICIT_BIT)

        } else if ctxt.version >= &Version(Api::Gl, 3, 0) ||
                  ctxt.extensions.gl_arb_map_buffer_range
        {
            let bind = bind_buffer(&mut ctxt, id, ty);
            ctxt.gl.MapBufferRange(bind, 0, size as gl::types::GLsizeiptr,
                                   gl::MAP_READ_BIT | gl::MAP_WRITE_BIT |
                                   gl::MAP_PERSISTENT_BIT | gl::MAP_FLUSH_EXPLICIT_BIT)
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

/// Returns true if a given buffer type is supported on a platform.
fn is_buffer_type_supported(ctxt: &mut CommandContext, ty: BufferType) -> bool {
    match ty {
        // glium fails to initialize if they are not supported
        BufferType::ArrayBuffer | BufferType::ElementArrayBuffer => true,

        BufferType::PixelPackBuffer | BufferType::PixelUnpackBuffer => {
            ctxt.version >= &Version(Api::Gl, 2, 1) || ctxt.version >= &Version(Api::GlEs, 3, 0) ||
            ctxt.extensions.gl_arb_pixel_buffer_object || ctxt.extensions.gl_nv_pixel_buffer_object
        },

        BufferType::UniformBuffer => {
            ctxt.version >= &Version(Api::Gl, 3, 1) || ctxt.version >= &Version(Api::GlEs, 3, 0) ||
            ctxt.extensions.gl_arb_uniform_buffer_object
        },

        BufferType::CopyReadBuffer => {
            ctxt.version >= &Version(Api::Gl, 3, 1) || ctxt.extensions.gl_arb_copy_buffer ||
            ctxt.version >= &Version(Api::GlEs, 3, 0) || ctxt.extensions.gl_nv_copy_buffer
        },

        BufferType::CopyWriteBuffer => {
            ctxt.version >= &Version(Api::Gl, 3, 0) || ctxt.extensions.gl_arb_copy_buffer ||
            ctxt.version >= &Version(Api::GlEs, 3, 0) || ctxt.extensions.gl_nv_copy_buffer
        },

        BufferType::DrawIndirectBuffer => {
            // TODO: draw indirect buffers are actually supported in OpenGL 4.0 or
            //       with GL_ARB_draw_indirect, but restricting to multidraw is more convenient
            //       for index/multidraw.rs
            ctxt.version >= &Version(Api::Gl, 4, 3) || ctxt.extensions.gl_arb_multi_draw_indirect ||
            ctxt.extensions.gl_ext_multi_draw_indirect
        },

        _ => false,     // FIXME: 
    }
}

/// Binds a buffer of the given type, and returns the GLenum of the bind point.
///
/// ## Unsafety
///
/// Assumes that the type of buffer is supported by the backend.
unsafe fn bind_buffer(mut ctxt: &mut CommandContext, id: gl::types::GLuint, ty: BufferType)
                      -> gl::types::GLenum
{
    macro_rules! check {
        ($ctxt:expr, $input_id:expr, $input_ty:expr, $check:ident, $state_var:ident) => (
            if $input_ty == BufferType::$check {
                let en = $input_ty.to_glenum();

                if ctxt.state.$state_var != $input_id {
                    ctxt.state.$state_var = $input_id;

                    if ctxt.version >= &Version(Api::Gl, 1, 5) ||
                       ctxt.version >= &Version(Api::GlEs, 2, 0)
                    {
                        ctxt.gl.BindBuffer(en, id);
                    } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                        ctxt.gl.BindBufferARB(en, id);
                    } else {
                        unreachable!();
                    }
                }

                return en;
            }
        );
    }

    check!(ctxt, id, ty, ArrayBuffer, array_buffer_binding);
    check!(ctxt, id, ty, PixelPackBuffer, pixel_pack_buffer_binding);
    check!(ctxt, id, ty, PixelUnpackBuffer, pixel_unpack_buffer_binding);
    check!(ctxt, id, ty, UniformBuffer, uniform_buffer_binding);
    check!(ctxt, id, ty, CopyReadBuffer, copy_read_buffer_binding);
    check!(ctxt, id, ty, CopyWriteBuffer, copy_write_buffer_binding);
    check!(ctxt, id, ty, DispatchIndirectBuffer, dispatch_indirect_buffer_binding);
    check!(ctxt, id, ty, DrawIndirectBuffer, draw_indirect_buffer_binding);
    check!(ctxt, id, ty, QueryBuffer, query_buffer_binding);
    check!(ctxt, id, ty, TextureBuffer, texture_buffer_binding);
    check!(ctxt, id, ty, AtomicCounterBuffer, atomic_counter_buffer_binding);
    check!(ctxt, id, ty, ShaderStorageBuffer, shader_storage_buffer_binding);

    if ty == BufferType::ElementArrayBuffer {
        // TODO: the state if the current buffer is not cached
        VertexAttributesSystem::hijack_current_element_array_buffer(ctxt);

        if ctxt.version >= &Version(Api::Gl, 1, 5) ||
           ctxt.version >= &Version(Api::GlEs, 2, 0)
        {
            ctxt.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, id);
        } else if ctxt.extensions.gl_arb_vertex_buffer_object {
            ctxt.gl.BindBufferARB(gl::ELEMENT_ARRAY_BUFFER, id);
        } else {
            unreachable!();
        }

        return gl::ELEMENT_ARRAY_BUFFER;
    }

    if ty == BufferType::TransformFeedbackBuffer {
        debug_assert!(ctxt.capabilities.max_indexed_transform_feedback_buffer >= 1);

        // FIXME: pause transform feedback if it is active

        if ctxt.state.indexed_transform_feedback_buffer_bindings[0].buffer != id {
            ctxt.state.indexed_transform_feedback_buffer_bindings[0].buffer = id;

            if ctxt.version >= &Version(Api::Gl, 1, 5) ||
               ctxt.version >= &Version(Api::GlEs, 2, 0)
            {
                ctxt.gl.BindBuffer(gl::TRANSFORM_FEEDBACK_BUFFER, id);
            } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                ctxt.gl.BindBufferARB(gl::TRANSFORM_FEEDBACK_BUFFER, id);
            } else {
                unreachable!();
            }
        }

        return gl::TRANSFORM_FEEDBACK_BUFFER;
    }

    unreachable!();
}

/// Binds a buffer of the given type to an indexed bind point.
///
/// # Panic
///
/// Panicks if the buffer type is not indexed.
///
/// # Unsafety
///
/// Assumes that the type of buffer is supported by the backend.
unsafe fn indexed_bind_buffer(mut ctxt: &mut CommandContext, id: gl::types::GLuint, ty: BufferType,
                              index: gl::types::GLuint, range: Range<usize>)
{
    let offset = range.start as gl::types::GLintptr;
    let size = (range.end - range.start) as gl::types::GLsizeiptr;

    macro_rules! check {
        ($ctxt:expr, $input_id:expr, $input_ty:expr, $input_index:expr, $check:ident,
         $state_var:ident, $max:ident) =>
        (
            if $input_ty == BufferType::$check {
                let en = $input_ty.to_glenum();

                if $input_index >= ctxt.capabilities.$max as gl::types::GLuint {
                    panic!("Indexed buffer out of range");
                }

                if ctxt.state.$state_var.len() <= $input_index as usize {
                    for _ in (0 .. 1 + ctxt.state.$state_var.len() - $input_index as usize) {
                        ctxt.state.$state_var.push(Default::default());
                    }
                }

                let unit = &mut ctxt.state.$state_var[$input_index as usize];
                if unit.buffer != $input_id || unit.offset != offset || unit.size != size {
                    unit.buffer = $input_id;
                    unit.offset = offset;
                    unit.size = size;

                    if ctxt.version >= &Version(Api::Gl, 3, 0) ||
                       ctxt.version >= &Version(Api::GlEs, 3, 0)
                    {
                        ctxt.gl.BindBufferRange(en, $input_index, id, offset, size);
                    } else if ctxt.extensions.gl_ext_transform_feedback {
                        ctxt.gl.BindBufferRangeEXT(en, $input_index, id, offset, size);
                    } else {
                        panic!("The backend doesn't support indexed buffer bind points");
                    }
                }

                return;
            }
        );
    }

    check!(ctxt, id, ty, index, UniformBuffer, indexed_uniform_buffer_bindings,
           max_indexed_uniform_buffer);
    check!(ctxt, id, ty, index, TransformFeedbackBuffer, indexed_transform_feedback_buffer_bindings,
           max_indexed_transform_feedback_buffer);
    check!(ctxt, id, ty, index, AtomicCounterBuffer, indexed_atomic_counter_buffer_bindings,
           max_indexed_atomic_counter_buffer);
    check!(ctxt, id, ty, index, ShaderStorageBuffer, indexed_shader_storage_buffer_bindings,
           max_indexed_shader_storage_buffer);

    panic!();
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
        // TODO: use proper error result
        panic!("Buffers copy are not supported");
    }
}

/// Destroys a buffer.
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

    if ctxt.state.copy_read_buffer_binding == id {
        ctxt.state.copy_read_buffer_binding = 0;
    }

    if ctxt.state.copy_write_buffer_binding == id {
        ctxt.state.copy_write_buffer_binding = 0;
    }

    if ctxt.state.dispatch_indirect_buffer_binding == id {
        ctxt.state.dispatch_indirect_buffer_binding = 0;
    }

    if ctxt.state.draw_indirect_buffer_binding == id {
        ctxt.state.draw_indirect_buffer_binding = 0;
    }

    if ctxt.state.query_buffer_binding == id {
        ctxt.state.query_buffer_binding = 0;
    }

    if ctxt.state.texture_buffer_binding == id {
        ctxt.state.texture_buffer_binding = 0;
    }

    if ctxt.state.atomic_counter_buffer_binding == id {
        ctxt.state.atomic_counter_buffer_binding = 0;
    }

    if ctxt.state.shader_storage_buffer_binding == id {
        ctxt.state.shader_storage_buffer_binding = 0;
    }

    for point in ctxt.state.indexed_atomic_counter_buffer_bindings.iter_mut() {
        if point.buffer == id {
            point.buffer = 0;
        }
    }

    for point in ctxt.state.indexed_shader_storage_buffer_bindings.iter_mut() {
        if point.buffer == id {
            point.buffer = 0;
        }
    }

    for point in ctxt.state.indexed_uniform_buffer_bindings.iter_mut() {
        if point.buffer == id {
            point.buffer = 0;
        }
    }

    for point in ctxt.state.indexed_transform_feedback_buffer_bindings.iter_mut() {
        // FIXME: end transform feedback if it is active
        if point.buffer == id {
            point.buffer = 0;
        }
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

/// Flushes a range of a mapped buffer.
unsafe fn flush_range(mut ctxt: &mut CommandContext, id: gl::types::GLuint, ty: BufferType,
                      range: Range<usize>)
{
    if ctxt.version >= &Version(Api::Gl, 4, 5) || ctxt.extensions.gl_arb_direct_state_access {
        ctxt.gl.FlushMappedNamedBufferRange(id, range.start as gl::types::GLintptr,
                                            (range.end - range.start) as gl::types::GLsizei);

    } else if ctxt.version >= &Version(Api::Gl, 3, 0) ||
              ctxt.version >= &Version(Api::GlEs, 3, 0) ||
              ctxt.extensions.gl_arb_map_buffer_range
    {
        let bind = bind_buffer(&mut ctxt, id, ty);
        ctxt.gl.FlushMappedBufferRange(bind, range.start as gl::types::GLintptr,
                                       (range.end - range.start) as gl::types::GLsizeiptr)

    } else {
        unreachable!();
    }
}

/// Maps a range of a buffer.
///
/// *Warning*: always passes `GL_MAP_FLUSH_EXPLICIT_BIT`.
unsafe fn map_buffer(mut ctxt: &mut CommandContext, id: gl::types::GLuint, ty: BufferType,
                     range: Range<usize>, read: bool, write: bool) -> Option<*const libc::c_void>
{
    let flags = match (read, write) {
        (true, true) => gl::MAP_FLUSH_EXPLICIT_BIT | gl::MAP_READ_BIT | gl::MAP_WRITE_BIT,
        (true, false) => gl::MAP_FLUSH_EXPLICIT_BIT | gl::MAP_READ_BIT,
        (false, true) => gl::MAP_FLUSH_EXPLICIT_BIT | gl::MAP_WRITE_BIT,
        (false, false) => gl::MAP_FLUSH_EXPLICIT_BIT
    };

    if ctxt.version >= &Version(Api::Gl, 4, 5) {
        Some(ctxt.gl.MapNamedBufferRange(id, range.start as gl::types::GLintptr,
                                         (range.end - range.start) as gl::types::GLsizei,
                                         flags))

    } else if ctxt.version >= &Version(Api::Gl, 3, 0) ||
        ctxt.version >= &Version(Api::GlEs, 3, 0) ||
        ctxt.extensions.gl_arb_map_buffer_range
    {
        let bind = bind_buffer(&mut ctxt, id, ty);
        Some(ctxt.gl.MapBufferRange(bind, range.start as gl::types::GLintptr,
                                    (range.end - range.start) as gl::types::GLsizeiptr,
                                    flags))

    } else {
        None       // FIXME: 
    }
}

/// Unmaps a previously-mapped buffer.
///
/// # Safety
///
/// Assumes that the buffer exists, that it is of the right type, and that it is already mapped.
unsafe fn unmap_buffer(mut ctxt: &mut CommandContext, id: gl::types::GLuint, ty: BufferType) {
    if ctxt.version >= &Version(Api::Gl, 4, 5) {
        ctxt.gl.UnmapNamedBuffer(id);

    } else if ctxt.version >= &Version(Api::Gl, 1, 5) ||
              ctxt.version >= &Version(Api::GlEs, 3, 0)
    {
        let bind = bind_buffer(&mut ctxt, id, ty);
        ctxt.gl.UnmapBuffer(bind);

    } else if ctxt.extensions.gl_arb_vertex_buffer_object {
        let bind = bind_buffer(&mut ctxt, id, ty);
        ctxt.gl.UnmapBufferARB(bind);

    } else {
        unreachable!();
    }
}
