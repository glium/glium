use gl;
use libc;

use std::ffi;
use std::mem;
use std::collections::hash_state::DefaultState;
use std::collections::HashMap;
use std::default::Default;
use util::FnvHasher;

use context;
use context::CommandContext;
use context::GlVersion;
use version::Api;

use uniforms::UniformType;
use vertex::AttributeType;

use Handle;

/// Information about a uniform (except its name).
#[derive(Debug, Copy)]
pub struct Uniform {
    /// The location of the uniform.
    ///
    /// This is internal information, you probably don't need to use it.
    pub location: i32,

    /// Type of the uniform.
    pub ty: UniformType,

    /// If it is an array, the number of elements.
    pub size: Option<usize>,
}

/// Information about a uniform block (except its name).
#[derive(Debug, Clone)]
pub struct UniformBlock {
    /// The binding point of the uniform.
    ///
    /// This is internal information, you probably don't need to use it.
    pub binding: i32,

    /// Size in bytes of the data in the block.
    pub size: usize,

    /// List of elements in the block.
    pub members: Vec<UniformBlockMember>,
}

/// Information about a uniform inside a block.
#[derive(Debug, Clone)]
pub struct UniformBlockMember {
    /// Name of the member.
    pub name: String,

    /// Offset of the member in the block.
    pub offset: usize,

    /// Type of the uniform.
    pub ty: UniformType,

    /// If it is an array, the number of elements.
    pub size: Option<usize>,
}

/// Information about an attribute of a program (except its name).
///
/// Internal struct. Not public.
#[derive(Debug, Copy)]
#[doc(hidden)]
pub struct Attribute {
    pub location: gl::types::GLint,
    pub ty: gl::types::GLenum,
    pub size: gl::types::GLint,
}

/// Describes a varying that is being output with transform feedback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformFeedbackVarying {
    /// Name of the variable.
    pub name: String,

    /// Size in bytes of this value.
    pub size: usize,

    /// Type of the value.
    pub ty: AttributeType,
}

/// Describes the mode that is used when transform feedback is enabled.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TransformFeedbackMode {
    /// Each value is interleaved in the same buffer.
    Interleaved,

    /// Each value will go in a separate buffer.
    Separate,
}

pub unsafe fn reflect_uniforms(ctxt: &mut CommandContext, program: Handle)
                               -> HashMap<String, Uniform, DefaultState<FnvHasher>>
{
    // reflecting program uniforms
    let mut uniforms = HashMap::with_hash_state(Default::default());

    // number of active uniforms
    let active_uniforms = {
        let mut active_uniforms: gl::types::GLint = mem::uninitialized();
        match program {
            Handle::Id(program) => {
                assert!(ctxt.version >= &GlVersion(Api::Gl, 2, 0));
                ctxt.gl.GetProgramiv(program, gl::ACTIVE_UNIFORMS, &mut active_uniforms);
            },
            Handle::Handle(program) => {
                assert!(ctxt.extensions.gl_arb_shader_objects);
                ctxt.gl.GetObjectParameterivARB(program, gl::OBJECT_ACTIVE_UNIFORMS_ARB,
                                                &mut active_uniforms);
            }
        };
        active_uniforms
    };

    for uniform_id in range(0, active_uniforms) {
        let mut uniform_name_tmp: Vec<u8> = Vec::with_capacity(64);
        let mut uniform_name_tmp_len = 63;

        let mut data_type: gl::types::GLenum = mem::uninitialized();
        let mut data_size: gl::types::GLint = mem::uninitialized();

        match program {
            Handle::Id(program) => {
                assert!(ctxt.version >= &GlVersion(Api::Gl, 2, 0));
                ctxt.gl.GetActiveUniform(program, uniform_id as gl::types::GLuint,
                                         uniform_name_tmp_len, &mut uniform_name_tmp_len,
                                         &mut data_size, &mut data_type,
                                         uniform_name_tmp.as_mut_slice().as_mut_ptr()
                                           as *mut gl::types::GLchar);
            },
            Handle::Handle(program) => {
                assert!(ctxt.extensions.gl_arb_shader_objects);
                ctxt.gl.GetActiveUniformARB(program, uniform_id as gl::types::GLuint,
                                            uniform_name_tmp_len, &mut uniform_name_tmp_len,
                                            &mut data_size, &mut data_type,
                                            uniform_name_tmp.as_mut_slice().as_mut_ptr()
                                              as *mut gl::types::GLchar);
            }
        };

        uniform_name_tmp.set_len(uniform_name_tmp_len as usize);

        let uniform_name = String::from_utf8(uniform_name_tmp).unwrap();
        let location = match program {
            Handle::Id(program) => {
                assert!(ctxt.version >= &GlVersion(Api::Gl, 2, 0));
                ctxt.gl.GetUniformLocation(program,
                                           ffi::CString::from_slice(uniform_name.as_bytes())
                                             .as_bytes_with_nul().as_ptr() as *const libc::c_char)
            },
            Handle::Handle(program) => {
                assert!(ctxt.extensions.gl_arb_shader_objects);
                ctxt.gl.GetUniformLocationARB(program,
                                              ffi::CString::from_slice(uniform_name.as_bytes())
                                                .as_bytes_with_nul().as_ptr() as *const libc::c_char)
            }
        };

        uniforms.insert(uniform_name, Uniform {
            location: location as i32,
            ty: glenum_to_uniform_type(data_type),
            size: if data_size == 1 { None } else { Some(data_size as usize) },
        });
    }

    uniforms
}

pub unsafe fn reflect_attributes(ctxt: &mut CommandContext, program: Handle)
                                 -> HashMap<String, Attribute, DefaultState<FnvHasher>>
{
    let mut attributes = HashMap::with_hash_state(Default::default());

    // number of active attributes
    let active_attributes = {
        let mut active_attributes: gl::types::GLint = mem::uninitialized();
        match program {
            Handle::Id(program) => {
                assert!(ctxt.version >= &GlVersion(Api::Gl, 2, 0));
                ctxt.gl.GetProgramiv(program, gl::ACTIVE_ATTRIBUTES, &mut active_attributes);
            },
            Handle::Handle(program) => {
                assert!(ctxt.extensions.gl_arb_vertex_shader);
                ctxt.gl.GetObjectParameterivARB(program, gl::OBJECT_ACTIVE_ATTRIBUTES_ARB,
                                                &mut active_attributes);
            }
        };
        active_attributes
    };

    for attribute_id in range(0, active_attributes) {
        let mut attr_name_tmp: Vec<u8> = Vec::with_capacity(64);
        let mut attr_name_tmp_len = 63;

        let mut data_type: gl::types::GLenum = mem::uninitialized();
        let mut data_size: gl::types::GLint = mem::uninitialized();

        match program {
            Handle::Id(program) => {
                assert!(ctxt.version >= &GlVersion(Api::Gl, 2, 0));
                ctxt.gl.GetActiveAttrib(program, attribute_id as gl::types::GLuint,
                                        attr_name_tmp_len, &mut attr_name_tmp_len, &mut data_size,
                                        &mut data_type, attr_name_tmp.as_mut_slice().as_mut_ptr()
                                          as *mut gl::types::GLchar);
            },
            Handle::Handle(program) => {
                assert!(ctxt.extensions.gl_arb_vertex_shader);
                ctxt.gl.GetActiveAttribARB(program, attribute_id as gl::types::GLuint,
                                           attr_name_tmp_len, &mut attr_name_tmp_len, &mut data_size,
                                           &mut data_type, attr_name_tmp.as_mut_slice().as_mut_ptr()
                                             as *mut gl::types::GLchar);
            }
        };

        attr_name_tmp.set_len(attr_name_tmp_len as usize);

        let attr_name = String::from_utf8(attr_name_tmp).unwrap();
        if attr_name.starts_with("gl_") {   // ignoring everything built-in
            continue;
        }

        let location = match program {
            Handle::Id(program) => {
                assert!(ctxt.version >= &GlVersion(Api::Gl, 2, 0));
                ctxt.gl.GetAttribLocation(program,
                                          ffi::CString::from_slice(attr_name.as_bytes())
                                            .as_bytes_with_nul().as_ptr() as *const libc::c_char)
            },
            Handle::Handle(program) => {
                assert!(ctxt.extensions.gl_arb_vertex_shader);
                ctxt.gl.GetAttribLocationARB(program,
                                             ffi::CString::from_slice(attr_name.as_bytes())
                                               .as_bytes_with_nul().as_ptr() as *const libc::c_char)
            }
        };

        attributes.insert(attr_name, Attribute {
            location: location,
            ty: data_type,
            size: data_size
        });
    }

    attributes
}

pub unsafe fn reflect_uniform_blocks(ctxt: &mut CommandContext, program: Handle)
                                     -> HashMap<String, UniformBlock, DefaultState<FnvHasher>>
{
    // uniform blocks are not supported, so there's none
    if ctxt.version < &context::GlVersion(Api::Gl, 3, 1) {
        return HashMap::with_hash_state(Default::default());
    }

    let program = match program {
        Handle::Id(id) => id,
        _ => unreachable!()
    };

    let mut blocks = HashMap::with_hash_state(Default::default());

    let mut active_blocks: gl::types::GLint = mem::uninitialized();
    ctxt.gl.GetProgramiv(program, gl::ACTIVE_UNIFORM_BLOCKS, &mut active_blocks);

    let mut active_blocks_max_name_len: gl::types::GLint = mem::uninitialized();
    ctxt.gl.GetProgramiv(program, gl::ACTIVE_UNIFORM_BLOCK_MAX_NAME_LENGTH,
                         &mut active_blocks_max_name_len);

    for block_id in range(0, active_blocks) {
        // getting the name of the block
        let name = {
            let mut name_tmp: Vec<u8> = Vec::with_capacity(1 + active_blocks_max_name_len
                                                           as usize);
            let mut name_tmp_len = active_blocks_max_name_len;

            ctxt.gl.GetActiveUniformBlockName(program, block_id as gl::types::GLuint,
                                              name_tmp_len, &mut name_tmp_len,
                                              name_tmp.as_mut_slice().as_mut_ptr()
                                              as *mut gl::types::GLchar);
            name_tmp.set_len(name_tmp_len as usize);
            String::from_utf8(name_tmp).unwrap()
        };

        // binding point for this block
        let mut binding: gl::types::GLint = mem::uninitialized();
        ctxt.gl.GetActiveUniformBlockiv(program, block_id as gl::types::GLuint,
                                        gl::UNIFORM_BLOCK_BINDING, &mut binding);

        // number of bytes
        let mut block_size: gl::types::GLint = mem::uninitialized();
        ctxt.gl.GetActiveUniformBlockiv(program, block_id as gl::types::GLuint,
                                        gl::UNIFORM_BLOCK_DATA_SIZE, &mut block_size);

        // number of members
        let mut num_members: gl::types::GLint = mem::uninitialized();
        ctxt.gl.GetActiveUniformBlockiv(program, block_id as gl::types::GLuint,
                                        gl::UNIFORM_BLOCK_ACTIVE_UNIFORMS, &mut num_members);

        // indices of the members
        let mut members_indices = ::std::iter::repeat(0).take(num_members as usize)
                                                        .collect::<Vec<gl::types::GLuint>>();
        ctxt.gl.GetActiveUniformBlockiv(program, block_id as gl::types::GLuint,
                                        gl::UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES,
                                        members_indices.as_mut_ptr() as *mut gl::types::GLint);

        // getting the offsets of the members
        let mut member_offsets = ::std::iter::repeat(0).take(num_members as usize)
                                                       .collect::<Vec<gl::types::GLint>>();
        ctxt.gl.GetActiveUniformsiv(program, num_members, members_indices.as_ptr(),
                                    gl::UNIFORM_OFFSET, member_offsets.as_mut_ptr());

        // getting the types of the members
        let mut member_types = ::std::iter::repeat(0).take(num_members as usize)
                                                     .collect::<Vec<gl::types::GLint>>();
        ctxt.gl.GetActiveUniformsiv(program, num_members, members_indices.as_ptr(),
                                    gl::UNIFORM_TYPE, member_types.as_mut_ptr());

        // getting the array sizes of the members
        let mut member_size = ::std::iter::repeat(0).take(num_members as usize)
                                                    .collect::<Vec<gl::types::GLint>>();
        ctxt.gl.GetActiveUniformsiv(program, num_members, members_indices.as_ptr(),
                                    gl::UNIFORM_SIZE, member_size.as_mut_ptr());

        // getting the length of the names of the members
        let mut member_name_len = ::std::iter::repeat(0).take(num_members as usize)
                                                         .collect::<Vec<gl::types::GLint>>();
        ctxt.gl.GetActiveUniformsiv(program, num_members, members_indices.as_ptr(),
                                    gl::UNIFORM_NAME_LENGTH, member_name_len.as_mut_ptr());

        // getting the names of the members
        let member_names = member_name_len.iter().zip(members_indices.iter())
                                          .map(|(&name_len, &index)|
        {
            let mut name_tmp: Vec<u8> = Vec::with_capacity(1 + name_len as usize);
            let mut name_len_tmp = name_len;
            ctxt.gl.GetActiveUniformName(program, index, name_len, &mut name_len_tmp,
                                         name_tmp.as_mut_ptr() as *mut gl::types::GLchar);
            name_tmp.set_len(name_len_tmp as usize);

            String::from_utf8(name_tmp).unwrap()
        }).collect::<Vec<_>>();

        // now computing the list of members
        let members = member_names.into_iter().enumerate().map(|(index, name)| {
            UniformBlockMember {
                name: name,
                offset: member_offsets[index] as usize,
                ty: glenum_to_uniform_type(member_types[index] as gl::types::GLenum),
                size: match member_size[index] {
                    1 => None,
                    a => Some(a as usize),
                },
            }
        }).collect::<Vec<_>>();

        // finally inserting into the blocks list
        blocks.insert(name, UniformBlock {
            binding: binding as i32,
            size: block_size as usize,
            members: members,
        });
    }

    blocks
}

pub unsafe fn reflect_transform_feedback(ctxt: &mut CommandContext, program: Handle)
                                         -> Option<(Vec<TransformFeedbackVarying>,
                                                    TransformFeedbackMode)>
{
    let program = match program {
        // transform feedback not supported
        Handle::Handle(_) => return None,
        Handle::Id(id) => id
    };

    // transform feedback not supported
    if ctxt.version < &GlVersion(Api::Gl, 3, 0) && !ctxt.extensions.gl_ext_transform_feedback {
        return None;
    }

    // querying the number of varying
    let num_varyings = {
        let mut num_varyings: gl::types::GLint = mem::uninitialized();

        if ctxt.version >= &GlVersion(Api::Gl, 3, 0) {
            ctxt.gl.GetProgramiv(program, gl::TRANSFORM_FEEDBACK_VARYINGS, &mut num_varyings);
        } else if ctxt.extensions.gl_ext_transform_feedback {
            ctxt.gl.GetProgramiv(program, gl::TRANSFORM_FEEDBACK_VARYINGS_EXT, &mut num_varyings);
        } else {
            unreachable!();
        }

        num_varyings
    };

    // no need to request other things if there are no varying
    if num_varyings == 0 {
        return None;
    }

    // querying "interleaved" or "separate"
    let buffer_mode = {
        let mut buffer_mode: gl::types::GLint = mem::uninitialized();

        if ctxt.version >= &GlVersion(Api::Gl, 3, 0) {
            ctxt.gl.GetProgramiv(program, gl::TRANSFORM_FEEDBACK_BUFFER_MODE, &mut buffer_mode);
        } else if ctxt.extensions.gl_ext_transform_feedback {
            ctxt.gl.GetProgramiv(program, gl::TRANSFORM_FEEDBACK_BUFFER_MODE_EXT, &mut buffer_mode);
        } else {
            unreachable!();
        }

        glenum_to_transform_feedback_mode(buffer_mode as gl::types::GLenum)
    };

    // the max length includes the null terminator
    let mut max_buffer_len: gl::types::GLint = mem::uninitialized();
    if ctxt.version >= &GlVersion(Api::Gl, 3, 0) {
        ctxt.gl.GetProgramiv(program, gl::TRANSFORM_FEEDBACK_VARYING_MAX_LENGTH,
                             &mut max_buffer_len);
    } else if ctxt.extensions.gl_ext_transform_feedback {
        ctxt.gl.GetProgramiv(program, gl::TRANSFORM_FEEDBACK_VARYING_MAX_LENGTH_EXT,
                             &mut max_buffer_len);
    } else {
        unreachable!();
    }

    let mut result = Vec::with_capacity(num_varyings as usize);

    for index in (0 .. num_varyings as gl::types::GLuint) {
        let mut name_tmp: Vec<u8> = Vec::with_capacity(max_buffer_len as usize);
        let mut name_tmp_len = max_buffer_len;

        let mut size = mem::uninitialized();
        let mut ty = mem::uninitialized();

        if ctxt.version >= &GlVersion(Api::Gl, 3, 0) {
            ctxt.gl.GetTransformFeedbackVarying(program, index, name_tmp_len, &mut name_tmp_len,
                                                &mut size, &mut ty, name_tmp.as_mut_ptr()
                                                as *mut gl::types::GLchar);
        } else if ctxt.extensions.gl_ext_transform_feedback {
            ctxt.gl.GetTransformFeedbackVaryingEXT(program, index, name_tmp_len,
                                                   &mut name_tmp_len, &mut size, &mut ty,
                                                   name_tmp.as_mut_ptr()
                                                   as *mut gl::types::GLchar);
        } else {
            unreachable!();
        }

        name_tmp.set_len(name_tmp_len as usize);
        let name = String::from_utf8(name_tmp).unwrap();

        result.push(TransformFeedbackVarying {
            name: name,
            size: size as usize,
            ty: glenum_to_attribute_type(ty as gl::types::GLenum),
        });
    }

    Some((result, buffer_mode))
}

fn glenum_to_uniform_type(ty: gl::types::GLenum) -> UniformType {
    match ty {
        gl::FLOAT => UniformType::Float,
        gl::FLOAT_VEC2 => UniformType::FloatVec2,
        gl::FLOAT_VEC3 => UniformType::FloatVec3,
        gl::FLOAT_VEC4 => UniformType::FloatVec4,
        gl::DOUBLE => UniformType::Double,
        gl::DOUBLE_VEC2 => UniformType::DoubleVec2,
        gl::DOUBLE_VEC3 => UniformType::DoubleVec3,
        gl::DOUBLE_VEC4 => UniformType::DoubleVec4,
        gl::INT => UniformType::Int,
        gl::INT_VEC2 => UniformType::IntVec2,
        gl::INT_VEC3 => UniformType::IntVec3,
        gl::INT_VEC4 => UniformType::IntVec4,
        gl::UNSIGNED_INT => UniformType::UnsignedInt,
        gl::UNSIGNED_INT_VEC2 => UniformType::UnsignedIntVec2,
        gl::UNSIGNED_INT_VEC3 => UniformType::UnsignedIntVec3,
        gl::UNSIGNED_INT_VEC4 => UniformType::UnsignedIntVec4,
        gl::BOOL => UniformType::Bool,
        gl::BOOL_VEC2 => UniformType::BoolVec2,
        gl::BOOL_VEC3 => UniformType::BoolVec3,
        gl::BOOL_VEC4 => UniformType::BoolVec4,
        gl::FLOAT_MAT2 => UniformType::FloatMat2,
        gl::FLOAT_MAT3 => UniformType::FloatMat3,
        gl::FLOAT_MAT4 => UniformType::FloatMat4,
        gl::FLOAT_MAT2x3 => UniformType::FloatMat2x3,
        gl::FLOAT_MAT2x4 => UniformType::FloatMat2x4,
        gl::FLOAT_MAT3x2 => UniformType::FloatMat3x2,
        gl::FLOAT_MAT3x4 => UniformType::FloatMat3x4,
        gl::FLOAT_MAT4x2 => UniformType::FloatMat4x2,
        gl::FLOAT_MAT4x3 => UniformType::FloatMat4x3,
        gl::DOUBLE_MAT2 => UniformType::DoubleMat2,
        gl::DOUBLE_MAT3 => UniformType::DoubleMat3,
        gl::DOUBLE_MAT4 => UniformType::DoubleMat4,
        gl::DOUBLE_MAT2x3 => UniformType::DoubleMat2x3,
        gl::DOUBLE_MAT2x4 => UniformType::DoubleMat2x4,
        gl::DOUBLE_MAT3x2 => UniformType::DoubleMat3x2,
        gl::DOUBLE_MAT3x4 => UniformType::DoubleMat3x4,
        gl::DOUBLE_MAT4x2 => UniformType::DoubleMat4x2,
        gl::DOUBLE_MAT4x3 => UniformType::DoubleMat4x3,
        gl::SAMPLER_1D => UniformType::Sampler1d,
        gl::SAMPLER_2D => UniformType::Sampler2d,
        gl::SAMPLER_3D => UniformType::Sampler3d,
        gl::SAMPLER_CUBE => UniformType::SamplerCube,
        gl::SAMPLER_1D_SHADOW => UniformType::Sampler1dShadow,
        gl::SAMPLER_2D_SHADOW => UniformType::Sampler2dShadow,
        gl::SAMPLER_1D_ARRAY => UniformType::Sampler1dArray,
        gl::SAMPLER_2D_ARRAY => UniformType::Sampler2dArray,
        gl::SAMPLER_1D_ARRAY_SHADOW => UniformType::Sampler1dArrayShadow,
        gl::SAMPLER_2D_ARRAY_SHADOW => UniformType::Sampler2dArrayShadow,
        gl::SAMPLER_2D_MULTISAMPLE => UniformType::Sampler2dMultisample,
        gl::SAMPLER_2D_MULTISAMPLE_ARRAY => UniformType::Sampler2dMultisampleArray,
        gl::SAMPLER_CUBE_SHADOW => UniformType::SamplerCubeShadow,
        gl::SAMPLER_BUFFER => UniformType::SamplerBuffer,
        gl::SAMPLER_2D_RECT => UniformType::Sampler2dRect,
        gl::SAMPLER_2D_RECT_SHADOW => UniformType::Sampler2dRectShadow,
        gl::INT_SAMPLER_1D => UniformType::ISampler1d,
        gl::INT_SAMPLER_2D => UniformType::ISampler2d,
        gl::INT_SAMPLER_3D => UniformType::ISampler3d,
        gl::INT_SAMPLER_CUBE => UniformType::ISamplerCube,
        gl::INT_SAMPLER_1D_ARRAY => UniformType::ISampler1dArray,
        gl::INT_SAMPLER_2D_ARRAY => UniformType::ISampler2dArray,
        gl::INT_SAMPLER_2D_MULTISAMPLE => UniformType::ISampler2dMultisample,
        gl::INT_SAMPLER_2D_MULTISAMPLE_ARRAY => UniformType::ISampler2dMultisampleArray,
        gl::INT_SAMPLER_BUFFER => UniformType::ISamplerBuffer,
        gl::INT_SAMPLER_2D_RECT => UniformType::ISampler2dRect,
        gl::UNSIGNED_INT_SAMPLER_1D => UniformType::USampler1d,
        gl::UNSIGNED_INT_SAMPLER_2D => UniformType::USampler2d,
        gl::UNSIGNED_INT_SAMPLER_3D => UniformType::USampler3d,
        gl::UNSIGNED_INT_SAMPLER_CUBE => UniformType::USamplerCube,
        gl::UNSIGNED_INT_SAMPLER_1D_ARRAY => UniformType::USampler2dArray,
        gl::UNSIGNED_INT_SAMPLER_2D_ARRAY => UniformType::USampler2dArray,
        gl::UNSIGNED_INT_SAMPLER_2D_MULTISAMPLE => UniformType::USampler2dMultisample,
        gl::UNSIGNED_INT_SAMPLER_2D_MULTISAMPLE_ARRAY => UniformType::USampler2dMultisampleArray,
        gl::UNSIGNED_INT_SAMPLER_BUFFER => UniformType::USamplerBuffer,
        gl::UNSIGNED_INT_SAMPLER_2D_RECT => UniformType::USampler2dRect,
        gl::IMAGE_1D => UniformType::Image1d,
        gl::IMAGE_2D => UniformType::Image2d,
        gl::IMAGE_3D => UniformType::Image3d,
        gl::IMAGE_2D_RECT => UniformType::Image2dRect,
        gl::IMAGE_CUBE => UniformType::ImageCube,
        gl::IMAGE_BUFFER => UniformType::ImageBuffer,
        gl::IMAGE_1D_ARRAY => UniformType::Image1dArray,
        gl::IMAGE_2D_ARRAY => UniformType::Image2dArray,
        gl::IMAGE_2D_MULTISAMPLE => UniformType::Image2dMultisample,
        gl::IMAGE_2D_MULTISAMPLE_ARRAY => UniformType::Image2dMultisampleArray,
        gl::INT_IMAGE_1D => UniformType::IImage1d,
        gl::INT_IMAGE_2D => UniformType::IImage2d,
        gl::INT_IMAGE_3D => UniformType::IImage3d,
        gl::INT_IMAGE_2D_RECT => UniformType::IImage2dRect,
        gl::INT_IMAGE_CUBE => UniformType::IImageCube,
        gl::INT_IMAGE_BUFFER => UniformType::IImageBuffer,
        gl::INT_IMAGE_1D_ARRAY => UniformType::IImage1dArray,
        gl::INT_IMAGE_2D_ARRAY => UniformType::IImage2dArray,
        gl::INT_IMAGE_2D_MULTISAMPLE => UniformType::IImage2dMultisample,
        gl::INT_IMAGE_2D_MULTISAMPLE_ARRAY => UniformType::IImage2dMultisampleArray,
        gl::UNSIGNED_INT_IMAGE_1D => UniformType::UImage1d,
        gl::UNSIGNED_INT_IMAGE_2D => UniformType::UImage2d,
        gl::UNSIGNED_INT_IMAGE_3D => UniformType::UImage3d,
        gl::UNSIGNED_INT_IMAGE_2D_RECT => UniformType::UImage2dRect,
        gl::UNSIGNED_INT_IMAGE_CUBE => UniformType::UImageCube,
        gl::UNSIGNED_INT_IMAGE_BUFFER => UniformType::UImageBuffer,
        gl::UNSIGNED_INT_IMAGE_1D_ARRAY => UniformType::UImage1dArray,
        gl::UNSIGNED_INT_IMAGE_2D_ARRAY => UniformType::UImage2dArray,
        gl::UNSIGNED_INT_IMAGE_2D_MULTISAMPLE => UniformType::UImage2dMultisample,
        gl::UNSIGNED_INT_IMAGE_2D_MULTISAMPLE_ARRAY => UniformType::UImage2dMultisampleArray,
        gl::UNSIGNED_INT_ATOMIC_COUNTER => UniformType::AtomicCounterUint,
        v => panic!("Unknown value returned by OpenGL uniform type: {}", v)
    }
}

fn glenum_to_attribute_type(value: gl::types::GLenum) -> AttributeType {
    match value {
        gl::FLOAT => AttributeType::F32,
        gl::FLOAT_VEC2 => AttributeType::F32F32,
        gl::FLOAT_VEC3 => AttributeType::F32F32F32,
        gl::FLOAT_VEC4 => AttributeType::F32F32F32F32,
        gl::INT => AttributeType::I32,
        gl::INT_VEC2 => AttributeType::I32I32,
        gl::INT_VEC3 => AttributeType::I32I32I32,
        gl::INT_VEC4 => AttributeType::I32I32I32I32,
        gl::UNSIGNED_INT => AttributeType::U32,
        gl::UNSIGNED_INT_VEC2 => AttributeType::U32U32,
        //gl::UNSIGNED_INT_VEC2_EXT => AttributeType::U32U32,
        gl::UNSIGNED_INT_VEC3 => AttributeType::U32U32U32,
        //gl::UNSIGNED_INT_VEC3_EXT => AttributeType::U32U32U32,
        gl::UNSIGNED_INT_VEC4 => AttributeType::U32U32U32U32,
        //gl::UNSIGNED_INT_VEC4_EXT => AttributeType::U32U32U32U32,
        gl::FLOAT_MAT2 => AttributeType::F32x2x2,
        gl::FLOAT_MAT3 => AttributeType::F32x3x3,
        gl::FLOAT_MAT4 => AttributeType::F32x4x4,
        v => panic!("Unknown value returned by OpenGL attribute type: {}", v)
    }
}

fn glenum_to_transform_feedback_mode(value: gl::types::GLenum) -> TransformFeedbackMode {
    match value {
        gl::INTERLEAVED_ATTRIBS/* | gl::INTERLEAVED_ATTRIBS_EXT*/ => {
            TransformFeedbackMode::Interleaved
        },
        gl::SEPARATE_ATTRIBS/* | gl::SEPARATE_ATTRIBS_EXT*/ => {
            TransformFeedbackMode::Separate
        },
        v => panic!("Unknown value returned by OpenGL varying mode: {}", v)
    }
}
