use gl;
use libc;

use context;
use context::CommandContext;
use context::GlVersion;
use version::Api;

use std::{ffi, fmt, mem};
use std::collections::hash_state::DefaultState;
use std::collections::HashMap;
use std::default::Default;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use util::FnvHasher;

use Display;
use GlObject;
use Handle;

use program::{COMPILER_GLOBAL_LOCK, IntoProgramCreationInput, ProgramCreationInput, Binary};

use program::reflection::{Uniform, UniformBlock};
use program::reflection::{Attribute, TransformFeedbackVarying, TransformFeedbackMode};
use program::reflection::{reflect_uniforms, reflect_attributes, reflect_uniform_blocks};
use program::reflection::{reflect_transform_feedback};
use program::shader::build_shader;


// TODO: remove this hack
#[doc(hidden)]
pub fn get_uniforms_locations(program: &Program) -> Arc<HashMap<String, Uniform, DefaultState<FnvHasher>>>
{
    program.uniforms.clone()
}

// TODO: remove this hack
#[doc(hidden)]
pub fn get_attributes(program: &Program) -> Arc<HashMap<String, Attribute, DefaultState<FnvHasher>>>
{
    program.attributes.clone()
}

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

    /// You have requested transform feedback varyings, but transform feedback is not supported
    /// by the backend.
    TransformFeedbackNotSupported,
}

impl ::std::fmt::Display for ProgramCreationError {
    fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        match self {
            &ProgramCreationError::CompilationError(ref s) =>
                formatter.write_fmt(format_args!("Compilation error in one of the shaders: {}", s)),
            &ProgramCreationError::LinkingError(ref s) =>
                formatter.write_fmt(format_args!("Error while linking shaders together: {}", s)),
            &ProgramCreationError::ShaderTypeNotSupported =>
                formatter.write_str("One of the request shader type is \
                                    not supported by the backend"),
            &ProgramCreationError::TransformFeedbackNotSupported => 
                formatter.write_str("You requested transform feedback, but this feature is not \
                                     supported by the backend"),
        }
    }
}

impl ::std::error::Error for ProgramCreationError {
    fn description(&self) -> &str {
        match self {
            &ProgramCreationError::CompilationError(_) => "Compilation error in one of the \
                                                           shaders",
            &ProgramCreationError::LinkingError(_) => "Error while linking shaders together",
            &ProgramCreationError::ShaderTypeNotSupported => "One of the request shader type is \
                                                              not supported by the backend",
            &ProgramCreationError::TransformFeedbackNotSupported => "Transform feedback is not \
                                                                     supported by the backend.",
        }
    }

    fn cause(&self) -> Option<&::std::error::Error> {
        None
    }
}

/// A combination of shaders linked together.
pub struct Program {
    display: Display,
    id: Handle,
    uniforms: Arc<HashMap<String, Uniform, DefaultState<FnvHasher>>>,
    uniform_blocks: Arc<HashMap<String, UniformBlock, DefaultState<FnvHasher>>>,
    attributes: Arc<HashMap<String, Attribute, DefaultState<FnvHasher>>>,
    frag_data_locations: Mutex<HashMap<String, Option<u32>, DefaultState<FnvHasher>>>,
    varyings: Option<(Vec<TransformFeedbackVarying>, TransformFeedbackMode)>,
    has_tessellation_shaders: bool,
}

impl Program {
    /// Builds a new program.
    pub fn new<'a, I>(display: &Display, input: I) -> Result<Program, ProgramCreationError>
                      where I: IntoProgramCreationInput<'a>
    {
        let input = input.into_program_creation_input();

        if let ProgramCreationInput::SourceCode { .. } = input {
            Program::from_source_impl(display, input)
        } else {
            Program::from_binary_impl(display, input)
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
    #[unstable = "The list of shaders and the result error will probably change"]
    pub fn from_source<'a>(display: &Display, vertex_shader: &'a str, fragment_shader: &'a str,
                           geometry_shader: Option<&'a str>)
                           -> Result<Program, ProgramCreationError>
    {
        Program::from_source_impl(display, ProgramCreationInput::SourceCode {
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
    fn from_source_impl(display: &Display, input: ProgramCreationInput)
                        -> Result<Program, ProgramCreationError>
    {
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
                (display.context.context.get_version() >= &GlVersion(Api::Gl, 3, 0) ||
                    !display.context.context.get_extensions().gl_ext_transform_feedback)
            {
                return Err(ProgramCreationError::TransformFeedbackNotSupported);
            }

            (shaders, transform_feedback_varyings)
        };

        let shaders_store = {
            let mut shaders_store = Vec::new();
            for (src, ty) in shaders.into_iter() {
                shaders_store.push(try!(build_shader(display, ty, src)));
            }
            shaders_store
        };

        let mut shaders_ids = Vec::new();
        for sh in shaders_store.iter() {
            shaders_ids.push(sh.get_id());
        }

        let (tx, rx) = channel();
        display.context.context.exec(move |mut ctxt| {
            unsafe {
                let id = create_program(&mut ctxt);

                // attaching shaders
                for sh in shaders_ids.iter() {
                    match (id, sh) {
                        (Handle::Id(id), &Handle::Id(sh)) => {
                            assert!(ctxt.version >= &GlVersion(Api::Gl, 2, 0));
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
                        ffi::CString::from_vec(name.into_bytes())
                    }).collect::<Vec<_>>();
                    let names_ptr = names.iter().map(|n| n.as_ptr()).collect::<Vec<_>>();

                    if ctxt.version >= &GlVersion(Api::Gl, 3, 0) {
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
                    let _lock = COMPILER_GLOBAL_LOCK.lock();

                    ctxt.shared_debug_output.report_errors.store(false, Ordering::Relaxed);

                    match id {
                        Handle::Id(id) => {
                            assert!(ctxt.version >= &GlVersion(Api::Gl, 2, 0));
                            ctxt.gl.LinkProgram(id);
                        },
                        Handle::Handle(id) => {
                            assert!(ctxt.extensions.gl_arb_shader_objects);
                            ctxt.gl.LinkProgramARB(id);
                        }
                    }

                    ctxt.shared_debug_output.report_errors.store(true, Ordering::Relaxed);
                }

                // checking for errors
                match check_program_link_errors(&mut ctxt, id) {
                    Ok(_) => (),
                    Err(err) => {
                        tx.send(Err(err)).ok();
                        return;
                    }
                }

                tx.send(Ok(id)).unwrap();
            }
        });

        let id = try!(rx.recv().unwrap());

        let (tx, rx) = channel();
        display.context.context.exec(move |mut ctxt| {
            unsafe {
                tx.send((
                    reflect_uniforms(&mut ctxt, id),
                    reflect_attributes(&mut ctxt, id),
                    reflect_uniform_blocks(&mut ctxt, id),
                    reflect_transform_feedback(&mut ctxt, id),
                )).ok();
            }
        });

        let (uniforms, attributes, blocks, varyings) = rx.recv().unwrap();

        Ok(Program {
            display: display.clone(),
            id: id,
            uniforms: Arc::new(uniforms),
            uniform_blocks: Arc::new(blocks),
            attributes: Arc::new(attributes),
            frag_data_locations: Mutex::new(HashMap::with_hash_state(Default::default())),
            varyings: varyings,
            has_tessellation_shaders: has_tessellation_shaders,
        })
    }

    /// Creates a program from binary.
    ///
    /// Must only be called if `input` is a `ProgramCreationInput::Binary`, will
    /// panic otherwise.
    fn from_binary_impl(display: &Display, input: ProgramCreationInput)
                        -> Result<Program, ProgramCreationError>
    {
        let binary = match input {
            ProgramCreationInput::Binary { data } => data,
            _ => unreachable!()
        };

        let (tx, rx) = channel();
        display.context.context.exec(move |mut ctxt| {
            unsafe {
                let id = create_program(&mut ctxt);

                match id {
                    Handle::Id(id) => {
                        assert!(ctxt.version >= &GlVersion(Api::Gl, 2, 0));
                        ctxt.gl.ProgramBinary(id, binary.format,
                                              binary.content.as_ptr() as *const _,
                                              binary.content.len() as gl::types::GLsizei);
                    },
                    Handle::Handle(id) => unreachable!()
                };

                // checking for errors
                match check_program_link_errors(&mut ctxt, id) {
                    Ok(_) => (),
                    Err(err) => {
                        tx.send(Err(err)).ok();
                        return;
                    }
                }

                tx.send(Ok(id)).unwrap();
            }
        });

        let id = try!(rx.recv().unwrap());

        let (tx, rx) = channel();
        display.context.context.exec(move |mut ctxt| {
            unsafe {
                tx.send((
                    reflect_uniforms(&mut ctxt, id),
                    reflect_attributes(&mut ctxt, id),
                    reflect_uniform_blocks(&mut ctxt, id),
                    reflect_transform_feedback(&mut ctxt, id),
                )).ok();
            }
        });

        let (uniforms, attributes, blocks, varyings) = rx.recv().unwrap();

        Ok(Program {
            display: display.clone(),
            id: id,
            uniforms: Arc::new(uniforms),
            uniform_blocks: Arc::new(blocks),
            attributes: Arc::new(attributes),
            frag_data_locations: Mutex::new(HashMap::with_hash_state(Default::default())),
            varyings: varyings,
            has_tessellation_shaders: true,     // FIXME: 
        })
    }

    /// Returns the program's compiled binary.
    ///
    /// You can store the result in a file, then reload it later. This avoids having to compile
    /// the source code every time.
    ///
    /// ## Features
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
        let id = self.get_id();

        let (tx, rx) = channel();
        self.display.context.context.exec(move |ctxt| {
            unsafe {
                if ctxt.version >= &context::GlVersion(Api::Gl, 4, 1) ||
                   ctxt.extensions.gl_arb_get_programy_binary
                {
                    let id = match id {
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

                    tx.send(Some((format, storage))).ok();

                } else {
                    tx.send(None).ok();
                }
            }
        });

        rx.recv().unwrap().map(|(format, storage)| {
            Binary {
                format: format,
                content: storage,
            }
        })
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
        if let Some(result) = self.frag_data_locations.lock().unwrap().get(name) {
            return result.clone();
        }

        // querying opengl
        let id = self.id.clone();
        let name_c = ffi::CString::from_slice(name.as_bytes());
        let (tx, rx) = channel();
        self.display.context.context.exec(move |ctxt| {
            unsafe {
                let value = match id {
                    Handle::Id(id) => {
                        assert!(ctxt.version >= &GlVersion(Api::Gl, 2, 0));
                        ctxt.gl.GetFragDataLocation(id, name_c.as_bytes_with_nul().as_ptr()
                                                    as *const libc::c_char)
                    },
                    Handle::Handle(id) => {
                        // not supported
                        -1
                    }
                };

                tx.send(value).ok();
            }
        });

        let location = match rx.recv().unwrap() {
            -1 => None,
            a => Some(a as u32),
        };

        self.frag_data_locations.lock().unwrap().insert(name.to_string(), location);
        location
    }

    /// Returns informations about a uniform variable, if it exists.
    pub fn get_uniform(&self, name: &str) -> Option<&Uniform> {
        self.uniforms.get(name)
    }
    
    /// Returns a list of uniform blocks.
    pub fn get_uniform_blocks(&self) -> &HashMap<String, UniformBlock, DefaultState<FnvHasher>> {
        &*self.uniform_blocks
    }

    /// Returns the list of transform feedback varyings.
    pub fn get_transform_feedback_varyings(&self) -> &[TransformFeedbackVarying] {
        self.varyings.as_ref().map(|&(ref v, _)| v.as_slice()).unwrap_or(&[])
    }

    /// Returns the mode used for transform feedback, or `None` is transform feedback is not
    /// enabled in this program or not supported.
    pub fn get_transform_feedback_mode(&self) -> Option<TransformFeedbackMode> {
        self.varyings.as_ref().map(|&(_, m)| m)
    }

    /// Returns true if the program contains a tessellation stage.
    pub fn has_tessellation_shaders(&self) -> bool {
        self.has_tessellation_shaders
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

impl Drop for Program {
    fn drop(&mut self) {
        // removing VAOs which contain this program
        {
            let mut vaos = self.display.context.vertex_array_objects.lock().unwrap();
            let to_delete = vaos.keys().filter(|&&(_, p)| p == self.id)
                .map(|k| k.clone()).collect::<Vec<_>>();
            for k in to_delete.into_iter() {
                vaos.remove(&k);
            }
        }

        // sending the destroy command
        let id = self.id.clone();
        self.display.context.context.exec(move |ctxt| {
            unsafe {
                match id {
                    Handle::Id(id) => {
                        assert!(ctxt.version >= &GlVersion(Api::Gl, 2, 0));

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
        });
    }
}

/// Builds an empty program from within the GL context.
unsafe fn create_program(ctxt: &mut CommandContext) -> Handle {
    let id = if ctxt.version >= &GlVersion(Api::Gl, 2, 0) {
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
            assert!(ctxt.version >= &GlVersion(Api::Gl, 2, 0));
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
                assert!(ctxt.version >= &GlVersion(Api::Gl, 2, 0));
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
                assert!(ctxt.version >= &GlVersion(Api::Gl, 2, 0));
                ctxt.gl.GetProgramInfoLog(id, error_log_size, &mut error_log_size,
                                          error_log.as_mut_slice().as_mut_ptr()
                                            as *mut gl::types::GLchar);
            },
            Handle::Handle(id) => {
                assert!(ctxt.extensions.gl_arb_shader_objects);
                ctxt.gl.GetInfoLogARB(id, error_log_size, &mut error_log_size,
                                      error_log.as_mut_slice().as_mut_ptr()
                                        as *mut gl::types::GLchar);
            }
        }

        error_log.set_len(error_log_size as usize);

        let msg = String::from_utf8(error_log).unwrap();
        return Err(LinkingError(msg));
    }

    Ok(())
}
