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
use context::GlVersion;
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

    // obtaining the identifier of the FBO to draw upon
    let fbo_id = {
        let mut ctxt = context.make_current();
        context.framebuffer_objects.as_ref().unwrap()
                            .get_framebuffer_for_drawing(framebuffer, &mut ctxt)
    };

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

    // list of the commands that can be executed
    enum DrawCommand {
        DrawArrays(gl::types::GLenum, gl::types::GLint, gl::types::GLsizei),
        DrawArraysInstanced(gl::types::GLenum, gl::types::GLint, gl::types::GLsizei,
                            gl::types::GLsizei),
        DrawElements(gl::types::GLenum, gl::types::GLsizei, gl::types::GLenum,
                     *const gl::types::GLvoid),
        DrawElementsInstanced(gl::types::GLenum, gl::types::GLsizei, gl::types::GLenum,
                              *const gl::types::GLvoid, gl::types::GLsizei),
    }

    // choosing the right command
    let draw_command = {
        let cmd = match &indices {
            &IndicesSource::IndexBuffer { ref buffer, offset, length, .. } => {
                let ptr: *const u8 = ptr::null_mut();
                let ptr = unsafe { ptr.offset((offset * buffer.get_indices_type().get_size()) as isize) };

                DrawCommand::DrawElements(buffer.get_primitives_type().to_glenum(),
                                          length as gl::types::GLsizei,
                                          buffer.get_indices_type().to_glenum(),
                                          ptr as *const libc::c_void)
            },
            &IndicesSource::Buffer { ref pointer, primitives, offset, length } => {
                assert!(offset == 0);       // not yet implemented

                DrawCommand::DrawElements(primitives.to_glenum(), length as gl::types::GLsizei,
                                          <I as index::Index>::get_type().to_glenum(),
                                          pointer.as_ptr() as *const gl::types::GLvoid)
            },
            &IndicesSource::NoIndices { primitives } => {
                let vertices_count = match vertices_count {
                    Some(c) => c,
                    None => return Err(DrawError::VerticesSourcesLengthMismatch)
                };

                DrawCommand::DrawArrays(primitives.to_glenum(), 0,
                                        vertices_count as gl::types::GLsizei)
            },
        };

        match (cmd, instances_count) {
            (DrawCommand::DrawElements(a, b, c, d), Some(e)) => {
                DrawCommand::DrawElementsInstanced(a, b, c, d, e as gl::types::GLsizei)
            },
            (DrawCommand::DrawArrays(a, b, c), Some(d)) => {
                DrawCommand::DrawArraysInstanced(a, b, c, d as gl::types::GLsizei)
            },
            (a, _) => a
        }
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

    // building the list of uniforms binders and the fences that must be fulfilled
    // TODO: panic if uniforms of the program are not found in the parameter
    let (uniforms, fences): (Vec<Box<Fn(&mut context::CommandContext) + Send>>, _) = {
        let mut active_texture = 0;
        let mut active_buffer_binding = 0;

        let mut uniforms_storage = Vec::new();
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

                let binder = match uniform_to_binder(&mut ctxt, &mut context.samplers.borrow_mut(),
                                                     value, uniform.location,
                                                     &mut active_texture, name)
                {
                    Ok(b) => b,
                    Err(e) => {
                        visiting_result = Err(e);
                        return;
                    }
                };

                uniforms_storage.push(binder);

            } else if let Some(block) = program.get_uniform_blocks().get(name) {
                let (binder, fence) = match block_to_binder(context, value, block,
                                                            program.get_id(),
                                                            &mut active_buffer_binding, name)
                {
                    Ok(b) => b,
                    Err(e) => {
                        visiting_result = Err(e);
                        return;
                    }
                };

                uniforms_storage.push(binder);

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

        (uniforms_storage, fences)
    };

    // copying some stuff in order to send them
    let program_id = program.get_id();
    let DrawParameters { depth_test, depth_write, depth_range, blending_function,
                         line_width, point_size, backface_culling, polygon_mode, multisampling,
                         dithering, viewport, scissor, draw_primitives } = *draw_parameters;
    
    // the vertex array object to bind
    let vao_id = context.vertex_array_objects.bind_vao(&mut ctxt,
                                                               vertex_buffers.iter().map(|v| v).collect::<Vec<_>>().as_slice(),
                                                               &indices, program);

    unsafe {
        fbo::bind_framebuffer(&mut ctxt, fbo_id, true, false);

        // binding program
        if ctxt.state.program != program_id {
            match program_id {
                Handle::Id(id) => ctxt.gl.UseProgram(id),
                Handle::Handle(id) => ctxt.gl.UseProgramObjectARB(id),
            }
            ctxt.state.program = program_id;
        }

        // binding program uniforms
        for binder in uniforms.into_iter() {
            binder.call((&mut ctxt,));
        }

        // sync-ing parameters
        sync_depth(&mut ctxt, depth_test, depth_write, depth_range);
        sync_blending(&mut ctxt, blending_function);
        sync_line_width(&mut ctxt, line_width);
        sync_point_size(&mut ctxt, point_size);
        sync_polygon_mode(&mut ctxt, backface_culling, polygon_mode);
        sync_multisampling(&mut ctxt, multisampling);
        sync_dithering(&mut ctxt, dithering);
        sync_viewport_scissor(&mut ctxt, viewport, scissor, dimensions);
        sync_rasterizer_discard(&mut ctxt, draw_primitives);
        sync_vertices_per_patch(&mut ctxt, vertices_per_patch);

        // drawing
        match draw_command {
            DrawCommand::DrawArrays(a, b, c) => {
                ctxt.gl.DrawArrays(a, b, c);
            },
            DrawCommand::DrawArraysInstanced(a, b, c, d) => {
                ctxt.gl.DrawArraysInstanced(a, b, c, d);
            },
            DrawCommand::DrawElements(a, b, c, d) => {
                ctxt.gl.DrawElements(a, b, c, d);
            },
            DrawCommand::DrawElementsInstanced(a, b, c, d, e) => {
                ctxt.gl.DrawElementsInstanced(a, b, c, d, e);
            },
        }

        // fulfilling the fences
        for fence in fences.into_iter() {
            fence.send(sync::new_linear_sync_fence_if_supported(&mut ctxt).unwrap()).unwrap();
        }
    }

    Ok(())
}

// TODO: we use a `Fn` instead of `FnOnce` because of that "std::thunk" issue
fn block_to_binder(context: &Context, value: &UniformValue, block: &program::UniformBlock,
                   program: Handle, current_bind_point: &mut gl::types::GLuint, name: &str)
                   -> Result<(Box<Fn(&mut context::CommandContext) + Send>,
                       Option<Sender<sync::LinearSyncFence>>), DrawError>
{
    Ok(match value {
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

            let bind_fn = Box::new(move |ctxt: &mut context::CommandContext| {
                unsafe {
                    ctxt.gl.BindBufferBase(gl::UNIFORM_BUFFER, bind_point as gl::types::GLuint,
                                           buffer);
                    ctxt.gl.UniformBlockBinding(program, binding,
                                                bind_point as gl::types::GLuint);
                }
            });

            (bind_fn, fence)
        },
        _ => {
            return Err(DrawError::UniformValueToBlock { name: name.to_string() });
        }
    })
}

// TODO: we use a `Fn` instead of `FnOnce` because of that "std::thunk" issue
fn uniform_to_binder(ctxt: &mut context::CommandContext,
                     samplers: &mut HashMap<SamplerBehavior, SamplerObject,
                                            DefaultState<util::FnvHasher>>,
                     value: &UniformValue, location: gl::types::GLint,
                     active_texture: &mut gl::types::GLenum, name: &str)
                     -> Result<Box<Fn(&mut context::CommandContext) + Send>, DrawError>
{
    macro_rules! uniform(
        ($ctxt:expr, $uniform:ident, $uniform_arb:ident, $($params:expr),+) => (
            unsafe {
                if $ctxt.version >= &context::GlVersion(Api::Gl, 1, 5) {
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
            return Err(DrawError::UniformBufferToValue {
                name: name.to_string(),
            });
        },
        UniformValue::SignedInt(val) => {
            Ok(Box::new(move |ctxt: &mut context::CommandContext| {
                uniform!(ctxt, Uniform1i, Uniform1iARB, location, val);
            }))
        },
        UniformValue::UnsignedInt(val) => {
            Ok(Box::new(move |ctxt: &mut context::CommandContext| {
                // Uniform1uiARB doesn't exist
                unsafe {
                    if ctxt.version >= &context::GlVersion(Api::Gl, 1, 5) {
                        ctxt.gl.Uniform1ui(location, val)
                    } else {
                        assert!(ctxt.extensions.gl_arb_shader_objects);
                        ctxt.gl.Uniform1iARB(location, val as gl::types::GLint)
                    }
                }
            }))
        },
        UniformValue::Float(val) => {
            Ok(Box::new(move |ctxt: &mut context::CommandContext| {
                uniform!(ctxt, Uniform1f, Uniform1fARB, location, val);
            }))
        },
        UniformValue::Mat2(val, transpose) => {
            Ok(Box::new(move |ctxt: &mut context::CommandContext| {
                uniform!(ctxt, UniformMatrix2fv, UniformMatrix2fvARB,
                         location, 1, if transpose { 1 } else { 0 }, val.as_ptr() as *const f32);
            }))
        },
        UniformValue::Mat3(val, transpose) => {
            Ok(Box::new(move |ctxt: &mut context::CommandContext| {
                uniform!(ctxt, UniformMatrix3fv, UniformMatrix3fvARB,
                         location, 1, if transpose { 1 } else { 0 }, val.as_ptr() as *const f32);
            }))
        },
        UniformValue::Mat4(val, transpose) => {
            Ok(Box::new(move |ctxt: &mut context::CommandContext| {
                uniform!(ctxt, UniformMatrix4fv, UniformMatrix4fvARB,
                         location, 1, if transpose { 1 } else { 0 }, val.as_ptr() as *const f32);
            }))
        },
        UniformValue::Vec2(val) => {
            Ok(Box::new(move |ctxt: &mut context::CommandContext| {
                uniform!(ctxt, Uniform2fv, Uniform2fvARB, location, 1, val.as_ptr() as *const f32);
            }))
        },
        UniformValue::Vec3(val) => {
            Ok(Box::new(move |ctxt: &mut context::CommandContext| {
                uniform!(ctxt, Uniform3fv, Uniform3fvARB, location, 1, val.as_ptr() as *const f32);
            }))
        },
        UniformValue::Vec4(val) => {
            Ok(Box::new(move |ctxt: &mut context::CommandContext| {
                uniform!(ctxt, Uniform4fv, Uniform4fvARB, location, 1, val.as_ptr() as *const f32);
            }))
        },
        UniformValue::Texture1d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_1D)
        },
        UniformValue::CompressedTexture1d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_1D)
        },
        UniformValue::IntegralTexture1d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_1D)
        },
        UniformValue::UnsignedTexture1d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_1D)
        },
        UniformValue::DepthTexture1d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_1D)
        },
        UniformValue::Texture2d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D)
        },
        UniformValue::CompressedTexture2d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D)
        },
        UniformValue::IntegralTexture2d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D)
        },
        UniformValue::UnsignedTexture2d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D)
        },
        UniformValue::DepthTexture2d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D)
        },
        UniformValue::Texture2dMultisample(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE)
        },
        UniformValue::IntegralTexture2dMultisample(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE)
        },
        UniformValue::UnsignedTexture2dMultisample(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE)
        },
        UniformValue::DepthTexture2dMultisample(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE)
        },
        UniformValue::Texture3d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_3D)
        },
        UniformValue::CompressedTexture3d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_3D)
        },
        UniformValue::IntegralTexture3d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_3D)
        },
        UniformValue::UnsignedTexture3d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_3D)
        },
        UniformValue::DepthTexture3d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_3D)
        },
        UniformValue::Texture1dArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_1D_ARRAY)
        },
        UniformValue::CompressedTexture1dArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_1D_ARRAY)
        },
        UniformValue::IntegralTexture1dArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_1D_ARRAY)
        },
        UniformValue::UnsignedTexture1dArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_1D_ARRAY)
        },
        UniformValue::DepthTexture1dArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_1D_ARRAY)
        },
        UniformValue::Texture2dArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_ARRAY)
        },
        UniformValue::CompressedTexture2dArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_ARRAY)
        },
        UniformValue::IntegralTexture2dArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_ARRAY)
        },
        UniformValue::UnsignedTexture2dArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_ARRAY)
        },
        UniformValue::DepthTexture2dArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_ARRAY)
        },
        UniformValue::Texture2dMultisampleArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE_ARRAY)
        },
        UniformValue::IntegralTexture2dMultisampleArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE_ARRAY)
        },
        UniformValue::UnsignedTexture2dMultisampleArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE_ARRAY)
        },
        UniformValue::DepthTexture2dMultisampleArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(ctxt, samplers, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE_ARRAY)
        },
    }
}

fn build_texture_binder(ctxt: &mut context::CommandContext,
                        samplers: &mut HashMap<SamplerBehavior, SamplerObject,
                                               DefaultState<util::FnvHasher>>,
                        texture: gl::types::GLuint,
                        sampler: Option<SamplerBehavior>, location: gl::types::GLint,
                        active_texture: &mut gl::types::GLenum,
                        bind_point: gl::types::GLenum)
                        -> Result<Box<Fn(&mut context::CommandContext) + Send>, DrawError>
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

    Ok(Box::new(move |ctxt: &mut context::CommandContext| {
        unsafe {
            // TODO: what if it's not supported?
            let active_tex_enum = current_texture + gl::TEXTURE0;
            if ctxt.state.active_texture != active_tex_enum {
                ctxt.gl.ActiveTexture(current_texture + gl::TEXTURE0);
                ctxt.state.active_texture = active_tex_enum;
            }

            ctxt.gl.BindTexture(bind_point, texture);

            if ctxt.version >= &context::GlVersion(Api::Gl, 1, 5) {
                ctxt.gl.Uniform1i(location, current_texture as gl::types::GLint);
            } else {
                assert!(ctxt.extensions.gl_arb_shader_objects);
                ctxt.gl.Uniform1iARB(location, current_texture as gl::types::GLint);
            }

            if let Some(sampler) = sampler {
                assert!(ctxt.version >= &context::GlVersion(Api::Gl, 3, 3) ||
                        ctxt.extensions.gl_arb_sampler_objects);
                ctxt.gl.BindSampler(current_texture, sampler);
            } else if ctxt.version >= &context::GlVersion(Api::Gl, 3, 3) ||
                ctxt.extensions.gl_arb_sampler_objects
            {
                ctxt.gl.BindSampler(current_texture, 0);
            }
        }
    }))
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
        if ctxt.version >= &GlVersion(Api::Gl, 3, 0) {
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
