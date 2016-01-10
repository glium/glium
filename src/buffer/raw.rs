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

/// Returns true if a given buffer type is supported on a platform.
pub fn is_buffer_type_supported(ctxt: &mut CommandContext, ty: BufferType) -> bool {
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

        BufferType::DispatchIndirectBuffer => {
            ctxt.version >= &Version(Api::Gl, 4, 3) || ctxt.version >= &Version(Api::GlEs, 3, 1) ||
            ctxt.extensions.gl_arb_compute_shader
        },

        BufferType::TextureBuffer => {
            ctxt.version >= &Version(Api::Gl, 3, 0) ||
            ctxt.extensions.gl_arb_texture_buffer_object ||
            ctxt.extensions.gl_ext_texture_buffer_object ||
            ctxt.extensions.gl_ext_texture_buffer || ctxt.extensions.gl_oes_texture_buffer
        },

        BufferType::QueryBuffer => {
            ctxt.version >= &Version(Api::Gl, 4, 4) ||
            ctxt.extensions.gl_arb_query_buffer_object ||
            ctxt.extensions.gl_amd_query_buffer_object
        },

        _ => false,     // FIXME:
    }
}

/// Binds a buffer of the given type, and returns the GLenum of the bind point.
/// `id` can be 0.
///
/// ## Unsafety
///
/// Assumes that the type of buffer is supported by the backend.
pub unsafe fn bind_buffer(mut ctxt: &mut CommandContext, id: gl::types::GLuint, ty: BufferType)
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
pub unsafe fn indexed_bind_buffer(mut ctxt: &mut CommandContext, id: gl::types::GLuint,
                                  ty: BufferType, index: gl::types::GLuint, range: Range<usize>)
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
                    for _ in 0 .. 1 + ctxt.state.$state_var.len() - $input_index as usize {
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
///
/// # Safety
///
/// The buffer IDs must be valid. The offsets and size must be valid.
///
pub unsafe fn copy_buffer(ctxt: &mut CommandContext, source: gl::types::GLuint,
                          source_offset: usize, dest: gl::types::GLuint, dest_offset: usize,
                          size: usize) -> Result<(), CopyError>
{
    if ctxt.version >= &Version(Api::Gl, 4, 5) || ctxt.extensions.gl_arb_direct_state_access {
        ctxt.gl.CopyNamedBufferSubData(source, dest, source_offset as gl::types::GLintptr,
                                       dest_offset as gl::types::GLintptr,
                                       size as gl::types::GLsizeiptr);

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
        return Err(CopyError::NotSupported);
    }

    Ok(())
}

/// Destroys a buffer.
pub unsafe fn destroy_buffer(mut ctxt: &mut CommandContext, id: gl::types::GLuint) {
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
pub unsafe fn flush_range(mut ctxt: &mut CommandContext, id: gl::types::GLuint, ty: BufferType,
                          range: Range<usize>)
{
    if ctxt.version >= &Version(Api::Gl, 4, 5) || ctxt.extensions.gl_arb_direct_state_access {
        ctxt.gl.FlushMappedNamedBufferRange(id, range.start as gl::types::GLintptr,
                                            (range.end - range.start) as gl::types::GLsizeiptr);

    } else if ctxt.extensions.gl_ext_direct_state_access {
        ctxt.gl.FlushMappedNamedBufferRangeEXT(id, range.start as gl::types::GLintptr,
                                               (range.end - range.start) as gl::types::GLsizeiptr);

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
pub unsafe fn map_buffer(mut ctxt: &mut CommandContext, id: gl::types::GLuint, ty: BufferType,
                         range: Range<usize>, read: bool, write: bool) -> Option<*mut ()>
{
    let flags = match (read, write) {
        (true, true) => gl::MAP_FLUSH_EXPLICIT_BIT | gl::MAP_READ_BIT | gl::MAP_WRITE_BIT,
        (true, false) => gl::MAP_READ_BIT,
        (false, true) => gl::MAP_FLUSH_EXPLICIT_BIT | gl::MAP_WRITE_BIT,
        (false, false) => 0,
    };

    if ctxt.version >= &Version(Api::Gl, 4, 5) {
        Some(ctxt.gl.MapNamedBufferRange(id, range.start as gl::types::GLintptr,
                                         (range.end - range.start) as gl::types::GLsizeiptr,
                                         flags) as *mut ())

    } else if ctxt.version >= &Version(Api::Gl, 3, 0) ||
        ctxt.version >= &Version(Api::GlEs, 3, 0) ||
        ctxt.extensions.gl_arb_map_buffer_range
    {
        let bind = bind_buffer(&mut ctxt, id, ty);
        Some(ctxt.gl.MapBufferRange(bind, range.start as gl::types::GLintptr,
                                    (range.end - range.start) as gl::types::GLsizeiptr,
                                    flags) as *mut ())

    } else {
        None       // FIXME:
    }
}

/// Unmaps a previously-mapped buffer.
///
/// # Safety
///
/// Assumes that the buffer exists, that it is of the right type, and that it is already mapped.
pub unsafe fn unmap_buffer(mut ctxt: &mut CommandContext, id: gl::types::GLuint, ty: BufferType) {
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
