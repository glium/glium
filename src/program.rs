use gl;
use libc;
use std::{ffi, fmt, mem, ptr};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, StaticMutex, MUTEX_INIT};
use std::sync::mpsc::channel;
use {Display, DisplayImpl, GlObject};
use context::{self, CommandContext};
use uniforms::UniformType;

/// Some shader compilers have race-condition issues, so we lock this mutex
/// in the GL thread every time we compile a shader or link a program.
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

/// Input when creating a program.
pub enum ProgramCreationInput<'a> {
    /// Use GLSL source code.
    SourceCode {
        /// Source code of the vertex shader.
        vertex_shader: &'a str,

        /// Source code of the optional tessellation control shader.
        tessellation_control_shader: Option<&'a str>,

        /// Source code of the optional tessellation evaluation shader.
        tessellation_evaluation_shader: Option<&'a str>,

        /// Source code of the optional geometry shader.
        geometry_shader: Option<&'a str>,

        /// Source code of the fragment shader.
        fragment_shader: &'a str,
    },
}

impl<'a> IntoProgramCreationInput<'a> for ProgramCreationInput<'a> {
    fn into_program_creation_input(self) -> ProgramCreationInput<'a> {
        self
    }
}

/// Traits for objects that can be turned into `ProgramCreationInput`.
pub trait IntoProgramCreationInput<'a> {
    /// Builds the `ProgramCreationInput`.
    fn into_program_creation_input(self) -> ProgramCreationInput<'a>;
}

/// Represents the source code of a program.
pub struct SourceCode<'a> {
    /// Source code of the vertex shader.
    pub vertex_shader: &'a str,

    /// Source code of the optional tessellation control shader.
    pub tessellation_control_shader: Option<&'a str>,

    /// Source code of the optional tessellation evaluation shader.
    pub tessellation_evaluation_shader: Option<&'a str>,

    /// Source code of the optional geometry shader.
    pub geometry_shader: Option<&'a str>,

    /// Source code of the fragment shader.
    pub fragment_shader: &'a str,
}

impl<'a> IntoProgramCreationInput<'a> for SourceCode<'a> {
    fn into_program_creation_input(self) -> ProgramCreationInput<'a> {
        let SourceCode { vertex_shader, fragment_shader, geometry_shader,
                         tessellation_control_shader, tessellation_evaluation_shader } = self;

        ProgramCreationInput::SourceCode {
            vertex_shader: vertex_shader,
            tessellation_control_shader: tessellation_control_shader,
            tessellation_evaluation_shader: tessellation_evaluation_shader,
            geometry_shader: geometry_shader,
            fragment_shader: fragment_shader,
        }
    }
}

/// Represents the compiled binary data of a program.
pub struct Binary {
    /// An implementation-defined format.
    pub format: u32,

    /// The binary data.
    pub content: Vec<u8>,
}

/// A combination of shaders linked together.
pub struct Program {
    display: Arc<DisplayImpl>,
    #[allow(dead_code)]
    shaders: Vec<Shader>,
    id: gl::types::GLuint,
    uniforms: Arc<HashMap<String, Uniform>>,
    uniform_blocks: Arc<HashMap<String, UniformBlock>>,
    attributes: Arc<HashMap<String, Attribute>>,
    frag_data_locations: Mutex<HashMap<String, Option<u32>>>,
    has_tessellation_shaders: bool,
}

/// Information about a uniform (except its name).
#[derive(Show, Copy)]
#[doc(hidden)]
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
#[derive(Show, Clone)]
#[doc(hidden)]
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
#[derive(Show, Clone)]
#[doc(hidden)]
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
#[derive(Show, Copy)]
#[doc(hidden)]
pub struct Attribute {
    pub location: gl::types::GLint,
    pub ty: gl::types::GLenum,
    pub size: gl::types::GLint,
}

/// Error that can be triggered when creating a `Program`.
#[derive(Clone, Show)]
pub enum ProgramCreationError {
    /// Error while compiling one of the shaders.
    CompilationError(String),

    /// Error while linking the program.
    LinkingError(String),

    /// One of the requested shader types is not supported by the backend.
    ///
    /// Usually the case for geometry shaders.
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
    pub fn new<'a, I>(display: &Display, input: I) -> Result<Program, ProgramCreationError>
                      where I: IntoProgramCreationInput<'a>
    {
        let input = input.into_program_creation_input();
        Program::from_source_impl(display, input)
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
        let shaders: Vec<(&str, gl::types::GLenum)> = {
            let ProgramCreationInput::SourceCode { vertex_shader, fragment_shader,
                                                   geometry_shader, tessellation_control_shader,
                                                   tessellation_evaluation_shader } = input;

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

            shaders
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
                                                                  GL_INVALID_VALUE")))).ok();
                                return;
                            },
                            gl::INVALID_OPERATION => {
                                tx.send(Err(LinkingError(format!("glLinkProgram triggered \
                                                                  GL_INVALID_OPERATION")))).ok();
                                return;
                            },
                            _ => {
                                tx.send(Err(LinkingError(format!("glLinkProgram triggered an \
                                                                  unknown error")))).ok();
                                return;
                            }
                        };

                        let mut error_log_size: gl::types::GLint = mem::uninitialized();
                        ctxt.gl.GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut error_log_size);

                        let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as usize);
                        ctxt.gl.GetProgramInfoLog(id, error_log_size, &mut error_log_size,
                            error_log.as_mut_slice().as_mut_ptr() as *mut gl::types::GLchar);
                        error_log.set_len(error_log_size as usize);

                        let msg = String::from_utf8(error_log).unwrap();
                        tx.send(Err(LinkingError(msg))).ok();
                        return;
                    }
                }

                tx.send(Ok(id)).unwrap();
            }
        });

        let id = try!(rx.recv().unwrap());

        let (tx, rx) = channel();
        display.context.context.exec(move |: mut ctxt| {
            unsafe {
                tx.send((
                    reflect_uniforms(&mut ctxt, id),
                    reflect_attributes(&mut ctxt, id),
                    reflect_uniform_blocks(&mut ctxt, id),
                )).ok();
            }
        });

        let (uniforms, attributes, blocks) = rx.recv().unwrap();

        Ok(Program {
            display: display.context.clone(),
            shaders: shaders_store,
            id: id,
            uniforms: Arc::new(uniforms),
            uniform_blocks: Arc::new(blocks),
            attributes: Arc::new(attributes),
            frag_data_locations: Mutex::new(HashMap::new()),
            has_tessellation_shaders: has_tessellation_shaders,
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
        self.display.context.exec(move |: ctxt| {
            unsafe {
                if ctxt.version >= &context::GlVersion(4, 1) ||
                   ctxt.extensions.gl_arb_get_programy_binary
                {
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
        self.display.context.exec(move |: ctxt| {
            unsafe {
                let value = ctxt.gl.GetFragDataLocation(id, name_c.as_slice_with_nul().as_ptr());
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
    pub fn get_uniform_blocks(&self) -> &HashMap<String, UniformBlock> {
        &*self.uniform_blocks
    }

    /// Returns true if the program contains a tessellation stage.
    pub fn has_tessellation_shaders(&self) -> bool {
        self.has_tessellation_shaders
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
#[doc(hidden)]
pub fn get_uniforms_locations(program: &Program) -> Arc<HashMap<String, Uniform>>
{
    program.uniforms.clone()
}

// TODO: remove this hack
#[doc(hidden)]
pub fn get_attributes(program: &Program) -> Arc<HashMap<String, Attribute>>
{
    program.attributes.clone()
}

impl Drop for Program {
    fn drop(&mut self) {
        // removing VAOs which contain this program
        {
            let mut vaos = self.display.vertex_array_objects.lock().unwrap();
            let to_delete = vaos.keys().filter(|&&(_, p)| p == self.id)
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
fn build_shader(display: &Display, shader_type: gl::types::GLenum, source_code: &str)
                -> Result<Shader, ProgramCreationError>
{
    let source_code = ffi::CString::from_slice(source_code.as_bytes());

    let (tx, rx) = channel();
    display.context.context.exec(move |: ctxt| {
        unsafe {
            if shader_type == gl::GEOMETRY_SHADER && ctxt.opengl_es {
                tx.send(Err(ProgramCreationError::ShaderTypeNotSupported)).ok();
                return;
            }

            let id = ctxt.gl.CreateShader(shader_type);

            if id == 0 {
                tx.send(Err(ProgramCreationError::ShaderTypeNotSupported)).ok();
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

                let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as usize);
                ctxt.gl.GetShaderInfoLog(id, error_log_size, &mut error_log_size,
                    error_log.as_mut_slice().as_mut_ptr() as *mut gl::types::GLchar);
                error_log.set_len(error_log_size as usize);

                let msg = String::from_utf8(error_log).unwrap();
                tx.send(Err(ProgramCreationError::CompilationError(msg))).ok();
                return;
            }

            tx.send(Ok(id)).unwrap();
        }
    });

    rx.recv().unwrap().map(|id| {
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
        uniform_name_tmp.set_len(uniform_name_tmp_len as usize);

        let uniform_name = String::from_utf8(uniform_name_tmp).unwrap();
        let location = ctxt.gl.GetUniformLocation(program, ffi::CString::from_slice(uniform_name.as_bytes()).as_slice_with_nul().as_ptr());

        uniforms.insert(uniform_name, Uniform {
            location: location as i32,
            ty: glenum_to_uniform_type(data_type),
            size: if data_size == 1 { None } else { Some(data_size as usize) },
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
        attr_name_tmp.set_len(attr_name_tmp_len as usize);

        let attr_name = String::from_utf8(attr_name_tmp).unwrap();
        if attr_name.starts_with("gl_") {   // ignoring everything built-in
            continue;
        }

        let location = ctxt.gl.GetAttribLocation(program, ffi::CString::from_slice(attr_name.as_bytes()).as_slice_with_nul().as_ptr());

        attributes.insert(attr_name, Attribute {
            location: location,
            ty: data_type,
            size: data_size
        });
    }

    attributes
}

unsafe fn reflect_uniform_blocks(ctxt: &mut CommandContext, program: gl::types::GLuint)
                                 -> HashMap<String, UniformBlock>
{
    // uniform blocks are not supported, so there's none
    if ctxt.version < &context::GlVersion(3, 1) {
        return HashMap::new();
    }

    let mut blocks = HashMap::new();

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
