use gl;

use context::CommandContext;
use backend::Facade;

use std::fmt;
use std::collections::hash_map::{self, HashMap};
use std::os::raw;
use std::hash::BuildHasherDefault;

use fnv::FnvHasher;

use CapabilitiesSource;
use GlObject;
use ProgramExt;
use Handle;
use RawUniformValue;

use program::{COMPILER_GLOBAL_LOCK, ProgramCreationError, Binary, GetBinaryError};

use program::reflection::{Uniform, UniformBlock};
use program::reflection::{ShaderStage, SubroutineData};
use program::shader::{build_shader, check_shader_type_compatibility};

use program::raw::RawProgram;

use buffer::BufferSlice;
use uniforms::Uniforms;

/// A combination of compute shaders linked together.
pub struct ComputeShader {
    raw: RawProgram,
}

impl ComputeShader {
    /// Returns true if the backend supports compute shaders.
    #[inline]
    pub fn is_supported<C: ?Sized>(ctxt: &C) -> bool where C: CapabilitiesSource {
        check_shader_type_compatibility(ctxt, gl::COMPUTE_SHADER)
    }

    /// Builds a new compute shader from some source code.
    #[inline]
    pub fn from_source<F: ?Sized>(facade: &F, src: &str) -> Result<ComputeShader, ProgramCreationError>
                          where F: Facade
    {
        let _lock = COMPILER_GLOBAL_LOCK.lock();

        let shader = build_shader(facade, gl::COMPUTE_SHADER, src)?;

        Ok(ComputeShader {
            raw: RawProgram::from_shaders(facade, &[shader], false, false, false, None)?
        })
    }

    /// Builds a new compute shader from some binary.
    #[inline]
    pub fn from_binary<F: ?Sized>(facade: &F, data: Binary) -> Result<ComputeShader, ProgramCreationError>
                          where F: Facade
    {
        let _lock = COMPILER_GLOBAL_LOCK.lock();

        Ok(ComputeShader {
            raw: RawProgram::from_binary(facade, data)?
        })
    }

    /// Executes the compute shader.
    ///
    /// `x * y * z` work groups will be started. The current work group can be retrieved with
    /// `gl_WorkGroupID`. Inside each work group, additional local work groups can be started
    /// depending on the attributes of the compute shader itself.
    #[inline]
    pub fn execute<U>(&self, uniforms: U, x: u32, y: u32, z: u32) where U: Uniforms {
        unsafe { self.raw.dispatch_compute(uniforms, x, y, z) }.unwrap();       // FIXME: return error
    }

    /// Executes the compute shader.
    ///
    /// This is similar to `execute`, except that the parameters are stored in a buffer.
    #[inline]
    pub fn execute_indirect<U>(&self, uniforms: U, buffer: BufferSlice<ComputeCommand>)
                               where U: Uniforms
    {
        unsafe { self.raw.dispatch_compute_indirect(uniforms, buffer) }.unwrap();       // FIXME: return error
    }

    /// Returns the program's compiled binary.
    ///
    /// You can store the result in a file, then reload it later. This avoids having to compile
    /// the source code every time.
    #[inline]
    pub fn get_binary(&self) -> Result<Binary, GetBinaryError> {
        self.raw.get_binary()
    }

    /// Returns informations about a uniform variable, if it exists.
    #[inline]
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
    #[inline]
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
    #[inline]
    pub fn get_uniform_blocks(&self)
                              -> &HashMap<String, UniformBlock, BuildHasherDefault<FnvHasher>> {
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
    #[inline]
    pub fn get_shader_storage_blocks(&self)
            -> &HashMap<String, UniformBlock, BuildHasherDefault<FnvHasher>> {
        self.raw.get_shader_storage_blocks()
    }
}

impl fmt::Debug for ComputeShader {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(formatter, "{:?}", self.raw)
    }
}

impl GlObject for ComputeShader {
    type Id = Handle;

    #[inline]
    fn get_id(&self) -> Handle {
        self.raw.get_id()
    }
}

impl ProgramExt for ComputeShader {
    #[inline]
    fn use_program(&self, ctxt: &mut CommandContext) {
        self.raw.use_program(ctxt)
    }

    #[inline]
    fn set_uniform(&self, ctxt: &mut CommandContext, uniform_location: gl::types::GLint,
                   value: &RawUniformValue)
    {
        self.raw.set_uniform(ctxt, uniform_location, value)
    }

    #[inline]
    fn set_uniform_block_binding(&self, ctxt: &mut CommandContext, block_location: gl::types::GLuint,
                                 value: gl::types::GLuint)
    {
        self.raw.set_uniform_block_binding(ctxt, block_location, value)
    }

    #[inline]
    fn set_shader_storage_block_binding(&self, ctxt: &mut CommandContext,
                                        block_location: gl::types::GLuint,
                                        value: gl::types::GLuint)
    {
        self.raw.set_shader_storage_block_binding(ctxt, block_location, value)
    }

    #[inline]
    fn set_subroutine_uniforms_for_stage(&self, ctxt: &mut CommandContext,
                                         stage: ShaderStage,
                                         indices: &[gl::types::GLuint])
    {
        self.raw.set_subroutine_uniforms_for_stage(ctxt, stage, indices);
    }

    #[inline]
    fn get_uniform(&self, name: &str) -> Option<&Uniform> {
        self.raw.get_uniform(name)
    }

    #[inline]
    fn get_uniform_blocks(&self) -> &HashMap<String, UniformBlock, BuildHasherDefault<FnvHasher>> {
        self.raw.get_uniform_blocks()
    }

    #[inline]
    fn get_shader_storage_blocks(&self)
                                 -> &HashMap<String, UniformBlock, BuildHasherDefault<FnvHasher>> {
        self.raw.get_shader_storage_blocks()
    }

    #[inline]
    fn get_subroutine_data(&self) -> &SubroutineData {
        self.raw.get_subroutine_data()
    }
}

/// Represents a compute shader command waiting to be dispatched.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ComputeCommand {
    /// Number of X groups.
    pub num_groups_x: raw::c_uint,
    /// Number of Y groups.
    pub num_groups_y: raw::c_uint,
    /// Number of Z groups.
    pub num_groups_z: raw::c_uint,
}

implement_uniform_block!(ComputeCommand, num_groups_x, num_groups_y, num_groups_z);
