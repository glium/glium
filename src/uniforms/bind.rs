/*!

Handles binding uniforms to the OpenGL state machine.

*/
use gl;

use BufferExt;
use BufferSliceExt;
use DrawError;
use ProgramExt;
use UniformsExt;
use RawUniformValue;
use TextureExt;

use texture::bind;

use uniforms::Uniforms;
use uniforms::UniformValue;
use uniforms::SamplerBehavior;

use context::CommandContext;
use buffer::Inserter;
use ContextExt;

use utils::bitsfield::Bitsfield;

use vertex::MultiVerticesSource;

use program;
use context;
use version::Version;
use version::Api;

impl<U> UniformsExt for U where U: Uniforms {
    fn bind_uniforms<'a, P>(&'a self, mut ctxt: &mut CommandContext, program: &P,
                            fences: &mut Vec<Inserter<'a>>)
                            -> Result<(), DrawError>
                            where P: ProgramExt
    {
        let mut textures_binder = bind::start_bind();
        let mut uniform_buffer_bind_points = Bitsfield::new();
        let mut shared_storage_buffer_bind_points = Bitsfield::new();

        let mut visiting_result = Ok(());
        self.visit_values(|name, value| {
            if visiting_result.is_err() { return; }

            if let Some(uniform) = program.get_uniform(name) {
                assert!(uniform.size.is_none(), "Uniform arrays not supported yet");

                if !value.is_usable_with(&uniform.ty) {
                    visiting_result = Err(DrawError::UniformTypeMismatch {
                        name: name.to_owned(),
                        expected: uniform.ty,
                    });
                    return;
                }

                match bind_uniform(&mut ctxt, &value, program, uniform.location,
                                   &mut textures_binder, name)
                {
                    Ok(_) => (),
                    Err(e) => {
                        visiting_result = Err(e);
                        return;
                    }
                };

            } else if let Some(block) = program.get_uniform_blocks().get(name) {
                let fence = match bind_uniform_block(&mut ctxt, &value, block,
                                                     program, &mut uniform_buffer_bind_points, name)
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

            } else if let Some(block) = program.get_shader_storage_blocks().get(name) {
                let fence = match bind_shared_storage_block(&mut ctxt, &value, block, program,
                                                            &mut shared_storage_buffer_bind_points,
                                                            name)
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

        visiting_result
    }
}

fn bind_uniform_block<'a, P>(ctxt: &mut context::CommandContext, value: &UniformValue<'a>,
                             block: &program::UniformBlock,
                             program: &P, buffer_bind_points: &mut Bitsfield, name: &str)
                             -> Result<Option<Inserter<'a>>, DrawError>
                             where P: ProgramExt
{
    match value {
        &UniformValue::Block(buffer, ref layout) => {
            match layout(block) {
                Ok(_) => (),
                Err(e) => {
                    return Err(DrawError::UniformBlockLayoutMismatch {
                        name: name.to_owned(),
                        err: e,
                    });
                }
            }

            let bind_point = buffer_bind_points.get_unused().expect("Not enough buffer units");
            buffer_bind_points.set_used(bind_point);

            assert!(buffer.get_offset_bytes() == 0);     // TODO: not implemented
            let fence = buffer.add_fence();
            let block_id = block.id as gl::types::GLuint;

            buffer.prepare_and_bind_for_uniform(ctxt, bind_point as gl::types::GLuint);
            program.set_uniform_block_binding(ctxt, block_id, bind_point as gl::types::GLuint);

            Ok(fence)
        },
        _ => {
            Err(DrawError::UniformValueToBlock { name: name.to_owned() })
        }
    }
}

fn bind_shared_storage_block<'a, P>(ctxt: &mut context::CommandContext, value: &UniformValue<'a>,
                                    block: &program::UniformBlock,
                                    program: &P, buffer_bind_points: &mut Bitsfield, name: &str)
                                    -> Result<Option<Inserter<'a>>, DrawError>
                                    where P: ProgramExt
{
    match value {
        &UniformValue::Block(buffer, ref layout) => {
            match layout(block) {
                Ok(_) => (),
                Err(e) => {
                    return Err(DrawError::UniformBlockLayoutMismatch {
                        name: name.to_owned(),
                        err: e,
                    });
                }
            }

            let bind_point = buffer_bind_points.get_unused().expect("Not enough buffer units");
            buffer_bind_points.set_used(bind_point);

            assert!(buffer.get_offset_bytes() == 0);     // TODO: not implemented
            let fence = buffer.add_fence();
            let block_id = block.id as gl::types::GLuint;

            buffer.prepare_and_bind_for_shared_storage(ctxt, bind_point as gl::types::GLuint);
            program.set_shader_storage_block_binding(ctxt, block_id, bind_point as gl::types::GLuint);

            Ok(fence)
        },
        _ => {
            Err(DrawError::UniformValueToBlock { name: name.to_owned() })
        }
    }
}

fn bind_uniform<P>(ctxt: &mut context::CommandContext,
                   value: &UniformValue, program: &P, location: gl::types::GLint,
                   binder: &mut bind::Binder, name: &str)
                   -> Result<(), DrawError> where P: ProgramExt
{
    assert!(location >= 0);

    match *value {
        UniformValue::Block(_, _) => {
            Err(DrawError::UniformBufferToValue {
                name: name.to_owned(),
            })
        },
        UniformValue::Bool(val) => {
            // Booleans get passed as integers.
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(val as i32));
            Ok(())
        },
        UniformValue::SignedInt(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(val));
            Ok(())
        },
        UniformValue::UnsignedInt(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::UnsignedInt(val));
            Ok(())
        },
        UniformValue::Float(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::Float(val));
            Ok(())
        },
        UniformValue::Mat2(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::Mat2(val));
            Ok(())
        },
        UniformValue::Mat3(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::Mat3(val));
            Ok(())
        },
        UniformValue::Mat4(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::Mat4(val));
            Ok(())
        },
        UniformValue::Vec2(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::Vec2(val));
            Ok(())
        },
        UniformValue::Vec3(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::Vec3(val));
            Ok(())
        },
        UniformValue::Vec4(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::Vec4(val));
            Ok(())
        },
        UniformValue::IntVec2(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::IntVec2(val));
            Ok(())
        },
        UniformValue::IntVec3(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::IntVec3(val));
            Ok(())
        },
        UniformValue::IntVec4(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::IntVec4(val));
            Ok(())
        },
        UniformValue::UnsignedIntVec2(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::UnsignedIntVec2(val));
            Ok(())
        },
        UniformValue::UnsignedIntVec3(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::UnsignedIntVec3(val));
            Ok(())
        },
        UniformValue::UnsignedIntVec4(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::UnsignedIntVec4(val));
            Ok(())
        },
        UniformValue::BoolVec2(val) => {
            let val_casted = [val[0] as i32, val[1] as i32];
            program.set_uniform(ctxt, location, &RawUniformValue::IntVec2(val_casted));
            Ok(())
        },
        UniformValue::BoolVec3(val) => {
            let val_casted = [val[0] as i32, val[1] as i32, val[2] as i32];
            program.set_uniform(ctxt, location, &RawUniformValue::IntVec3(val_casted));
            Ok(())
        },
        UniformValue::BoolVec4(val) => {
            let val_casted = [val[0] as i32, val[1] as i32, val[2] as i32, val[3] as i32];
            program.set_uniform(ctxt, location, &RawUniformValue::IntVec4(val_casted));
            Ok(())
        },
        UniformValue::Double(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::Double(val));
            Ok(())
        },
        UniformValue::DoubleMat2(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::DoubleMat2(val));
            Ok(())
        },
        UniformValue::DoubleMat3(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::DoubleMat3(val));
            Ok(())
        },
        UniformValue::DoubleMat4(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::DoubleMat4(val));
            Ok(())
        },
        UniformValue::DoubleVec2(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::DoubleVec2(val));
            Ok(())
        },
        UniformValue::DoubleVec3(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::DoubleVec3(val));
            Ok(())
        },
        UniformValue::DoubleVec4(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::DoubleVec4(val));
            Ok(())
        },
        UniformValue::Int64(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::Int64(val));
            Ok(())
        },
        UniformValue::Int64Vec2(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::Int64Vec2(val));
            Ok(())
        },
        UniformValue::Int64Vec3(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::Int64Vec3(val));
            Ok(())
        },
        UniformValue::Int64Vec4(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::Int64Vec4(val));
            Ok(())
        },
        UniformValue::UnsignedInt64(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::UnsignedInt64(val));
            Ok(())
        },
        UniformValue::UnsignedInt64Vec2(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::UnsignedInt64Vec2(val));
            Ok(())
        },
        UniformValue::UnsignedInt64Vec3(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::UnsignedInt64Vec3(val));
            Ok(())
        },
        UniformValue::UnsignedInt64Vec4(val) => {
            program.set_uniform(ctxt, location, &RawUniformValue::UnsignedInt64Vec4(val));
            Ok(())
        },
        UniformValue::Texture1d(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::CompressedTexture1d(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::SrgbTexture1d(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::CompressedSrgbTexture1d(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::IntegralTexture1d(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::UnsignedTexture1d(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::DepthTexture1d(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::Texture2d(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::CompressedTexture2d(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::SrgbTexture2d(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::CompressedSrgbTexture2d(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::IntegralTexture2d(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::UnsignedTexture2d(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::DepthTexture2d(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::Texture2dMultisample(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::SrgbTexture2dMultisample(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::IntegralTexture2dMultisample(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::UnsignedTexture2dMultisample(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::DepthTexture2dMultisample(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::Texture3d(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::CompressedTexture3d(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::SrgbTexture3d(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::CompressedSrgbTexture3d(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::IntegralTexture3d(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::UnsignedTexture3d(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::DepthTexture3d(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::Texture1dArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::CompressedTexture1dArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::SrgbTexture1dArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::CompressedSrgbTexture1dArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::IntegralTexture1dArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::UnsignedTexture1dArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::DepthTexture1dArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::Texture2dArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::CompressedTexture2dArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::SrgbTexture2dArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::CompressedSrgbTexture2dArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::IntegralTexture2dArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::UnsignedTexture2dArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::DepthTexture2dArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::Texture2dMultisampleArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::SrgbTexture2dMultisampleArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::IntegralTexture2dMultisampleArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::UnsignedTexture2dMultisampleArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::DepthTexture2dMultisampleArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::Cubemap(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::CompressedCubemap(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::SrgbCubemap(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::CompressedSrgbCubemap(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::IntegralCubemap(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::UnsignedCubemap(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::DepthCubemap(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::CubemapArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::CompressedCubemapArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::SrgbCubemapArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::CompressedSrgbCubemapArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::IntegralCubemapArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::UnsignedCubemapArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::DepthCubemapArray(texture, sampler) => {
            let pt = try!(binder.add(ctxt, &**texture, sampler));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
        UniformValue::BufferTexture(texture) => {
            let pt = try!(binder.add(ctxt, &texture, None));
            program.set_uniform(ctxt, location, &RawUniformValue::SignedInt(pt));
            Ok(())
        },
    }
}
