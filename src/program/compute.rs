use gl;

use context::CommandContext;
use backend::Facade;

use std::fmt;
use std::error::Error;
use std::collections::hash_map::{self, HashMap};

use GlObject;
use ProgramExt;
use Handle;
use RawUniformValue;

use program::{COMPILER_GLOBAL_LOCK, ProgramCreationError, Binary};

use program::reflection::{Uniform, UniformBlock};
use program::shader::build_shader;

use program::raw::RawProgram;

use uniforms::Uniforms;

/// A combination of compute shaders linked together.
pub struct ComputeShader {
    raw: RawProgram,
}

impl ComputeShader {
    /// Builds a new compute shader from some source code.
    pub fn from_source<F>(facade: &F, src: &str) -> Result<ComputeShader, ProgramCreationError>
                          where F: Facade
    {
        let _lock = COMPILER_GLOBAL_LOCK.lock();

        let shader = try!(build_shader(facade, gl::COMPUTE_SHADER, src));

        Ok(ComputeShader {
            raw: try!(RawProgram::from_shaders(facade, &[shader], false, false, None))
        })
    }

    /// Builds a new compute shader from some binary.
    pub fn from_binary<F>(facade: &F, data: Binary) -> Result<ComputeShader, ProgramCreationError>
                          where F: Facade
    {
        let _lock = COMPILER_GLOBAL_LOCK.lock();

        Ok(ComputeShader {
            raw: try!(RawProgram::from_binary(facade, data))
        })
    }

    /// Executes the compute shader.
    ///
    /// `x * y * z` work groups will be started. The current work group can be retreived with
    /// `gl_WorkGroupID`. Inside each work group, additional local work groups can be started
    /// depending on the attributes of the compute shader itself.
    pub fn execute<U>(&self, uniforms: U, x: u32, y: u32, z: u32) where U: Uniforms {
        unsafe { self.raw.dispatch_compute(uniforms, x, y, z) }.unwrap();       // FIXME: return error
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
        self.raw.get_binary()
    }

    /// Returns the program's compiled binary.
    ///
    /// Same as `get_binary` but always available. Returns `None` if the backend doesn't support
    /// getting or reloading the program's binary.
    pub fn get_binary_if_supported(&self) -> Option<Binary> {
        self.raw.get_binary_if_supported()
    }

    /// Returns informations about a uniform variable, if it exists.
    pub fn get_uniform(&self, name: &str) -> Option<&Uniform> {
        self.raw.get_uniform(name)
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
        self.raw.uniforms()
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
        self.raw.get_uniform_blocks()
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
    pub fn get_shader_storage_blocks(&self) -> &HashMap<String, UniformBlock> {
        self.raw.get_shader_storage_blocks()
    }
}

impl fmt::Debug for ComputeShader {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(formatter, "{:?}", self.raw)
    }
}

impl GlObject for ComputeShader {
    type Id = Handle;

    fn get_id(&self) -> Handle {
        self.raw.get_id()
    }
}

impl ProgramExt for ComputeShader {
    fn use_program(&self, ctxt: &mut CommandContext) {
        self.raw.use_program(ctxt)
    }

    fn set_uniform(&self, ctxt: &mut CommandContext, uniform_location: gl::types::GLint,
                   value: &RawUniformValue)
    {
        self.raw.set_uniform(ctxt, uniform_location, value)
    }

    fn set_uniform_block_binding(&self, ctxt: &mut CommandContext, block_location: gl::types::GLuint,
                                 value: gl::types::GLuint)
    {
        self.raw.set_uniform_block_binding(ctxt, block_location, value)
    }

    fn set_shader_storage_block_binding(&self, ctxt: &mut CommandContext,
                                        block_location: gl::types::GLuint,
                                        value: gl::types::GLuint)
    {
        self.raw.set_shader_storage_block_binding(ctxt, block_location, value)
    }

    fn get_uniform(&self, name: &str) -> Option<&Uniform> {
        self.raw.get_uniform(name)
    }

    fn get_uniform_blocks(&self) -> &HashMap<String, UniformBlock> {
        self.raw.get_uniform_blocks()
    }

    fn get_shader_storage_blocks(&self) -> &HashMap<String, UniformBlock> {
        self.raw.get_shader_storage_blocks()
    }
}
