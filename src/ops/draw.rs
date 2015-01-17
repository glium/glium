use libc;
use std::sync::mpsc::{channel, Sender};

use Display;
use DrawError;

use fbo::{self, FramebufferAttachments};

use sync;
use uniforms::{Uniforms, UniformValue, SamplerBehavior};
use {Program, DrawParameters, GlObject, ToGlEnum};
use index_buffer::IndicesSource;
use vertex::VerticesSource;

use {program, vertex_array_object};
use {gl, context};

/// Draws everything.
pub fn draw<'a, I, U>(display: &Display,
    framebuffer: Option<&FramebufferAttachments>, mut vertex_buffer: VerticesSource,
    mut indices: IndicesSource<I>, program: &Program, uniforms: U, draw_parameters: &DrawParameters,
    dimensions: (u32, u32)) -> Result<(), DrawError> where U: Uniforms, I: ::index_buffer::Index
{
    try!(draw_parameters.validate());

    let fbo_id = display.context.framebuffer_objects.as_ref().unwrap()
                        .get_framebuffer_for_drawing(framebuffer, &display.context.context);

    let vao_id = vertex_array_object::get_vertex_array_object(&display.context, &vertex_buffer,
                                                              &indices, program);

    let program_id = program.get_id();

    let pointer = ::std::ptr::Unique(match &indices {
        &IndicesSource::IndexBuffer { .. } => ::std::ptr::null_mut(),
        &IndicesSource::Buffer { ref pointer, .. } => pointer.as_ptr() as *mut ::libc::c_void,
    });

    let primitives = indices.get_primitives_type().to_glenum();
    let data_type = indices.get_indices_type().to_glenum();
    assert!(indices.get_offset() == 0); // not yet implemented
    let indices_count = indices.get_length();

    // building the list of uniforms binders and the fences that must be fulfilled
    let (uniforms, fences): (Vec<Box<Fn(&mut context::CommandContext) + Send>>, _) = {
        let uniforms_locations = program::get_uniforms_locations(program);
        let mut active_texture = 0;
        let mut active_buffer_binding = 0;

        let mut uniforms_storage = Vec::new();
        let mut fences = Vec::new();

        let mut visiting_result = Ok(());
        uniforms.visit_values(|&mut: name, value| {
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
                                                            program_id,
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
        match &mut vertex_buffer {
            &mut VerticesSource::VertexBuffer(_, ref mut fence) => {
                if let Some(fence) = fence.take() {
                    fences.push(fence);
                }
            }
        };
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

    // TODO: panick if uniforms of the program are not found in the parameter

    let draw_parameters = draw_parameters.clone();

    // in some situations, we have to wait for the draw command to finish before returning
    let (tx, rx) = {
        let needs_sync = if let &IndicesSource::Buffer{..} = &indices {
            true
        } else {
            false
        };

        if needs_sync {
            let (tx, rx) = channel();
            (Some(tx), Some(rx))
        } else {
            (None, None)
        }
    };

    display.context.context.exec(move |: mut ctxt| {
        unsafe {
            fbo::bind_framebuffer(&mut ctxt, fbo_id, true, false);

            // binding program
            if ctxt.state.program != program_id {
                ctxt.gl.UseProgram(program_id);
                ctxt.state.program = program_id;
            }

            // binding program uniforms
            for binder in uniforms.into_iter() {
                binder.call((&mut ctxt,));
            }

            // binding VAO
            if ctxt.state.vertex_array != vao_id {
                ctxt.gl.BindVertexArray(vao_id);
                ctxt.state.vertex_array = vao_id;
            }

            // sync-ing parameters
            draw_parameters.sync(&mut ctxt, dimensions);

            // drawing
            ctxt.gl.DrawElements(primitives, indices_count as i32, data_type, pointer.0);

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
                   program: gl::types::GLuint, current_bind_point: &mut gl::types::GLuint,
                   name: &str)
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
    Ok(match *value {
        UniformValue::Block(_, _, _) => {
            return Err(DrawError::UniformBufferToValue {
                name: name.to_string(),
            })
        },
        UniformValue::SignedInt(val) => {
            Box::new(move |&: ctxt: &mut context::CommandContext| {
                unsafe {
                    ctxt.gl.Uniform1i(location, val)
                }
            })
        },
        UniformValue::UnsignedInt(val) => {
            Box::new(move |&: ctxt: &mut context::CommandContext| {
                unsafe {
                    ctxt.gl.Uniform1ui(location, val)
                }
            })
        },
        UniformValue::Float(val) => {
            Box::new(move |&: ctxt: &mut context::CommandContext| {
                unsafe {
                    ctxt.gl.Uniform1f(location, val)
                }
            })
        },
        UniformValue::Mat2(val) => {
            Box::new(move |&: ctxt: &mut context::CommandContext| {
                unsafe {
                    ctxt.gl.UniformMatrix2fv(location, 1, 0, val.as_ptr() as *const f32)
                }
            })
        },
        UniformValue::Mat3(val) => {
            Box::new(move |&: ctxt: &mut context::CommandContext| {
                unsafe {
                    ctxt.gl.UniformMatrix3fv(location, 1, 0, val.as_ptr() as *const f32)
                }
            })
        },
        UniformValue::Mat4(val) => {
            Box::new(move |&: ctxt: &mut context::CommandContext| {
                unsafe {
                    ctxt.gl.UniformMatrix4fv(location, 1, 0, val.as_ptr() as *const f32)
                }
            })
        },
        UniformValue::Vec2(val) => {
            Box::new(move |&: ctxt: &mut context::CommandContext| {
                unsafe {
                    ctxt.gl.Uniform2fv(location, 1, val.as_ptr() as *const f32)
                }
            })
        },
        UniformValue::Vec3(val) => {
            Box::new(move |&: ctxt: &mut context::CommandContext| {
                unsafe {
                    ctxt.gl.Uniform3fv(location, 1, val.as_ptr() as *const f32)
                }
            })
        },
        UniformValue::Vec4(val) => {
            Box::new(move |&: ctxt: &mut context::CommandContext| {
                unsafe {
                    ctxt.gl.Uniform4fv(location, 1, val.as_ptr() as *const f32)
                }
            })
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
    })
}

fn build_texture_binder(display: &Display, texture: gl::types::GLuint,
                        sampler: Option<SamplerBehavior>, location: gl::types::GLint,
                        active_texture: &mut gl::types::GLenum,
                        bind_point: gl::types::GLenum)
                        -> Box<Fn(&mut context::CommandContext) + Send>
{
    assert!(*active_texture < display.context.context.capabilities()
                                     .max_combined_texture_image_units as gl::types::GLenum);

    let sampler = sampler.map(|b| ::uniforms::get_sampler(display, &b));

    let current_texture = *active_texture;
    *active_texture += 1;

    Box::new(move |&: ctxt: &mut context::CommandContext| {
        unsafe {
            ctxt.gl.ActiveTexture(current_texture + gl::TEXTURE0);
            ctxt.gl.BindTexture(bind_point, texture);
            ctxt.gl.Uniform1i(location, current_texture as gl::types::GLint);

            if let Some(sampler) = sampler {
                ctxt.gl.BindSampler(current_texture, sampler);
            } else {
                ctxt.gl.BindSampler(current_texture, 0);
            }
        }
    })
}
