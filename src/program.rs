use gl;
use std::{fmt, mem, ptr};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, StaticMutex, MUTEX_INIT};
use {Display, DisplayImpl, GlObject};
use context::CommandContext;

/// Some shader compilers have race-condition issues.
/// We lock this mutex in the GL thread every time we compile a shader or link a program.
static COMPILER_GLOBAL_LOCK: StaticMutex = MUTEX_INIT;

struct Shader {
    display: Arc<DisplayImpl>,
    id: gl::types::GLuint,
}

impl Drop for Shader {
    fn drop(&mut self) {
        let id = self.id.clone();
        self.display.context.exec(move |: ctxt| {
            unsafe {
                ctxt.gl.DeleteShader(id);
            }
        });
    }
}

/// A combinaison of shaders linked together.
pub struct Program {
    display: Arc<DisplayImpl>,
    #[allow(dead_code)]
    shaders: Vec<Shader>,
    id: gl::types::GLuint,
    uniforms: Arc<HashMap<String, Uniform>>,
    attributes: Arc<HashMap<String, Attribute>>,
    frag_data_locations: Mutex<HashMap<String, Option<u32>>>,
}

/// Informations about a uniform (except its name).
///
/// Internal struct. Not public.
struct Uniform {
    pub location: gl::types::GLint,
    pub ty: gl::types::GLenum,
    pub size: gl::types::GLint,
}

/// Informations about an attribute of a program (except its name).
///
/// Internal struct. Not public.
#[deriving(Show)]
struct Attribute {
    pub location: gl::types::GLint,
    pub ty: gl::types::GLenum,
    pub size: gl::types::GLint,
}

/// Error that can be triggered when creating a `Program`.
#[deriving(Clone, Show)]
pub enum ProgramCreationError {
    /// Error while compiling one of the shaders.
    CompilationError(String),

    /// Error while linking the program.
    LinkingError(String),

    /// One of the request shader type is not supported by the backend.
    ///
    /// Usually the case of geometry shaders.
    ShaderTypeNotSupported,
}

impl ::std::error::Error for ProgramCreationError {
    fn description(&self) -> &str {
        match self {
            &ProgramCreationError::CompilationError(_) => "Compilation error in one of the \
                                                           shaders",
            &ProgramCreationError::LinkingError(_) => "Error while linking shaders together",
            &ProgramCreationError::ShaderTypeNotSupported => "One of the request shader type is \
                                                              not supported by the backend",
        }
    }

    fn detail(&self) -> Option<String> {
        match self {
            &ProgramCreationError::CompilationError(ref s) => Some(s.clone()),
            &ProgramCreationError::LinkingError(ref s) => Some(s.clone()),
            &ProgramCreationError::ShaderTypeNotSupported => None,
        }
    }

    fn cause(&self) -> Option<&::std::error::Error> {
        None
    }
}

impl Program {
    /// Builds a new program.
    #[deprecated = "Use Program::from_source. The `new` function will soon change its API."]
    pub fn new(display: &Display, vertex_shader: &str, fragment_shader: &str,
               geometry_shader: Option<&str>) -> Result<Program, ProgramCreationError>
    {
        Program::from_source(display, vertex_shader, fragment_shader, geometry_shader)
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
    #[experimental = "The list of shaders and the result error will probably change"]
    pub fn from_source(display: &Display, vertex_shader: &str, fragment_shader: &str,
                       geometry_shader: Option<&str>) -> Result<Program, ProgramCreationError>
    {
        let mut shaders_store = Vec::new();
        shaders_store.push(try!(build_shader(display, gl::VERTEX_SHADER, vertex_shader)));
        match geometry_shader {
            Some(gs) => shaders_store.push(try!(build_shader(display, gl::GEOMETRY_SHADER, gs))),
            None => ()
        }
        shaders_store.push(try!(build_shader(display, gl::FRAGMENT_SHADER, fragment_shader)));

        let mut shaders_ids = Vec::new();
        for sh in shaders_store.iter() {
            shaders_ids.push(sh.id);
        }

        let (tx, rx) = channel();
        display.context.context.exec(move |: ctxt| {
            unsafe {
                let id = ctxt.gl.CreateProgram();
                if id == 0 {
                    panic!("glCreateProgram failed");
                }

                // attaching shaders
                for sh in shaders_ids.iter() {
                    ctxt.gl.AttachShader(id, sh.clone());
                }

                // linking
                {
                    let _lock = COMPILER_GLOBAL_LOCK.lock();
                    ctxt.gl.LinkProgram(id);
                }

                // checking for errors
                {   let mut link_success: gl::types::GLint = mem::uninitialized();
                    ctxt.gl.GetProgramiv(id, gl::LINK_STATUS, &mut link_success);
                    if link_success == 0 {
                        use ProgramCreationError::LinkingError;

                        match ctxt.gl.GetError() {
                            gl::NO_ERROR => (),
                            gl::INVALID_VALUE => {
                                tx.send(Err(LinkingError(format!("glLinkProgram triggered \
                                                                  GL_INVALID_VALUE"))));
                                return;
                            },
                            gl::INVALID_OPERATION => {
                                tx.send(Err(LinkingError(format!("glLinkProgram triggered \
                                                                  GL_INVALID_OPERATION"))));
                                return;
                            },
                            _ => {
                                tx.send(Err(LinkingError(format!("glLinkProgram triggered an \
                                                                  unknown error"))));
                                return;
                            }
                        };

                        let mut error_log_size: gl::types::GLint = mem::uninitialized();
                        ctxt.gl.GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut error_log_size);

                        let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as uint);
                        ctxt.gl.GetProgramInfoLog(id, error_log_size, &mut error_log_size,
                            error_log.as_mut_slice().as_mut_ptr() as *mut gl::types::GLchar);
                        error_log.set_len(error_log_size as uint);

                        let msg = String::from_utf8(error_log).unwrap();
                        tx.send(Err(LinkingError(msg)));
                        return;
                    }
                }

                tx.send(Ok(id));
            }
        });

        let id = try!(rx.recv());

        let (tx, rx) = channel();
        display.context.context.exec(move |: mut ctxt| {
            unsafe {
                tx.send((
                    reflect_uniforms(&mut ctxt, id),
                    reflect_attributes(&mut ctxt, id)
                ))
            }
        });

        let (uniforms, attributes) = rx.recv();

        Ok(Program {
            display: display.context.clone(),
            shaders: shaders_store,
            id: id,
            uniforms: Arc::new(uniforms),
            attributes: Arc::new(attributes),
            frag_data_locations: Mutex::new(HashMap::new()),
        })
    }

    /// Returns the *location* of an output fragment, if it exists.
    ///
    /// The *location* is a low-level information that is used internally by glium.
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
        let name_c = name.to_c_str();
        let (tx, rx) = channel();
        self.display.context.exec(move |: ctxt| {
            unsafe {
                let value = ctxt.gl.GetFragDataLocation(id, name_c.as_ptr());
                tx.send(value);
            }
        });

        let location = match rx.recv() {
            -1 => None,
            a => Some(a as u32),
        };

        self.frag_data_locations.lock().unwrap().insert(name.to_string(), location);
        location
    }
}

impl fmt::Show for Program {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        (format!("Program #{}", self.id)).fmt(formatter)
    }
}

impl GlObject for Program {
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

// TODO: remove this hack
pub fn get_uniforms_locations(program: &Program) -> Arc<HashMap<String, Uniform>>
{
    program.uniforms.clone()
}

// TODO: remove this hack
pub fn get_attributes(program: &Program) -> Arc<HashMap<String, Attribute>>
{
    program.attributes.clone()
}

impl Drop for Program {
    fn drop(&mut self) {
        // removing VAOs which contain this program
        {
            let mut vaos = self.display.vertex_array_objects.lock().unwrap();
            let to_delete = vaos.keys().filter(|&&(_, _, p)| p == self.id)
                .map(|k| k.clone()).collect::<Vec<_>>();
            for k in to_delete.into_iter() {
                vaos.remove(&k);
            }
        }

        // sending the destroy command
        let id = self.id.clone();
        self.display.context.exec(move |: ctxt| {
            unsafe {
                if ctxt.state.program == id {
                    ctxt.gl.UseProgram(0);
                    ctxt.state.program = 0;
                }

                ctxt.gl.DeleteProgram(id);
            }
        });
    }
}

/// Builds an individual shader.
fn build_shader<S: ToCStr>(display: &Display, shader_type: gl::types::GLenum, source_code: S)
    -> Result<Shader, ProgramCreationError>
{
    let source_code = source_code.to_c_str();

    let (tx, rx) = channel();
    display.context.context.exec(move |: ctxt| {
        unsafe {
            if shader_type == gl::GEOMETRY_SHADER && ctxt.opengl_es {
                tx.send(Err(ProgramCreationError::ShaderTypeNotSupported));
                return;
            }

            let id = ctxt.gl.CreateShader(shader_type);

            if id == 0 {
                tx.send(Err(ProgramCreationError::ShaderTypeNotSupported));
                return;
            }

            ctxt.gl.ShaderSource(id, 1, [ source_code.as_ptr() ].as_ptr(), ptr::null());

            // compiling
            {
                let _lock = COMPILER_GLOBAL_LOCK.lock();
                ctxt.gl.CompileShader(id);
            }

            // checking compilation success
            let compilation_success = {
                let mut compilation_success: gl::types::GLint = mem::uninitialized();
                ctxt.gl.GetShaderiv(id, gl::COMPILE_STATUS, &mut compilation_success);
                compilation_success
            };

            if compilation_success == 0 {
                // compilation error
                let mut error_log_size: gl::types::GLint = mem::uninitialized();
                ctxt.gl.GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut error_log_size);

                let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as uint);
                ctxt.gl.GetShaderInfoLog(id, error_log_size, &mut error_log_size,
                    error_log.as_mut_slice().as_mut_ptr() as *mut gl::types::GLchar);
                error_log.set_len(error_log_size as uint);

                let msg = String::from_utf8(error_log).unwrap();
                tx.send(Err(ProgramCreationError::CompilationError(msg)));
                return;
            }

            tx.send(Ok(id));
        }
    });

    rx.recv().map(|id| {
        Shader {
            display: display.context.clone(),
            id: id
        }
    })
}

unsafe fn reflect_uniforms(ctxt: &mut CommandContext, program: gl::types::GLuint)
    -> HashMap<String, Uniform>
{
    // reflecting program uniforms
    let mut uniforms = HashMap::new();

    let mut active_uniforms: gl::types::GLint = mem::uninitialized();
    ctxt.gl.GetProgramiv(program, gl::ACTIVE_UNIFORMS, &mut active_uniforms);

    for uniform_id in range(0, active_uniforms) {
        let mut uniform_name_tmp: Vec<u8> = Vec::with_capacity(64);
        let mut uniform_name_tmp_len = 63;

        let mut data_type: gl::types::GLenum = mem::uninitialized();
        let mut data_size: gl::types::GLint = mem::uninitialized();
        ctxt.gl.GetActiveUniform(program, uniform_id as gl::types::GLuint, uniform_name_tmp_len,
            &mut uniform_name_tmp_len, &mut data_size, &mut data_type,
            uniform_name_tmp.as_mut_slice().as_mut_ptr() as *mut gl::types::GLchar);
        uniform_name_tmp.set_len(uniform_name_tmp_len as uint);

        let uniform_name = String::from_utf8(uniform_name_tmp).unwrap();
        let location = ctxt.gl.GetUniformLocation(program, uniform_name.to_c_str().into_inner());

        uniforms.insert(uniform_name, Uniform {
            location: location, 
            ty: data_type, 
            size: data_size
        });
    }

    uniforms
}

unsafe fn reflect_attributes(ctxt: &mut CommandContext, program: gl::types::GLuint)
    -> HashMap<String, Attribute>
{
    let mut attributes = HashMap::new();

    let mut active_attributes: gl::types::GLint = mem::uninitialized();
    ctxt.gl.GetProgramiv(program, gl::ACTIVE_ATTRIBUTES, &mut active_attributes);

    for attribute_id in range(0, active_attributes) {
        let mut attr_name_tmp: Vec<u8> = Vec::with_capacity(64);
        let mut attr_name_tmp_len = 63;

        let mut data_type: gl::types::GLenum = mem::uninitialized();
        let mut data_size: gl::types::GLint = mem::uninitialized();
        ctxt.gl.GetActiveAttrib(program, attribute_id as gl::types::GLuint, attr_name_tmp_len,
            &mut attr_name_tmp_len, &mut data_size, &mut data_type,
            attr_name_tmp.as_mut_slice().as_mut_ptr() as *mut gl::types::GLchar);
        attr_name_tmp.set_len(attr_name_tmp_len as uint);

        let attr_name = String::from_utf8(attr_name_tmp).unwrap();
        let location = ctxt.gl.GetAttribLocation(program, attr_name.to_c_str().into_inner());

        attributes.insert(attr_name, Attribute {
            location: location, 
            ty: data_type, 
            size: data_size
        });
    }

    attributes
}
