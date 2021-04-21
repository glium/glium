use std::ptr;

use crate::BufferExt;
use crate::BufferSliceExt;
use crate::ProgramExt;
use crate::DrawError;
use crate::UniformsExt;

use crate::context::Context;
use crate::ContextExt;
use crate::TransformFeedbackSessionExt;

use crate::fbo::{self, ValidatedAttachments};

use crate::uniforms::Uniforms;
use crate::{Program, ToGlEnum};
use crate::index::{self, IndicesSource};
use crate::vertex::{MultiVerticesSource, VerticesSource, TransformFeedbackSession};
use crate::vertex_array_object::VertexAttributesSystem;

use crate::draw_parameters::DrawParameters;

use crate::{gl, context, draw_parameters};
use crate::version::Version;
use crate::version::Api;

/// Draws everything.
pub fn draw<'a, U, V>(context: &Context, framebuffer: Option<&ValidatedAttachments<'_>>,
                      vertex_buffers: V, indices: IndicesSource<'_>,
                      program: &Program, uniforms: &U, draw_parameters: &DrawParameters<'_>,
                      dimensions: (u32, u32)) -> Result<(), DrawError>
                      where U: Uniforms, V: MultiVerticesSource<'a>
{
    // this contains the list of fences that will need to be fulfilled after the draw command
    // has started
    let mut fences = Vec::with_capacity(0);

    // handling tessellation
    let vertices_per_patch = match indices.get_primitives_type() {
        index::PrimitiveType::Patches { vertices_per_patch } => {
            if let Some(max) = context.capabilities().max_patch_vertices {
                if vertices_per_patch == 0 || vertices_per_patch as gl::types::GLint > max {
                    return Err(DrawError::UnsupportedVerticesPerPatch);
                }
            } else {
                return Err(DrawError::TessellationNotSupported);
            }

            // TODO: programs created from binaries have the wrong value
            // for `has_tessellation_shaders`
            /*if !program.has_tessellation_shaders() {    // TODO:
                panic!("Default tessellation level is not supported yet");
            }*/

            Some(vertices_per_patch)
        },
        _ => {
            // TODO: programs created from binaries have the wrong value
            // for `has_tessellation_shaders`
            /*if program.has_tessellation_shaders() {
                return Err(DrawError::TessellationWithoutPatches);
            }*/

            None
        },
    };

    // starting the state changes
    let mut ctxt = context.make_current();

    // handling vertices source
    let (vertices_count, instances_count, base_vertex) = {
        let index_buffer = match indices {
            IndicesSource::IndexBuffer { buffer, .. } => Some(buffer),
            IndicesSource::MultidrawArray { .. } => None,
            IndicesSource::MultidrawElement { indices, .. } => Some(indices),
            IndicesSource::NoIndices { .. } => None,
        };

        // determining whether we can use the `base_vertex` variants for drawing
        let use_base_vertex = match indices {
            IndicesSource::MultidrawArray { .. } => false,
            IndicesSource::MultidrawElement { .. } => false,
            IndicesSource::NoIndices { .. } => true,
            _ => ctxt.version >= &Version(Api::Gl, 3, 2) ||
                 ctxt.version >= &Version(Api::GlEs, 3, 2) ||
                 ctxt.extensions.gl_arb_draw_elements_base_vertex ||
                 ctxt.extensions.gl_oes_draw_elements_base_vertex
        };

        // object that is used to build the bindings
        let mut binder = VertexAttributesSystem::start(&mut ctxt, program, index_buffer,
                                                       use_base_vertex);
        // number of vertices in the vertices sources, or `None` if there is a mismatch
        let mut vertices_count: Option<usize> = None;
        // number of instances to draw
        let mut instances_count: Option<usize> = None;

        for src in vertex_buffers.iter() {
            // Allow single match for consistency with the match below.
            // Integrating the two matches wouldn't improve the code either.
            #[allow(clippy::single_match)]
            match src {
                VerticesSource::VertexBuffer(buffer, format, per_instance) => {
                    // TODO: assert!(buffer.get_elements_size() == total_size(format));

                    if let Some(fence) = buffer.add_fence() {
                        fences.push(fence);
                    }

                    binder = binder.add(&buffer, format, if per_instance { Some(1) } else { None });
                },
                _ => {}
            }

            match src {
                VerticesSource::VertexBuffer(ref buffer, _, false) => {
                    if let Some(curr) = vertices_count {
                        if curr != buffer.get_elements_count() {
                            vertices_count = None;
                            break;
                        }
                    } else {
                        vertices_count = Some(buffer.get_elements_count());
                    }
                },
                VerticesSource::VertexBuffer(ref buffer, _, true) => {
                    if let Some(curr) = instances_count {
                        if curr != buffer.get_elements_count() {
                            return Err(DrawError::InstancesCountMismatch);
                        }
                    } else {
                        instances_count = Some(buffer.get_elements_count());
                    }
                },
                VerticesSource::Marker { len, per_instance } if !per_instance => {
                    if let Some(curr) = vertices_count {
                        if curr != len {
                            vertices_count = None;
                            break;
                        }
                    } else {
                        vertices_count = Some(len);
                    }
                },
                VerticesSource::Marker { len, per_instance } if per_instance => {
                    if let Some(curr) = instances_count {
                        if curr != len {
                            return Err(DrawError::InstancesCountMismatch);
                        }
                    } else {
                        instances_count = Some(len);
                    }
                },
                _ => ()
            }
        }

        (vertices_count, instances_count, binder.bind().unwrap_or(0))
    };

    // binding the FBO to draw upon
    {
        let fbo_id = fbo::FramebuffersContainer::get_framebuffer_for_drawing(&mut ctxt, framebuffer);
        unsafe { fbo::bind_framebuffer(&mut ctxt, fbo_id, true, false) };
    };

    // binding the program and uniforms
    program.use_program(&mut ctxt);
    uniforms.bind_uniforms(&mut ctxt, program, &mut fences)?;

    // sync-ing draw_parameters
    unsafe {
        draw_parameters::sync(&mut ctxt, draw_parameters, dimensions, indices.get_primitives_type())?;
        sync_vertices_per_patch(&mut ctxt, vertices_per_patch);

        // TODO: make sure that the program is the right one
        // TODO: changing the current transform feedback requires pausing/unbinding before changing the program
        if let Some(ref tf) = draw_parameters.transform_feedback {
            tf.bind(&mut ctxt, indices.get_primitives_type());
        } else {
            TransformFeedbackSession::unbind(&mut ctxt);
        }
    }

    // drawing
    // TODO: make this code more readable
    {
        match &indices {
            IndicesSource::IndexBuffer { ref buffer, data_type, primitives } => {
                let ptr: *const u8 = ptr::null_mut();
                let ptr = unsafe { ptr.add(buffer.get_offset_bytes()) };

                if let Some(fence) = buffer.add_fence() {
                    fences.push(fence);
                }

                unsafe {
                    if let Some(instances_count) = instances_count {
                        if base_vertex != 0 {
                            if ctxt.version >= &Version(Api::Gl, 3, 2) ||
                               ctxt.version >= &Version(Api::GlEs, 3, 2) ||
                               ctxt.extensions.gl_arb_draw_elements_base_vertex
                            {
                                ctxt.gl.DrawElementsInstancedBaseVertex(primitives.to_glenum(),
                                                                     buffer.get_elements_count() as
                                                                        gl::types::GLsizei,
                                                                        data_type.to_glenum(),
                                                                        ptr as *const _,
                                                                        instances_count as
                                                                        gl::types::GLsizei,
                                                                        base_vertex);

                            } else if ctxt.extensions.gl_oes_draw_elements_base_vertex {
                                ctxt.gl.DrawElementsInstancedBaseVertexOES(primitives.to_glenum(),
                                                                     buffer.get_elements_count() as
                                                                           gl::types::GLsizei,
                                                                           data_type.to_glenum(),
                                                                        ptr as *const _,
                                                                           instances_count as
                                                                           gl::types::GLsizei,
                                                                           base_vertex);
                            } else {
                                unreachable!();
                            }

                        } else {
                            ctxt.gl.DrawElementsInstanced(primitives.to_glenum(),
                                                          buffer.get_elements_count() as
                                                          gl::types::GLsizei,
                                                          data_type.to_glenum(),
                                                          ptr as *const _,
                                                          instances_count as gl::types::GLsizei);
                        }

                    } else if base_vertex != 0 {
                        if ctxt.version >= &Version(Api::Gl, 3, 2) ||
                           ctxt.version >= &Version(Api::GlEs, 3, 2) ||
                           ctxt.extensions.gl_arb_draw_elements_base_vertex
                        {
                            ctxt.gl.DrawElementsBaseVertex(primitives.to_glenum(),
                                                           buffer.get_elements_count() as
                                                           gl::types::GLsizei,
                                                           data_type.to_glenum(),
                                                           ptr as *const _,
                                                           base_vertex);

                        } else if ctxt.extensions.gl_oes_draw_elements_base_vertex {
                            ctxt.gl.DrawElementsBaseVertexOES(primitives.to_glenum(),
                                                              buffer.get_elements_count() as
                                                              gl::types::GLsizei,
                                                              data_type.to_glenum(),
                                                              ptr as *const _,
                                                              base_vertex);
                        } else {
                            unreachable!();
                        }

                    } else {
                        ctxt.gl.DrawElements(primitives.to_glenum(),
                                             buffer.get_elements_count() as gl::types::GLsizei,
                                             data_type.to_glenum(),
                                             ptr as *const _);
                    }
                }
            },

            IndicesSource::MultidrawArray { ref buffer, primitives } => {
                let ptr: *const u8 = ptr::null_mut();
                let ptr = unsafe { ptr.add(buffer.get_offset_bytes()) };

                debug_assert_eq!(base_vertex, 0);       // enforced earlier in this function

                if let Some(fence) = buffer.add_fence() {
                    fences.push(fence);
                }

                unsafe {
                    buffer.prepare_and_bind_for_draw_indirect(&mut ctxt);
                    ctxt.gl.MultiDrawArraysIndirect(primitives.to_glenum(), ptr as *const _,
                                                    buffer.get_elements_count() as gl::types::GLsizei,
                                                    0);
                }
            },

            IndicesSource::MultidrawElement { ref commands, ref indices, data_type, primitives } => {
                let cmd_ptr: *const u8 = ptr::null_mut();
                let cmd_ptr = unsafe { cmd_ptr.add(commands.get_offset_bytes()) };

                if let Some(fence) = commands.add_fence() {
                    fences.push(fence);
                }

                if let Some(fence) = indices.add_fence() {
                    fences.push(fence);
                }

                unsafe {
                    commands.prepare_and_bind_for_draw_indirect(&mut ctxt);
                    debug_assert_eq!(base_vertex, 0);       // enforced earlier in this function
                    ctxt.gl.MultiDrawElementsIndirect(primitives.to_glenum(), data_type.to_glenum(),
                                                      cmd_ptr as *const _,
                                                      commands.get_elements_count() as gl::types::GLsizei,
                                                      0);
                }
            },

            IndicesSource::NoIndices { primitives } => {
                let vertices_count = match vertices_count {
                    Some(c) => c,
                    None => return Err(DrawError::VerticesSourcesLengthMismatch)
                };

                unsafe {
                    if let Some(instances_count) = instances_count {
                        ctxt.gl.DrawArraysInstanced(primitives.to_glenum(), base_vertex,
                                                    vertices_count as gl::types::GLsizei,
                                                    instances_count as gl::types::GLsizei);
                    } else {
                        ctxt.gl.DrawArrays(primitives.to_glenum(), base_vertex,
                                           vertices_count as gl::types::GLsizei);
                    }
                }
            },
        };
    };

    ctxt.state.next_draw_call_id += 1;

    // fulfilling the fences
    for fence in fences.into_iter() {
        fence.insert(&mut ctxt);
    }

    Ok(())
}

unsafe fn sync_vertices_per_patch(ctxt: &mut context::CommandContext<'_>, vertices_per_patch: Option<u16>) {
    if let Some(vertices_per_patch) = vertices_per_patch {
        let vertices_per_patch = vertices_per_patch as gl::types::GLint;
        if ctxt.state.patch_patch_vertices != vertices_per_patch {
            ctxt.gl.PatchParameteri(gl::PATCH_VERTICES, vertices_per_patch);
            ctxt.state.patch_patch_vertices = vertices_per_patch;
        }
    }
}
