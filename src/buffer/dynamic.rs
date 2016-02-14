use backend::Facade;
use context::CommandContext;
use context::Context;
use version::Version;
use CapabilitiesSource;
use ContextExt;
use gl;
use std::error::Error;
use std::{fmt, mem, ptr};
use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::os::raw::c_void;
use std::ops::{Deref, DerefMut, Range};
use GlObject;
use TransformFeedbackSessionExt;

use buffer::{BufferType, BufferCreationError, CopyError, BufferAnySlice};
use vertex::TransformFeedbackSession;
use vertex_array_object::VertexAttributesSystem;

use version::Api;

use buffer::raw;
use buffer::ArrayContent;
use buffer::Content;
use buffer::CopyTo;
use buffer::Create;
use buffer::Invalidate;
use buffer::Storage;

/// 
pub struct DynamicBuffer<T: ?Sized> where T: Content {
    marker: PhantomData<T>,

    context: Rc<Context>,

    /// OpenGL identifier ; can't be zero.
    id: gl::types::GLuint,

    /// Type of buffer.
    ty: BufferType,

    /// The flag used when creating the buffer (eg. `STREAM_DRAW`, `STATIC_READ`, etc.).
    usage: gl::types::GLenum,

    /// Size in bytes of the buffer.
    size: usize,

    /// True if the buffer is currently mapped with something else than persistent mapping.
    ///
    /// The purpose of this flag is to detect if the user mem::forgets the `Mapping` object.
    mapped: Cell<bool>,

    /// ID of the draw call where the buffer was last written as an SSBO.
    latest_shader_write: Cell<u64>,
}

impl<T: ?Sized> DynamicBuffer<T> where T: Content {
    /// Builds a new buffer containing the given data. The size of the buffer is equal to the size
    /// of the data.
    pub fn new<F>(facade: &F, data: &T, ty: BufferType, mode: BufferMode)
                  -> Result<$ty<T>, BufferCreationError>
                  where F: Facade
    {
        let mut ctxt = facade.get_context().make_current();

        let size = mem::size_of_val(data);
        let id = try!(unsafe { create_buffer(&mut ctxt, size, Some(data), ty, gl::STATIC_DRAW) });

        Ok(DynamicBuffer {
            marker: PhantomData,
            context: facade.get_context().clone(),
            id: id,
            ty: ty,
            usage: gl::STATIC_DRAW,
            size: size,
            mapped: Cell::new(false),
            latest_shader_write: Cell::new(0),
        })
    }

    pub fn empty<F>(facade: &F, ty: BufferType) -> Result<ImmutableBuffer<T>, BufferCreationError>
        where F: Facade, T: Sized, T: Copy
    {
        let mut ctxt = facade.get_context().make_current();

        let size = mem::size_of::<T>();
        let id = try!(unsafe { create_buffer::<()>(&mut ctxt, size, None, ty, gl::STATIC_DRAW) });

        Ok(DynamicBuffer {
            marker: PhantomData,
            context: facade.get_context().clone(),
            id: id,
            ty: ty,
            usage: gl::STATIC_DRAW,
            size: size,
            mapped: Cell::new(false),
            latest_shader_write: Cell::new(0),
        })
    }

    pub fn empty_array<F>(facade: &F, len: usize, ty: BufferType)
                          -> Result<ImmutableBuffer<T>, BufferCreationError>
        where F: Facade, T: ArrayContent
    {
        let mut ctxt = facade.get_context().make_current();

        let size = len * <T as ArrayContent>::element_size();
        let id = try!(unsafe { create_buffer::<()>(&mut ctxt, size, None, ty, gl::STATIC_DRAW) });

        Ok(DynamicBuffer {
            marker: PhantomData,
            context: facade.get_context().clone(),
            id: id,
            ty: ty,
            usage: gl::STATIC_DRAW,
            size: size,
            mapped: Cell::new(false),
            latest_shader_write: Cell::new(0),
        })
    }

    pub fn empty_unsized<F>(facade: &F, size: usize, ty: BufferType)
                            -> Result<ImmutableBuffer<T>, BufferCreationError>
        where F: Facade, T: Copy
    {
        let mut ctxt = facade.get_context().make_current();
        let id = try!(unsafe { create_buffer::<()>(&mut ctxt, size, None, ty, gl::STATIC_DRAW) });

        Ok(DynamicBuffer {
            marker: PhantomData,
            context: facade.get_context().clone(),
            id: id,
            ty: ty,
            usage: gl::STATIC_DRAW,
            size: size,
            mapped: Cell::new(false),
            latest_shader_write: Cell::new(0),
        })
    }

    /// Copies the content of the buffer to another buffer.
    ///
    /// # Panic
    ///
    /// Panics if `T` is unsized and the other buffer is too small.
    ///
    pub fn copy_to<'a, S>(&self, target: S) -> Result<(), CopyError>
                          where S: Into<$slice_ty<'a, T>>, T: 'a
    {
        let target = target.into();
        let alloc = self.alloc.as_ref().unwrap();

        try!(alloc.copy_to(0 .. self.get_size(), &target.alloc, target.get_offset_bytes()));

        if let Some(inserter) = self.as_slice().add_fence() {
            let mut ctxt = alloc.get_context().make_current();
            inserter.insert(&mut ctxt);
        }

        if let Some(inserter) = target.add_fence() {
            let mut ctxt = alloc.get_context().make_current();
            inserter.insert(&mut ctxt);
        }

        Ok(())
    }

    /// Uploads data in the buffer.
    ///
    /// The data must fit inside the buffer.
    ///
    /// # Panic
    ///
    /// Panics if `T` is unsized and the data is not the same size as the buffer.
    ///
    pub fn upload(&self, data: &T) {
        if ctxt.version >= &Version(Api::Gl, 4, 5) || ctxt.extensions.gl_arb_direct_state_access {
            ctxt.gl.NamedBufferData(self.id, offset_bytes as gl::types::GLintptr,
                                    mem::size_of_val(data) as gl::types::GLsizeiptr,
                                    data.to_void_ptr() as *const _, self.usage)

        } else if ctxt.extensions.gl_ext_direct_state_access {
            ctxt.gl.NamedBufferDataEXT(self.id, offset_bytes as gl::types::GLintptr,
                                       mem::size_of_val(data) as gl::types::GLsizeiptr,
                                       data.to_void_ptr() as *const _, self.usage)

        } else if ctxt.version >= &Version(Api::Gl, 1, 5) ||
            ctxt.version >= &Version(Api::GlEs, 2, 0)
        {
            let bind = bind_buffer(&mut ctxt, self.id, self.ty);
            ctxt.gl.BufferData(bind, offset_bytes as gl::types::GLintptr,
                               mem::size_of_val(data) as gl::types::GLsizeiptr,
                               data.to_void_ptr() as *const _, self.usage);

        } else if ctxt.extensions.gl_arb_vertex_buffer_object {
            let bind = bind_buffer(&mut ctxt, self.id, self.ty);
            ctxt.gl.BufferDataARB(bind, offset_bytes as gl::types::GLintptr,
                                  mem::size_of_val(data) as gl::types::GLsizeiptr,
                                  data.to_void_ptr() as *const _, self.usage);

        } else {
            unreachable!();
        }
    }

    /// Implementation of the `upload` function meant to called by the `upload` functions
    /// in the slices.
    ///
    /// # Panic
    ///
    /// Panics if `offset_bytes` is out of range or the data is too large to fit in the buffer.
    ///
    /// # Safety
    ///
    /// Type safety is not enforced. This is mostly the equivalent of `std::ptr::write`.
    pub unsafe fn upload_impl<D: ?Sized>(&self, offset_bytes: usize, data: &D)
        where D: Content
    {
        assert!(offset_bytes + mem::size_of_val(data) <= self.size);
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

        if ctxt.version >= &Version(Api::Gl, 4, 5) || ctxt.extensions.gl_arb_direct_state_access {
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

    /// Invalidates the content of the buffer. The data becomes undefined.
    ///
    /// This is done by calling `glBufferData` with a null pointer for the data.
    ///
    /// # Panic
    ///
    /// Panics if out of range.
    ///
    pub fn invalidate(&self) {
        let flags = match self.creation_mode {
            BufferMode::Default | BufferMode::Immutable => gl::STATIC_DRAW,
            BufferMode::Persistent | BufferMode::Dynamic => gl::DYNAMIC_DRAW,
        };

        if ctxt.version >= &Version(Api::Gl, 1, 5) || ctxt.version >= &Version(Api::GlEs, 2, 0) {
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
    pub fn map<D: ?Sized>(&mut self, bytes_range: Range<usize>)
                                 -> Mapping<D> where D: Content
    {
        self.map_impl(bytes_range, true, true)
    }

    fn map_impl<D: ?Sized>(&mut self, bytes_range: Range<usize>, read: bool, write: bool)
                           -> MappingImpl<D>
        where D: Content
    {
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

        Mapping {
            buffer: self,
            data: data,
            needs_flushing: write,
        }
    }

    /// Reads the content of the buffer.
    #[inline]
    pub fn read(&self) -> Result<T::Owned, ReadError> {
        self.read_impl::<T>(0 .. self.size())
    }

    /// Implementation of the `read` function. Takes a range as parameter so that it can be called
    /// as well from the `read` functions implemented on slices.
    ///
    /// # Panic
    ///
    /// Panicks if `range` is out of range.
    ///
    /// # Unsafe
    ///
    /// The caller must make sure that the content within `range` corresponds to an object of
    /// type `D`.
    unsafe fn read_impl<D: ?Sized>(&self, range: Range<usize>) -> Result<D::Owned, ReadError>
        where D: Content
    {
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
                Ok(())

            } else if ctxt.version >= &Version(Api::Gl, 1, 5) {
                let bind = bind_buffer(&mut ctxt, self.id, self.ty);
                ctxt.gl.GetBufferSubData(bind, range.start as gl::types::GLintptr,
                                         size_to_read as gl::types::GLsizeiptr,
                                         output as *mut _ as *mut _);
                Ok(())

            } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                let bind = bind_buffer(&mut ctxt, self.id, self.ty);
                ctxt.gl.GetBufferSubDataARB(bind, range.start as gl::types::GLintptr,
                                            size_to_read as gl::types::GLsizeiptr,
                                            output as *mut _ as *mut _);
                Ok(())

            } else if ctxt.version >= &Version(Api::GlEs, 1, 0) {
                Err(ReadError::NotSupported);

            } else {
                unreachable!()
            }
        })
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

buffers_base!(DynamicBuffer, DynamicBufferSlice, DynamicBufferMutSlice);

impl<'a, T: ?Sized> DynamicBufferSlice<'a, T> where T: Content {

    /// Uploads data in the buffer.
    ///
    /// The data must fit inside the buffer.
    ///
    /// # Panic
    ///
    /// Panics if `T` is unsized and the data is not the same size as the buffer.
    ///
    pub fn upload(&self, data: &T) {
    }
}

impl<'a, T: ?Sized> DynamicBufferMutSlice<'a, T> where T: Content {
    pub fn map(&mut self) -> Mapping<T> {
        self.buffer.map_impl(self.bytes_start .. self.bytes_end, true, true)
    }
}

/// A mapping of a buffer for reading and writing.
pub struct Mapping<'a, T: ?Sized> {
    context: Rc<Context>,
    id: gl::types::GLuint,
    ty: BufferType,
    data: *mut D,
    marker: PhantomData<&'a mut T>,
    is_mapped: &'a mut Cell<bool>,
}

unsafe impl<'a, T: ?Sized> Sync for Mapping<'a, T> where T: Send + Sync {}

impl<'a, T: ?Sized> Deref for Mapping<'a, T> where T: Content {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &*data
    }
}

impl<'a, T: ?Sized> DerefMut for Mapping<'a, T> where T: Content {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *data }
    }
}

impl<'a, T: ?Sized> Drop for Mapping<'a, T> {
    fn drop(&mut self) {
        let mut ctxt = self.context.make_current();

        unsafe {
            raw::flush_range(&mut ctxt, self.id, self.ty, 0 .. mem::size_of_val(&*data));
            raw::unmap_buffer(&mut ctxt, self.id, self.ty);
        }

        self.is_mapped.set(false);
    }
}

/// Creates a new buffer.
///
/// # Panic
///
/// Panics if `mem::size_of_val(&data) != size`.
unsafe fn create_buffer<D: ?Sized>(mut ctxt: &mut CommandContext, size: usize, data: Option<&D>,
                                   ty: BufferType, usage: gl::types::GLenum)
                                   -> Result<gl::types::GLuint, BufferCreationError>
    where D: Content
{
    if !raw::is_buffer_type_supported(ctxt, ty) {
        return Err(BufferCreationError::BufferTypeNotSupported);
    }

    if let Some(data) = data {
        assert!(mem::size_of_val(data) == size);
    }

    // creating the id of the buffer
    let id = raw::create_buffer_name(ctxt);

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

    // will store the actual size of the buffer so that we can compare it with the expected size
    let mut obtained_size: gl::types::GLint = mem::uninitialized();

    if ctxt.version >= &Version(Api::Gl, 1, 5) || ctxt.version >= &Version(Api::GlEs, 2, 0) {
        let bind = bind_buffer(&mut ctxt, id, ty);
        ctxt.gl.BufferData(bind, size as gl::types::GLsizeiptr, data_ptr as *const _, usage);
        ctxt.gl.GetBufferParameteriv(bind, gl::BUFFER_SIZE, &mut obtained_size);

    } else if ctxt.extensions.gl_arb_vertex_buffer_object {
        let bind = bind_buffer(&mut ctxt, id, ty);
        ctxt.gl.BufferDataARB(bind, size as gl::types::GLsizeiptr, data_ptr as *const _, usage);
        ctxt.gl.GetBufferParameterivARB(bind, gl::BUFFER_SIZE, &mut obtained_size);

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

    Ok(id)
}
