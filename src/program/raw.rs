use gl;

use context::CommandContext;
use version::Version;
use version::Api;

use backend::Facade;
use context::Context;
use ContextExt;
use UniformsExt;

use std::{ffi, fmt};
use std::collections::hash_map::{self, HashMap};
use std::rc::Rc;
use std::cell::RefCell;
use std::os::raw;
use std::hash::BuildHasherDefault;

use fnv::FnvHasher;

use DrawError;
use GlObject;
use ProgramExt;
use Handle;
use RawUniformValue;

use QueryExt;
use draw_parameters::TimeElapsedQuery;

use buffer::BufferSlice;
use BufferExt;
use BufferSliceExt;

use program::{ProgramCreationError, Binary, GetBinaryError};
use program::uniforms_storage::UniformsStorage;

use program::compute::ComputeCommand;
use program::reflection::{Uniform, UniformBlock, OutputPrimitives};
use program::reflection::{Attribute, TransformFeedbackMode, TransformFeedbackBuffer};
use program::reflection::{SubroutineData, ShaderStage};
use program::reflection::{reflect_uniforms, reflect_attributes, reflect_uniform_blocks};
use program::reflection::{reflect_transform_feedback, reflect_geometry_output_type};
use program::reflection::{reflect_tess_eval_output_type, reflect_shader_storage_blocks};
use program::reflection::{reflect_subroutine_data};
use program::shader::Shader;
use program::binary_header::{attach_glium_header, process_glium_header};

use uniforms::Uniforms;

use vertex::VertexFormat;
use vertex_array_object::VertexAttributesSystem;

/// A combination of shaders linked together.
pub struct RawProgram {
    context: Rc<Context>,
    id: Handle,
    uniform_values: UniformsStorage,
    uniforms: HashMap<String, Uniform, BuildHasherDefault<FnvHasher>>,
    uniform_blocks: HashMap<String, UniformBlock, BuildHasherDefault<FnvHasher>>,
    subroutine_data: SubroutineData,
    attributes: HashMap<String, Attribute, BuildHasherDefault<FnvHasher>>,
    frag_data_locations: RefCell<HashMap<String, Option<u32>, BuildHasherDefault<FnvHasher>>>,
    tf_buffers: Vec<TransformFeedbackBuffer>,
    ssbos: HashMap<String, UniformBlock, BuildHasherDefault<FnvHasher>>,
    output_primitives: Option<OutputPrimitives>,
    has_geometry_shader: bool,
    has_tessellation_control_shader: bool,
    has_tessellation_evaluation_shader: bool,
}

impl RawProgram {
    /// Builds a new program from a list of shaders.
    // TODO: the "has_*" parameters are bad
    pub fn from_shaders<'a, F: ?Sized, I>(facade: &'a F, shaders: I, has_geometry_shader: bool,
                                  has_tessellation_control_shader: bool,
                                  has_tessellation_evaluation_shader: bool,
                                  transform_feedback: Option<(Vec<String>, TransformFeedbackMode)>)
                                  -> Result<RawProgram, ProgramCreationError>
                                  where F: Facade, I: IntoIterator<Item = &'a Shader>
    {
        let mut ctxt = facade.get_context().make_current();

        let shaders_ids = shaders.into_iter().map(|s| s.get_id()).collect::<Vec<_>>();

        let id = unsafe {
            let id = create_program(&mut ctxt);

            // attaching shaders
            for sh in shaders_ids.iter() {
                match (id, sh) {
                    (Handle::Id(id), &Handle::Id(sh)) => {
                        assert!(ctxt.version >= &Version(Api::Gl, 2, 0) ||
                                ctxt.version >= &Version(Api::GlEs, 2, 0));
                        ctxt.gl.AttachShader(id, sh);
                    },
                    (Handle::Handle(id), &Handle::Handle(sh)) => {
                        assert!(ctxt.extensions.gl_arb_shader_objects);
                        ctxt.gl.AttachObjectARB(id, sh);
                    },
                    _ => unreachable!()
                }
            }

            // transform feedback varyings
            if let Some((names, mode)) = transform_feedback {
                let id = match id {
                    Handle::Id(id) => id,
                    Handle::Handle(id) => unreachable!()    // transf. feedback shouldn't be
                                                            // available with handles
                };

                let names = names.into_iter().map(|name| {
                    ffi::CString::new(name.into_bytes()).unwrap()
                }).collect::<Vec<_>>();
                let names_ptr = names.iter().map(|n| n.as_ptr()).collect::<Vec<_>>();

                if ctxt.version >= &Version(Api::Gl, 3, 0) {
                    let mode = match mode {
                        TransformFeedbackMode::Interleaved => gl::INTERLEAVED_ATTRIBS,
                        TransformFeedbackMode::Separate => gl::SEPARATE_ATTRIBS,
                    };

                    ctxt.gl.TransformFeedbackVaryings(id, names_ptr.len() as gl::types::GLsizei,
                                                      names_ptr.as_ptr(), mode);

                } else if ctxt.extensions.gl_ext_transform_feedback {
                    let mode = match mode {
                        TransformFeedbackMode::Interleaved => gl::INTERLEAVED_ATTRIBS_EXT,
                        TransformFeedbackMode::Separate => gl::SEPARATE_ATTRIBS_EXT,
                    };

                    ctxt.gl.TransformFeedbackVaryingsEXT(id, names_ptr.len()
                                                         as gl::types::GLsizei,
                                                         names_ptr.as_ptr(), mode);

                } else {
                    unreachable!();     // has been checked in the frontend
                }
            }

            // linking
            {
                ctxt.report_debug_output_errors.set(false);

                match id {
                    Handle::Id(id) => {
                        assert!(ctxt.version >= &Version(Api::Gl, 2, 0) ||
                                ctxt.version >= &Version(Api::GlEs, 2, 0));
                        ctxt.gl.LinkProgram(id);
                    },
                    Handle::Handle(id) => {
                        assert!(ctxt.extensions.gl_arb_shader_objects);
                        ctxt.gl.LinkProgramARB(id);
                    }
                }

                ctxt.report_debug_output_errors.set(true);
            }

            // checking for errors
            check_program_link_errors(&mut ctxt, id)?;

            id
        };

        let uniforms = unsafe { reflect_uniforms(&mut ctxt, id) };
        let attributes = unsafe { reflect_attributes(&mut ctxt, id) };
        let blocks = unsafe { reflect_uniform_blocks(&mut ctxt, id) };
        let tf_buffers = unsafe { reflect_transform_feedback(&mut ctxt, id) };
        let ssbos = unsafe { reflect_shader_storage_blocks(&mut ctxt, id) };
        let subroutine_data = unsafe {
            reflect_subroutine_data(&mut ctxt, id, has_geometry_shader,
                                    has_tessellation_control_shader,
                                    has_tessellation_evaluation_shader)
            };

        let output_primitives = if has_geometry_shader {
            Some(unsafe { reflect_geometry_output_type(&mut ctxt, id) })
        } else if has_tessellation_evaluation_shader {
            Some(unsafe { reflect_tess_eval_output_type(&mut ctxt, id) })
        } else {
            None
        };

        Ok(RawProgram {
            context: facade.get_context().clone(),
            id: id,
            uniforms: uniforms,
            uniform_values: UniformsStorage::new(),
            uniform_blocks: blocks,
            subroutine_data: subroutine_data,
            attributes: attributes,
            frag_data_locations: RefCell::new(HashMap::with_hasher(Default::default())),
            tf_buffers: tf_buffers,
            ssbos: ssbos,
            output_primitives: output_primitives,
            has_geometry_shader: has_geometry_shader,
            has_tessellation_control_shader: has_tessellation_control_shader,
            has_tessellation_evaluation_shader: has_tessellation_evaluation_shader,
        })
    }

    /// Creates a program from binary.
    pub fn from_binary<F: ?Sized>(facade: &F, binary: Binary)
                          -> Result<RawProgram, ProgramCreationError> where F: Facade
    {
        let (has_geometry_shader, has_tessellation_control_shader, has_tessellation_evaluation_shader) = {
            match process_glium_header(&binary.content) {
                Some(flags) => flags,
                None => return Err(ProgramCreationError::BinaryHeaderError)
            }
        };

        let mut ctxt = facade.get_context().make_current();

        let id = unsafe {
            let id = create_program(&mut ctxt);

            match id {
                Handle::Id(id) => {
                    assert!(ctxt.version >= &Version(Api::Gl, 2, 0));
                    ctxt.gl.ProgramBinary(id, binary.format,
                                          binary.content[1..].as_ptr() as *const _,
                                          (binary.content.len() - 1) as gl::types::GLsizei);
                },
                Handle::Handle(id) => unreachable!()
            };

            // checking for errors
            check_program_link_errors(&mut ctxt, id)?;

            id
        };

        let (uniforms, attributes, blocks, tf_buffers, ssbos, subroutine_data) = unsafe {
            (
                reflect_uniforms(&mut ctxt, id),
                reflect_attributes(&mut ctxt, id),
                reflect_uniform_blocks(&mut ctxt, id),
                reflect_transform_feedback(&mut ctxt, id),
                reflect_shader_storage_blocks(&mut ctxt, id),
                reflect_subroutine_data(&mut ctxt, id, has_geometry_shader,
                                        has_tessellation_control_shader,
                                        has_tessellation_evaluation_shader),
            )
        };

        let output_primitives = if has_geometry_shader {
            Some(unsafe { reflect_geometry_output_type(&mut ctxt, id) })
        } else if has_tessellation_evaluation_shader {
            Some(unsafe { reflect_tess_eval_output_type(&mut ctxt, id) })
        } else {
            None
        };

        Ok(RawProgram {
            context: facade.get_context().clone(),
            id: id,
            uniforms: uniforms,
            uniform_values: UniformsStorage::new(),
            uniform_blocks: blocks,
            subroutine_data: subroutine_data,
            attributes: attributes,
            frag_data_locations: RefCell::new(HashMap::with_hasher(Default::default())),
            tf_buffers: tf_buffers,
            ssbos: ssbos,
            output_primitives: output_primitives,
            has_geometry_shader: has_geometry_shader,
            has_tessellation_control_shader: has_tessellation_control_shader,
            has_tessellation_evaluation_shader: has_tessellation_evaluation_shader,
        })
    }

    /// Returns the program's compiled binary.
    ///
    /// You can store the result in a file, then reload it later. This avoids having to compile
    /// the source code every time.
    pub fn get_binary(&self) -> Result<Binary, GetBinaryError> {
        unsafe {
            let ctxt = self.context.make_current();

            if ctxt.version >= &Version(Api::Gl, 4, 1) ||
               ctxt.extensions.gl_arb_get_programy_binary
            {
                let id = match self.id {
                    Handle::Id(id) => id,
                    Handle::Handle(_) => unreachable!()
                };

                let mut num_supported_formats = 0;
                ctxt.gl.GetIntegerv(gl::NUM_PROGRAM_BINARY_FORMATS, &mut num_supported_formats);
                if num_supported_formats == 0 {
                    return Err(GetBinaryError::NoFormats)
                }

                let mut buf_len = 0;
                ctxt.gl.GetProgramiv(id, gl::PROGRAM_BINARY_LENGTH, &mut buf_len);

                let mut format = 0;
                let mut storage: Vec<u8> = Vec::with_capacity(buf_len as usize);
                ctxt.gl.GetProgramBinary(id, buf_len, &mut buf_len, &mut format,
                                         storage.as_mut_ptr() as *mut _);
                storage.set_len(buf_len as usize);
                attach_glium_header(&self, &mut storage);
                Ok(Binary {
                    format: format,
                    content: storage,
                })

            } else {
                Err(GetBinaryError::NotSupported)
            }
        }
    }

    /// Returns the *location* of an output fragment, if it exists.
    ///
    /// The *location* is low-level information that is used internally by glium.
    /// You probably don't need to call this function.
    ///
    /// You can declare output fragments in your shaders by writing:
    ///
    /// ```notrust
    /// out vec4 foo;
    /// ```
    ///
    pub fn get_frag_data_location(&self, name: &str) -> Option<u32> {
        // looking for a cached value
        if let Some(result) = self.frag_data_locations.borrow_mut().get(name) {
            return result.clone();
        }

        // querying opengl
        let name_c = ffi::CString::new(name.as_bytes()).unwrap();

        let ctxt = self.context.make_current();

        let value = unsafe {
            match self.id {
                Handle::Id(id) => {
                    assert!(ctxt.version >= &Version(Api::Gl, 2, 0));
                    ctxt.gl.GetFragDataLocation(id, name_c.as_bytes_with_nul().as_ptr()
                                                as *const raw::c_char)
                },
                Handle::Handle(id) => {
                    // not supported
                    -1
                }
            }
        };

        let location = match value {
            -1 => None,
            a => Some(a as u32),
        };

        self.frag_data_locations.borrow_mut().insert(name.to_owned(), location);
        location
    }

    /// Returns informations about a uniform variable, if it exists.
    #[inline]
    pub fn get_uniform(&self, name: &str) -> Option<&Uniform> {
        self.uniforms.get(name)
    }

    /// Returns an iterator to the list of uniforms.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # let program: glium::Program = unsafe { std::mem::uninitialized() };
    /// for (name, uniform) in program.uniforms() {
    ///     println!("Name: {} - Type: {:?}", name, uniform.ty);
    /// }
    /// ```
    #[inline]
    pub fn uniforms(&self) -> hash_map::Iter<String, Uniform> {
        self.uniforms.iter()
    }

    /// Returns a list of uniform blocks.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # let program: glium::Program = unsafe { std::mem::uninitialized() };
    /// for (name, uniform) in program.get_uniform_blocks() {
    ///     println!("Name: {}", name);
    /// }
    /// ```
    #[inline]
    pub fn get_uniform_blocks(&self)
                              -> &HashMap<String, UniformBlock, BuildHasherDefault<FnvHasher>> {
        &self.uniform_blocks
    }

    /// Returns the list of transform feedback varyings.
    #[inline]
    pub fn get_transform_feedback_buffers(&self) -> &[TransformFeedbackBuffer] {
        &self.tf_buffers
    }

    /// True if the transform feedback output of this program matches the specified `VertexFormat`
    /// and `stride`.
    ///
    /// The `stride` is the number of bytes between two vertices.
    ///
    /// This function only check the identity for type and offset(exclude the naming) between vertex attributes.
    ///
    /// The correctness of semantic meaning(i.e. naming) between vertex attributes is the responsibility of user.
    pub fn transform_feedback_matches(&self, format: &VertexFormat, stride: usize) -> bool {
        // TODO: doesn't support multiple buffers

        if self.get_transform_feedback_buffers().len() != 1 {
            return false;
        }

        let buf = &self.get_transform_feedback_buffers()[0];

        if buf.stride != stride {
            return false;
        }

        for elem in buf.elements.iter() {
            if format.iter().find(|e| e.1 == elem.offset && e.2 == elem.ty)
                            .is_none()
            {
                return false;
            }
            

            if format.iter().any(|e| e.1 != elem.offset && e.0 == elem.name) {
                return false;
            }
        }

        true
    }

    /// Returns the type of geometry that transform feedback would generate, or `None` if it
    /// depends on the vertex/index data passed when drawing.
    ///
    /// This corresponds to `GL_GEOMETRY_OUTPUT_TYPE` or `GL_TESS_GEN_MODE`. If the program doesn't
    /// contain either a geometry shader or a tessellation evaluation shader, returns `None`.
    #[inline]
    pub fn get_output_primitives(&self) -> Option<OutputPrimitives> {
        self.output_primitives
    }

    /// Returns true if the program contains a tessellation stage.
    #[inline]
    pub fn has_tessellation_shaders(&self) -> bool {
        self.has_tessellation_control_shader() | self.has_tessellation_evaluation_shader()
    }

    /// Returns true if the program contains a tessellation control stage.
    #[inline]
    pub fn has_tessellation_control_shader(&self) -> bool {
        self.has_tessellation_control_shader
    }

    /// Returns true if the program contains a tessellation evaluation stage.
    #[inline]
    pub fn has_tessellation_evaluation_shader(&self) -> bool {
        self.has_tessellation_evaluation_shader
    }

    /// Returns true if the program contains a geometry shader.
    #[inline]
    pub fn has_geometry_shader(&self) -> bool {
        self.has_geometry_shader
    }

    /// Returns informations about an attribute, if it exists.
    #[inline]
    pub fn get_attribute(&self, name: &str) -> Option<&Attribute> {
        self.attributes.get(name)
    }

    /// Returns an iterator to the list of attributes.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # let program: glium::Program = unsafe { std::mem::uninitialized() };
    /// for (name, attribute) in program.attributes() {
    ///     println!("Name: {} - Type: {:?}", name, attribute.ty);
    /// }
    /// ```
    #[inline]
    pub fn attributes(&self) -> hash_map::Iter<String, Attribute> {
        self.attributes.iter()
    }

    /// Returns the list of shader storage blocks.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # let program: glium::Program = unsafe { std::mem::uninitialized() };
    /// for (name, uniform) in program.get_shader_storage_blocks() {
    ///     println!("Name: {}", name);
    /// }
    /// ```
    #[inline]
    pub fn get_shader_storage_blocks(&self)
            -> &HashMap<String, UniformBlock, BuildHasherDefault<FnvHasher>> {
        &self.ssbos
    }

    /// Returns data associated with the programs subroutines.
    #[inline]
    pub fn get_subroutine_data(&self) -> &SubroutineData {
        &self.subroutine_data
    }

    /// Assumes that the program contains a compute shader and executes it.
    ///
    /// # Safety
    ///
    /// The program *must* contain a compute shader.
    /// TODO: check inside the program if it has a compute shader instead of being unsafe
    pub unsafe fn dispatch_compute<U>(&self, uniforms: U, x: u32, y: u32, z: u32)
                                      -> Result<(), DrawError>      // TODO: other error?
                                      where U: Uniforms
    {
        let mut ctxt = self.context.make_current();

        // TODO: return an error instead
        assert!(x < ctxt.capabilities.max_compute_work_group_count.0 as u32);
        assert!(y < ctxt.capabilities.max_compute_work_group_count.1 as u32);
        assert!(z < ctxt.capabilities.max_compute_work_group_count.2 as u32);

        assert!(ctxt.version >= &Version(Api::Gl, 4, 3) ||
                ctxt.version >= &Version(Api::GlEs, 3, 1) ||
                ctxt.extensions.gl_arb_compute_shader);

        TimeElapsedQuery::end_conditional_render(&mut ctxt);

        let mut fences = Vec::with_capacity(0);

        self.use_program(&mut ctxt);
        uniforms.bind_uniforms(&mut ctxt, self, &mut fences)?;
        ctxt.gl.DispatchCompute(x, y, z);

        for fence in fences {
            fence.insert(&mut ctxt);
        }

        Ok(())
    }

    /// Assumes that the program contains a compute shader and executes it.
    ///
    /// # Safety
    ///
    /// The program *must* contain a compute shader.
    /// TODO: check inside the program if it has a compute shader instead of being unsafe
    pub unsafe fn dispatch_compute_indirect<U>(&self, uniforms: U,
                                               buffer: BufferSlice<ComputeCommand>)
                                               -> Result<(), DrawError>      // TODO: other error?
                                               where U: Uniforms
    {
        let mut ctxt = self.context.make_current();

        assert!(ctxt.version >= &Version(Api::Gl, 4, 3) ||
                ctxt.version >= &Version(Api::GlEs, 3, 1) ||
                ctxt.extensions.gl_arb_compute_shader);

        TimeElapsedQuery::end_conditional_render(&mut ctxt);

        buffer.prepare_and_bind_for_dispatch_indirect(&mut ctxt);
        let offset = buffer.get_offset_bytes();

        // an error is generated if the offset is not a multiple of 4
        assert!(offset % 4 == 0);

        if let Some(fence) = buffer.add_fence() {
            fence.insert(&mut ctxt);
        }

        self.use_program(&mut ctxt);

        let mut fences = Vec::with_capacity(0);
        uniforms.bind_uniforms(&mut ctxt, self, &mut fences)?;

        ctxt.gl.DispatchComputeIndirect(offset as gl::types::GLintptr);

        for fence in fences {
            fence.insert(&mut ctxt);
        }

        Ok(())
    }
}

impl fmt::Debug for RawProgram {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        (format!("Program #{:?}", self.id)).fmt(formatter)
    }
}

impl GlObject for RawProgram {
    type Id = Handle;

    #[inline]
    fn get_id(&self) -> Handle {
        self.id
    }
}

impl ProgramExt for RawProgram {
    #[inline]
    fn use_program(&self, ctxt: &mut CommandContext) {
        unsafe {
            let program_id = self.get_id();
            if ctxt.state.program != program_id {
                match program_id {
                    Handle::Id(id) => ctxt.gl.UseProgram(id),
                    Handle::Handle(id) => ctxt.gl.UseProgramObjectARB(id),
                }
                ctxt.state.program = program_id;
            }
        }
    }

    #[inline]
    fn set_uniform(&self, ctxt: &mut CommandContext, uniform_location: gl::types::GLint,
                   value: &RawUniformValue)
    {
        self.uniform_values.set_uniform_value(ctxt, self.id, uniform_location, value);
    }

    #[inline]
    fn set_uniform_block_binding(&self, ctxt: &mut CommandContext, block_location: gl::types::GLuint,
                                 value: gl::types::GLuint)
    {
        self.uniform_values.set_uniform_block_binding(ctxt, self.id, block_location, value);
    }

    #[inline]
    fn set_shader_storage_block_binding(&self, ctxt: &mut CommandContext,
                                        block_location: gl::types::GLuint,
                                        value: gl::types::GLuint)
    {
        self.uniform_values.set_shader_storage_block_binding(ctxt, self.id, block_location, value);
    }

    #[inline]
    fn set_subroutine_uniforms_for_stage(&self, ctxt: &mut CommandContext,
                                         stage: ShaderStage,
                                         indices: &[gl::types::GLuint])
    {
        self.uniform_values.set_subroutine_uniforms_for_stage(ctxt, self.id, stage, indices);
    }

    #[inline]
    fn get_uniform(&self, name: &str) -> Option<&Uniform> {
        self.uniforms.get(name)
    }

    #[inline]
    fn get_uniform_blocks(&self) -> &HashMap<String, UniformBlock, BuildHasherDefault<FnvHasher>> {
        &self.uniform_blocks
    }

    #[inline]
    fn get_shader_storage_blocks(&self)
                                 -> &HashMap<String, UniformBlock, BuildHasherDefault<FnvHasher>> {
        &self.ssbos
    }

    #[inline]
    fn get_subroutine_data(&self) -> &SubroutineData {
        &self.subroutine_data
    }
}

impl Drop for RawProgram {
    fn drop(&mut self) {
        let mut ctxt = self.context.make_current();

        // removing VAOs which contain this program
        VertexAttributesSystem::purge_program(&mut ctxt, self.id);

        // sending the destroy command
        unsafe {
            match self.id {
                Handle::Id(id) => {
                    assert!(ctxt.version >= &Version(Api::Gl, 2, 0) ||
                            ctxt.version >= &Version(Api::GlEs, 2, 0));

                    if ctxt.state.program == Handle::Id(id) {
                        ctxt.gl.UseProgram(0);
                        ctxt.state.program = Handle::Id(0);
                    }

                    ctxt.gl.DeleteProgram(id);
                },
                Handle::Handle(id) => {
                    assert!(ctxt.extensions.gl_arb_shader_objects);

                    if ctxt.state.program == Handle::Handle(id) {
                        ctxt.gl.UseProgramObjectARB(0 as gl::types::GLhandleARB);
                        ctxt.state.program = Handle::Handle(0 as gl::types::GLhandleARB);
                    }

                    ctxt.gl.DeleteObjectARB(id);
                }
            }
        }
    }
}

/// Builds an empty program from within the GL context.
unsafe fn create_program(ctxt: &mut CommandContext) -> Handle {
    let id = if ctxt.version >= &Version(Api::Gl, 2, 0) ||
                ctxt.version >= &Version(Api::GlEs, 2, 0)
    {
        Handle::Id(ctxt.gl.CreateProgram())
    } else if ctxt.extensions.gl_arb_shader_objects {
        Handle::Handle(ctxt.gl.CreateProgramObjectARB())
    } else {
        unreachable!()
    };

    if id == Handle::Id(0) || id == Handle::Handle(0 as gl::types::GLhandleARB) {
        panic!("glCreateProgram failed");
    }

    id
}

unsafe fn check_program_link_errors(ctxt: &mut CommandContext, id: Handle)
                                    -> Result<(), ProgramCreationError>
{
    let mut link_success: gl::types::GLint = 0;

    match id {
        Handle::Id(id) => {
            assert!(ctxt.version >= &Version(Api::Gl, 2, 0) ||
                    ctxt.version >= &Version(Api::GlEs, 2, 0));
            ctxt.gl.GetProgramiv(id, gl::LINK_STATUS, &mut link_success);
        },
        Handle::Handle(id) => {
            assert!(ctxt.extensions.gl_arb_shader_objects);
            ctxt.gl.GetObjectParameterivARB(id, gl::OBJECT_LINK_STATUS_ARB,
                                            &mut link_success);
        }
    }

    if link_success == 0 {
        use ProgramCreationError::LinkingError;

        match ctxt.gl.GetError() {
            gl::NO_ERROR => (),
            gl::INVALID_VALUE => {
                return Err(LinkingError(format!("glLinkProgram triggered \
                                                 GL_INVALID_VALUE")));
            },
            gl::INVALID_OPERATION => {
                return Err(LinkingError(format!("glLinkProgram triggered \
                                                 GL_INVALID_OPERATION")));
            },
            _ => {
                return Err(LinkingError(format!("glLinkProgram triggered an \
                                                 unknown error")));
            }
        };

        let mut error_log_size: gl::types::GLint = 0;

        match id {
            Handle::Id(id) => {
                assert!(ctxt.version >= &Version(Api::Gl, 2, 0) ||
                    ctxt.version >= &Version(Api::GlEs, 2, 0));
                ctxt.gl.GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut error_log_size);
            },
            Handle::Handle(id) => {
                assert!(ctxt.extensions.gl_arb_shader_objects);
                ctxt.gl.GetObjectParameterivARB(id, gl::OBJECT_INFO_LOG_LENGTH_ARB,
                                                &mut error_log_size);
            }
        }

        let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as usize);

        match id {
            Handle::Id(id) => {
                assert!(ctxt.version >= &Version(Api::Gl, 2, 0) ||
                    ctxt.version >= &Version(Api::GlEs, 2, 0));
                ctxt.gl.GetProgramInfoLog(id, error_log_size, &mut error_log_size,
                                          error_log.as_mut_ptr() as *mut gl::types::GLchar);
            },
            Handle::Handle(id) => {
                assert!(ctxt.extensions.gl_arb_shader_objects);
                ctxt.gl.GetInfoLogARB(id, error_log_size, &mut error_log_size,
                                      error_log.as_mut_ptr() as *mut gl::types::GLchar);
            }
        }

        error_log.set_len(error_log_size as usize);

        let msg = String::from_utf8(error_log).unwrap();
        return Err(LinkingError(msg));
    }

    Ok(())
}
