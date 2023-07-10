/*!

Handles binding uniforms to the OpenGL state machine.

*/
use crate::gl;

use std::collections::HashMap;
use std::hash::BuildHasherDefault;

use fnv::FnvHasher;

use crate::BufferExt;
use crate::BufferSliceExt;
use crate::DrawError;
use crate::ProgramExt;
use crate::UniformsExt;
use crate::RawUniformValue;
use crate::TextureExt;

use crate::uniforms::Uniforms;
use crate::uniforms::UniformValue;
use crate::uniforms::SamplerBehavior;
use crate::uniforms::ImageUnitBehavior;


use crate::context::CommandContext;
use crate::buffer::Inserter;

use crate::utils::bitsfield::Bitsfield;

use crate::program;
use crate::context;
use crate::version::Version;
use crate::version::Api;

impl<U> UniformsExt for U where U: Uniforms {
    fn bind_uniforms<'a, P>(&'a self, mut ctxt: &mut CommandContext<'_>, program: &P,
                            fences: &mut Vec<Inserter<'a>>)
                            -> Result<(), DrawError>
                            where P: ProgramExt
    {
        let mut texture_bind_points = Bitsfield::new();
        let mut uniform_buffer_bind_points = Bitsfield::new();
        let mut shared_storage_buffer_bind_points = Bitsfield::new();
        let mut image_unit_bind_points = Bitsfield::new();
        image_unit_bind_points.set_used(0); // Trying to attach data to image unit 0 would not go well

        // Subroutine uniforms must be bound all at once, so we collect them first and process them at the end.
        // The vec contains the uniform we want to set and the value we want to set it to.
        let mut subroutine_bindings: HashMap<program::ShaderStage, Vec<(&program::SubroutineUniform, &str)>, _>
            = HashMap::with_hasher(Default::default());

        let mut visiting_result = Ok(());
        self.visit_values(|name, value| {
            if visiting_result.is_err() { return; }

            if let Some(uniform) = program.get_uniform(name) {
                // TODO: remove the size member
                debug_assert!(uniform.size.is_none());

                if !value.is_usable_with(&uniform.ty) {
                    visiting_result = Err(DrawError::UniformTypeMismatch {
                        name: name.to_owned(),
                        expected: uniform.ty,
                    });
                    return;
                }

                match bind_uniform(&mut ctxt, &value, program, uniform.location,
                                   &mut texture_bind_points, &mut image_unit_bind_points, name)
                {
                    Ok(_) => (),
                    Err(e) => {
                        visiting_result = Err(e);
                        
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
            } else if let Some(block) = program.get_atomic_counters().get(name) {
                let fence = match bind_atomic_counter(&mut ctxt, &value, block, program,
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
            } else if let UniformValue::Subroutine(stage, sr_name) = value {
                if let Some(subroutine_uniform) = program.get_subroutine_data().subroutine_uniforms.get(&(name.into(), stage)) {
                    subroutine_bindings.entry(stage).or_insert(Vec::new());
                    let vec = subroutine_bindings.get_mut(&stage).unwrap();
                    vec.push((subroutine_uniform, sr_name));
                }
            }
        });

        // Process all subroutine uniforms in one batch.
        if !subroutine_bindings.is_empty() {
            match bind_subroutine_uniforms(&mut ctxt, program, &subroutine_bindings) {
                Ok(_) => (),
                Err(e) => {
                    visiting_result = Err(e);
                }
            }
        }

        visiting_result
    }
}

fn bind_subroutine_uniforms<P>(ctxt: &mut context::CommandContext<'_>, program: &P,
                            subroutine_bindings: &HashMap<program::ShaderStage, Vec<(&program::SubroutineUniform, &str)>, BuildHasherDefault<FnvHasher>>)
                            -> Result<(), DrawError>
                            where P: ProgramExt
{
    let subroutine_data = program.get_subroutine_data();
    for (stage, bindings) in subroutine_bindings {
        // Validate that all subroutine uniforms of this stage are set, otherwise OpenGL will throw an error.
        let set_cnt = bindings.len();
        let expected_cnt = subroutine_data.subroutine_uniforms.iter()
                                  .filter(|&(&(_, uni_stage), _)| *stage == uni_stage)
                                  .count();
        if set_cnt != expected_cnt {
            return Err(DrawError::SubroutineUniformMissing {
                stage: *stage,
                real_count: set_cnt,
                expected_count: expected_cnt,
            })
        }

        // Build the indices array
        let mut indices = vec![0 as gl::types::GLuint; *subroutine_data.location_counts.get(stage).unwrap()];
        for binding in bindings {
            let uniform = binding.0;
            let subroutine_str = binding.1;
            let subroutine = match uniform.compatible_subroutines.iter()
                                   .find(|subroutine| subroutine.name == subroutine_str) {
                Some(subroutine) => subroutine,
                None => return Err(DrawError::SubroutineNotFound {
                                    stage: *stage,
                                    name: subroutine_str.into(),
                                })
            };

            indices[uniform.location as usize] = subroutine.index;
        }
        program.set_subroutine_uniforms_for_stage(ctxt, *stage, &indices);
    }
    Ok(())
}

fn bind_uniform_block<'a, P>(ctxt: &mut context::CommandContext<'_>, value: &UniformValue<'a>,
                             block: &program::UniformBlock,
                             program: &P, buffer_bind_points: &mut Bitsfield, name: &str)
                             -> Result<Option<Inserter<'a>>, DrawError>
                             where P: ProgramExt
{
    match value {
        UniformValue::Block(buffer, ref layout) => {
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

fn bind_shared_storage_block<'a, P>(ctxt: &mut context::CommandContext<'_>, value: &UniformValue<'a>,
                                    block: &program::UniformBlock,
                                    program: &P, buffer_bind_points: &mut Bitsfield, name: &str)
                                    -> Result<Option<Inserter<'a>>, DrawError>
                                    where P: ProgramExt
{
    match value {
        UniformValue::Block(buffer, ref layout) => {
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


fn bind_atomic_counter<'a, P>(ctxt: &mut context::CommandContext<'_>, value: &UniformValue<'a>,
                              block: &program::UniformBlock,
                              program: &P, name: &str)
                              -> Result<Option<Inserter<'a>>, DrawError>
                              where P: ProgramExt
{
    match value {
        UniformValue::Block(buffer, ref layout) => {
            match layout(block) {
                Ok(_) => (),
                Err(e) => {
                    return Err(DrawError::UniformBlockLayoutMismatch {
                        name: name.to_owned(),
                        err: e,
                    });
                }
            }

            assert!(buffer.get_offset_bytes() == 0);     // TODO: not implemented
            let fence = buffer.add_fence();

            buffer.prepare_and_bind_for_atomic_counter(ctxt, block.initial_binding as gl::types::GLuint);

            Ok(fence)
        },
        _ => {
            Err(DrawError::UniformValueToBlock { name: name.to_owned() })
        }
    }
}

fn bind_uniform<P>(ctxt: &mut context::CommandContext,
                   value: &UniformValue, program: &P, location: gl::types::GLint,
                   texture_bind_points: &mut Bitsfield,
                   image_unit_bind_points: &mut Bitsfield,
                   name: &str)
                   -> Result<(), DrawError> where P: ProgramExt
{
    // The spec only states that a negative location is an error.
    // This fact is abused by some implementations (e.g. vitaGL)
    // which actually use negative values for uniform locations.
    #[cfg(target_os = "vita")]
    assert!(location != -1);

    #[cfg(not(target_os = "vita"))]
    assert!(location >= 0);

    match *value {
        UniformValue::Block(_, _) => {
            Err(DrawError::UniformBufferToValue {
                name: name.to_owned(),
            })
        },
        UniformValue::Subroutine(_, _) => {
            Err(DrawError::SubroutineUniformToValue {
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
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::CompressedTexture1d(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::SrgbTexture1d(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::CompressedSrgbTexture1d(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::IntegralTexture1d(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::UnsignedTexture1d(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::DepthTexture1d(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::Texture2d(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::CompressedTexture2d(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::SrgbTexture2d(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::CompressedSrgbTexture2d(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::IntegralTexture2d(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::UnsignedTexture2d(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::DepthTexture2d(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::Texture2dMultisample(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::SrgbTexture2dMultisample(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::IntegralTexture2dMultisample(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::UnsignedTexture2dMultisample(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::DepthTexture2dMultisample(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::Texture3d(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::CompressedTexture3d(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::SrgbTexture3d(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::CompressedSrgbTexture3d(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::IntegralTexture3d(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::UnsignedTexture3d(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::DepthTexture3d(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::Texture1dArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::CompressedTexture1dArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::SrgbTexture1dArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::CompressedSrgbTexture1dArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::IntegralTexture1dArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::UnsignedTexture1dArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::DepthTexture1dArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::Texture2dArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::CompressedTexture2dArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::SrgbTexture2dArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::CompressedSrgbTexture2dArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::IntegralTexture2dArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::UnsignedTexture2dArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::DepthTexture2dArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::Texture2dMultisampleArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::SrgbTexture2dMultisampleArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::IntegralTexture2dMultisampleArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::UnsignedTexture2dMultisampleArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::DepthTexture2dMultisampleArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::Cubemap(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::CompressedCubemap(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::SrgbCubemap(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::CompressedSrgbCubemap(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::IntegralCubemap(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::UnsignedCubemap(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::DepthCubemap(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::CubemapArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::CompressedCubemapArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::SrgbCubemapArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::CompressedSrgbCubemapArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::IntegralCubemapArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::UnsignedCubemapArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::DepthCubemapArray(texture, sampler) => {
            bind_texture_uniform(ctxt, &**texture, sampler, location, program, texture_bind_points)
        },
        UniformValue::BufferTexture(texture) => {
            bind_texture_uniform(ctxt, &texture, None, location, program, texture_bind_points)
        },
        UniformValue::Image1d(texture, unit) => {
            bind_image_uniform(ctxt, &**texture, unit, location, program, image_unit_bind_points)
        },
        UniformValue::IntegralImage1d(texture, unit) => {
            bind_image_uniform(ctxt, &**texture, unit, location, program, image_unit_bind_points)
        },
        UniformValue::UnsignedImage1d(texture, unit) => {
            bind_image_uniform(ctxt, &**texture, unit, location, program, image_unit_bind_points)
        },
        UniformValue::Image2d(texture, unit) => {
            bind_image_uniform(ctxt, &**texture, unit, location, program, image_unit_bind_points)
        },
        UniformValue::IntegralImage2d(texture, unit) => {
            bind_image_uniform(ctxt, &**texture, unit, location, program, image_unit_bind_points)
        },
        UniformValue::UnsignedImage2d(texture, unit) => {
            bind_image_uniform(ctxt, &**texture, unit, location, program, image_unit_bind_points)
        },
        UniformValue::Image3d(texture, unit) => {
            bind_image_uniform(ctxt, &**texture, unit, location, program, image_unit_bind_points)
        },
        UniformValue::IntegralImage3d(texture, unit) => {
            bind_image_uniform(ctxt, &**texture, unit, location, program, image_unit_bind_points)
        },
        UniformValue::UnsignedImage3d(texture, unit) => {
            bind_image_uniform(ctxt, &**texture, unit, location, program, image_unit_bind_points)
        },
        UniformValue::Image1dArray(texture, unit) => {
            bind_image_uniform(ctxt, &**texture, unit, location, program, image_unit_bind_points)
        },
        UniformValue::IntegralImage1dArray(texture, unit) => {
            bind_image_uniform(ctxt, &**texture, unit, location, program, image_unit_bind_points)
        },
        UniformValue::UnsignedImage1dArray(texture, unit) => {
            bind_image_uniform(ctxt, &**texture, unit, location, program, image_unit_bind_points)
        },
        UniformValue::Image2dArray(texture, unit) => {
            bind_image_uniform(ctxt, &**texture, unit, location, program, image_unit_bind_points)
        },
        UniformValue::IntegralImage2dArray(texture, unit) => {
            bind_image_uniform(ctxt, &**texture, unit, location, program, image_unit_bind_points)
        },
        UniformValue::UnsignedImage2dArray(texture, unit) => {
            bind_image_uniform(ctxt, &**texture, unit, location, program, image_unit_bind_points)
        },
        UniformValue::ImageCube(texture, unit) => {
            bind_image_uniform(ctxt, &**texture, unit, location, program, image_unit_bind_points)
        },
        UniformValue::IntegralImageCube(texture, unit) => {
            bind_image_uniform(ctxt, &**texture, unit, location, program, image_unit_bind_points)
        },
        UniformValue::UnsignedImageCube(texture, unit) => {
            bind_image_uniform(ctxt, &**texture, unit, location, program, image_unit_bind_points)
        },
        UniformValue::ImageCubeArray(texture, unit) => {
            bind_image_uniform(ctxt, &**texture, unit, location, program, image_unit_bind_points)
        },
        UniformValue::IntegralImageCubeArray(texture, unit) => {
            bind_image_uniform(ctxt, &**texture, unit, location, program, image_unit_bind_points)
        },
        UniformValue::UnsignedImageCubeArray(texture, unit) => {
            bind_image_uniform(ctxt, &**texture, unit, location, program, image_unit_bind_points)
        },
    }
}

fn bind_texture_uniform<P, T>(ctxt: &mut context::CommandContext<'_>,
                              texture: &T, sampler: Option<SamplerBehavior>,
                              location: gl::types::GLint, program: &P,
                              texture_bind_points: &mut Bitsfield)
                              -> Result<(), DrawError> where P: ProgramExt, T: TextureExt
{
    texture.prepare_for_access(ctxt, crate::TextureAccess::TextureFetch);
    
    let sampler = if let Some(sampler) = sampler {
        Some(crate::sampler_object::get_sampler(ctxt, &sampler)?)
    } else {
        None
    };

    let sampler = sampler.unwrap_or(0);

    // finding an appropriate texture unit
    let texture_unit =
        ctxt.state.texture_units
            .iter().enumerate()
            .find(|&(unit, content)| {
                content.texture == texture.get_texture_id() && (content.sampler == sampler ||
                                                        !texture_bind_points.is_used(unit as u16))
            })
            .map(|(unit, _)| unit as u16)
            .or_else(|| {
                if ctxt.state.texture_units.len() <
                    ctxt.capabilities.max_combined_texture_image_units as usize
                {
                    Some(ctxt.state.texture_units.len() as u16)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                texture_bind_points.get_unused().expect("Not enough texture units available")
            });
    assert!((texture_unit as gl::types::GLint) <
            ctxt.capabilities.max_combined_texture_image_units);
    texture_bind_points.set_used(texture_unit);

    // updating the program to use the right unit
    program.set_uniform(ctxt, location,
                        &RawUniformValue::SignedInt(texture_unit as gl::types::GLint));

    // updating the state of the texture unit
    if ctxt.state.texture_units.len() <= texture_unit as usize {
        for _ in ctxt.state.texture_units.len() .. texture_unit as usize + 1 {
            ctxt.state.texture_units.push(Default::default());
        }
    }

    // TODO: do better
    if ctxt.state.texture_units[texture_unit as usize].texture != texture.get_texture_id() ||
       ctxt.state.texture_units[texture_unit as usize].sampler != sampler
    {
        // TODO: what if it's not supported?
        if ctxt.state.active_texture != texture_unit as gl::types::GLenum {
            unsafe { ctxt.gl.ActiveTexture(texture_unit as gl::types::GLenum + gl::TEXTURE0) };
            ctxt.state.active_texture = texture_unit as gl::types::GLenum;
        }

        texture.bind_to_current(ctxt);

        if ctxt.state.texture_units[texture_unit as usize].sampler != sampler {
            assert!(ctxt.version >= &Version(Api::Gl, 3, 3) ||
                    ctxt.version >= &Version(Api::GlEs, 3, 0) ||
                    ctxt.extensions.gl_arb_sampler_objects);

            unsafe { ctxt.gl.BindSampler(texture_unit as gl::types::GLenum, sampler); }
            ctxt.state.texture_units[texture_unit as usize].sampler = sampler;
        }
    }

    Ok(())
}

fn bind_image_uniform<P, T>(
    ctxt: &mut context::CommandContext,
    texture: &T, unit_behavior: Option<ImageUnitBehavior>,
    location: gl::types::GLint, program: &P,
    image_unit_bind_points: &mut Bitsfield
) -> Result<(), DrawError>
  where P: ProgramExt, T: TextureExt
{
    use crate::ToGlEnum;

    let unit_behavior = unit_behavior.expect("Unit behavior should always be provided");
    let image_unit = match image_unit_bind_points.get_unused() {
        Some(unit) => unit,
        None => return Err(DrawError::InsufficientImageUnits),
    };
    image_unit_bind_points.set_used(image_unit);

    // Update the program to use the right unit
    program.set_uniform(ctxt, location,
                        &RawUniformValue::SignedInt(image_unit as gl::types::GLint));

    // "If layered is false, then only a single level identified by <layer> will be bound"
    let (layered, layer) = match unit_behavior.layer {
        None => (true, 0),
        Some(l) => (false, l)
    };
    use crate::uniforms::ImageUnitAccess;
    let will_write =
        unit_behavior.access.eq(&ImageUnitAccess::Write) | unit_behavior.access.eq(&ImageUnitAccess::ReadWrite);
    
    texture.prepare_for_access(ctxt, crate::TextureAccess::ImageUnit { will_write });

    unsafe {
        ctxt.gl.BindImageTexture(
            image_unit as gl::types::GLuint,
            texture.get_texture_id(),
            unit_behavior.level as i32,
            if layered { 1 } else { 0 },
            layer as i32,
            unit_behavior.access.to_glenum(),
            unit_behavior.format.to_glenum(),
        )
    }

    Ok(())
}
