use gl;

use context::CommandContext;
use version::Version;
use version::Api;

use backend::Facade;
use ContextExt;

use std::fmt;
use std::error::Error;
use std::collections::hash_map::{self, HashMap};

use GlObject;
use ProgramExt;
use Handle;
use RawUniformValue;

use program::{COMPILER_GLOBAL_LOCK, ProgramCreationInput, ProgramCreationError, Binary};
use program::GetBinaryError;

use program::reflection::{Uniform, UniformBlock, OutputPrimitives};
use program::reflection::{Attribute, TransformFeedbackBuffer};
use program::shader::build_shader;

use program::raw::RawProgram;

use vertex::VertexFormat;

/// A combination of shaders linked together.
pub struct Program {
    raw: RawProgram,
    uses_point_size: bool,
}

impl Program {
    /// Builds a new program.
    pub fn new<'a, F, I>(facade: &F, input: I) -> Result<Program, ProgramCreationError>
                         where I: Into<ProgramCreationInput<'a>>, F: Facade
    {
        let input = input.into();

        let (raw, uses_point_size) = match input {
            ProgramCreationInput::SourceCode { vertex_shader, tessellation_control_shader,
                                               tessellation_evaluation_shader, geometry_shader,
                                               fragment_shader, transform_feedback_varyings,
                                               uses_point_size } =>
            {
                let mut has_geometry_shader = false;
                let mut has_tessellation_shaders = false;

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
                    has_tessellation_shaders = true;
                }

                if let Some(ts) = tessellation_evaluation_shader {
                    shaders.push((ts, gl::TESS_EVALUATION_SHADER));
                    has_tessellation_shaders = true;
                }

                // TODO: move somewhere else
                if transform_feedback_varyings.is_some() &&
                    (facade.get_context().get_version() >= &Version(Api::Gl, 3, 0) ||
                        !facade.get_context().get_extensions().gl_ext_transform_feedback)
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
                        shaders_store.push(try!(build_shader(facade, ty, src)));
                    }
                    shaders_store
                };

                (try!(RawProgram::from_shaders(facade, &shaders_store, has_geometry_shader,
                                               has_tessellation_shaders, transform_feedback_varyings)),
                 uses_point_size)
            },

            ProgramCreationInput::Binary { data, uses_point_size } => {
                if uses_point_size && !(facade.get_context().get_version() >= &Version(Api::Gl, 3, 0)) {
                    return Err(ProgramCreationError::PointSizeNotSupported);
                }

                (try!(RawProgram::from_binary(facade, data)),
                 uses_point_size)
            },
        };

        Ok(Program { raw: raw, uses_point_size: uses_point_size })
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
        Program::new(facade, ProgramCreationInput::SourceCode {
            vertex_shader: vertex_shader,
            fragment_shader: fragment_shader,
            geometry_shader: geometry_shader,
            tessellation_control_shader: None,
            tessellation_evaluation_shader: None,
            transform_feedback_varyings: None,
            uses_point_size: false,
        })
    }

    /// Returns the program's compiled binary.
    ///
    /// You can store the result in a file, then reload it later. This avoids having to compile
    /// the source code every time.
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
    pub fn get_frag_data_location(&self, name: &str) -> Option<u32> {
        self.raw.get_frag_data_location(name)
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

    /// Returns the list of transform feedback varyings.
    pub fn get_transform_feedback_buffers(&self) -> &[TransformFeedbackBuffer] {
        self.raw.get_transform_feedback_buffers()
    }

    /// True if the transform feedback output of this program matches the specified `VertexFormat`
    /// and `stride`.
    ///
    /// The `stride` is the number of bytes between two vertices.
    pub fn transform_feedback_matches(&self, format: &VertexFormat, stride: usize) -> bool {
        self.raw.transform_feedback_matches(format, stride)
    }

    /// Returns the type of geometry that transform feedback would generate, or `None` if it
    /// depends on the vertex/index data passed when drawing.
    ///
    /// This corresponds to `GL_GEOMETRY_OUTPUT_TYPE` or `GL_TESS_GEN_MODE`. If the program doesn't
    /// contain either a geometry shader or a tessellation evaluation shader, returns `None`.
    pub fn get_output_primitives(&self) -> Option<OutputPrimitives> {
        self.raw.get_output_primitives()
    }

    /// Returns true if the program contains a tessellation stage.
    pub fn has_tessellation_shaders(&self) -> bool {
        self.raw.has_tessellation_shaders()
    }

    /// Returns informations about an attribute, if it exists.
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
    pub fn attributes(&self) -> hash_map::Iter<String, Attribute> {
        self.raw.attributes()
    }

    /// Returns true if the program has been configured to output sRGB instead of RGB.
    pub fn has_srgb_output(&self) -> bool {
        self.raw.has_srgb_output()
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

    /// Returns true if the program has been configured to use the `gl_PointSize` variable.
    ///
    /// If the program uses `gl_PointSize` without having been configured appropriately, then
    /// setting the value of `gl_PointSize` will have no effect.
    pub fn uses_point_size(&self) -> bool {
      self.uses_point_size
    }
}

impl fmt::Debug for Program {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(formatter, "{:?}", self.raw)
    }
}

impl GlObject for Program {
    type Id = Handle;

    fn get_id(&self) -> Handle {
        self.raw.get_id()
    }
}

impl ProgramExt for Program {
    fn use_program(&self, ctxt: &mut CommandContext) {
        if self.uses_point_size && !ctxt.state.enabled_program_point_size {
            unsafe { ctxt.gl.Enable(gl::PROGRAM_POINT_SIZE); }
        }
        else if !self.uses_point_size && ctxt.state.enabled_program_point_size {
            unsafe { ctxt.gl.Disable(gl::PROGRAM_POINT_SIZE); }
        }

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
