use backend::Facade;
use context::CommandContext;
use context::Context;
use version::Version;
use CapabilitiesSource;
use ContextExt;
use gl;
use std::os::raw;
use std::error::Error;
use std::{fmt, mem, ptr};
use std::cell::Cell;
use std::rc::Rc;
use std::ops::{Deref, DerefMut, Range};
use GlObject;
use TransformFeedbackSessionExt;

use buffer::{Content, BufferType, BufferMode, BufferCreationError, CopyError};
use vertex::TransformFeedbackSession;
use vertex_array_object::VertexAttributesSystem;

use version::Api;

use buffer::raw::*;

/// Error that can happen when reading from a buffer.
#[derive(Debug, Copy, Clone)]
pub enum ReadError {
    /// The backend doesn't support reading from a buffer.
    NotSupported,

    /// The context has been lost. Reading from the buffer would return garbage data.
    ContextLost,
}

impl fmt::Display for ReadError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

impl Error for ReadError {
    fn description(&self) -> &str {
        use self::ReadError::*;
        match *self {
            NotSupported => "The backend doesn't support reading from a buffer",
            ContextLost => "The context has been lost. Reading from the buffer would return garbage data",
        }
    }
}

/// A buffer in the graphics card's memory.
pub struct Alloc {
    context: Rc<Context>,

    /// OpenGL identifier ; can't be zero.
    id: gl::types::GLuint,

    /// Type of buffer.
    ty: BufferType,

    /// Size in bytes of the buffer.
    size: usize,

    /// A pointer to the persistent mapping of this buffer in memory, if there is one.
    persistent_mapping: Option<*mut raw::c_void>,

    /// If true, then this buffer can only be modified by calls to `glCopyBufferSubData` or through
    /// the persistent mapping.
    immutable: bool,

    /// If true, the buffer was created with the "dynamic" flag.
    creation_mode: BufferMode,

    /// If true, the buffer was created with `glBufferStorage`.
    created_with_buffer_storage: bool,

    /// True if the buffer is currently mapped with something else than persistent mapping.
    ///
    /// The purpose of this flag is to detect if the user mem::forgets the `Mapping` object.
    mapped: Cell<bool>,

    /// ID of the draw call where the buffer was last written as an SSBO.
    latest_shader_write: Cell<u64>,
}

impl Alloc {
    /// Builds a new buffer containing the given data. The size of the buffer is equal to the
    /// size of the data.
    pub fn new<D: ?Sized, F>(facade: &F, data: &D, ty: BufferType, mode: BufferMode)
                             -> Result<Alloc, BufferCreationError>
                             where D: Content, F: Facade
    {
        let mut ctxt = facade.get_context().make_current();

        let size = mem::size_of_val(data);

        let (id, immutable, created_with_buffer_storage, persistent_mapping) = try!(unsafe {
            create_buffer(&mut ctxt, size, Some(data), ty, mode)
        });

        Ok(Alloc {
            context: facade.get_context().clone(),
            id: id,
            ty: ty,
            size: size,
            persistent_mapping: persistent_mapping,
            immutable: immutable,
            created_with_buffer_storage: created_with_buffer_storage,
            creation_mode: mode,
            mapped: Cell::new(false),
            latest_shader_write: Cell::new(0),
        })
    }

    /// Builds a new empty buffer of the given size.
    pub fn empty<F>(facade: &F, ty: BufferType, size: usize, mode: BufferMode)
                    -> Result<Alloc, BufferCreationError> where F: Facade
    {
        let mut ctxt = facade.get_context().make_current();

        let (id, immutable, created_with_buffer_storage, persistent_mapping) = try!(unsafe {
            create_buffer::<()>(&mut ctxt, size, None, ty, mode)
        });

        Ok(Alloc {
            context: facade.get_context().clone(),
            id: id,
            ty: ty,
            size: size,
            persistent_mapping: persistent_mapping,
            immutable: immutable,
            created_with_buffer_storage: created_with_buffer_storage,
            creation_mode: mode,
            mapped: Cell::new(false),
            latest_shader_write: Cell::new(0),
        })
    }

    /// Returns the context corresponding to this buffer.
    #[inline]
    pub fn get_context(&self) -> &Rc<Context> {
        &self.context
    }

    /// Returns the total size in bytes of this buffer.
    #[inline]
    pub fn get_size(&self) -> usize {
        self.size
    }

    /// Returns true if the buffer is persistently mapped in memory.
    #[inline]
    pub fn uses_persistent_mapping(&self) -> bool {
        self.persistent_mapping.is_some()
    }

    /// Changes the type of the buffer. Returns `Err` if this is forbidden.
    pub fn set_type(mut self, ty: BufferType) -> Result<Alloc, Alloc> {
        // FIXME: return Err for GLES2
        self.ty = ty;
        Ok(self)
    }

    /// Asserts that the buffer is not mapped and available for operations.
    /// No-op for persistent mapping.
    fn assert_unmapped(&self, ctxt: &mut CommandContext) {
        if self.mapped.get() {
            unsafe { unmap_buffer(ctxt, self.id, self.ty) };
            self.mapped.set(false);
        }
    }

    /// Ensures that the buffer isn't used by the transform feedback process.
    #[inline]
    fn assert_not_transform_feedback(&self, ctxt: &mut CommandContext) {
        TransformFeedbackSession::ensure_buffer_out_of_transform_feedback(ctxt, self.id);
    }

    /// Calls `glMemoryBarrier(GL_BUFFER_UPDATE_BARRIER_BIT)` if necessary.
    fn barrier_for_buffer_update(&self, ctxt: &mut CommandContext) {
        if self.latest_shader_write.get() >= ctxt.state.latest_memory_barrier_buffer_update {
            unsafe { ctxt.gl.MemoryBarrier(gl::BUFFER_UPDATE_BARRIER_BIT); }
            ctxt.state.latest_memory_barrier_buffer_update = ctxt.state.next_draw_call_id;
        }
    }

    /// Calls `glMemoryBarrier(GL_VERTEX_ATTRIB_ARRAY_BARRIER_BIT)` if necessary.
    pub fn prepare_for_vertex_attrib_array(&self, ctxt: &mut CommandContext) {
        self.assert_unmapped(ctxt);
        self.assert_not_transform_feedback(ctxt);

        if self.latest_shader_write.get() >= ctxt.state.latest_memory_barrier_vertex_attrib_array {
            unsafe { ctxt.gl.MemoryBarrier(gl::VERTEX_ATTRIB_ARRAY_BARRIER_BIT); }
            ctxt.state.latest_memory_barrier_vertex_attrib_array = ctxt.state.next_draw_call_id;
        }
    }

    /// Calls `glMemoryBarrier(ELEMENT_ARRAY_BARRIER_BIT)` if necessary.
    pub fn prepare_for_element_array(&self, ctxt: &mut CommandContext) {
        self.assert_unmapped(ctxt);
        self.assert_not_transform_feedback(ctxt);

        if self.latest_shader_write.get() >= ctxt.state.latest_memory_barrier_element_array {
            unsafe { ctxt.gl.MemoryBarrier(gl::ELEMENT_ARRAY_BARRIER_BIT); }
            ctxt.state.latest_memory_barrier_element_array = ctxt.state.next_draw_call_id;
        }

    }

    /// Binds the buffer to `GL_ELEMENT_ARRAY_BUFFER` regardless of the current vertex array object.
    pub fn bind_to_element_array(&self, ctxt: &mut CommandContext) {
        if ctxt.version >= &Version(Api::Gl, 1, 5) ||
           ctxt.version >= &Version(Api::GlEs, 2, 0)
        {
            unsafe { ctxt.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id); }
        } else if ctxt.extensions.gl_arb_vertex_buffer_object {
            unsafe { ctxt.gl.BindBufferARB(gl::ELEMENT_ARRAY_BUFFER, self.id); }
        } else {
            unreachable!();
        }
    }

    /// Makes sure that the buffer is binded to the `GL_PIXEL_PACK_BUFFER` and calls
    /// `glMemoryBarrier(GL_PIXEL_BUFFER_BARRIER_BIT)` if necessary.
    pub fn prepare_and_bind_for_pixel_pack(&self, mut ctxt: &mut CommandContext) {
        self.assert_unmapped(ctxt);
        self.assert_not_transform_feedback(ctxt);

        if self.latest_shader_write.get() >= ctxt.state.latest_memory_barrier_pixel_buffer {
            unsafe { ctxt.gl.MemoryBarrier(gl::PIXEL_BUFFER_BARRIER_BIT); }
            ctxt.state.latest_memory_barrier_pixel_buffer = ctxt.state.next_draw_call_id;
        }

        unsafe { bind_buffer(ctxt, self.id, BufferType::PixelPackBuffer); }
    }

    /// Makes sure that nothing is binded to `GL_PIXEL_PACK_BUFFER`.
    #[inline]
    pub fn unbind_pixel_pack(ctxt: &mut CommandContext) {
        unsafe { bind_buffer(ctxt, 0, BufferType::PixelPackBuffer); }
    }

    /// Makes sure that the buffer is binded to the `GL_PIXEL_UNPACK_BUFFER` and calls
    /// `glMemoryBarrier(GL_PIXEL_BUFFER_BARRIER_BIT)` if necessary.
    pub fn prepare_and_bind_for_pixel_unpack(&self, mut ctxt: &mut CommandContext) {
        self.assert_unmapped(ctxt);
        self.assert_not_transform_feedback(ctxt);

        if self.latest_shader_write.get() >= ctxt.state.latest_memory_barrier_pixel_buffer {
            unsafe { ctxt.gl.MemoryBarrier(gl::PIXEL_BUFFER_BARRIER_BIT); }
            ctxt.state.latest_memory_barrier_pixel_buffer = ctxt.state.next_draw_call_id;
        }

        unsafe { bind_buffer(ctxt, self.id, BufferType::PixelUnpackBuffer); }
    }

    /// Makes sure that nothing is binded to `GL_PIXEL_UNPACK_BUFFER`.
    #[inline]
    pub fn unbind_pixel_unpack(ctxt: &mut CommandContext) {
        unsafe { bind_buffer(ctxt, 0, BufferType::PixelUnpackBuffer); }
    }

    /// Makes sure that the buffer is binded to the `GL_QUERY_BUFFER` and calls
    /// `glMemoryBarrier(GL_PIXEL_BUFFER_BARRIER_BIT)` if necessary.
    pub fn prepare_and_bind_for_query(&self, mut ctxt: &mut CommandContext) {
        assert!(ctxt.version >= &Version(Api::Gl, 4, 4) ||
                ctxt.extensions.gl_arb_query_buffer_object ||
                ctxt.extensions.gl_amd_query_buffer_object);

        self.assert_unmapped(ctxt);
        self.assert_not_transform_feedback(ctxt);

        if self.latest_shader_write.get() >= ctxt.state.latest_memory_barrier_pixel_buffer {
            unsafe { ctxt.gl.MemoryBarrier(gl::QUERY_BUFFER_BARRIER_BIT); }
            ctxt.state.latest_memory_barrier_query_buffer = ctxt.state.next_draw_call_id;
        }

        unsafe { bind_buffer(ctxt, self.id, BufferType::QueryBuffer); }
    }

    /// Makes sure that nothing is binded to `GL_QUERY_BUFFER`.
    #[inline]
    pub fn unbind_query(ctxt: &mut CommandContext) {
        unsafe { bind_buffer(ctxt, 0, BufferType::QueryBuffer); }
    }

    /// Makes sure that the buffer is binded to the `GL_DRAW_INDIRECT_BUFFER` and calls
    /// `glMemoryBarrier(GL_COMMAND_BARRIER_BIT)` if necessary.
    pub fn prepare_and_bind_for_draw_indirect(&self, mut ctxt: &mut CommandContext) {
        self.assert_unmapped(ctxt);
        self.assert_not_transform_feedback(ctxt);

        if self.latest_shader_write.get() >= ctxt.state.latest_memory_barrier_command {
            unsafe { ctxt.gl.MemoryBarrier(gl::COMMAND_BARRIER_BIT); }
            ctxt.state.latest_memory_barrier_command = ctxt.state.next_draw_call_id;
        }

        unsafe { bind_buffer(ctxt, self.id, BufferType::DrawIndirectBuffer); }
    }

    /// Makes sure that the buffer is binded to the `GL_DISPATCH_INDIRECT_BUFFER` and calls
    /// `glMemoryBarrier(GL_COMMAND_BARRIER_BIT)` if necessary.
    pub fn prepare_and_bind_for_dispatch_indirect(&self, mut ctxt: &mut CommandContext) {
        self.assert_unmapped(ctxt);
        self.assert_not_transform_feedback(ctxt);

        if self.latest_shader_write.get() >= ctxt.state.latest_memory_barrier_command {
            unsafe { ctxt.gl.MemoryBarrier(gl::COMMAND_BARRIER_BIT); }
            ctxt.state.latest_memory_barrier_command = ctxt.state.next_draw_call_id;
        }

        unsafe { bind_buffer(ctxt, self.id, BufferType::DispatchIndirectBuffer); }
    }

    /// Makes sure that the buffer is binded to the indexed `GL_UNIFORM_BUFFER` point and calls
    /// `glMemoryBarrier(GL_UNIFORM_BARRIER_BIT)` if necessary.
    pub fn prepare_and_bind_for_uniform(&self, ctxt: &mut CommandContext, index: gl::types::GLuint,
                                        range: Range<usize>)
    {
        self.assert_unmapped(ctxt);
        self.assert_not_transform_feedback(ctxt);

        if self.latest_shader_write.get() >= ctxt.state.latest_memory_barrier_uniform {
            unsafe { ctxt.gl.MemoryBarrier(gl::UNIFORM_BARRIER_BIT); }
            ctxt.state.latest_memory_barrier_uniform = ctxt.state.next_draw_call_id;
        }

        self.indexed_bind(ctxt, BufferType::UniformBuffer, index, range);
    }

    /// Makes sure that the buffer is binded to the indexed `GL_SHARED_STORAGE_BUFFER` point and calls
    /// `glMemoryBarrier(GL_SHADER_STORAGE_BARRIER_BIT)` if necessary.
    pub fn prepare_and_bind_for_shared_storage(&self, ctxt: &mut CommandContext, index: gl::types::GLuint,
                                               range: Range<usize>)
    {
        self.assert_unmapped(ctxt);
        self.assert_not_transform_feedback(ctxt);

        if self.latest_shader_write.get() >= ctxt.state.latest_memory_barrier_shader_storage {
            unsafe { ctxt.gl.MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT); }
            ctxt.state.latest_memory_barrier_shader_storage = ctxt.state.next_draw_call_id;
        }

        self.indexed_bind(ctxt, BufferType::ShaderStorageBuffer, index, range);

        self.latest_shader_write.set(ctxt.state.next_draw_call_id);        // TODO: put this somewhere else
    }

    /// Binds the buffer to `GL_TRANSFORM_FEEDBACk_BUFFER` regardless of the current transform
    /// feedback object.
    #[inline]
    pub fn bind_to_transform_feedback(&self, ctxt: &mut CommandContext, index: gl::types::GLuint,
                                      range: Range<usize>)
    {
        self.indexed_bind(ctxt, BufferType::TransformFeedbackBuffer, index, range);
    }

    /// Makes sure that the buffer is binded to a specific bind point.
    ///
    /// The bind point is the value passed to `ty`.
    ///
    /// # Panic
    ///
    /// Panicks if the backend doesn't allow binding this buffer to the specified point.
    #[inline]
    fn bind(&self, mut ctxt: &mut CommandContext, ty: BufferType) {
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
    #[inline]
    fn indexed_bind(&self, mut ctxt: &mut CommandContext, ty: BufferType,
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
    pub unsafe fn upload<D: ?Sized>(&self, offset_bytes: usize, data: &D)
                                    where D: Content
    {
        assert!(offset_bytes + mem::size_of_val(data) <= self.size);

        if self.persistent_mapping.is_some() {
            let mapping = Mapping { mapping: self.map_shared(offset_bytes .. offset_bytes + mem::size_of_val(data), false, true) };
            ptr::copy_nonoverlapping(data.to_void_ptr() as *const u8, <D as Content>::to_void_ptr(&mapping) as *mut u8, mem::size_of_val(data));

        } else if self.immutable {
            let mut ctxt = self.context.make_current();
            self.barrier_for_buffer_update(&mut ctxt);

            self.assert_unmapped(&mut ctxt);
            self.assert_not_transform_feedback(&mut ctxt);

            let (tmp_buffer, _, _, _) = create_buffer(&mut ctxt, mem::size_of_val(data), Some(data),
                                                      BufferType::CopyReadBuffer,
                                                      BufferMode::Dynamic).unwrap();
            copy_buffer(&mut ctxt, tmp_buffer, 0, self.id, offset_bytes, mem::size_of_val(data)).unwrap();
            destroy_buffer(&mut ctxt, tmp_buffer);

        } else {
            assert!(offset_bytes < self.size);

            let mut ctxt = self.context.make_current();
            self.barrier_for_buffer_update(&mut ctxt);

            let invalidate_all = offset_bytes == 0 && mem::size_of_val(data) == self.size;

            self.assert_unmapped(&mut ctxt);
            self.assert_not_transform_feedback(&mut ctxt);

            if invalidate_all && (ctxt.version >= &Version(Api::Gl, 4, 3) ||
                ctxt.extensions.gl_arb_invalidate_subdata)
            {
                ctxt.gl.InvalidateBufferData(self.id);
            }

            if ctxt.version >= &Version(Api::Gl, 4, 5) {
                ctxt.gl.NamedBufferSubData(self.id, offset_bytes as gl::types::GLintptr,
                                           mem::size_of_val(data) as gl::types::GLsizeiptr,
                                           data.to_void_ptr() as *const _)

            } else if ctxt.extensions.gl_ext_direct_state_access {
                ctxt.gl.NamedBufferSubDataEXT(self.id, offset_bytes as gl::types::GLintptr,
                                              mem::size_of_val(data) as gl::types::GLsizeiptr,
                                              data.to_void_ptr() as *const _)

            } else if ctxt.version >= &Version(Api::Gl, 1, 5) ||
                ctxt.version >= &Version(Api::GlEs, 2, 0)
            {
                let bind = bind_buffer(&mut ctxt, self.id, self.ty);
                ctxt.gl.BufferSubData(bind, offset_bytes as gl::types::GLintptr,
                                      mem::size_of_val(data) as gl::types::GLsizeiptr,
                                      data.to_void_ptr() as *const _);

            } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                let bind = bind_buffer(&mut ctxt, self.id, self.ty);
                ctxt.gl.BufferSubDataARB(bind, offset_bytes as gl::types::GLintptr,
                                         mem::size_of_val(data) as gl::types::GLsizeiptr,
                                         data.to_void_ptr() as *const _);

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

        if self.persistent_mapping.is_none() &&
           (ctxt.version >= &Version(Api::Gl, 4, 3) || ctxt.extensions.gl_arb_invalidate_subdata)
        {
            if is_whole_buffer {
                unsafe { ctxt.gl.InvalidateBufferData(self.id) };
            } else {
                unsafe { ctxt.gl.InvalidateBufferSubData(self.id, offset as gl::types::GLintptr,
                                                         size as gl::types::GLsizeiptr) };
            }

        } else if !self.created_with_buffer_storage {
            if is_whole_buffer {
                let flags = match self.creation_mode {
                    BufferMode::Default | BufferMode::Immutable => gl::STATIC_DRAW,
                    BufferMode::Persistent | BufferMode::Dynamic => gl::DYNAMIC_DRAW,
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
    /// Contrary to `map_mut`, this function only requires a `&self`.
    ///
    /// # Panic
    ///
    /// Panicks if the `bytes_range` is not aligned to a mappable slice.
    ///
    /// # Unsafety
    ///
    /// If the buffer uses persistent mapping, the caller of this function must handle
    /// synchronization.
    ///
    /// If you pass `false` for `read`, you **must not** read the returned buffer. If you pass
    /// `false` for `write`, you **must not** write the returned buffer.
    ///
    unsafe fn map_shared<D: ?Sized>(&self, bytes_range: Range<usize>, read: bool, write: bool)
                                    -> MappingImpl<D> where D: Content
    {
        if let Some(existing_mapping) = self.persistent_mapping.clone() {
            // TODO: optimize so that it's not always necessary to make the context current
            let mut ctxt = self.context.make_current();
            self.barrier_for_buffer_update(&mut ctxt);

            let data = (existing_mapping as *mut u8).offset(bytes_range.start as isize);
            let data = Content::ref_from_ptr(data as *mut (),
                                             bytes_range.end - bytes_range.start).unwrap();

            MappingImpl::PersistentMapping {
                buffer: self,
                offset_bytes: bytes_range.start,
                data: data,
                needs_flushing: write,
            }

        } else {
            let size_bytes = bytes_range.end - bytes_range.start;

            let mut ctxt = self.context.make_current();

            // we have to construct a temporary buffer that we will map in memory
            // then after the Mapping is destroyed, we will copy from the temporary buffer to the
            // real one
            let temporary_buffer = {
                let (temporary_buffer, _, _, _) = create_buffer::<D>(&mut ctxt, size_bytes,
                                                                     None, BufferType::CopyWriteBuffer,
                                                                     BufferMode::Dynamic).unwrap();
                temporary_buffer
            };

            let ptr = {
                self.assert_unmapped(&mut ctxt);
                self.assert_not_transform_feedback(&mut ctxt);

                if read {
                    copy_buffer(&mut ctxt, self.id, bytes_range.start,
                                temporary_buffer, 0, size_bytes).unwrap();
                }

                map_buffer(&mut ctxt, temporary_buffer, self.ty, 0 .. size_bytes, true, true)
                                    .expect("Buffer mapping is not supported by the backend")
            };

            let data = match Content::ref_from_ptr(ptr, bytes_range.end - bytes_range.start) {
                Some(data) => data,
                None => {
                    unmap_buffer(&mut ctxt, temporary_buffer, self.ty);
                    panic!("Wrong bytes range");
                }
            };

            MappingImpl::TemporaryBuffer {
                original_buffer: self,
                original_buffer_offset: bytes_range.start,
                temporary_buffer: temporary_buffer,
                temporary_buffer_data: data,
                needs_flushing: write,
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
    /// Contrary to `map_shared`, this function requires a `&mut self`. It can only be used if
    /// you have exclusive access to the buffer.
    ///
    /// # Panic
    ///
    /// Panicks if the `bytes_range` is not aligned to a mappable slice.
    ///
    /// # Unsafety
    ///
    /// If the buffer uses persistent mapping, the caller of this function must handle
    /// synchronization.
    ///
    /// If you pass `false` for `read`, you **must not** read the returned buffer. If you pass
    /// `false` for `write`, you **must not** write the returned buffer.
    ///
    unsafe fn map_impl<D: ?Sized>(&mut self, bytes_range: Range<usize>, read: bool, write: bool)
                                  -> MappingImpl<D> where D: Content
    {
        if self.persistent_mapping.is_some() || self.immutable {
            self.map_shared(bytes_range, read, write)

        } else {
            let data = {
                let mut ctxt = self.context.make_current();

                let ptr = {
                    self.assert_unmapped(&mut ctxt);
                    self.assert_not_transform_feedback(&mut ctxt);
                    self.barrier_for_buffer_update(&mut ctxt);
                    let ptr = map_buffer(&mut ctxt, self.id, self.ty, bytes_range.clone(),
                                         read, write)
                                        .expect("Buffer mapping is not supported by the backend");
                    self.mapped.set(true);
                    ptr
                };

                match Content::ref_from_ptr(ptr, bytes_range.end - bytes_range.start) {
                    Some(data) => data,
                    None => {
                        unmap_buffer(&mut ctxt, self.id, self.ty);
                        panic!("Wrong bytes range");
                    }
                }
            };

            MappingImpl::RegularMapping {
                buffer: self,
                data: data,
                needs_flushing: write,
            }
        }
    }

    /// Returns a read and write mapping in memory of the content of the buffer.
    ///
    /// # Panic
    ///
    /// Panicks if the `bytes_range` is not aligned to a mappable slice.
    ///
    /// # Unsafety
    ///
    /// If the buffer uses persistent mapping, the caller of this function must handle
    /// synchronization.
    ///
    #[inline]
    pub unsafe fn map<D: ?Sized>(&mut self, bytes_range: Range<usize>)
                                 -> Mapping<D> where D: Content
    {
        Mapping {
            mapping: self.map_impl(bytes_range, true, true)
        }
    }

    /// Returns a read-only mapping in memory of the content of the buffer.
    ///
    /// # Panic
    ///
    /// Panicks if the `bytes_range` is not aligned to a mappable slice.
    ///
    /// # Unsafety
    ///
    /// If the buffer uses persistent mapping, the caller of this function must handle
    /// synchronization.
    ///
    #[inline]
    pub unsafe fn map_read<D: ?Sized>(&mut self, bytes_range: Range<usize>)
                                      -> ReadMapping<D> where D: Content
    {
        ReadMapping {
            mapping: self.map_impl(bytes_range, true, false)
        }
    }

    /// Returns a write-only mapping in memory of the content of the buffer.
    ///
    /// # Panic
    ///
    /// Panicks if the `bytes_range` is not aligned to a mappable slice.
    ///
    /// # Unsafety
    ///
    /// If the buffer uses persistent mapping, the caller of this function must handle
    /// synchronization.
    ///
    #[inline]
    pub unsafe fn map_write<D: ?Sized>(&mut self, bytes_range: Range<usize>)
                                       -> WriteMapping<D> where D: Content
    {
        WriteMapping {
            mapping: self.map_impl(bytes_range, false, true)
        }
    }

    /// Reads the content of the buffer.
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
    pub unsafe fn read<D: ?Sized>(&self, range: Range<usize>)
                                  -> Result<D::Owned, ReadError>
                                  where D: Content
    {
        let size_to_read = range.end - range.start;

        if self.persistent_mapping.is_some() {
            let mapping = ReadMapping { mapping: self.map_shared(range, true, false) };
            <D as Content>::read(size_to_read, |output| {
                ptr::copy_nonoverlapping(<D as Content>::to_void_ptr(&mapping) as *const u8, output as *mut D as *mut u8, size_to_read);
                Ok(())
            })

        } else {
            let mut ctxt = self.context.make_current();

            if ctxt.state.lost_context {
                return Err(ReadError::ContextLost);
            }

            self.assert_unmapped(&mut ctxt);
            self.barrier_for_buffer_update(&mut ctxt);

            <D as Content>::read(size_to_read, |output| {
                if ctxt.version >= &Version(Api::Gl, 4, 5) {
                    ctxt.gl.GetNamedBufferSubData(self.id, range.start as gl::types::GLintptr,
                                                  size_to_read as gl::types::GLsizeiptr,
                                                  output as *mut _ as *mut _);

                } else if ctxt.version >= &Version(Api::Gl, 1, 5) {
                    let bind = bind_buffer(&mut ctxt, self.id, self.ty);
                    ctxt.gl.GetBufferSubData(bind, range.start as gl::types::GLintptr,
                                             size_to_read as gl::types::GLsizeiptr,
                                             output as *mut _ as *mut _);

                } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                    let bind = bind_buffer(&mut ctxt, self.id, self.ty);
                    ctxt.gl.GetBufferSubDataARB(bind, range.start as gl::types::GLintptr,
                                                size_to_read as gl::types::GLsizeiptr,
                                                output as *mut _ as *mut _);

                } else if ctxt.version >= &Version(Api::GlEs, 1, 0) {
                    return Err(ReadError::NotSupported);

                } else {
                    unreachable!()
                }

                Ok(())
            })
        }
    }

    /// Copies data from this buffer to another one.
    ///
    /// With persistent-mapped buffers you must create a sync fence *after* this operation.
    ///
    /// # Panic
    ///
    /// Panics if the offset/sizes are out of range.
    ///
    pub fn copy_to(&self, range: Range<usize>, target: &Alloc, dest_offset: usize)
                   -> Result<(), CopyError>
    {
        // TODO: read+write manually
        // TODO: check that the other buffer belongs to the same context

        assert!(range.end >= range.start);
        assert!(range.end <= self.size);
        assert!(dest_offset + range.end - range.start <= target.size);

        let mut ctxt = self.context.make_current();

        unsafe {
            copy_buffer(&mut ctxt, self.id, range.start, target.id, dest_offset,
                        range.end - range.start)
        }
    }
}

impl fmt::Debug for Alloc {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "Buffer #{} (size: {} bytes)", self.id, self.size)
    }
}

impl Drop for Alloc {
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

impl GlObject for Alloc {
    type Id = gl::types::GLuint;

    #[inline]
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

/// A mapping of a buffer. Private object.
enum MappingImpl<'b, D: ?Sized> {
    PersistentMapping {
        buffer: &'b Alloc,
        offset_bytes: usize,
        data: *mut D,
        needs_flushing: bool,
    },

    TemporaryBuffer {
        original_buffer: &'b Alloc,
        original_buffer_offset: usize,
        temporary_buffer: gl::types::GLuint,
        temporary_buffer_data: *mut D,
        needs_flushing: bool,
    },

    RegularMapping {
        buffer: &'b mut Alloc,
        data: *mut D,
        needs_flushing: bool,
    },
}

unsafe impl<'a, D: ?Sized> Sync for MappingImpl<'a, D> where D: Send + Sync {}

impl<'a, D: ?Sized> Drop for MappingImpl<'a, D> {
    fn drop(&mut self) {
        match self {
            &mut MappingImpl::PersistentMapping { buffer, offset_bytes, data, needs_flushing } => {
                let mut ctxt = buffer.context.make_current();
                unsafe {
                    if needs_flushing {
                        flush_range(&mut ctxt, buffer.id, buffer.ty,
                                    offset_bytes .. offset_bytes + mem::size_of_val(&*data));
                    }
                }
            },

            &mut MappingImpl::TemporaryBuffer { original_buffer, original_buffer_offset,
                                                temporary_buffer, temporary_buffer_data,
                                                needs_flushing } =>
            {
                let mut ctxt = original_buffer.context.make_current();
                original_buffer.barrier_for_buffer_update(&mut ctxt);

                unsafe {
                    if needs_flushing {
                        flush_range(&mut ctxt, temporary_buffer, original_buffer.ty,
                                    0 .. mem::size_of_val(&*temporary_buffer_data));
                    }
                    unmap_buffer(&mut ctxt, temporary_buffer, original_buffer.ty);
                    if needs_flushing {
                        copy_buffer(&mut ctxt, temporary_buffer, 0, original_buffer.id,
                                    original_buffer_offset, mem::size_of_val(&*temporary_buffer_data)).unwrap();
                    }

                    destroy_buffer(&mut ctxt, temporary_buffer);
                }
            },

            &mut MappingImpl::RegularMapping { ref mut buffer, data, needs_flushing } => {
                let mut ctxt = buffer.context.make_current();

                unsafe {
                    if needs_flushing {
                        flush_range(&mut ctxt, buffer.id, buffer.ty,
                                    0 .. mem::size_of_val(&*data));
                    }
                    unmap_buffer(&mut ctxt, buffer.id, buffer.ty);
                }

                buffer.mapped.set(false);
            },
        }
    }
}

/// A mapping of a buffer for reading and writing.
pub struct Mapping<'b, D: ?Sized> where D: Content {
    mapping: MappingImpl<'b, D>,
}

impl<'a, D: ?Sized> Deref for Mapping<'a, D> where D: Content {
    type Target = D;

    #[inline]
    fn deref(&self) -> &D {
        match self.mapping {
            MappingImpl::PersistentMapping { data, .. } => {
                unsafe { &*data }
            },

            MappingImpl::TemporaryBuffer { temporary_buffer_data, .. } => {
                unsafe { &*temporary_buffer_data }
            },

            MappingImpl::RegularMapping { data, .. } => {
                unsafe { &*data }
            },
        }
    }
}

impl<'a, D: ?Sized> DerefMut for Mapping<'a, D> where D: Content {
    #[inline]
    fn deref_mut(&mut self) -> &mut D {
        match self.mapping {
            MappingImpl::PersistentMapping { data, .. } => {
                unsafe { &mut *data }
            },

            MappingImpl::TemporaryBuffer { temporary_buffer_data, .. } => {
                unsafe { &mut *temporary_buffer_data }
            },

            MappingImpl::RegularMapping { data, .. } => {
                unsafe { &mut *data }
            },
        }
    }
}

/// A mapping of a buffer for reading.
pub struct ReadMapping<'b, D: ?Sized> where D: Content {
    mapping: MappingImpl<'b, D>,
}

impl<'a, D: ?Sized> Deref for ReadMapping<'a, D> where D: Content {
    type Target = D;

    #[inline]
    fn deref(&self) -> &D {
        match self.mapping {
            MappingImpl::PersistentMapping { data, .. } => {
                unsafe { &*data }
            },

            MappingImpl::TemporaryBuffer { temporary_buffer_data, .. } => {
                unsafe { &*temporary_buffer_data }
            },

            MappingImpl::RegularMapping { data, .. } => {
                unsafe { &*data }
            },
        }
    }
}

/// A mapping of a buffer for write only.
pub struct WriteMapping<'b, D: ?Sized> where D: Content {
    mapping: MappingImpl<'b, D>,
}

impl<'b, D: ?Sized> WriteMapping<'b, D> where D: Content {
    #[inline]
    fn get_slice(&mut self) -> &mut D {
        match self.mapping {
            MappingImpl::PersistentMapping { data, .. } => {
                unsafe { &mut *data }
            },

            MappingImpl::TemporaryBuffer { temporary_buffer_data, .. } => {
                unsafe { &mut *temporary_buffer_data }
            },

            MappingImpl::RegularMapping { data, .. } => {
                unsafe { &mut *data }
            },
        }
    }
}

impl<'b, D> WriteMapping<'b, D> where D: Content + Copy {
    /// Writes the whole content.
    #[inline]
    pub fn write(&mut self, value: D) {
        let slice = self.get_slice();
        *slice = value;
    }
}

impl<'b, D> WriteMapping<'b, [D]> where [D]: Content, D: Copy {
    /// Returns the length of the mapping.
    #[inline]
    pub fn len(&self) -> usize {
        match self.mapping {
            MappingImpl::PersistentMapping { data, .. } => unsafe { (&*data).len() },
            MappingImpl::TemporaryBuffer { temporary_buffer_data, .. } => unsafe { (&*temporary_buffer_data).len() },
            MappingImpl::RegularMapping { data, .. } => unsafe { (&*data).len() },
        }
    }

    /// Changes an element of the mapping.
    ///
    /// # Panic
    ///
    /// Panics if out of range.
    ///
    #[inline]
    pub fn set(&mut self, index: usize, value: D) {
        let slice = self.get_slice();
        slice[index] = value;
    }
}

/// Returns true if reading from a buffer is supported by the backend.
pub fn is_buffer_read_supported<C>(ctxt: &C) -> bool where C: CapabilitiesSource {
    if ctxt.get_version() >= &Version(Api::Gl, 4, 5) {
        true

    } else if ctxt.get_version() >= &Version(Api::Gl, 1, 5) {
        true

    } else if ctxt.get_extensions().gl_arb_vertex_buffer_object {
        true

    } else if ctxt.get_version() >= &Version(Api::GlEs, 1, 0) {
        false

    } else {
        unreachable!();
    }
}

/// Creates a new buffer.
///
/// # Panic
///
/// Panics if `mem::size_of_val(&data) != size`.
unsafe fn create_buffer<D: ?Sized>(mut ctxt: &mut CommandContext, size: usize, data: Option<&D>,
                                   ty: BufferType, mode: BufferMode)
                                   -> Result<(gl::types::GLuint, bool, bool, Option<*mut raw::c_void>),
                                             BufferCreationError>
                                   where D: Content
{
    if !is_buffer_type_supported(ctxt, ty) {
        return Err(BufferCreationError::BufferTypeNotSupported);
    }

    if let Some(data) = data {
        assert!(mem::size_of_val(data) == size);
    }

    // creating the id of the buffer
    let id = {
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
        id
    };

    // raw pointer to data
    let data_ptr = if let Some(data) = data {
        if size == 0 {      // if the size is `0` we pass `1` instead (see below),
            ptr::null()     // so it's important to have `null` here
        } else {
            data.to_void_ptr()
        }
    } else {
        ptr::null()
    };

    // if the `size` is 0 bytes then we use 1 instead, otherwise nvidia drivers complain
    // note that according to glium the size of the buffer will remain 0
    let size = match size {
        0 => 1,
        a => a
    };

    // the flags to use in the case where only `glBufferData` is supported
    let mutable_storage_flags = match mode {
        BufferMode::Persistent | BufferMode::Dynamic => gl::DYNAMIC_DRAW,
        BufferMode::Default | BufferMode::Immutable => gl::STATIC_DRAW,
    };

    // the flags to use if `glBufferStorage` is supported
    let immutable_storage_flags = match mode {
        BufferMode::Default => gl::DYNAMIC_STORAGE_BIT | gl::MAP_READ_BIT | gl::MAP_WRITE_BIT,
        BufferMode::Dynamic => gl::DYNAMIC_STORAGE_BIT | gl::CLIENT_STORAGE_BIT | gl::MAP_READ_BIT | gl::MAP_WRITE_BIT,
        BufferMode::Persistent => gl::MAP_PERSISTENT_BIT | gl::MAP_READ_BIT | gl::MAP_WRITE_BIT,
        BufferMode::Immutable => 0,
    };

    // if true, there is a possibility that the buffer won't be modifiable with regular OpenGL
    // function calls
    let could_be_immutable = match mode {
        BufferMode::Default | BufferMode::Dynamic => false,
        BufferMode::Immutable | BufferMode::Persistent => true,
    };

    // will store the actual size of the buffer so that we can compare it with the expected size
    let mut obtained_size: gl::types::GLint = mem::uninitialized();

    // the value of `immutable` is determined below
    // if true, the buffer won't be modifiable with regular OpenGL function calls
    let immutable: bool;

    // whether the buffer was created with `glBufferStorage`
    let created_with_buffer_storage: bool;

    if ctxt.version >= &Version(Api::Gl, 4, 5) || ctxt.extensions.gl_arb_direct_state_access {
        ctxt.gl.NamedBufferStorage(id, size as gl::types::GLsizeiptr,
                                   data_ptr as *const _,
                                   immutable_storage_flags);
        ctxt.gl.GetNamedBufferParameteriv(id, gl::BUFFER_SIZE, &mut obtained_size);
        immutable = could_be_immutable;
        created_with_buffer_storage = true;

    } else if ctxt.extensions.gl_arb_buffer_storage &&
              ctxt.extensions.gl_ext_direct_state_access
    {
        ctxt.gl.NamedBufferStorageEXT(id, size as gl::types::GLsizeiptr,
                                      data_ptr as *const _,
                                      immutable_storage_flags);
        ctxt.gl.GetNamedBufferParameterivEXT(id, gl::BUFFER_SIZE, &mut obtained_size);
        immutable = could_be_immutable;
        created_with_buffer_storage = true;

    } else if ctxt.version >= &Version(Api::Gl, 4, 4) ||
              ctxt.extensions.gl_arb_buffer_storage
    {
        let bind = bind_buffer(&mut ctxt, id, ty);
        ctxt.gl.BufferStorage(bind, size as gl::types::GLsizeiptr,
                              data_ptr as *const _,
                              immutable_storage_flags);
        ctxt.gl.GetBufferParameteriv(bind, gl::BUFFER_SIZE, &mut obtained_size);
        immutable = could_be_immutable;
        created_with_buffer_storage = true;

    } else if ctxt.extensions.gl_ext_buffer_storage {
        let bind = bind_buffer(&mut ctxt, id, ty);
        ctxt.gl.BufferStorageEXT(bind, size as gl::types::GLsizeiptr,
                                 data_ptr as *const _,
                                 immutable_storage_flags);
        ctxt.gl.GetBufferParameteriv(bind, gl::BUFFER_SIZE, &mut obtained_size);
        immutable = could_be_immutable;
        created_with_buffer_storage = true;

    } else if ctxt.version >= &Version(Api::Gl, 1, 5) ||
        ctxt.version >= &Version(Api::GlEs, 2, 0)
    {
        let bind = bind_buffer(&mut ctxt, id, ty);
        ctxt.gl.BufferData(bind, size as gl::types::GLsizeiptr,
                           data_ptr as *const _, mutable_storage_flags);
        ctxt.gl.GetBufferParameteriv(bind, gl::BUFFER_SIZE, &mut obtained_size);
        immutable = false;
        created_with_buffer_storage = false;

    } else if ctxt.extensions.gl_arb_vertex_buffer_object {
        let bind = bind_buffer(&mut ctxt, id, ty);
        ctxt.gl.BufferDataARB(bind, size as gl::types::GLsizeiptr,
                              data_ptr as *const _, mutable_storage_flags);
        ctxt.gl.GetBufferParameterivARB(bind, gl::BUFFER_SIZE, &mut obtained_size);
        immutable = false;
        created_with_buffer_storage = false;

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

    let persistent_mapping = if let BufferMode::Persistent = mode {
        if immutable {
            let ptr = if ctxt.version >= &Version(Api::Gl, 4, 5) {
                ctxt.gl.MapNamedBufferRange(id, 0, size as gl::types::GLsizeiptr,
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
        }
    } else {
        None
    };

    Ok((id, immutable, created_with_buffer_storage, persistent_mapping))
}
