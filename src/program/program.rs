use gl;
use libc;

use context::CommandContext;
use version::Version;
use version::Api;

use backend::Facade;
use context::Context;
use ContextExt;

use std::{ffi, fmt, mem};
use std::error::Error;
use std::collections::hash_map::{self, HashMap};
use std::rc::Rc;
use std::cell::RefCell;

use GlObject;
use ProgramExt;
use Handle;
use RawUniformValue;

use program::{COMPILER_GLOBAL_LOCK, ProgramCreationInput, Binary};
use program::uniforms_storage::UniformsStorage;

use program::reflection::{Uniform, UniformBlock, OutputPrimitives};
use program::reflection::{Attribute, TransformFeedbackMode, TransformFeedbackBuffer};
use program::reflection::{reflect_uniforms, reflect_attributes, reflect_uniform_blocks};
use program::reflection::{reflect_transform_feedback, reflect_geometry_output_type};
use program::reflection::{reflect_tess_eval_output_type};
use program::shader::build_shader;

use vertex::VertexFormat;
use vertex_array_object::VertexAttributesSystem;

/// Error that can be triggered when creating a `Program`.
#[derive(Clone, Debug)]
pub enum ProgramCreationError {
    /// Error while compiling one of the shaders.
    CompilationError(String),

    /// Error while linking the program.
    LinkingError(String),

    /// One of the requested shader types is not supported by the backend.
    ///
    /// Usually the case for geometry shaders.
    ShaderTypeNotSupported,

    /// The OpenGL implementation doesn't provide a compiler.
    CompilationNotSupported,

    /// You have requested transform feedback varyings, but transform feedback is not supported
    /// by the backend.
    TransformFeedbackNotSupported,
}

impl fmt::Display for ProgramCreationError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &ProgramCreationError::CompilationError(ref s) =>
                formatter.write_fmt(format_args!("Compilation error in one of the shaders: {}", s)),
            &ProgramCreationError::LinkingError(ref s) =>
                formatter.write_fmt(format_args!("Error while linking shaders together: {}", s)),
            &ProgramCreationError::ShaderTypeNotSupported =>
                formatter.write_str("One of the request shader type is \
                                    not supported by the backend"),
            &ProgramCreationError::CompilationNotSupported =>
                formatter.write_str("The backend doesn't support shaders compilation"),
            &ProgramCreationError::TransformFeedbackNotSupported => 
                formatter.write_str("You requested transform feedback, but this feature is not \
                                     supported by the backend"),
        }
    }
}

impl Error for ProgramCreationError {
    fn description(&self) -> &str {
        match self {
            &ProgramCreationError::CompilationError(_) => "Compilation error in one of the \
                                                           shaders",
            &ProgramCreationError::LinkingError(_) => "Error while linking shaders together",
            &ProgramCreationError::ShaderTypeNotSupported => "One of the request shader type is \
                                                              not supported by the backend",
            &ProgramCreationError::CompilationNotSupported => "The backend doesn't support \
                                                               shaders compilation",
            &ProgramCreationError::TransformFeedbackNotSupported => "Transform feedback is not \
                                                                     supported by the backend.",
        }
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

/// A combination of shaders linked together.
pub struct Program {
    context: Rc<Context>,
    id: Handle,
    uniform_values: UniformsStorage,
    uniforms: HashMap<String, Uniform>,
    uniform_blocks: HashMap<String, UniformBlock>,
    attributes: HashMap<String, Attribute>,
    frag_data_locations: RefCell<HashMap<String, Option<u32>>>,
    tf_buffers: Vec<TransformFeedbackBuffer>,
    output_primitives: Option<OutputPrimitives>,
    has_tessellation_shaders: bool,
}

impl Program {
    /// Builds a new program.
    pub fn new<'a, F, I>(facade: &F, input: I) -> Result<Program, ProgramCreationError>
                         where I: Into<ProgramCreationInput<'a>>, F: Facade
    {
        let input = input.into();

        if let ProgramCreationInput::SourceCode { .. } = input {
            Program::from_source_impl(facade, input)
        } else {
            Program::from_binary_impl(facade, input)
        }
    }

    /// Builds a new program from GLSL source code.
    ///
    /// A program is a group of shaders linked together.
    ///
    /// # Parameters
    ///
    /// - `vertex_shader`: Source code of the vertex shader.
    /// - `fragment_shader`: Source code of the fragment shader.
    /// - `geometry_shader`: Source code of the geometry shader.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let display: glium::Display = unsafe { std::mem::uninitialized() };
    /// # let vertex_source = ""; let fragment_source = ""; let geometry_source = "";
    /// let program = glium::Program::from_source(&display, vertex_source, fragment_source,
    ///     Some(geometry_source));
    /// ```
    ///
    pub fn from_source<'a, F>(facade: &F, vertex_shader: &'a str, fragment_shader: &'a str,
                              geometry_shader: Option<&'a str>)
                              -> Result<Program, ProgramCreationError> where F: Facade
    {
        Program::from_source_impl(facade, ProgramCreationInput::SourceCode {
            vertex_shader: vertex_shader,
            fragment_shader: fragment_shader,
            geometry_shader: geometry_shader,
            tessellation_control_shader: None,
            tessellation_evaluation_shader: None,
            transform_feedback_varyings: None,
        })
    }

    /// Compiles a program from source.
    ///
    /// Must only be called if `input` is a `ProgramCreationInput::SourceCode`, will
    /// panic otherwise.
    fn from_source_impl<F>(facade: &F, input: ProgramCreationInput)
                           -> Result<Program, ProgramCreationError>
                           where F: Facade
    {
        let _lock = COMPILER_GLOBAL_LOCK.lock();

        let mut has_geometry_shader = false;
        let mut has_tessellation_shaders = false;

        // getting an array of the source codes and their type
        let (shaders, transform_feedback_varyings): (Vec<(&str, gl::types::GLenum)>, _) = {
            let (vertex_shader, fragment_shader, geometry_shader,
                 tessellation_control_shader, tessellation_evaluation_shader,
                 transform_feedback_varyings) = match input
            {
                ProgramCreationInput::SourceCode { vertex_shader, fragment_shader,
                                                   geometry_shader, tessellation_control_shader,
                                                   tessellation_evaluation_shader,
                                                   transform_feedback_varyings } =>
                {
                    (vertex_shader, fragment_shader, geometry_shader,
                     tessellation_control_shader, tessellation_evaluation_shader,
                     transform_feedback_varyings)
                },
                _ => unreachable!()     // the function shouldn't be called with anything else
            };

            let mut shaders = vec![
                (vertex_shader, gl::VERTEX_SHADER),
                (fragment_shader, gl::FRAGMENT_SHADER)
            ];

            if let Some(gs) = geometry_shader {
                has_geometry_shader = true;
                shaders.push((gs, gl::GEOMETRY_SHADER));
            }

            if let Some(ts) = tessellation_control_shader {
                has_tessellation_shaders = true;
                shaders.push((ts, gl::TESS_CONTROL_SHADER));
            }

            if let Some(ts) = tessellation_evaluation_shader {
                has_tessellation_shaders = true;
                shaders.push((ts, gl::TESS_EVALUATION_SHADER));
            }

            if transform_feedback_varyings.is_some() &&
                (facade.get_context().get_version() >= &Version(Api::Gl, 3, 0) ||
                    !facade.get_context().get_extensions().gl_ext_transform_feedback)
            {
                return Err(ProgramCreationError::TransformFeedbackNotSupported);
            }

            (shaders, transform_feedback_varyings)
        };

        let shaders_store = {
            let mut shaders_store = Vec::new();
            for (src, ty) in shaders.into_iter() {
                shaders_store.push(try!(build_shader(facade, ty, src)));
            }
            shaders_store
        };

        let mut shaders_ids = Vec::new();
        for sh in shaders_store.iter() {
            shaders_ids.push(sh.get_id());
        }

        let mut ctxt = facade.get_context().make_current();

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
            if let Some((names, mode)) = transform_feedback_varyings {
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
            try!(check_program_link_errors(&mut ctxt, id));

            id
        };

        let uniforms = unsafe { reflect_uniforms(&mut ctxt, id) };
        let attributes = unsafe { reflect_attributes(&mut ctxt, id) };
        let blocks = unsafe { reflect_uniform_blocks(&mut ctxt, id) };
        let tf_buffers = unsafe { reflect_transform_feedback(&mut ctxt, id) };

        let output_primitives = if has_geometry_shader {
            Some(unsafe { reflect_geometry_output_type(&mut ctxt, id) })
        } else if has_tessellation_shaders {
            Some(unsafe { reflect_tess_eval_output_type(&mut ctxt, id) })
        } else {
            None
        };

        Ok(Program {
            context: facade.get_context().clone(),
            id: id,
            uniforms: uniforms,
            uniform_values: UniformsStorage::new(),
            uniform_blocks: blocks,
            attributes: attributes,
            frag_data_locations: RefCell::new(HashMap::new()),
            tf_buffers: tf_buffers,
            output_primitives: output_primitives,
            has_tessellation_shaders: has_tessellation_shaders,
        })
    }

    /// Creates a program from binary.
    ///
    /// Must only be called if `input` is a `ProgramCreationInput::Binary`, will
    /// panic otherwise.
    fn from_binary_impl<F>(facade: &F, input: ProgramCreationInput)
                           -> Result<Program, ProgramCreationError> where F: Facade
    {
        let binary = match input {
            ProgramCreationInput::Binary { data } => data,
            _ => unreachable!()
        };

        let mut ctxt = facade.get_context().make_current();

        let id = unsafe {
            let id = create_program(&mut ctxt);

            match id {
                Handle::Id(id) => {
                    assert!(ctxt.version >= &Version(Api::Gl, 2, 0));
                    ctxt.gl.ProgramBinary(id, binary.format,
                                          binary.content.as_ptr() as *const _,
                                          binary.content.len() as gl::types::GLsizei);
                },
                Handle::Handle(id) => unreachable!()
            };

            // checking for errors
            try!(check_program_link_errors(&mut ctxt, id));

            id
        };

        let (uniforms, attributes, blocks, tf_buffers) = unsafe {
            (
                reflect_uniforms(&mut ctxt, id),
                reflect_attributes(&mut ctxt, id),
                reflect_uniform_blocks(&mut ctxt, id),
                reflect_transform_feedback(&mut ctxt, id),
            )
        };

        Ok(Program {
            context: facade.get_context().clone(),
            id: id,
            uniforms: uniforms,
            uniform_values: UniformsStorage::new(),
            uniform_blocks: blocks,
            attributes: attributes,
            frag_data_locations: RefCell::new(HashMap::new()),
            tf_buffers: tf_buffers,
            output_primitives: None,            // FIXME: 
            has_tessellation_shaders: true,     // FIXME: 
        })
    }

    /// Returns the program's compiled binary.
    ///
    /// You can store the result in a file, then reload it later. This avoids having to compile
    /// the source code every time.
    ///
    /// # Features
    ///
    /// Only available if the `gl_program_binary` feature is enabled.
    #[cfg(feature = "gl_program_binary")]
    pub fn get_binary(&self) -> Binary {
        self.get_binary_if_supported().unwrap()
    }

    /// Returns the program's compiled binary.
    ///
    /// Same as `get_binary` but always available. Returns `None` if the backend doesn't support
    /// getting or reloading the program's binary.
    pub fn get_binary_if_supported(&self) -> Option<Binary> {
        unsafe {
            let ctxt = self.context.make_current();

            if ctxt.version >= &Version(Api::Gl, 4, 1) ||
               ctxt.extensions.gl_arb_get_programy_binary
            {
                let id = match self.id {
                    Handle::Id(id) => id,
                    Handle::Handle(_) => unreachable!()
                };

                let mut buf_len = mem::uninitialized();
                ctxt.gl.GetProgramiv(id, gl::PROGRAM_BINARY_LENGTH, &mut buf_len);

                let mut format = mem::uninitialized();
                let mut storage: Vec<u8> = Vec::with_capacity(buf_len as usize);
                ctxt.gl.GetProgramBinary(id, buf_len, &mut buf_len, &mut format,
                                         storage.as_mut_ptr() as *mut libc::c_void);
                storage.set_len(buf_len as usize);

                Some(Binary {
                    format: format,
                    content: storage,
                })

            } else {
                None
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
                                                as *const libc::c_char)
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

        self.frag_data_locations.borrow_mut().insert(name.to_string(), location);
        location
    }

    /// Returns informations about a uniform variable, if it exists.
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
    pub fn get_uniform_blocks(&self) -> &HashMap<String, UniformBlock> {
        &self.uniform_blocks
    }

    /// Returns the list of transform feedback varyings.
    pub fn get_transform_feedback_buffers(&self) -> &[TransformFeedbackBuffer] {
        &self.tf_buffers
    }

    /// True if the transform feedback output of this program matches the specified `VertexFormat`
    /// and `stride`.
    ///
    /// The `stride` is the number of bytes between two vertices.
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
            if format.iter().find(|e| &e.0 == &*elem.name && e.1 == elem.offset && e.2 == elem.ty)
                            .is_none()
            {
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
    pub fn get_output_primitives(&self) -> Option<OutputPrimitives> {
        self.output_primitives
    }

    /// Returns true if the program contains a tessellation stage.
    pub fn has_tessellation_shaders(&self) -> bool {
        self.has_tessellation_shaders
    }

    /// Returns informations about an attribute, if it exists.
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
    pub fn attributes(&self) -> hash_map::Iter<String, Attribute> {
        self.attributes.iter()
    }

    /// Returns true if the program has been configured to output sRGB instead of RGB.
    pub fn has_srgb_output(&self) -> bool {
        false
    }
}

impl fmt::Debug for Program {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        (format!("Program #{:?}", self.id)).fmt(formatter)
    }
}

impl GlObject for Program {
    type Id = Handle;
    fn get_id(&self) -> Handle {
        self.id
    }
}

impl ProgramExt for Program {
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

    fn set_uniform(&self, ctxt: &mut CommandContext, uniform_location: gl::types::GLint,
                   value: &RawUniformValue)
    {
        self.uniform_values.set_uniform_value(ctxt, self.id, uniform_location, value);
    }

    fn set_block(&self, ctxt: &mut CommandContext, block_location: gl::types::GLuint,
                 value: gl::types::GLuint)
    {
        self.uniform_values.set_block_binding(ctxt, self.id, block_location, value);
    }
}

impl Drop for Program {
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
    let mut link_success: gl::types::GLint = mem::uninitialized();

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

        let mut error_log_size: gl::types::GLint = mem::uninitialized();

        match id {
            Handle::Id(id) => {
                assert!(ctxt.version >= &Version(Api::Gl, 2, 0));
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
                assert!(ctxt.version >= &Version(Api::Gl, 2, 0));
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
