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

/// A buffer whose content is inaccessible from the CPU. In other words, whose content is
/// inaccessible from your application.
///
/// The fact that the data is not accessible often gives a performance boost, for two reasons:
///
/// 1) The driver doesn't have to worry about synchronization, as everything that touches the
///    buffer executes serially.
/// 2) The data can be stored in memory that is inaccessible from the CPU. The GPU can usually read
///    faster from this kind of memory.
///
/// There is no way to modify an immutable buffer's content, except by copying data from another
/// buffer or by issuing rendering commands.
pub struct ImmutableBuffer<T: ?Sized> where T: Content {
    marker: PhantomData<T>,

    context: Rc<Context>,

    /// OpenGL identifier ; can't be zero.
    id: gl::types::GLuint,

    /// Type of buffer.
    ty: BufferType,

    /// Size in bytes of the buffer.
    size: usize,
}

impl<T: ?Sized> ImmutableBuffer<T> where T: Content {
    pub fn new<F>(facade: &F, data: &T, ty: BufferType)
                  -> Result<ImmutableBuffer<T>, BufferCreationError>
        where F: Facade
    {
        let mut ctxt = facade.get_context().make_current();

        let size = mem::size_of_val(data);
        let id = try!(unsafe { create_buffer(&mut ctxt, size, Some(data), ty) });

        Ok(ImmutableBuffer {
            marker: PhantomData,
            context: facade.get_context().clone(),
            id: id,
            ty: ty,
            size: size,
        })
    }

    pub fn empty<F>(facade: &F, ty: BufferType) -> Result<ImmutableBuffer<T>, BufferCreationError>
        where F: Facade, T: Sized, T: Copy
    {
        let mut ctxt = facade.get_context().make_current();

        let size = mem::size_of::<T>();
        let id = try!(unsafe { create_buffer::<()>(&mut ctxt, size, None, ty) });

        Ok(ImmutableBuffer {
            marker: PhantomData,
            context: facade.get_context().clone(),
            id: id,
            ty: ty,
            size: size,
        })
    }

    pub fn empty_array<F>(facade: &F, len: usize, ty: BufferType)
                          -> Result<ImmutableBuffer<T>, BufferCreationError>
        where F: Facade, T: ArrayContent
    {
        let mut ctxt = facade.get_context().make_current();

        let size = len * <T as ArrayContent>::element_size();
        let id = try!(unsafe { create_buffer::<()>(&mut ctxt, size, None, ty) });

        Ok(ImmutableBuffer {
            marker: PhantomData,
            context: facade.get_context().clone(),
            id: id,
            ty: ty,
            size: size,
        })
    }

    pub fn empty_unsized<F>(facade: &F, size: usize, ty: BufferType)
                            -> Result<ImmutableBuffer<T>, BufferCreationError>
        where F: Facade, T: Copy
    {
        let mut ctxt = facade.get_context().make_current();
        let id = try!(unsafe { create_buffer::<()>(&mut ctxt, size, None, ty) });

        Ok(ImmutableBuffer {
            marker: PhantomData,
            context: facade.get_context().clone(),
            id: id,
            ty: ty,
            size: size,
        })
    }

    /// See the `CopyTo` trait.
    pub fn copy_to<S>(&self, target: &S) -> Result<(), CopyError>
        where S: Storage
    {
        // TODO: check that the other buffer belongs to the same context
        let mut ctxt = self.context.make_current();

        unimplemented!()
        /*unsafe {
            raw::copy_buffer(&mut ctxt, self.id, range.start, target.id, dest_offset,
                             range.end - range.start)
        }*/
    }

    /// See the `Invalidate` trait.
    pub fn invalidate(&self) {
        let mut ctxt = self.context.make_current();

        if ctxt.version >= &Version(Api::Gl, 4, 3) || ctxt.extensions.gl_arb_invalidate_subdata {
            unsafe {
                ctxt.gl.InvalidateBufferData(self.id);
            }
        }
    }
}

impl<T: ?Sized> Drop for ImmutableBuffer<T> where T: Content {
    fn drop(&mut self) {
        unsafe {
            let mut ctxt = self.context.make_current();
            unimplemented!();//self.assert_not_transform_feedback(&mut ctxt);        // TODO:
            VertexAttributesSystem::purge_buffer(&mut ctxt, self.id);
            raw::destroy_buffer(&mut ctxt, self.id);
        }
    }
}

impl<'a, T: ?Sized> ImmutableBufferSlice<'a, T> where T: Content {
    /// See the `Invalidate` trait.
    pub fn invalidate(&self) {
        let mut ctxt = self.context.make_current();

        if ctxt.version >= &Version(Api::Gl, 4, 3) || ctxt.extensions.gl_arb_invalidate_subdata {
            unsafe {
                ctxt.gl.InvalidateBufferSubData(self.buffer, self.bytes_start as gl::types::GLintptr,
                                                self.size() as gl::types::GLsizeiptr);
            }
        }
    }
}

impl<'a, T: ?Sized> ImmutableBufferMutSlice<'a, T> where T: Content {
    /// See the `Invalidate` trait.
    pub fn invalidate(&self) {
        let mut ctxt = self.context.make_current();

        if ctxt.version >= &Version(Api::Gl, 4, 3) || ctxt.extensions.gl_arb_invalidate_subdata {
            unsafe {
                ctxt.gl.InvalidateBufferSubData(self.buffer, self.bytes_start as gl::types::GLintptr,
                                                self.size() as gl::types::GLsizeiptr);
            }
        }
    }
}

impl<T: ?Sized> Storage for ImmutableBuffer<T> where T: Content {
    type Content = T;

    fn as_slice_any(&self) -> BufferAnySlice {
        unimplemented!()
    }

    #[inline]
    fn size(&self) -> usize {
        self.size
    }
}

impl<T: ?Sized> Invalidate for ImmutableBuffer<T> where T: Content {
    #[inline]
    fn invalidate(&self) {
        self.invalidate()
    }
}

impl<'a, T: ?Sized + 'a> Invalidate for ImmutableBufferSlice<'a, T> where T: Content {
    #[inline]
    fn invalidate(&self) {
        self.invalidate()
    }
}

impl<'a, T: ?Sized + 'a> Invalidate for ImmutableBufferMutSlice<'a, T> where T: Content {
    #[inline]
    fn invalidate(&self) {
        self.invalidate()
    }
}

impl<T: ?Sized> Create for ImmutableBuffer<T> where T: Content {
    #[inline]
    fn new<F>(facade: &F, data: &T, ty: BufferType)
              -> Result<ImmutableBuffer<T>, BufferCreationError>
        where F: Facade
    {
        ImmutableBuffer::new(facade, data, ty)
    }

    #[inline]
    fn empty<F>(facade: &F, ty: BufferType)
                -> Result<ImmutableBuffer<T>, BufferCreationError>
        where F: Facade, T: Copy
    {
        ImmutableBuffer::empty(facade, ty)
    }

    #[inline]
    fn empty_array<F>(facade: &F, len: usize, ty: BufferType)
                      -> Result<ImmutableBuffer<T>, BufferCreationError>
        where F: Facade, T: ArrayContent
    {
        ImmutableBuffer::empty_array(facade, len, ty)
    }

    #[inline]
    fn empty_unsized<F>(facade: &F, size: usize, ty: BufferType)
                        -> Result<ImmutableBuffer<T>, BufferCreationError>
        where F: Facade, T: Copy
    {
        ImmutableBuffer::empty_unsized(facade, size, ty)
    }
}

impl<T: ?Sized> CopyTo for ImmutableBuffer<T> where T: Content {
    #[inline]
    fn copy_to<S>(&self, target: &S) -> Result<(), CopyError>
        where S: Storage
    {
        self.copy_to(target)
    }
}

buffers_base!(ImmutableBuffer, ImmutableBufferSlice, ImmutableBufferMutSlice);

/// Creates a new buffer.
///
/// # Panic
///
/// Panics if `mem::size_of_val(&data) != size`.
unsafe fn create_buffer<D: ?Sized>(mut ctxt: &mut CommandContext, size: usize, data: Option<&D>,
                                   ty: BufferType) -> Result<gl::types::GLuint, BufferCreationError>
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

    if ctxt.version >= &Version(Api::Gl, 4, 5) || ctxt.extensions.gl_arb_direct_state_access {
        ctxt.gl.NamedBufferStorage(id, size as gl::types::GLsizeiptr, data_ptr as *const _, 0);
        ctxt.gl.GetNamedBufferParameteriv(id, gl::BUFFER_SIZE, &mut obtained_size);

    } else if ctxt.extensions.gl_arb_buffer_storage &&
              ctxt.extensions.gl_ext_direct_state_access
    {
        ctxt.gl.NamedBufferStorageEXT(id, size as gl::types::GLsizeiptr, data_ptr as *const _, 0);
        ctxt.gl.GetNamedBufferParameterivEXT(id, gl::BUFFER_SIZE, &mut obtained_size);

    } else if ctxt.version >= &Version(Api::Gl, 4, 4) ||
              ctxt.extensions.gl_arb_buffer_storage
    {
        let bind = raw::bind_buffer(&mut ctxt, id, ty);
        ctxt.gl.BufferStorage(bind, size as gl::types::GLsizeiptr, data_ptr as *const _, 0);
        ctxt.gl.GetBufferParameteriv(bind, gl::BUFFER_SIZE, &mut obtained_size);

    } else if ctxt.extensions.gl_ext_buffer_storage {
        let bind = raw::bind_buffer(&mut ctxt, id, ty);
        ctxt.gl.BufferStorageEXT(bind, size as gl::types::GLsizeiptr, data_ptr as *const _, 0);
        ctxt.gl.GetBufferParameteriv(bind, gl::BUFFER_SIZE, &mut obtained_size);

    } else {
        // FIXME: return error instead
        panic!()
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
