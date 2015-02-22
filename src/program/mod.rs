use std::sync::{StaticMutex, MUTEX_INIT};

pub use self::program::{Program, ProgramCreationError};
pub use self::reflection::{Uniform, UniformBlock, UniformBlockMember};
pub use self::reflection::{Attribute, TransformFeedbackVarying, TransformFeedbackMode};

// TODO: remove this hack
pub use self::program::{get_uniforms_locations, get_attributes};

mod program;
mod reflection;
mod shader;

/// Some shader compilers have race-condition issues, so we lock this mutex
/// in the GL thread every time we compile a shader or link a program.
static COMPILER_GLOBAL_LOCK: StaticMutex = MUTEX_INIT;

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

        /// The list of variables and mode to use for transform feedback.
        ///
        /// The information specified here will be passed to the OpenGL linker. If you pass
        /// `None`, then you won't be able to use transform feedback.
        transform_feedback_varyings: Option<(Vec<String>, TransformFeedbackMode)>,
    },

    /// Use a precompiled binary.
    Binary {
        /// The data.
        data: Binary,
    }
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
            transform_feedback_varyings: None,
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

impl IntoProgramCreationInput<'static> for Binary {
    fn into_program_creation_input(self) -> ProgramCreationInput<'static> {
        ProgramCreationInput::Binary {
            data: self,
        }
    }
}
