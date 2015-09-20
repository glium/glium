use std::fmt;
use std::error::Error;
use std::sync::Mutex;
use CapabilitiesSource;

use gl;
use version::Api;
use version::Version;

pub use self::compute::{ComputeShader, ComputeCommand};
pub use self::program::Program;
pub use self::reflection::{Uniform, UniformBlock, BlockLayout, OutputPrimitives};
pub use self::reflection::{Attribute, TransformFeedbackVarying, TransformFeedbackBuffer, TransformFeedbackMode};

mod compute;
mod program;
mod raw;
mod reflection;
mod shader;
mod uniforms_storage;

/// Returns true if the backend supports geometry shaders.
#[inline]
pub fn is_geometry_shader_supported<C>(ctxt: &C) -> bool where C: CapabilitiesSource {
    shader::check_shader_type_compatibility(ctxt, gl::GEOMETRY_SHADER)
}

/// Returns true if the backend supports tessellation shaders.
#[inline]
pub fn is_tessellation_shader_supported<C>(ctxt: &C) -> bool where C: CapabilitiesSource {
    shader::check_shader_type_compatibility(ctxt, gl::TESS_CONTROL_SHADER)
}

/// Returns true if the backend supports creating and retreiving binary format.
#[inline]
pub fn is_binary_supported<C>(ctxt: &C) -> bool where C: CapabilitiesSource {
    ctxt.get_version() >= &Version(Api::Gl, 4, 1) || ctxt.get_version() >= &Version(Api::GlEs, 2, 0)
        || ctxt.get_extensions().gl_arb_get_programy_binary
}

/// Some shader compilers have race-condition issues, so we lock this mutex
/// in the GL thread every time we compile a shader or link a program.
// TODO: replace by a StaticMutex
lazy_static! {
    static ref COMPILER_GLOBAL_LOCK: Mutex<()> = Mutex::new(());
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

    /// The OpenGL implementation doesn't provide a compiler.
    CompilationNotSupported,

    /// You have requested transform feedback varyings, but transform feedback is not supported
    /// by the backend.
    TransformFeedbackNotSupported,

    /// You have requested point size setting from the shader, but it's not
    /// supported by the backend.
    PointSizeNotSupported,
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
            &ProgramCreationError::PointSizeNotSupported =>
                formatter.write_str("You requested point size setting, but it's not \
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
            &ProgramCreationError::PointSizeNotSupported => "Point size is not supported by \
                                                             the backend.",
        }
    }

    #[inline]
    fn cause(&self) -> Option<&Error> {
        None
    }
}

/// Error while retreiving the binary representation of a program.
#[derive(Copy, Clone, Debug)]
pub enum GetBinaryError {
    /// The backend doesn't support binary.
    NotSupported,
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

        /// The list of variables and mode to use for transform feedback.
        ///
        /// The information specified here will be passed to the OpenGL linker. If you pass
        /// `None`, then you won't be able to use transform feedback.
        transform_feedback_varyings: Option<(Vec<String>, TransformFeedbackMode)>,

        /// Whether the fragment shader outputs colors in `sRGB` or `RGB`. This is false by default,
        /// meaning that the program outputs `RGB`.
        ///
        /// If this is false, then `GL_FRAMEBUFFER_SRGB` will be enabled when this program is used
        /// (if it is supported).
        outputs_srgb: bool,

        /// Whether the shader uses point size.
        uses_point_size: bool,
    },

    /// Use a precompiled binary.
    Binary {
        /// The data.
        data: Binary,

        /// See `SourceCode::outputs_srgb`.
        outputs_srgb: bool,

        /// Whether the shader uses point size.
        uses_point_size: bool,
    }
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

impl<'a> From<SourceCode<'a>> for ProgramCreationInput<'a> {
    #[inline]
    fn from(code: SourceCode<'a>) -> ProgramCreationInput<'a> {
        let SourceCode { vertex_shader, fragment_shader, geometry_shader,
                         tessellation_control_shader, tessellation_evaluation_shader } = code;

        ProgramCreationInput::SourceCode {
            vertex_shader: vertex_shader,
            tessellation_control_shader: tessellation_control_shader,
            tessellation_evaluation_shader: tessellation_evaluation_shader,
            geometry_shader: geometry_shader,
            fragment_shader: fragment_shader,
            transform_feedback_varyings: None,
            outputs_srgb: false,
            uses_point_size: false,
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

impl<'a> From<Binary> for ProgramCreationInput<'a> {
    #[inline]
    fn from(binary: Binary) -> ProgramCreationInput<'a> {
        ProgramCreationInput::Binary {
            data: binary,
            outputs_srgb: false,
            uses_point_size: false,
        }
    }
}
