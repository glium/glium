use data_types;
use gl;
use libc;
use std::{fmt, mem, ptr};
use std::collections::HashMap;
use std::sync::Arc;
use texture;
use {Display, DisplayImpl, Texture};

struct ShaderImpl {
    display: Arc<DisplayImpl>,
    id: gl::types::GLuint,
}

impl Drop for ShaderImpl {
    fn drop(&mut self) {
        let id = self.id.clone();
        self.display.context.exec(proc(gl, _state) {
            gl.DeleteShader(id);
        });
    }
}

/// A combinaison of shaders linked together.
pub struct Program {
    program: Arc<ProgramImpl>
}

impl Program {
    /// Builds a new program.
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
    /// # let display: glium_core::Display = unsafe { std::mem::uninitialized() };
    /// # let vertex_source = ""; let fragment_source = ""; let geometry_source = "";
    /// let program = glium_core::Program::new(&display, vertex_source, fragment_source, Some(geometry_source));
    /// ```
    /// 
    #[experimental = "The list of shaders and the result error will probably change"]
    pub fn new(display: &Display, vertex_shader: &str, fragment_shader: &str,
               geometry_shader: Option<&str>) -> Result<Program, String>
    {
        let mut shaders_store = Vec::new();
        shaders_store.push(try!(build_shader(display, gl::VERTEX_SHADER, vertex_shader)));
        match geometry_shader {
            Some(gs) => shaders_store.push(try!(build_geometry_shader(display, gs))),
            None => ()
        }
        shaders_store.push(try!(build_shader(display, gl::FRAGMENT_SHADER, fragment_shader)));

        let mut shaders_ids = Vec::new();
        for sh in shaders_store.iter() {
            shaders_ids.push(sh.id);
        }

        let (tx, rx) = channel();
        display.context.context.exec(proc(gl, _state) {
            unsafe {
                let id = gl.CreateProgram();
                if id == 0 {
                    tx.send(Err(format!("glCreateProgram failed")));
                    return;
                }

                // attaching shaders
                for sh in shaders_ids.iter() {
                    gl.AttachShader(id, sh.clone());
                }

                // linking and checking for errors
                gl.LinkProgram(id);
                {   let mut link_success: gl::types::GLint = mem::uninitialized();
                    gl.GetProgramiv(id, gl::LINK_STATUS, &mut link_success);
                    if link_success == 0 {
                        match gl.GetError() {
                            gl::NO_ERROR => (),
                            gl::INVALID_VALUE => {
                                tx.send(Err(format!("glLinkProgram triggered GL_INVALID_VALUE")));
                                return;
                            },
                            gl::INVALID_OPERATION => {
                                tx.send(Err(format!("glLinkProgram triggered GL_INVALID_OPERATION")));
                                return;
                            },
                            _ => {
                                tx.send(Err(format!("glLinkProgram triggered an unknown error")));
                                return;
                            }
                        };

                        let mut error_log_size: gl::types::GLint = mem::uninitialized();
                        gl.GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut error_log_size);

                        let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as uint);
                        gl.GetProgramInfoLog(id, error_log_size, &mut error_log_size, error_log.as_mut_slice().as_mut_ptr() as *mut gl::types::GLchar);
                        error_log.set_len(error_log_size as uint);

                        let msg = String::from_utf8(error_log).unwrap();
                        tx.send(Err(msg));
                        return;
                    }
                }

                tx.send(Ok(id));
            }
        });

        let id = try!(rx.recv());

        let (tx, rx) = channel();
        display.context.context.exec(proc(gl, _state) {
            unsafe {
                // reflecting program uniforms
                let mut uniforms = HashMap::new();

                let mut active_uniforms: gl::types::GLint = mem::uninitialized();
                gl.GetProgramiv(id, gl::ACTIVE_UNIFORMS, &mut active_uniforms);

                for uniform_id in range(0, active_uniforms) {
                    let mut uniform_name_tmp: Vec<u8> = Vec::with_capacity(64);
                    let mut uniform_name_tmp_len = 63;

                    let mut data_type: gl::types::GLenum = mem::uninitialized();
                    let mut data_size: gl::types::GLint = mem::uninitialized();
                    gl.GetActiveUniform(id, uniform_id as gl::types::GLuint, uniform_name_tmp_len, &mut uniform_name_tmp_len, &mut data_size, &mut data_type, uniform_name_tmp.as_mut_slice().as_mut_ptr() as *mut gl::types::GLchar);
                    uniform_name_tmp.set_len(uniform_name_tmp_len as uint);

                    let uniform_name = String::from_utf8(uniform_name_tmp).unwrap();
                    let location = gl.GetUniformLocation(id, uniform_name.to_c_str().unwrap());

                    uniforms.insert(uniform_name, (location, data_type, data_size));
                }

                tx.send(Arc::new(uniforms));
            }
        });

        Ok(Program {
            program: Arc::new(ProgramImpl {
                display: display.context.clone(),
                shaders: shaders_store,
                id: id,
                uniforms: rx.recv(),
            })
        })
    }

    /// Creates a new `ProgramUniforms` object.
    ///
    /// A `ProgramUniforms` object is a link between a program and its uniforms values.
    pub fn build_uniforms(&self) -> ProgramUniforms {
        ProgramUniforms {
            display: self.program.display.clone(),
            program: self.program.clone(),
            textures: HashMap::new(),
            values: HashMap::new(),
            uniforms: self.program.uniforms.clone()
        }
    }
}

impl fmt::Show for Program {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::FormatError> {
        (format!("Program #{}", self.program.id)).fmt(formatter)
    }
}

struct ProgramImpl {
    display: Arc<DisplayImpl>,
    #[allow(dead_code)]
    shaders: Vec<Arc<ShaderImpl>>,
    id: gl::types::GLuint,
    uniforms: Arc<HashMap<String, (gl::types::GLint, gl::types::GLenum, gl::types::GLint)>>     // location, type and size of each uniform, ordered by name
}

impl Drop for ProgramImpl {
    fn drop(&mut self) {
        let id = self.id.clone();
        self.display.context.exec(proc(gl, state) {
            if state.program == id {
                gl.UseProgram(0);
                state.program = 0;
            }

            gl.DeleteProgram(id);
        });
    }
}

/// A program which stores values of uniforms.
#[deriving(Clone)]
pub struct ProgramUniforms {
    display: Arc<DisplayImpl>,
    program: Arc<ProgramImpl>,
    textures: HashMap<gl::types::GLint, Arc<texture::TextureImpl>>,
    values: HashMap<gl::types::GLint, (gl::types::GLenum, Vec<char>)>,
    uniforms: Arc<HashMap<String, (gl::types::GLint, gl::types::GLenum, gl::types::GLint)>>     // same as the program's variable
}

/// Function accessible from within glium_core but not the outside.
pub fn get_program_id(uniforms: &ProgramUniforms) -> gl::types::GLuint {
    uniforms.program.id
}

/// Function accessible from within glium_core but not the outside.
pub fn unwrap_uniforms(uniforms: &ProgramUniforms) -> (&HashMap<gl::types::GLint, Arc<texture::TextureImpl>>, &HashMap<gl::types::GLint, (gl::types::GLenum, Vec<char>)>) {
    (&uniforms.textures, &uniforms.values)
}

impl ProgramUniforms {
    /// Modifies the value of a uniform of the program.
    ///
    /// `uniform_name` must be the name of a uniform in the program.
    /// Nothing happens if the program doesn't contain a uniform with this name.
    /// However the function will fail if the type of data doesn't match the type required
    ///  by the shader source code.
    pub fn set_value<T: data_types::UniformValue>(&mut self, uniform_name: &str, value: T) {
        let &(location, gltype, _) = match self.uniforms.find(&uniform_name.to_string()) {
            Some(a) => a,
            None => return      // the uniform is not used, we ignore it
        };

        if gltype != data_types::UniformValue::get_gl_type(None::<T>) {
            fail!("Type of data passed to set_value must match the type of data requested by the shader")
        }

        let mut data: Vec<char> = Vec::with_capacity(mem::size_of_val(&value));
        unsafe { data.set_len(mem::size_of_val(&value)); }

        let data_inside = data.as_mut_ptr() as *mut T;
        unsafe { (*data_inside) = value; }

        self.values.insert(location.clone(), (gltype, data));
    }

    /// Modifies the value of a texture uniform of the program.
    ///
    /// `uniform_name` must be the name of a uniform in the program.
    /// Nothing happens if the program doesn't contain a uniform with this name.
    /// However the function will fail if you call this function for a non-texture uniform.
    pub fn set_texture(&mut self, uniform_name: &str, texture: &Texture) {
        let &(location, gltype, _) = match self.uniforms.find(&uniform_name.to_string()) {
            Some(a) => a,
            None => return      // the uniform is not used, we ignore it
        };

        // TODO: fix the check for GLES
        /*match gltype {
            gl::SAMPLER_1D | gl::SAMPLER_2D | gl::SAMPLER_3D | gl::SAMPLER_CUBE |
            gl::SAMPLER_1D_SHADOW | gl::SAMPLER_2D_SHADOW | gl::SAMPLER_1D_ARRAY |
            gl::SAMPLER_2D_ARRAY | gl::SAMPLER_1D_ARRAY_SHADOW | gl::SAMPLER_2D_ARRAY_SHADOW |
            gl::SAMPLER_2D_MULTISAMPLE | gl::SAMPLER_2D_MULTISAMPLE_ARRAY |
            gl::SAMPLER_CUBE_SHADOW | gl::SAMPLER_BUFFER | gl::SAMPLER_2D_RECT |
            gl::SAMPLER_2D_RECT_SHADOW | gl::INT_SAMPLER_1D | gl::INT_SAMPLER_2D |
            gl::INT_SAMPLER_3D | gl::INT_SAMPLER_CUBE | gl::INT_SAMPLER_1D_ARRAY |
            gl::INT_SAMPLER_2D_ARRAY | gl::INT_SAMPLER_2D_MULTISAMPLE |
            gl::INT_SAMPLER_2D_MULTISAMPLE_ARRAY | gl::INT_SAMPLER_BUFFER |
            gl::INT_SAMPLER_2D_RECT | gl::UNSIGNED_INT_SAMPLER_1D | gl::UNSIGNED_INT_SAMPLER_2D |
            gl::UNSIGNED_INT_SAMPLER_3D | gl::UNSIGNED_INT_SAMPLER_CUBE |
            gl::UNSIGNED_INT_SAMPLER_1D_ARRAY | gl::UNSIGNED_INT_SAMPLER_2D_ARRAY |
            gl::UNSIGNED_INT_SAMPLER_2D_MULTISAMPLE |
            gl::UNSIGNED_INT_SAMPLER_2D_MULTISAMPLE_ARRAY | gl::UNSIGNED_INT_SAMPLER_BUFFER |
            gl::UNSIGNED_INT_SAMPLER_2D_RECT
                => (),
            _ => fail!("Trying to bind a texture to a non-texture uniform")
        };*/

        self.textures.insert(location.clone(), texture::get_impl(texture).clone());
    }
}

/// Builds an individual shader.
fn build_shader<S: ToCStr>(display: &Display, shader_type: gl::types::GLenum, source_code: S)
    -> Result<Arc<ShaderImpl>, String>
{
    let source_code = source_code.to_c_str();

    let (tx, rx) = channel();
    display.context.context.exec(proc(gl, _state) {
        unsafe {
            let id = gl.CreateShader(shader_type);

            gl.ShaderSource(id, 1, [ source_code.as_ptr() ].as_ptr(), ptr::null());
            gl.CompileShader(id);

            // checking compilation success
            let compilation_success = {
                let mut compilation_success: gl::types::GLint = mem::uninitialized();
                gl.GetShaderiv(id, gl::COMPILE_STATUS, &mut compilation_success);
                compilation_success
            };

            if compilation_success == 0 {
                // compilation error
                let mut error_log_size: gl::types::GLint = mem::uninitialized();
                gl.GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut error_log_size);

                let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as uint);
                gl.GetShaderInfoLog(id, error_log_size, &mut error_log_size, error_log.as_mut_slice().as_mut_ptr() as *mut gl::types::GLchar);
                error_log.set_len(error_log_size as uint);

                let msg = String::from_utf8(error_log).unwrap();
                tx.send(Err(msg));
                return;
            }

            tx.send(Ok(id));
        }
    });

    rx.recv().map(|id| {
        Arc::new(ShaderImpl {
            display: display.context.clone(),
            id: id
        })
    })
}

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
fn build_geometry_shader<S: ToCStr>(display: &Display, source_code: S)
    -> Result<Arc<ShaderImpl>, String>
{
    build_shader(display, gl::GEOMETRY_SHADER, source_code)
}

#[cfg(target_os = "android")]
fn build_geometry_shader<S: ToCStr>(display: &Display, source_code: S)
    -> Result<Arc<ShaderImpl>, String>
{
    Err(format!("Geometry shaders are not supported on this platform"))
}
