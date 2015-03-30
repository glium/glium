use std::ptr;
use std::sync::mpsc::Sender;
use std::collections::hash_state::DefaultState;
use std::collections::HashMap;

use BufferExt;
use DrawError;
use Handle;

use context::Context;
use ContextExt;

use fbo::{self, FramebufferAttachments};

use sync;
use uniforms::{Uniforms, UniformValue, SamplerBehavior};
use sampler_object::SamplerObject;
use {Program, GlObject, ToGlEnum};
use index::{self, IndicesSource};
use vertex::{MultiVerticesSource, VerticesSource};

use draw_parameters::DrawParameters;
use draw_parameters::{BlendingFunction, BackfaceCullingMode};
use draw_parameters::{DepthTest, PolygonMode};
use Rect;

use program;
use libc;
use util;
use {gl, context, draw_parameters};
use version::Version;
use version::Api;

/// Draws everything.
pub fn draw<'a, I, U, V>(context: &Context, framebuffer: Option<&FramebufferAttachments>,
                         vertex_buffers: V, mut indices: IndicesSource<I>,
                         program: &Program, uniforms: U, draw_parameters: &DrawParameters,
                         dimensions: (u32, u32)) -> Result<(), DrawError>
                         where U: Uniforms, I: index::Index, V: MultiVerticesSource<'a>
{
    // TODO: avoid this allocation
    let mut vertex_buffers = vertex_buffers.iter().collect::<Vec<_>>();

    try!(draw_parameters::validate(context, draw_parameters));

    // using a base vertex is not yet supported
    // TODO: 
    for src in vertex_buffers.iter() {
        match src {
            &VerticesSource::VertexBuffer(_, offset, _) => {
                if offset != 0 {
                    panic!("Using a base vertex different from 0 is not yet implemented");
                }
            },
            _ => ()
        }
    }

    // getting the number of vertices in the vertices sources, or `None` if there is a
    // mismatch
    let vertices_count = {
        let mut vertices_count: Option<usize> = None;
        for src in vertex_buffers.iter() {
            match src {
                &VerticesSource::VertexBuffer(_, _, len) => {
                    if let Some(curr) = vertices_count {
                        if curr != len {
                            vertices_count = None;
                            break;
                        }
                    } else {
                        vertices_count = Some(len);
                    }
                },
                _ => ()
            }
        }
        vertices_count
    };

    // getting the number of instances to draw
    let instances_count = {
        let mut instances_count: Option<usize> = None;
        for src in vertex_buffers.iter() {
            match src {
                &VerticesSource::PerInstanceBuffer(ref buffer) => {
                    if let Some(curr) = instances_count {
                        if curr != buffer.len() {
                            return Err(DrawError::InstancesCountMismatch);
                        }
                    } else {
                        instances_count = Some(buffer.len());
                    }
                },
                _ => ()
            }
        }
        instances_count
    };

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

    // sending the command
    let mut ctxt = context.make_current();

    // binding the vertex array object or vertex attributes
    context.vertex_array_objects.bind_vao(&mut ctxt,
                                          &vertex_buffers.iter().map(|v| v).collect::<Vec<_>>(),
                                          &indices, program);

    // binding the FBO to draw upon
    {
        let fbo_id = context.framebuffer_objects.as_ref().unwrap()
                            .get_framebuffer_for_drawing(framebuffer, &mut ctxt);
        fbo::bind_framebuffer(&mut ctxt, fbo_id, true, false);
    };

    // binding the program
    unsafe {
        let program_id = program.get_id();
        if ctxt.state.program != program_id {
            match program_id {
                Handle::Id(id) => ctxt.gl.UseProgram(id),
                Handle::Handle(id) => ctxt.gl.UseProgramObjectARB(id),
            }
            ctxt.state.program = program_id;
        }
    }

    // building the list of uniforms binders and the fences that must be fulfilled
    // TODO: panic if uniforms of the program are not found in the parameter
    let fences = {
        let mut active_texture = 0;
        let mut active_buffer_binding = 0;

        let mut fences = Vec::new();

        let mut visiting_result = Ok(());
        uniforms.visit_values(|name, value| {
            if visiting_result.is_err() { return; }

            if let Some(uniform) = program.get_uniform(name) {
                assert!(uniform.size.is_none(), "Uniform arrays not supported yet");

                if !value.is_usable_with(&uniform.ty) {
                    visiting_result = Err(DrawError::UniformTypeMismatch {
                        name: name.to_string(),
                        expected: uniform.ty,
                    });
                    return;
                }

                match bind_uniform(&mut ctxt, &mut context.samplers.borrow_mut(),
                                   value, uniform.location,
                                   &mut active_texture, name)
                {
                    Ok(_) => (),
                    Err(e) => {
                        visiting_result = Err(e);
                        return;
                    }
                };

            } else if let Some(block) = program.get_uniform_blocks().get(name) {
                let fence = match bind_uniform_block(&mut ctxt, value, block,
                                                     program.get_id(),
                                                    &mut active_buffer_binding, name)
                {
                    Ok(f) => f,
                    Err(e) => {
                        visiting_result = Err(e);
                        return;
                    }
                };

                if let Some(fence) = fence {
                    fences.push(fence);
                }
            }
        });

        if let Err(e) = visiting_result {
            return Err(e);
        }

        // adding the vertex buffer and index buffer to the list of fences
        for vertex_buffer in vertex_buffers.iter_mut() {
            match vertex_buffer {
                &mut VerticesSource::VertexBuffer(ref buffer, _, _) => {
                    if let Some(fence) = buffer.add_fence() {
                        fences.push(fence);
                    }
                }
                &mut VerticesSource::PerInstanceBuffer(ref buffer) => {
                    if let Some(fence) = buffer.add_fence() {
                        fences.push(fence);
                    }
                }
            };
        }
        match &mut indices {
            &mut IndicesSource::IndexBuffer { ref buffer, .. } => {
                if let Some(fence) = buffer.add_fence() {
                    fences.push(fence);
                }
            },
            _ => ()
        };

        fences
    };

    // sync-ing draw_parameters
    unsafe {
        sync_depth(&mut ctxt, draw_parameters.depth_test, draw_parameters.depth_write,
                   draw_parameters.depth_range);
        sync_blending(&mut ctxt, draw_parameters.blending_function);
        sync_line_width(&mut ctxt, draw_parameters.line_width);
        sync_point_size(&mut ctxt, draw_parameters.point_size);
        sync_polygon_mode(&mut ctxt, draw_parameters.backface_culling, draw_parameters.polygon_mode);
        sync_multisampling(&mut ctxt, draw_parameters.multisampling);
        sync_dithering(&mut ctxt, draw_parameters.dithering);
        sync_viewport_scissor(&mut ctxt, draw_parameters.viewport, draw_parameters.scissor,
                              dimensions);
        sync_rasterizer_discard(&mut ctxt, draw_parameters.draw_primitives);
        sync_vertices_per_patch(&mut ctxt, vertices_per_patch);
    }

    // drawing
    {
        match &indices {
            &IndicesSource::IndexBuffer { ref buffer, offset, length, .. } => {
                let ptr: *const u8 = ptr::null_mut();
                let ptr = unsafe { ptr.offset((offset * buffer.get_indices_type().get_size()) as isize) };

                unsafe {
                    if let Some(instances_count) = instances_count {
                        ctxt.gl.DrawElementsInstanced(buffer.get_primitives_type().to_glenum(),
                                                      length as gl::types::GLsizei,
                                                      buffer.get_indices_type().to_glenum(),
                                                      ptr as *const libc::c_void,
                                                      instances_count as gl::types::GLsizei);
                    } else {
                        ctxt.gl.DrawElements(buffer.get_primitives_type().to_glenum(),
                                             length as gl::types::GLsizei,
                                             buffer.get_indices_type().to_glenum(),
                                             ptr as *const libc::c_void);
                    }
                }
            },

            &IndicesSource::Buffer { ref pointer, primitives, offset, length } => {
                assert!(offset == 0);       // not yet implemented

                unsafe {
                    if let Some(instances_count) = instances_count {
                        ctxt.gl.DrawElementsInstanced(primitives.to_glenum(),
                                                      length as gl::types::GLsizei,
                                                      <I as index::Index>::get_type().to_glenum(),
                                                      pointer.as_ptr() as *const gl::types::GLvoid,
                                                      instances_count as gl::types::GLsizei);
                    } else {
                        ctxt.gl.DrawElements(primitives.to_glenum(), length as gl::types::GLsizei,
                                             <I as index::Index>::get_type().to_glenum(),
                                             pointer.as_ptr() as *const gl::types::GLvoid);
                    }
                }
            },

            &IndicesSource::NoIndices { primitives } => {
                let vertices_count = match vertices_count {
                    Some(c) => c,
                    None => return Err(DrawError::VerticesSourcesLengthMismatch)
                };

                unsafe {
                    if let Some(instances_count) = instances_count {
                        ctxt.gl.DrawArraysInstanced(primitives.to_glenum(), 0,
                                                    vertices_count as gl::types::GLsizei,
                                                    instances_count as gl::types::GLsizei);
                    } else {
                        ctxt.gl.DrawArrays(primitives.to_glenum(), 0,
                                           vertices_count as gl::types::GLsizei);
                    }
                }
            },
        };
    };

    unsafe {
        // fulfilling the fences
        for fence in fences.into_iter() {
            fence.send(sync::new_linear_sync_fence_if_supported(&mut ctxt).unwrap()).unwrap();
        }
    }

    Ok(())
}

fn bind_uniform_block(ctxt: &mut context::CommandContext, value: &UniformValue,
                      block: &program::UniformBlock,
                      program: Handle, current_bind_point: &mut gl::types::GLuint, name: &str)
                      -> Result<Option<Sender<sync::LinearSyncFence>>, DrawError>
{
    match value {
        &UniformValue::Block(ref buffer, ref layout) => {
            if !layout(block) {
                return Err(DrawError::UniformBlockLayoutMismatch { name: name.to_string() });
            }

            let bind_point = *current_bind_point;
            *current_bind_point += 1;

            let fence = buffer.add_fence();
            let buffer = buffer.get_id();
            let binding = block.binding as gl::types::GLuint;

            let program = match program {
                Handle::Id(id) => id,
                _ => unreachable!()
            };

            unsafe {
                ctxt.gl.BindBufferBase(gl::UNIFORM_BUFFER, bind_point as gl::types::GLuint,
                                       buffer);
                ctxt.gl.UniformBlockBinding(program, binding,
                                            bind_point as gl::types::GLuint);
            }

            Ok(fence)
        },
        _ => {
            Err(DrawError::UniformValueToBlock { name: name.to_string() })
        }
    }
}

fn bind_uniform(ctxt: &mut context::CommandContext,
                samplers: &mut HashMap<SamplerBehavior, SamplerObject,
                                       DefaultState<util::FnvHasher>>,
                value: &UniformValue, location: gl::types::GLint,
                active_texture: &mut gl::types::GLenum, name: &str)
                -> Result<(), DrawError>
{
    macro_rules! uniform(
        ($ctxt:expr, $uniform:ident, $uniform_arb:ident, $($params:expr),+) => (
            unsafe {
                if $ctxt.version >= &Version(Api::Gl, 1, 5) {
                    $ctxt.gl.$uniform($($params),+)
                } else {
                    assert!($ctxt.extensions.gl_arb_shader_objects);
                    $ctxt.gl.$uniform_arb($($params),+)
                }
            }
        )
    );

    match *value {
        UniformValue::Block(_, _) => {
            Err(DrawError::UniformBufferToValue {
                name: name.to_string(),
            })
        },
        UniformValue::SignedInt(val) => {
            uniform!(ctxt, Uniform1i, Uniform1iARB, location, val);
            Ok(())
        },
        UniformValue::UnsignedInt(val) => {
            // Uniform1uiARB doesn't exist
            unsafe {
                if ctxt.version >= &Version(Api::Gl, 1, 5) {
                    ctxt.gl.Uniform1ui(location, val)
                } else {
                    assert!(ctxt.extensions.gl_arb_shader_objects);
                    ctxt.gl.Uniform1iARB(location, val as gl::types::GLint)
                }
            }

            Ok(())
        },
        UniformValue::Float(val) => {
            uniform!(ctxt, Uniform1f, Uniform1fARB, location, val);
            Ok(())
        },
        UniformValue::Mat2(val) => {
            uniform!(ctxt, UniformMatrix2fv, UniformMatrix2fvARB,
                     location, 1, gl::FALSE, val.as_ptr() as *const f32);
            Ok(())
        },
        UniformValue::Mat3(val) => {
            uniform!(ctxt, UniformMatrix3fv, UniformMatrix3fvARB,
                     location, 1, gl::FALSE, val.as_ptr() as *const f32);
            Ok(())
        },
        UniformValue::Mat4(val) => {
            uniform!(ctxt, UniformMatrix4fv, UniformMatrix4fvARB,
                     location, 1, gl::FALSE, val.as_ptr() as *const f32);
            Ok(())
        },
        UniformValue::Vec2(val) => {
            uniform!(ctxt, Uniform2fv, Uniform2fvARB, location, 1, val.as_ptr() as *const f32);
            Ok(())
        },
        UniformValue::Vec3(val) => {
            uniform!(ctxt, Uniform3fv, Uniform3fvARB, location, 1, val.as_ptr() as *const f32);
            Ok(())
        },
        UniformValue::Vec4(val) => {
            uniform!(ctxt, Uniform4fv, Uniform4fvARB, location, 1, val.as_ptr() as *const f32);
            Ok(())
        },
        UniformValue::Texture1d(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_1D)
        },
        UniformValue::CompressedTexture1d(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_1D)
        },
        UniformValue::IntegralTexture1d(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_1D)
        },
        UniformValue::UnsignedTexture1d(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_1D)
        },
        UniformValue::DepthTexture1d(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_1D)
        },
        UniformValue::Texture2d(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D)
        },
        UniformValue::CompressedTexture2d(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D)
        },
        UniformValue::IntegralTexture2d(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D)
        },
        UniformValue::UnsignedTexture2d(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D)
        },
        UniformValue::DepthTexture2d(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D)
        },
        UniformValue::Texture2dMultisample(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE)
        },
        UniformValue::IntegralTexture2dMultisample(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE)
        },
        UniformValue::UnsignedTexture2dMultisample(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE)
        },
        UniformValue::DepthTexture2dMultisample(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE)
        },
        UniformValue::Texture3d(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_3D)
        },
        UniformValue::CompressedTexture3d(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_3D)
        },
        UniformValue::IntegralTexture3d(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_3D)
        },
        UniformValue::UnsignedTexture3d(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_3D)
        },
        UniformValue::DepthTexture3d(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_3D)
        },
        UniformValue::Texture1dArray(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_1D_ARRAY)
        },
        UniformValue::CompressedTexture1dArray(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_1D_ARRAY)
        },
        UniformValue::IntegralTexture1dArray(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_1D_ARRAY)
        },
        UniformValue::UnsignedTexture1dArray(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_1D_ARRAY)
        },
        UniformValue::DepthTexture1dArray(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_1D_ARRAY)
        },
        UniformValue::Texture2dArray(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_ARRAY)
        },
        UniformValue::CompressedTexture2dArray(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_ARRAY)
        },
        UniformValue::IntegralTexture2dArray(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_ARRAY)
        },
        UniformValue::UnsignedTexture2dArray(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_ARRAY)
        },
        UniformValue::DepthTexture2dArray(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_ARRAY)
        },
        UniformValue::Texture2dMultisampleArray(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE_ARRAY)
        },
        UniformValue::IntegralTexture2dMultisampleArray(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE_ARRAY)
        },
        UniformValue::UnsignedTexture2dMultisampleArray(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE_ARRAY)
        },
        UniformValue::DepthTexture2dMultisampleArray(texture, sampler) => {
            let texture = texture.get_id();
            bind_texture_uniform(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE_ARRAY)
        },
    }
}

fn bind_texture_uniform(ctxt: &mut context::CommandContext,
                        samplers: &mut HashMap<SamplerBehavior, SamplerObject,
                                               DefaultState<util::FnvHasher>>,
                        texture: gl::types::GLuint,
                        sampler: Option<SamplerBehavior>, location: gl::types::GLint,
                        active_texture: &mut gl::types::GLenum,
                        bind_point: gl::types::GLenum)
                        -> Result<(), DrawError>
{
    assert!(*active_texture < ctxt.capabilities
                                  .max_combined_texture_image_units as gl::types::GLenum);

    let sampler = if let Some(sampler) = sampler {
        Some(try!(::sampler_object::get_sampler(ctxt, samplers, &sampler)))
    } else {
        None
    };

    let current_texture = *active_texture;
    *active_texture += 1;

    unsafe {
        // TODO: what if it's not supported?
        let active_tex_enum = current_texture + gl::TEXTURE0;
        if ctxt.state.active_texture != active_tex_enum {
            ctxt.gl.ActiveTexture(current_texture + gl::TEXTURE0);
            ctxt.state.active_texture = active_tex_enum;
        }

        ctxt.gl.BindTexture(bind_point, texture);

        if ctxt.version >= &Version(Api::Gl, 1, 5) {
            ctxt.gl.Uniform1i(location, current_texture as gl::types::GLint);
        } else {
            assert!(ctxt.extensions.gl_arb_shader_objects);
            ctxt.gl.Uniform1iARB(location, current_texture as gl::types::GLint);
        }

        if let Some(sampler) = sampler {
            assert!(ctxt.version >= &Version(Api::Gl, 3, 3) ||
                    ctxt.extensions.gl_arb_sampler_objects);
            ctxt.gl.BindSampler(current_texture, sampler);
        } else if ctxt.version >= &Version(Api::Gl, 3, 3) ||
            ctxt.extensions.gl_arb_sampler_objects
        {
            ctxt.gl.BindSampler(current_texture, 0);
        }
    }

    Ok(())
}

fn sync_depth(ctxt: &mut context::CommandContext, depth_test: DepthTest, depth_write: bool,
              depth_range: (f32, f32))
{
    // depth test
    match depth_test {
        DepthTest::Overwrite => unsafe {
            if ctxt.state.enabled_depth_test {
                ctxt.gl.Disable(gl::DEPTH_TEST);
                ctxt.state.enabled_depth_test = false;
            }
        },
        depth_function => unsafe {
            let depth_function = depth_function.to_glenum();
            if ctxt.state.depth_func != depth_function {
                ctxt.gl.DepthFunc(depth_function);
                ctxt.state.depth_func = depth_function;
            }
            if !ctxt.state.enabled_depth_test {
                ctxt.gl.Enable(gl::DEPTH_TEST);
                ctxt.state.enabled_depth_test = true;
            }
        }
    }

    // depth mask
    if depth_write != ctxt.state.depth_mask {
        unsafe {
            ctxt.gl.DepthMask(if depth_write { gl::TRUE } else { gl::FALSE });
        }
        ctxt.state.depth_mask = depth_write;
    }

    // depth range
    if depth_range != ctxt.state.depth_range {
        unsafe {
            ctxt.gl.DepthRange(depth_range.0 as f64, depth_range.1 as f64);
        }
        ctxt.state.depth_range = depth_range;
    }
}

fn sync_blending(ctxt: &mut context::CommandContext, blending_function: Option<BlendingFunction>) {
    let blend_factors = match blending_function {
        Some(BlendingFunction::AlwaysReplace) => unsafe {
            if ctxt.state.enabled_blend {
                ctxt.gl.Disable(gl::BLEND);
                ctxt.state.enabled_blend = false;
            }
            None
        },
        Some(BlendingFunction::Min) => unsafe {
            if ctxt.state.blend_equation != gl::MIN {
                ctxt.gl.BlendEquation(gl::MIN);
                ctxt.state.blend_equation = gl::MIN;
            }
            if !ctxt.state.enabled_blend {
                ctxt.gl.Enable(gl::BLEND);
                ctxt.state.enabled_blend = true;
            }
            None
        },
        Some(BlendingFunction::Max) => unsafe {
            if ctxt.state.blend_equation != gl::MAX {
                ctxt.gl.BlendEquation(gl::MAX);
                ctxt.state.blend_equation = gl::MAX;
            }
            if !ctxt.state.enabled_blend {
                ctxt.gl.Enable(gl::BLEND);
                ctxt.state.enabled_blend = true;
            }
            None
        },
        Some(BlendingFunction::Addition { source, destination }) => unsafe {
            if ctxt.state.blend_equation != gl::FUNC_ADD {
                ctxt.gl.BlendEquation(gl::FUNC_ADD);
                ctxt.state.blend_equation = gl::FUNC_ADD;
            }
            if !ctxt.state.enabled_blend {
                ctxt.gl.Enable(gl::BLEND);
                ctxt.state.enabled_blend = true;
            }
            Some((source, destination))
        },
        Some(BlendingFunction::Subtraction { source, destination }) => unsafe {
            if ctxt.state.blend_equation != gl::FUNC_SUBTRACT {
                ctxt.gl.BlendEquation(gl::FUNC_SUBTRACT);
                ctxt.state.blend_equation = gl::FUNC_SUBTRACT;
            }
            if !ctxt.state.enabled_blend {
                ctxt.gl.Enable(gl::BLEND);
                ctxt.state.enabled_blend = true;
            }
            Some((source, destination))
        },
        Some(BlendingFunction::ReverseSubtraction { source, destination }) => unsafe {
            if ctxt.state.blend_equation != gl::FUNC_REVERSE_SUBTRACT {
                ctxt.gl.BlendEquation(gl::FUNC_REVERSE_SUBTRACT);
                ctxt.state.blend_equation = gl::FUNC_REVERSE_SUBTRACT;
            }
            if !ctxt.state.enabled_blend {
                ctxt.gl.Enable(gl::BLEND);
                ctxt.state.enabled_blend = true;
            }
            Some((source, destination))
        },
        _ => None
    };
    if let Some((source, destination)) = blend_factors {
        let source = source.to_glenum();
        let destination = destination.to_glenum();

        if ctxt.state.blend_func != (source, destination) {
            unsafe { ctxt.gl.BlendFunc(source, destination) };
            ctxt.state.blend_func = (source, destination);
        }
    };
}

fn sync_line_width(ctxt: &mut context::CommandContext, line_width: Option<f32>) {
    if let Some(line_width) = line_width {
        if ctxt.state.line_width != line_width {
            unsafe {
                ctxt.gl.LineWidth(line_width);
                ctxt.state.line_width = line_width;
            }
        }
    }
}

fn sync_point_size(ctxt: &mut context::CommandContext, point_size: Option<f32>) {
    if let Some(point_size) = point_size {
        if ctxt.state.point_size != point_size {
            unsafe {
                ctxt.gl.PointSize(point_size);
                ctxt.state.point_size = point_size;
            }
        }
    }
}

fn sync_polygon_mode(ctxt: &mut context::CommandContext, backface_culling: BackfaceCullingMode,
                     polygon_mode: PolygonMode)
{
    // back-face culling
    // note: we never change the value of `glFrontFace`, whose default is GL_CCW
    //  that's why `CullClockWise` uses `GL_BACK` for example
    match backface_culling {
        BackfaceCullingMode::CullingDisabled => unsafe {
            if ctxt.state.enabled_cull_face {
                ctxt.gl.Disable(gl::CULL_FACE);
                ctxt.state.enabled_cull_face = false;
            }
        },
        BackfaceCullingMode::CullCounterClockWise => unsafe {
            if !ctxt.state.enabled_cull_face {
                ctxt.gl.Enable(gl::CULL_FACE);
                ctxt.state.enabled_cull_face = true;
            }
            if ctxt.state.cull_face != gl::FRONT {
                ctxt.gl.CullFace(gl::FRONT);
                ctxt.state.cull_face = gl::FRONT;
            }
        },
        BackfaceCullingMode::CullClockWise => unsafe {
            if !ctxt.state.enabled_cull_face {
                ctxt.gl.Enable(gl::CULL_FACE);
                ctxt.state.enabled_cull_face = true;
            }
            if ctxt.state.cull_face != gl::BACK {
                ctxt.gl.CullFace(gl::BACK);
                ctxt.state.cull_face = gl::BACK;
            }
        },
    }

    // polygon mode
    unsafe {
        let polygon_mode = polygon_mode.to_glenum();
        if ctxt.state.polygon_mode != polygon_mode {
            ctxt.gl.PolygonMode(gl::FRONT_AND_BACK, polygon_mode);
            ctxt.state.polygon_mode = polygon_mode;
        }
    }
}

fn sync_multisampling(ctxt: &mut context::CommandContext, multisampling: bool) {
    if ctxt.state.enabled_multisample != multisampling {
        unsafe {
            if multisampling {
                ctxt.gl.Enable(gl::MULTISAMPLE);
                ctxt.state.enabled_multisample = true;
            } else {
                ctxt.gl.Disable(gl::MULTISAMPLE);
                ctxt.state.enabled_multisample = false;
            }
        }
    }
}

fn sync_dithering(ctxt: &mut context::CommandContext, dithering: bool) {
    if ctxt.state.enabled_dither != dithering {
        unsafe {
            if dithering {
                ctxt.gl.Enable(gl::DITHER);
                ctxt.state.enabled_dither = true;
            } else {
                ctxt.gl.Disable(gl::DITHER);
                ctxt.state.enabled_dither = false;
            }
        }
    }
}

fn sync_viewport_scissor(ctxt: &mut context::CommandContext, viewport: Option<Rect>,
                         scissor: Option<Rect>, surface_dimensions: (u32, u32))
{
    // viewport
    if let Some(viewport) = viewport {
        assert!(viewport.width <= ctxt.capabilities.max_viewport_dims.0 as u32,
                "Viewport dimensions are too large");
        assert!(viewport.height <= ctxt.capabilities.max_viewport_dims.1 as u32,
                "Viewport dimensions are too large");

        let viewport = (viewport.left as gl::types::GLint, viewport.bottom as gl::types::GLint,
                        viewport.width as gl::types::GLsizei,
                        viewport.height as gl::types::GLsizei);

        if ctxt.state.viewport != Some(viewport) {
            unsafe { ctxt.gl.Viewport(viewport.0, viewport.1, viewport.2, viewport.3); }
            ctxt.state.viewport = Some(viewport);
        }

    } else {
        assert!(surface_dimensions.0 <= ctxt.capabilities.max_viewport_dims.0 as u32,
                "Viewport dimensions are too large");
        assert!(surface_dimensions.1 <= ctxt.capabilities.max_viewport_dims.1 as u32,
                "Viewport dimensions are too large");

        let viewport = (0, 0, surface_dimensions.0 as gl::types::GLsizei,
                        surface_dimensions.1 as gl::types::GLsizei);

        if ctxt.state.viewport != Some(viewport) {
            unsafe { ctxt.gl.Viewport(viewport.0, viewport.1, viewport.2, viewport.3); }
            ctxt.state.viewport = Some(viewport);
        }
    }

    // scissor
    if let Some(scissor) = scissor {
        let scissor = (scissor.left as gl::types::GLint, scissor.bottom as gl::types::GLint,
                       scissor.width as gl::types::GLsizei,
                       scissor.height as gl::types::GLsizei);

        unsafe {
            if ctxt.state.scissor != Some(scissor) {
                ctxt.gl.Scissor(scissor.0, scissor.1, scissor.2, scissor.3);
                ctxt.state.scissor = Some(scissor);
            }

            if !ctxt.state.enabled_scissor_test {
                ctxt.gl.Enable(gl::SCISSOR_TEST);
                ctxt.state.enabled_scissor_test = true;
            }
        }
    } else {
        unsafe {
            if ctxt.state.enabled_scissor_test {
                ctxt.gl.Disable(gl::SCISSOR_TEST);
                ctxt.state.enabled_scissor_test = false;
            }
        }
    }
}

fn sync_rasterizer_discard(ctxt: &mut context::CommandContext, draw_primitives: bool) {
    if ctxt.state.enabled_rasterizer_discard == draw_primitives {
        if ctxt.version >= &Version(Api::Gl, 3, 0) {
            if draw_primitives {
                unsafe { ctxt.gl.Disable(gl::RASTERIZER_DISCARD); }
                ctxt.state.enabled_rasterizer_discard = false;
            } else {
                unsafe { ctxt.gl.Enable(gl::RASTERIZER_DISCARD); }
                ctxt.state.enabled_rasterizer_discard = true;
            }

        } else if ctxt.extensions.gl_ext_transform_feedback {
            if draw_primitives {
                unsafe { ctxt.gl.Disable(gl::RASTERIZER_DISCARD_EXT); }
                ctxt.state.enabled_rasterizer_discard = false;
            } else {
                unsafe { ctxt.gl.Enable(gl::RASTERIZER_DISCARD_EXT); }
                ctxt.state.enabled_rasterizer_discard = true;
            }

        } else {
            unreachable!();
        }
    }
}

unsafe fn sync_vertices_per_patch(ctxt: &mut context::CommandContext, vertices_per_patch: Option<u16>) {
    if let Some(vertices_per_patch) = vertices_per_patch {
        let vertices_per_patch = vertices_per_patch as gl::types::GLint;
        if ctxt.state.patch_patch_vertices != vertices_per_patch {
            ctxt.gl.PatchParameteri(gl::PATCH_VERTICES, vertices_per_patch);
            ctxt.state.patch_patch_vertices = vertices_per_patch;
        }
    }
}
