use gl;
use libc;

use std::ffi;
use std::mem;
use std::ptr;
use std::collections::HashMap;

use context::CommandContext;
use version::Version;
use version::Api;

use uniforms::UniformType;
use vertex::AttributeType;

use Handle;

/// Information about a uniform (except its name).
#[derive(Debug, Copy, Clone)]
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
    /// The binding point of the block.
    ///
    /// This is internal information, you probably don't need to use it.
    pub binding: i32,

    /// Size in bytes of the data in the block.
    pub size: usize,

    /// Layout of the block.
    pub layout: BlockLayout,
}

/// Layout of a shader storage buffer or a uniform buffer.
#[derive(Debug, Clone)]
pub enum BlockLayout {
    /// Multiple elements, each having a name.
    Struct {
        /// The list of elements, with `name`/`layout` pairs.
        members: Vec<(String, BlockLayout)>,
    },

    /// A basic element.
    BasicType {
        /// Type of data.
        ty: UniformType,

        /// Offset of this element in bytes from the start of the buffer.
        offset_in_buffer: usize,
    },

    /// A fixed-size array.
    ///
    /// For example:
    ///
    /// ```notrust
    /// uint data[12];
    /// ```
    Array {
        /// Type of data of each element.
        content: Box<BlockLayout>,

        /// Number of elements in the array.
        length: usize,
    },

    /// An array whose size isn't known at compile-time. Can only be used as the last element of
    /// a buffer.
    ///
    /// Its actual size depends on the size of the buffer.
    ///
    /// For example:
    ///
    /// ```notrust
    /// buffer MyBuffer {
    ///     uint data[];
    /// }
    /// ```
    DynamicSizedArray {
        /// Type of data of each element.
        content: Box<BlockLayout>,
    },
}

/// Information about an attribute of a program (except its name).
///
/// Internal struct. Not public.
#[derive(Debug, Copy, Clone)]
pub struct Attribute {
    /// The index of the uniform.
    ///
    /// This is internal information, you probably don't need to use it.
    pub location: i32,

    /// Type of the attribute.
    pub ty: AttributeType,

    /// Number of elements of the attribute.
    pub size: usize,
}

/// Describes the layout of a buffer that can receive transform feedback output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformFeedbackBuffer {
    /// Slot of this buffer.
    ///
    /// This is internal information, you probably don't need to use it.
    pub id: i32,

    /// List of elements inside the buffer.
    pub elements: Vec<TransformFeedbackVarying>,

    /// Size in bytes between two consecutive elements.
    pub stride: usize,
}

/// Describes a varying that is being output with transform feedback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformFeedbackVarying {
    /// Name of the variable.
    pub name: String,

    /// Number of bytes between the start of the first element and the start of this one.
    pub offset: usize,

    /// Size in bytes of this value.
    pub size: usize,

    /// Type of the value.
    pub ty: AttributeType,
}

/// Type of transform feedback. Only used with the legacy interface.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TransformFeedbackMode {
    /// Each value is interleaved in the same buffer.
    Interleaved,

    /// Each value will go in a separate buffer.
    Separate,
}

/// Type of primitives that is being output by transform feedback.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OutputPrimitives {
    /// Points.
    Points,
    /// Lines.
    Lines,
    /// Triangles.
    Triangles,
    /// Quads.
    Quads,
}

pub unsafe fn reflect_uniforms(ctxt: &mut CommandContext, program: Handle)
                               -> HashMap<String, Uniform>
{
    // number of active uniforms
    let active_uniforms = {
        let mut active_uniforms: gl::types::GLint = mem::uninitialized();
        match program {
            Handle::Id(program) => {
                assert!(ctxt.version >= &Version(Api::Gl, 2, 0) ||
                        ctxt.version >= &Version(Api::GlEs, 2, 0));
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

    // the result of this function
    let mut uniforms = HashMap::with_capacity(active_uniforms as usize);

    for uniform_id in (0 .. active_uniforms) {
        let mut uniform_name_tmp: Vec<u8> = Vec::with_capacity(64);
        let mut uniform_name_tmp_len = 63;

        let mut data_type: gl::types::GLenum = mem::uninitialized();
        let mut data_size: gl::types::GLint = mem::uninitialized();

        match program {
            Handle::Id(program) => {
                assert!(ctxt.version >= &Version(Api::Gl, 2, 0) ||
                        ctxt.version >= &Version(Api::GlEs, 2, 0));
                ctxt.gl.GetActiveUniform(program, uniform_id as gl::types::GLuint,
                                         uniform_name_tmp_len, &mut uniform_name_tmp_len,
                                         &mut data_size, &mut data_type,
                                         uniform_name_tmp.as_mut_ptr() as *mut gl::types::GLchar);
            },
            Handle::Handle(program) => {
                assert!(ctxt.extensions.gl_arb_shader_objects);
                ctxt.gl.GetActiveUniformARB(program, uniform_id as gl::types::GLuint,
                                            uniform_name_tmp_len, &mut uniform_name_tmp_len,
                                            &mut data_size, &mut data_type,
                                            uniform_name_tmp.as_mut_ptr()
                                              as *mut gl::types::GLchar);
            }
        };

        uniform_name_tmp.set_len(uniform_name_tmp_len as usize);

        let uniform_name = String::from_utf8(uniform_name_tmp).unwrap();
        let location = match program {
            Handle::Id(program) => {
                assert!(ctxt.version >= &Version(Api::Gl, 2, 0) ||
                        ctxt.version >= &Version(Api::GlEs, 2, 0));
                ctxt.gl.GetUniformLocation(program,
                                           ffi::CString::new(uniform_name.as_bytes()).unwrap()
                                             .as_bytes_with_nul().as_ptr() as *const libc::c_char)
            },
            Handle::Handle(program) => {
                assert!(ctxt.extensions.gl_arb_shader_objects);
                ctxt.gl.GetUniformLocationARB(program,
                                              ffi::CString::new(uniform_name.as_bytes()).unwrap()
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
                                 -> HashMap<String, Attribute>
{
    // number of active attributes
    let active_attributes = {
        let mut active_attributes: gl::types::GLint = mem::uninitialized();
        match program {
            Handle::Id(program) => {
                assert!(ctxt.version >= &Version(Api::Gl, 2, 0) ||
                        ctxt.version >= &Version(Api::GlEs, 2, 0));
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

    // the result of this function
    let mut attributes = HashMap::with_capacity(active_attributes as usize);

    for attribute_id in (0 .. active_attributes) {
        let mut attr_name_tmp: Vec<u8> = Vec::with_capacity(64);
        let mut attr_name_tmp_len = 63;

        let mut data_type: gl::types::GLenum = mem::uninitialized();
        let mut data_size: gl::types::GLint = mem::uninitialized();

        match program {
            Handle::Id(program) => {
                assert!(ctxt.version >= &Version(Api::Gl, 2, 0) ||
                        ctxt.version >= &Version(Api::GlEs, 2, 0));
                ctxt.gl.GetActiveAttrib(program, attribute_id as gl::types::GLuint,
                                        attr_name_tmp_len, &mut attr_name_tmp_len, &mut data_size,
                                        &mut data_type, attr_name_tmp.as_mut_ptr()
                                          as *mut gl::types::GLchar);
            },
            Handle::Handle(program) => {
                assert!(ctxt.extensions.gl_arb_vertex_shader);
                ctxt.gl.GetActiveAttribARB(program, attribute_id as gl::types::GLuint,
                                           attr_name_tmp_len, &mut attr_name_tmp_len, &mut data_size,
                                           &mut data_type, attr_name_tmp.as_mut_ptr()
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
                assert!(ctxt.version >= &Version(Api::Gl, 2, 0) ||
                        ctxt.version >= &Version(Api::GlEs, 2, 0));
                ctxt.gl.GetAttribLocation(program,
                                          ffi::CString::new(attr_name.as_bytes()).unwrap()
                                            .as_bytes_with_nul().as_ptr() as *const libc::c_char)
            },
            Handle::Handle(program) => {
                assert!(ctxt.extensions.gl_arb_vertex_shader);
                ctxt.gl.GetAttribLocationARB(program,
                                             ffi::CString::new(attr_name.as_bytes()).unwrap()
                                               .as_bytes_with_nul().as_ptr() as *const libc::c_char)
            }
        };

        attributes.insert(attr_name, Attribute {
            location: location,
            ty: glenum_to_attribute_type(data_type),
            size: data_size as usize,
        });
    }

    attributes
}

pub unsafe fn reflect_uniform_blocks(ctxt: &mut CommandContext, program: Handle)
                                     -> HashMap<String, UniformBlock>
{
    // uniform blocks are not supported, so there's none
    if !(ctxt.version >= &Version(Api::Gl, 3, 1) || ctxt.version >= &Version(Api::GlEs, 3, 0)) {
        return HashMap::new();
    }

    let program = match program {
        Handle::Id(id) => id,
        _ => unreachable!()
    };

    let mut active_blocks: gl::types::GLint = mem::uninitialized();
    ctxt.gl.GetProgramiv(program, gl::ACTIVE_UNIFORM_BLOCKS, &mut active_blocks);

    let mut active_blocks_max_name_len: gl::types::GLint = mem::uninitialized();
    ctxt.gl.GetProgramiv(program, gl::ACTIVE_UNIFORM_BLOCK_MAX_NAME_LENGTH,
                         &mut active_blocks_max_name_len);

    let mut blocks = HashMap::with_capacity(active_blocks as usize);

    for block_id in (0 .. active_blocks) {
        // getting the name of the block
        let name = {
            let mut name_tmp: Vec<u8> = Vec::with_capacity(1 + active_blocks_max_name_len
                                                           as usize);
            let mut name_tmp_len = active_blocks_max_name_len;

            ctxt.gl.GetActiveUniformBlockName(program, block_id as gl::types::GLuint,
                                              name_tmp_len, &mut name_tmp_len,
                                              name_tmp.as_mut_ptr() as *mut gl::types::GLchar);
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
            (name, member_offsets[index] as usize,
             glenum_to_uniform_type(member_types[index] as gl::types::GLenum),
             match member_size[index] {
                 1 => None,
                 a => Some(a as usize),
             }
            )
        });

        // finally inserting into the blocks list
        blocks.insert(name, UniformBlock {
            binding: binding as i32,
            size: block_size as usize,
            layout: introspection_output_to_layout(members),
        });
    }

    blocks
}

pub unsafe fn reflect_transform_feedback(ctxt: &mut CommandContext, program: Handle)
                                         -> Vec<TransformFeedbackBuffer>
{
    let program = match program {
        // transform feedback not supported
        Handle::Handle(_) => return Vec::with_capacity(0),
        Handle::Id(id) => id
    };

    // transform feedback not supported
    if !(ctxt.version >= &Version(Api::Gl, 3, 0)) && !ctxt.extensions.gl_ext_transform_feedback {
        return Vec::with_capacity(0);
    }

    // querying the number of varying
    let num_varyings = {
        let mut num_varyings: gl::types::GLint = mem::uninitialized();

        if ctxt.version >= &Version(Api::Gl, 3, 0) {
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
        return Vec::with_capacity(0);
    }

    // querying "interleaved" or "separate"
    let buffer_mode = {
        let mut buffer_mode: gl::types::GLint = mem::uninitialized();

        if ctxt.version >= &Version(Api::Gl, 3, 0) {
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
    if ctxt.version >= &Version(Api::Gl, 3, 0) {
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

        if ctxt.version >= &Version(Api::Gl, 3, 0) {
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

        if buffer_mode == TransformFeedbackMode::Interleaved {
            if result.len() == 0 {
                result.push(TransformFeedbackBuffer {
                    id: 0,
                    elements: vec![],
                    stride: 0,
                });
            }

            let ty = glenum_to_attribute_type(ty as gl::types::GLenum);

            let prev_size = result[0].stride;
            result[0].stride += size as usize * ty.get_size_bytes();
            result[0].elements.push(TransformFeedbackVarying {        // TODO: handle arrays
                name: name,
                size: size as usize * ty.get_size_bytes(),
                offset: prev_size,
                ty: ty,
            });

        } else if buffer_mode == TransformFeedbackMode::Separate {
            let id = result.len();
            let ty = glenum_to_attribute_type(ty as gl::types::GLenum);
            result.push(TransformFeedbackBuffer {
                id: id as i32,
                elements: vec![
                    TransformFeedbackVarying {
                        name: name,
                        size: size as usize * ty.get_size_bytes(),
                        offset: 0,
                        ty: ty,
                    }
                ],
                stride: size as usize * ty.get_size_bytes(),
            });

        } else {
            unreachable!();
        }
    }

    result
}

/// Obtains the type of data that the geometry shader stage outputs.
///
/// # Unsafety
///
/// - `program` must be a valid handle to a program.
/// - The program **must** contain a geometry shader.
pub unsafe fn reflect_geometry_output_type(ctxt: &mut CommandContext, program: Handle)
                                           -> OutputPrimitives
{
    let mut value = mem::uninitialized();

    match program {
        Handle::Id(program) => {
            assert!(ctxt.version >= &Version(Api::Gl, 2, 0) ||
                    ctxt.version >= &Version(Api::GlEs, 2, 0));
            ctxt.gl.GetProgramiv(program, gl::GEOMETRY_OUTPUT_TYPE, &mut value);
        },
        Handle::Handle(program) => {
            assert!(ctxt.extensions.gl_arb_vertex_shader);
            ctxt.gl.GetObjectParameterivARB(program, gl::GEOMETRY_OUTPUT_TYPE, &mut value);
        }
    };

    match value as gl::types::GLenum {
        gl::POINTS => OutputPrimitives::Points,
        gl::LINE_STRIP => OutputPrimitives::Lines,
        gl::TRIANGLE_STRIP => OutputPrimitives::Triangles,
        _ => unreachable!()
    }
}

/// Obtains the type of data that the tessellation evaluation shader stage outputs.
///
/// # Unsafety
///
/// - `program` must be a valid handle to a program.
/// - The program **must** contain a tessellation evaluation shader.
pub unsafe fn reflect_tess_eval_output_type(ctxt: &mut CommandContext, program: Handle)
                                            -> OutputPrimitives
{
    let mut value = mem::uninitialized();

    match program {
        Handle::Id(program) => {
            assert!(ctxt.version >= &Version(Api::Gl, 2, 0) ||
                    ctxt.version >= &Version(Api::GlEs, 2, 0));
            ctxt.gl.GetProgramiv(program, gl::TESS_GEN_MODE, &mut value);
        },
        Handle::Handle(program) => {
            assert!(ctxt.extensions.gl_arb_vertex_shader);
            ctxt.gl.GetObjectParameterivARB(program, gl::TESS_GEN_MODE, &mut value);
        }
    };

    match value as gl::types::GLenum {
        gl::TRIANGLES => OutputPrimitives::Triangles,
        gl::ISOLINES => OutputPrimitives::Lines,
        gl::QUADS => OutputPrimitives::Quads,
        _ => unreachable!()
    }
}

/// Returns the list of shader storage blocks of a program.
pub unsafe fn reflect_shader_storage_blocks(ctxt: &mut CommandContext, program: Handle)
                                            -> HashMap<String, UniformBlock>
{
    if !(ctxt.version >= &Version(Api::Gl, 4, 3) || ctxt.version >= &Version(Api::GlEs, 3, 1) ||
         (ctxt.extensions.gl_arb_program_interface_query && ctxt.extensions.gl_arb_shader_storage_buffer_object))
    {
        // not supported
        return HashMap::with_capacity(0);
    }
    
    let program = match program {
        Handle::Id(program) => program,
        Handle::Handle(program) => return HashMap::with_capacity(0)
    };

    // number of active SSBOs
    let active_blocks = {
        let mut active_blocks: gl::types::GLint = mem::uninitialized();
        ctxt.gl.GetProgramInterfaceiv(program, gl::SHADER_STORAGE_BLOCK,
                                      gl::ACTIVE_RESOURCES, &mut active_blocks);
        active_blocks as gl::types::GLuint
    };

    // the result of this function
    let mut blocks = HashMap::with_capacity(active_blocks as usize);

    for block_id in (0 .. active_blocks) {
        // getting basic infos
        let (name_len, num_variables, binding, total_size) = {
            let mut output: [gl::types::GLint; 4] = mem::uninitialized();
            ctxt.gl.GetProgramResourceiv(program, gl::SHADER_STORAGE_BLOCK, block_id, 4,
                                         [gl::NAME_LENGTH, gl::NUM_ACTIVE_VARIABLES,
                                          gl::BUFFER_BINDING, gl::BUFFER_DATA_SIZE].as_ptr(), 4,
                                         ptr::null_mut(), output.as_mut_ptr() as *mut _);
            (output[0] as usize, output[1] as usize, output[2], output[3] as usize)
        };

        // getting the name of the block
        let name = {
            let mut name_tmp: Vec<u8> = Vec::with_capacity(1 + name_len);
            let mut name_tmp_len = name_len as gl::types::GLsizei;

            ctxt.gl.GetProgramResourceName(program, gl::SHADER_STORAGE_BLOCK, block_id,
                                           name_tmp_len, &mut name_tmp_len,
                                           name_tmp.as_mut_ptr() as *mut _);
            name_tmp.set_len(name_tmp_len as usize);
            String::from_utf8(name_tmp).unwrap()
        };

        // indices of the active variables
        let active_variables: Vec<gl::types::GLint> = {
            let mut variables = Vec::with_capacity(num_variables);
            ctxt.gl.GetProgramResourceiv(program, gl::SHADER_STORAGE_BLOCK, block_id, 1,
                                         [gl::ACTIVE_VARIABLES].as_ptr(),
                                         num_variables as gl::types::GLsizei,
                                         ptr::null_mut(), variables.as_mut_ptr() as *mut _);
            variables.set_len(num_variables);
            variables
        };

        // iterator over variables
        let members = active_variables.into_iter().map(|variable| {
            let (ty, array_size, offset, _array_stride, name_len) = {
                let mut output: [gl::types::GLint; 5] = mem::uninitialized();
                ctxt.gl.GetProgramResourceiv(program, gl::BUFFER_VARIABLE,
                                             variable as gl::types::GLuint, 5,
                                             [gl::TYPE, gl::ARRAY_SIZE, gl::OFFSET,
                                              gl::ARRAY_STRIDE, gl::NAME_LENGTH].as_ptr(), 5,
                                             ptr::null_mut(), output.as_mut_ptr() as *mut _);
                (glenum_to_uniform_type(output[0] as gl::types::GLenum), output[1] as usize,
                 output[2] as usize, output[3] as usize, output[4] as usize)
            };

            let name = {
                let mut name_tmp: Vec<u8> = Vec::with_capacity(1 + name_len);
                let mut name_tmp_len = name_len as gl::types::GLsizei;

                ctxt.gl.GetProgramResourceName(program, gl::BUFFER_VARIABLE,
                                               variable as gl::types::GLuint,
                                               name_tmp_len, &mut name_tmp_len,
                                               name_tmp.as_mut_ptr() as *mut _);
                name_tmp.set_len(name_tmp_len as usize);
                String::from_utf8(name_tmp).unwrap()
            };

            (
                name, offset, ty,
                match array_size {
                    1 => None,
                    a => Some(a as usize),
                },
            )
        });

        // finally inserting into the blocks list
        blocks.insert(name, UniformBlock {
            binding: binding as i32,
            size: total_size,
            layout: introspection_output_to_layout(members),
        });
    }

    blocks
}

/// Takes a list of elements produced by OpenGL's introspection API and turns them into
/// a `BlockLayout` object.
///
/// The iterator must produce a list of `(name, offset, ty, array_size)`.
///
/// # Panic
///
/// Panic if the input doesn't conform to the OpenGL specs.
///
fn introspection_output_to_layout<I>(elements: I) -> BlockLayout
                                     where I: Iterator<Item = (String, usize, UniformType,
                                                               Option<usize>)>
{
    // `output` must be a BlockLayout::Struct, otherwise this function will panic
    fn process(output: &mut BlockLayout, name: &str, offset: usize, ty: UniformType,
               array_size: Option<usize>)
    {
        let mut components = name.splitn(2, '.');
        let current_component = components.next().unwrap();
        let name_rest = components.next();

        // finding the appropriate place in `output` to write the element
        let member = if let &mut BlockLayout::Struct { ref mut members } = output {
            // splitting the name and array size
            let (current_component, array) = if current_component.ends_with(']') {
                let open_bracket_pos = current_component.rfind('[').unwrap();
                let array = current_component[open_bracket_pos + 1 .. current_component.len() - 1]
                                        .parse().unwrap();
                (&current_component[.. open_bracket_pos], Some(array))
            } else {
                (&current_component[..], None)
            };

            // because of a bug in Rust's borrow checker, we have to loop twice instead of just
            // call `if let Some() { } else { }`
            let existing = members.iter_mut().find(|m| m.0 == current_component).is_some();
            if existing {
                let member = &mut members.iter_mut().find(|m| m.0 == current_component)
                                         .unwrap().1;

                if let Some(array) = array {
                    match member {
                        &mut BlockLayout::Array { ref mut content, ref mut length } => {
                            if *length <= array { *length = array + 1; }
                            &mut **content
                        },
                        _ => unreachable!()
                    }
                } else {
                    member
                }

            } else {
                // member doesn't exist yet in the output, adding it
                if let Some(array) = array {
                    members.push((current_component.to_string(), BlockLayout::Array {
                        content: Box::new(BlockLayout::Struct { members: Vec::new() }),
                        length: if name_rest.is_some() { array } else { array_size.unwrap() },
                    }));

                    match &mut members.last_mut().unwrap().1 {
                        &mut BlockLayout::Array { ref mut content, .. } => &mut **content,
                        _ => unreachable!()
                    }

                } else {
                    members.push((current_component.to_string(), BlockLayout::Struct {
                        members: Vec::new()
                    }));
                    &mut members.last_mut().unwrap().1
                }
            }

        } else {
            unreachable!();
        };

        // now adding either the other elements or the final element itself
        if let Some(name_rest) = name_rest {
            process(member, name_rest, offset, ty, array_size);

        } else {
            *member = BlockLayout::BasicType {
                offset_in_buffer: offset,
                ty: ty,
            };
        }
    }

    // ↓ actual body of `introspection_output_to_layout` starts here ↓
    let mut layout = BlockLayout::Struct { members: Vec::new() };
    for (name, offset, ty, array_size) in elements {
        process(&mut layout, &name, offset, ty, array_size);
    }
    layout
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
        gl::FLOAT_MAT2x3 => AttributeType::F32x2x3,
        gl::FLOAT_MAT2x4 => AttributeType::F32x2x4,
        gl::FLOAT_MAT3x2 => AttributeType::F32x3x2,
        gl::FLOAT_MAT3x4 => AttributeType::F32x3x4,
        gl::FLOAT_MAT4x2 => AttributeType::F32x4x2,
        gl::FLOAT_MAT4x3 => AttributeType::F32x4x3,
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
