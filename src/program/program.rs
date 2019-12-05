use gl;

use context::CommandContext;
use version::Version;
use version::Api;

use backend::Facade;
use CapabilitiesSource;

use std::fmt;
use std::collections::hash_map::{self, HashMap};
use std::hash::BuildHasherDefault;

use fnv::FnvHasher;

use GlObject;
use ProgramExt;
use Handle;
use RawUniformValue;

use program::{COMPILER_GLOBAL_LOCK, ProgramCreationInput, ProgramCreationError, Binary};
use program::GetBinaryError;

use program::reflection::{Uniform, UniformBlock, OutputPrimitives};
use program::reflection::{Attribute, TransformFeedbackBuffer};
use program::reflection::{SubroutineData, ShaderStage, SubroutineUniform};
use program::shader::build_shader;

use program::raw::RawProgram;

use vertex::VertexFormat;

/// A combination of shaders linked together.
pub struct Program {
    raw: RawProgram,
    outputs_srgb: bool,
    uses_point_size: bool,
}

impl Program {
    /// Builds a new program.
    pub fn new<'a, F: ?Sized, I>(facade: &F, input: I) -> Result<Program, ProgramCreationError>
                         where I: Into<ProgramCreationInput<'a>>, F: Facade
    {
        let input = input.into();

        let (raw, outputs_srgb, uses_point_size) = match input {
            ProgramCreationInput::SourceCode { vertex_shader, tessellation_control_shader,
                                               tessellation_evaluation_shader, geometry_shader,
                                               fragment_shader, transform_feedback_varyings,
                                               outputs_srgb, uses_point_size } =>
            {
                let mut has_geometry_shader = false;
                let mut has_tessellation_control_shader = false;
                let mut has_tessellation_evaluation_shader = false;

                let mut shaders = vec![
                    (vertex_shader, gl::VERTEX_SHADER),
                    (fragment_shader, gl::FRAGMENT_SHADER)
                ];

                if let Some(gs) = geometry_shader {
                    shaders.push((gs, gl::GEOMETRY_SHADER));
                    has_geometry_shader = true;
                }

                if let Some(ts) = tessellation_control_shader {
                    shaders.push((ts, gl::TESS_CONTROL_SHADER));
                    has_tessellation_control_shader = true;
                }

                if let Some(ts) = tessellation_evaluation_shader {
                    shaders.push((ts, gl::TESS_EVALUATION_SHADER));
                    has_tessellation_evaluation_shader = true;
                }

                // TODO: move somewhere else
                if transform_feedback_varyings.is_some() &&
                    !(facade.get_context().get_version() >= &Version(Api::Gl, 3, 0)) &&
                    !facade.get_context().get_extensions().gl_ext_transform_feedback
                {
                    return Err(ProgramCreationError::TransformFeedbackNotSupported);
                }

                if uses_point_size && !(facade.get_context().get_version() >= &Version(Api::Gl, 3, 0)) {
                    return Err(ProgramCreationError::PointSizeNotSupported);
                }

                let _lock = COMPILER_GLOBAL_LOCK.lock();

                let shaders_store = {
                    let mut shaders_store = Vec::new();
                    for (src, ty) in shaders.into_iter() {
                        shaders_store.push(build_shader(facade, ty, src)?);
                    }
                    shaders_store
                };

                (RawProgram::from_shaders(facade, &shaders_store, has_geometry_shader,
                                               has_tessellation_control_shader, has_tessellation_evaluation_shader,
                                               transform_feedback_varyings)?,
                 outputs_srgb, uses_point_size)
            },

            ProgramCreationInput::Binary { data, outputs_srgb, uses_point_size } => {
                if uses_point_size && !(facade.get_context().get_version() >= &Version(Api::Gl, 3, 0)) {
                    return Err(ProgramCreationError::PointSizeNotSupported);
                }

                (RawProgram::from_binary(facade, data)?, outputs_srgb, uses_point_size)
            },
        };
        Ok(Program {
            raw: raw,
            outputs_srgb: outputs_srgb,
            uses_point_size: uses_point_size,
        })
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
    #[inline]
    pub fn from_source<'a, F: ?Sized>(facade: &F, vertex_shader: &'a str, fragment_shader: &'a str,
                              geometry_shader: Option<&'a str>)
                              -> Result<Program, ProgramCreationError> where F: Facade
    {
        Program::new(facade, ProgramCreationInput::SourceCode {
            vertex_shader: vertex_shader,
            fragment_shader: fragment_shader,
            geometry_shader: geometry_shader,
            tessellation_control_shader: None,
            tessellation_evaluation_shader: None,
            transform_feedback_varyings: None,
            outputs_srgb: false,
            uses_point_size: false,
        })
    }

    /// Returns the program's compiled binary.
    ///
    /// You can store the result in a file, then reload it later. This avoids having to compile
    /// the source code every time.
    #[inline]
    pub fn get_binary(&self) -> Result<Binary, GetBinaryError> {
        self.raw.get_binary()
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
    #[inline]
    pub fn get_frag_data_location(&self, name: &str) -> Option<u32> {
        self.raw.get_frag_data_location(name)
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

    /// Returns the list of transform feedback varyings.
    #[inline]
    pub fn get_transform_feedback_buffers(&self) -> &[TransformFeedbackBuffer] {
        self.raw.get_transform_feedback_buffers()
    }

    /// True if the transform feedback output of this program matches the specified `VertexFormat`
    /// and `stride`.
    ///
    /// The `stride` is the number of bytes between two vertices.
    #[inline]
    pub fn transform_feedback_matches(&self, format: &VertexFormat, stride: usize) -> bool {
        self.raw.transform_feedback_matches(format, stride)
    }

    /// Returns the type of geometry that transform feedback would generate, or `None` if it
    /// depends on the vertex/index data passed when drawing.
    ///
    /// This corresponds to `GL_GEOMETRY_OUTPUT_TYPE` or `GL_TESS_GEN_MODE`. If the program doesn't
    /// contain either a geometry shader or a tessellation evaluation shader, returns `None`.
    #[inline]
    pub fn get_output_primitives(&self) -> Option<OutputPrimitives> {
        self.raw.get_output_primitives()
    }

    /// Returns true if the program contains a tessellation stage.
    #[inline]
    pub fn has_tessellation_shaders(&self) -> bool {
        self.raw.has_tessellation_shaders()
    }

    /// Returns true if the program contains a tessellation control stage.
    #[inline]
    pub fn has_tessellation_control_shader(&self) -> bool {
        self.raw.has_tessellation_control_shader()
    }

    /// Returns true if the program contains a tessellation evaluation stage.
    #[inline]
    pub fn has_tessellation_evaluation_shader(&self) -> bool {
        self.raw.has_tessellation_evaluation_shader()
    }

    /// Returns true if the program contains a geometry shader.
    #[inline]
    pub fn has_geometry_shader(&self) -> bool {
        self.raw.has_geometry_shader()
    }

    /// Returns informations about an attribute, if it exists.
    #[inline]
    pub fn get_attribute(&self, name: &str) -> Option<&Attribute> {
        self.raw.get_attribute(name)
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
        self.raw.attributes()
    }

    /// Returns true if the program has been configured to output sRGB instead of RGB.
    #[inline]
    pub fn has_srgb_output(&self) -> bool {
        self.outputs_srgb
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

    /// Returns the subroutine uniforms of this program.
    ///
    /// Since subroutine uniforms are unique per shader and *not* per program,
    /// the keys of the `HashMap` are in the format `("subroutine_name", ShaderStage)`.
    /// ## Example
    ///
    /// ```no_run
    /// # let program: glium::Program = unsafe { std::mem::uninitialized() };
    /// for (&(ref name, shader), uniform) in program.get_subroutine_uniforms() {
    ///     println!("Name: {}", name);
    /// }
    /// ```
    #[inline]
    pub fn get_subroutine_uniforms(&self)
            -> &HashMap<(String, ShaderStage), SubroutineUniform, BuildHasherDefault<FnvHasher>> {
        &self.raw.get_subroutine_data().subroutine_uniforms
    }

    /// Returns true if the program has been configured to use the `gl_PointSize` variable.
    ///
    /// If the program uses `gl_PointSize` without having been configured appropriately, then
    /// setting the value of `gl_PointSize` will have no effect.
    #[inline]
    pub fn uses_point_size(&self) -> bool {
      self.uses_point_size
    }
}

impl fmt::Debug for Program {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(formatter, "{:?}", self.raw)
    }
}

impl GlObject for Program {
    type Id = Handle;

    #[inline]
    fn get_id(&self) -> Handle {
        self.raw.get_id()
    }
}

impl ProgramExt for Program {
    fn use_program(&self, ctxt: &mut CommandContext) {
        // compatibility was checked at program creation
        if self.uses_point_size && !ctxt.state.enabled_program_point_size {
            unsafe { ctxt.gl.Enable(gl::PROGRAM_POINT_SIZE); }
        } else if !self.uses_point_size && ctxt.state.enabled_program_point_size {
            unsafe { ctxt.gl.Disable(gl::PROGRAM_POINT_SIZE); }
        }

        if ctxt.version >= &Version(Api::Gl, 3, 0) || ctxt.extensions.gl_arb_framebuffer_srgb ||
           ctxt.extensions.gl_ext_framebuffer_srgb || ctxt.extensions.gl_ext_srgb_write_control
        {
            if ctxt.state.enabled_framebuffer_srgb == self.outputs_srgb {
                ctxt.state.enabled_framebuffer_srgb = !self.outputs_srgb;

                if self.outputs_srgb {
                    unsafe { ctxt.gl.Disable(gl::FRAMEBUFFER_SRGB) };
                } else {
                    unsafe { ctxt.gl.Enable(gl::FRAMEBUFFER_SRGB) };
                }
            }
        }

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
