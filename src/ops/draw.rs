use std::ptr;
use std::sync::mpsc::{channel, Sender};

use Display;
use DrawError;
use Handle;

use fbo::{self, FramebufferAttachments};

use sync;
use uniforms::{Uniforms, UniformValue, SamplerBehavior};
use {Program, DrawParameters, GlObject, ToGlEnum};
use index::{self, IndicesSource};
use vertex::{MultiVerticesSource, VerticesSource};

use {program, vertex_array_object};
use {gl, context, draw_parameters};
use version::Api;

/// Draws everything.
pub fn draw<'a, I, U, V>(display: &Display, framebuffer: Option<&FramebufferAttachments>,
                         vertex_buffers: V, mut indices: IndicesSource<I>,
                         program: &Program, uniforms: U, draw_parameters: &DrawParameters,
                         dimensions: (u32, u32)) -> Result<(), DrawError>
                         where U: Uniforms, I: index::Index, V: MultiVerticesSource<'a>
{
    // TODO: avoid this allocation
    let mut vertex_buffers = vertex_buffers.iter().collect::<Vec<_>>();

    try!(draw_parameters::validate(display, draw_parameters));

    // obtaining the identifier of the FBO to draw upon
    let fbo_id = display.context.framebuffer_objects.as_ref().unwrap()
                        .get_framebuffer_for_drawing(framebuffer, &display.context.context);

    // using a base vertex is not yet supported
    // TODO: 
    for src in vertex_buffers.iter() {
        match src {
            &VerticesSource::VertexBuffer(_, _, offset, _) => {
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
                &VerticesSource::VertexBuffer(_, _, _, len) => {
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
                &VerticesSource::PerInstanceBuffer(ref buffer, _) => {
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

    // the vertex array object to bind
    let vao_id = vertex_array_object::get_vertex_array_object(&display.context,
                                                              vertex_buffers.iter().map(|v| v).collect::<Vec<_>>().as_slice(),
                                                              &indices, program);

    // list of the commands that can be executed
    enum DrawCommand {
        DrawArrays(gl::types::GLenum, gl::types::GLint, gl::types::GLsizei),
        DrawArraysInstanced(gl::types::GLenum, gl::types::GLint, gl::types::GLsizei,
                            gl::types::GLsizei),
        DrawElements(gl::types::GLenum, gl::types::GLsizei, gl::types::GLenum,
                     ptr::Unique<gl::types::GLvoid>),
        DrawElementsInstanced(gl::types::GLenum, gl::types::GLsizei, gl::types::GLenum,
                              ptr::Unique<gl::types::GLvoid>, gl::types::GLsizei),
    }

    // choosing the right command
    let must_sync;
    let draw_command = {
        let cmd = match &indices {
            &IndicesSource::IndexBuffer { ref buffer, offset, length, .. } => {
                assert!(offset == 0);       // not yet implemented
                must_sync = false;
                DrawCommand::DrawElements(buffer.get_primitives_type().to_glenum(),
                                          length as gl::types::GLsizei,
                                          buffer.get_indices_type().to_glenum(),
                                          unsafe { ptr::Unique::new(ptr::null_mut()) })
            },
            &IndicesSource::Buffer { ref pointer, primitives, offset, length } => {
                assert!(offset == 0);       // not yet implemented
                must_sync = true;
                DrawCommand::DrawElements(primitives.to_glenum(), length as gl::types::GLsizei,
                                          <I as index::Index>::get_type().to_glenum(),
                                          unsafe { ptr::Unique::new(pointer.as_ptr() as *mut gl::types::GLvoid) })
            },
            &IndicesSource::NoIndices { primitives } => {
                must_sync = false;

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
            if let Some(max) = display.context.context.capabilities().max_patch_vertices {
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

    // building the list of uniforms binders and the fences that must be fulfilled
    // TODO: panic if uniforms of the program are not found in the parameter
    let (uniforms, fences): (Vec<Box<Fn(&mut context::CommandContext) + Send>>, _) = {
        let uniforms_locations = program::get_uniforms_locations(program);
        let mut active_texture = 0;
        let mut active_buffer_binding = 0;

        let mut uniforms_storage = Vec::new();
        let mut fences = Vec::new();

        let mut visiting_result = Ok(());
        uniforms.visit_values(|name, value| {
            if visiting_result.is_err() { return; }

            if let Some(uniform) = uniforms_locations.get(name) {
                assert!(uniform.size.is_none(), "Uniform arrays not supported yet");

                if !value.is_usable_with(&uniform.ty) {
                    visiting_result = Err(DrawError::UniformTypeMismatch {
                        name: name.to_string(),
                        expected: uniform.ty,
                    });
                    return;
                }

                let binder = match uniform_to_binder(display, value, uniform.location,
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
                let (binder, fence) = match block_to_binder(display, value, block,
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
                &mut VerticesSource::VertexBuffer(_, ref mut fence, _, _) => {
                    if let Some(fence) = fence.take() {
                        fences.push(fence);
                    }
                }
                &mut VerticesSource::PerInstanceBuffer(_, ref mut fence) => {
                    if let Some(fence) = fence.take() {
                        fences.push(fence);
                    }
                }
            };
        }
        match &mut indices {
            &mut IndicesSource::IndexBuffer { ref mut fence, .. } => {
                if let Some(fence) = fence.take() {
                    fences.push(fence);
                }
            },
            _ => ()
        };

        (uniforms_storage, fences)
    };

    // if the command uses data in the RAM, we have to wait for the draw command to
    // finish before returning
    // if so, we build a channel for this purpose
    let (tx, rx) = if must_sync {
        let (tx, rx) = channel();
        (Some(tx), Some(rx))
    } else {
        (None, None)
    };

    // copying some stuff in order to send them
    let program_id = program.get_id();
    let draw_parameters = draw_parameters.clone();

    // sending the command
    display.context.context.exec(move |mut ctxt| {
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

            // binding VAO
            if ctxt.state.vertex_array != vao_id {
                if ctxt.version >= &context::GlVersion(Api::Gl, 3, 0) ||
                    ctxt.extensions.gl_arb_vertex_array_object
                {
                    ctxt.gl.BindVertexArray(vao_id);
                } else if ctxt.extensions.gl_apple_vertex_array_object {
                    ctxt.gl.BindVertexArrayAPPLE(vao_id);
                } else {
                    unreachable!()
                }
                
                ctxt.state.vertex_array = vao_id;
            }

            // sync-ing parameters
            draw_parameters::sync(&draw_parameters, &mut ctxt, dimensions);

            // vertices per patch
            if let Some(vertices_per_patch) = vertices_per_patch {
                let vertices_per_patch = vertices_per_patch as gl::types::GLint;
                if ctxt.state.patch_patch_vertices != vertices_per_patch {
                    ctxt.gl.PatchParameteri(gl::PATCH_VERTICES, vertices_per_patch);
                    ctxt.state.patch_patch_vertices = vertices_per_patch;
                }
            }

            // drawing
            match draw_command {
                DrawCommand::DrawArrays(a, b, c) => {
                    ctxt.gl.DrawArrays(a, b, c);
                },
                DrawCommand::DrawArraysInstanced(a, b, c, d) => {
                    ctxt.gl.DrawArraysInstanced(a, b, c, d);
                },
                DrawCommand::DrawElements(a, b, c, d) => {
                    ctxt.gl.DrawElements(a, b, c, d.get());
                },
                DrawCommand::DrawElementsInstanced(a, b, c, d, e) => {
                    ctxt.gl.DrawElementsInstanced(a, b, c, d.get(), e);
                },
            }

            // fulfilling the fences
            for fence in fences.into_iter() {
                fence.send(sync::new_linear_sync_fence_if_supported(&mut ctxt).unwrap()).unwrap();
            }
        }

        // sync-ing if necessary
        if let Some(tx) = tx {
            tx.send(()).ok();
        }
    });

    // sync-ing if necessary
    if let Some(rx) = rx {
        rx.recv().unwrap();
    }

    Ok(())
}

// TODO: we use a `Fn` instead of `FnOnce` because of that "std::thunk" issue
fn block_to_binder(display: &Display, value: &UniformValue, block: &program::UniformBlock,
                   program: Handle, current_bind_point: &mut gl::types::GLuint, name: &str)
                   -> Result<(Box<Fn(&mut context::CommandContext) + Send>,
                       Option<Sender<sync::LinearSyncFence>>), DrawError>
{
    Ok(match value {
        &UniformValue::Block(ref buffer, ref layout, ref fence) => {
            if !layout.call((block,)) {
                return Err(DrawError::UniformBlockLayoutMismatch { name: name.to_string() });
            }

            let bind_point = *current_bind_point;
            *current_bind_point += 1;

            let buffer = buffer.get_id();
            let binding = block.binding as gl::types::GLuint;

            let program = match program {
                Handle::Id(id) => id,
                _ => unreachable!()
            };

            let bind_fn = Box::new(move |&: ctxt: &mut context::CommandContext| {
                unsafe {
                    ctxt.gl.BindBufferBase(gl::UNIFORM_BUFFER, bind_point as gl::types::GLuint,
                                           buffer);
                    ctxt.gl.UniformBlockBinding(program, binding,
                                                bind_point as gl::types::GLuint);
                }
            });

            (bind_fn, fence.clone())
        },
        _ => {
            return Err(DrawError::UniformValueToBlock { name: name.to_string() });
        }
    })
}

// TODO: we use a `Fn` instead of `FnOnce` because of that "std::thunk" issue
fn uniform_to_binder(display: &Display, value: &UniformValue, location: gl::types::GLint,
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
        UniformValue::Block(_, _, _) => {
            return Err(DrawError::UniformBufferToValue {
                name: name.to_string(),
            });
        },
        UniformValue::SignedInt(val) => {
            Ok(Box::new(move |&: ctxt: &mut context::CommandContext| {
                uniform!(ctxt, Uniform1i, Uniform1iARB, location, val);
            }))
        },
        UniformValue::UnsignedInt(val) => {
            Ok(Box::new(move |&: ctxt: &mut context::CommandContext| {
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
            Ok(Box::new(move |&: ctxt: &mut context::CommandContext| {
                uniform!(ctxt, Uniform1f, Uniform1fARB, location, val);
            }))
        },
        UniformValue::Mat2(val, transpose) => {
            Ok(Box::new(move |&: ctxt: &mut context::CommandContext| {
                uniform!(ctxt, UniformMatrix2fv, UniformMatrix2fvARB,
                         location, 1, if transpose { 1 } else { 0 }, val.as_ptr() as *const f32);
            }))
        },
        UniformValue::Mat3(val, transpose) => {
            Ok(Box::new(move |&: ctxt: &mut context::CommandContext| {
                uniform!(ctxt, UniformMatrix3fv, UniformMatrix3fvARB,
                         location, 1, if transpose { 1 } else { 0 }, val.as_ptr() as *const f32);
            }))
        },
        UniformValue::Mat4(val, transpose) => {
            Ok(Box::new(move |&: ctxt: &mut context::CommandContext| {
                uniform!(ctxt, UniformMatrix4fv, UniformMatrix4fvARB,
                         location, 1, if transpose { 1 } else { 0 }, val.as_ptr() as *const f32);
            }))
        },
        UniformValue::Vec2(val) => {
            Ok(Box::new(move |&: ctxt: &mut context::CommandContext| {
                uniform!(ctxt, Uniform2fv, Uniform2fvARB, location, 1, val.as_ptr() as *const f32);
            }))
        },
        UniformValue::Vec3(val) => {
            Ok(Box::new(move |&: ctxt: &mut context::CommandContext| {
                uniform!(ctxt, Uniform3fv, Uniform3fvARB, location, 1, val.as_ptr() as *const f32);
            }))
        },
        UniformValue::Vec4(val) => {
            Ok(Box::new(move |&: ctxt: &mut context::CommandContext| {
                uniform!(ctxt, Uniform4fv, Uniform4fvARB, location, 1, val.as_ptr() as *const f32);
            }))
        },
        UniformValue::Texture1d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_1D)
        },
        UniformValue::CompressedTexture1d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_1D)
        },
        UniformValue::IntegralTexture1d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_1D)
        },
        UniformValue::UnsignedTexture1d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_1D)
        },
        UniformValue::DepthTexture1d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_1D)
        },
        UniformValue::Texture2d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_2D)
        },
        UniformValue::CompressedTexture2d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_2D)
        },
        UniformValue::IntegralTexture2d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_2D)
        },
        UniformValue::UnsignedTexture2d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_2D)
        },
        UniformValue::DepthTexture2d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_2D)
        },
        UniformValue::Texture2dMultisample(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE)
        },
        UniformValue::IntegralTexture2dMultisample(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE)
        },
        UniformValue::UnsignedTexture2dMultisample(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE)
        },
        UniformValue::DepthTexture2dMultisample(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE)
        },
        UniformValue::Texture3d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_3D)
        },
        UniformValue::CompressedTexture3d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_3D)
        },
        UniformValue::IntegralTexture3d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_3D)
        },
        UniformValue::UnsignedTexture3d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_3D)
        },
        UniformValue::DepthTexture3d(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_3D)
        },
        UniformValue::Texture1dArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_1D_ARRAY)
        },
        UniformValue::CompressedTexture1dArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_1D_ARRAY)
        },
        UniformValue::IntegralTexture1dArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_1D_ARRAY)
        },
        UniformValue::UnsignedTexture1dArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_1D_ARRAY)
        },
        UniformValue::DepthTexture1dArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_1D_ARRAY)
        },
        UniformValue::Texture2dArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_2D_ARRAY)
        },
        UniformValue::CompressedTexture2dArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_2D_ARRAY)
        },
        UniformValue::IntegralTexture2dArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_2D_ARRAY)
        },
        UniformValue::UnsignedTexture2dArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_2D_ARRAY)
        },
        UniformValue::DepthTexture2dArray(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_2D_ARRAY)
        },
        UniformValue::Texture2dArrayMultisample(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE_ARRAY)
        },
        UniformValue::IntegralTexture2dArrayMultisample(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE_ARRAY)
        },
        UniformValue::UnsignedTexture2dArrayMultisample(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE_ARRAY)
        },
        UniformValue::DepthTexture2dArrayMultisample(texture, sampler) => {
            let texture = texture.get_id();
            build_texture_binder(display, texture, sampler, location, active_texture, gl::TEXTURE_2D_MULTISAMPLE_ARRAY)
        },
    }
}

fn build_texture_binder(display: &Display, texture: gl::types::GLuint,
                        sampler: Option<SamplerBehavior>, location: gl::types::GLint,
                        active_texture: &mut gl::types::GLenum,
                        bind_point: gl::types::GLenum)
                        -> Result<Box<Fn(&mut context::CommandContext) + Send>, DrawError>
{
    assert!(*active_texture < display.context.context.capabilities()
                                     .max_combined_texture_image_units as gl::types::GLenum);

    let sampler = if let Some(sampler) = sampler {
        Some(try!(::sampler_object::get_sampler(display, &sampler)))
    } else {
        None
    };

    let current_texture = *active_texture;
    *active_texture += 1;

    Ok(Box::new(move |&: ctxt: &mut context::CommandContext| {
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
