//! Items related to creating an OpenGL program.

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
pub use self::reflection::{ShaderStage, SubroutineData, SubroutineUniform};

mod compute;
mod program;
mod raw;
mod reflection;
mod shader;
mod uniforms_storage;
mod binary_header;

/// Returns true if the backend supports geometry shaders.
#[inline]
pub fn is_geometry_shader_supported<C: ?Sized>(ctxt: &C) -> bool where C: CapabilitiesSource {
    shader::check_shader_type_compatibility(ctxt, gl::GEOMETRY_SHADER)
}

/// Returns true if the backend supports tessellation shaders.
#[inline]
pub fn is_tessellation_shader_supported<C: ?Sized>(ctxt: &C) -> bool where C: CapabilitiesSource {
    shader::check_shader_type_compatibility(ctxt, gl::TESS_CONTROL_SHADER)
}

/// Returns true if the backend supports creating and retrieving binary format.
#[inline]
pub fn is_binary_supported<C: ?Sized>(ctxt: &C) -> bool where C: CapabilitiesSource {
    ctxt.get_version() >= &Version(Api::Gl, 4, 1) || ctxt.get_version() >= &Version(Api::GlEs, 2, 0)
        || ctxt.get_extensions().gl_arb_get_programy_binary
}

/// Returns true if the backend supports shader subroutines.
#[inline]
pub fn is_subroutine_supported<C: ?Sized>(ctxt: &C) -> bool where C: CapabilitiesSource {
    // WORKAROUND: Windows only; NVIDIA doesn't actually return a valid function pointer for
    //              GetProgramStageiv despite supporting ARB_shader_subroutine; see #1439
    if cfg!(target_os = "windows")
        && ctxt.get_version() <= &Version(Api::Gl, 4, 0)
        && ctxt.get_capabilities().vendor == "NVIDIA Corporation" {
        return false;
    }
    ctxt.get_version() >= &Version(Api::Gl, 4, 0) || ctxt.get_extensions().gl_arb_shader_subroutine
}

// Some shader compilers have race-condition issues, so we lock this mutex
// in the GL thread every time we compile a shader or link a program.
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

    /// The glium-specific binary header was not found or is corrupt.
    BinaryHeaderError,
}

impl fmt::Display for ProgramCreationError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::ProgramCreationError::*;
        match *self {
            CompilationError(ref s) =>
                write!(fmt, "{}: {}", self.description(), s),
            LinkingError(ref s) =>
                write!(fmt, "{}: {}", self.description(), s),
            _ =>
                write!(fmt, "{}", self.description()),
        }
    }
}

impl Error for ProgramCreationError {
    fn description(&self) -> &str {
        use self::ProgramCreationError::*;
        match *self {
            CompilationError(_) =>
                "Compilation error in one of the shaders",
            LinkingError(_) =>
                "Error while linking shaders together",
            ShaderTypeNotSupported =>
                "One of the request shader type is not supported by the backend",
            CompilationNotSupported =>
                "The backend doesn't support shaders compilation",
            TransformFeedbackNotSupported =>
                "Transform feedback is not supported by the backend.",
            PointSizeNotSupported =>
                "Point size is not supported by the backend.",
            BinaryHeaderError =>
                "The glium-specific binary header was not found or is corrupt.",
        }
    }
}

/// Error type that is returned by the `program!` macro.
#[derive(Clone, Debug)]
pub enum ProgramChooserCreationError {
    /// No available version has been found.
    NoVersion,

    /// A version has been found but it triggered the given error.
    ProgramCreationError(ProgramCreationError),
}

impl fmt::Display for ProgramChooserCreationError {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", self.description())
    }
}

impl Error for ProgramChooserCreationError {
    #[inline]
    fn description(&self) -> &str {
        use self::ProgramChooserCreationError::*;
        match *self {
            ProgramCreationError(ref err) => err.description(),
            NoVersion => "No version of the program has been found for the current OpenGL version.",
        }
    }

    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::ProgramChooserCreationError::*;
        match *self {
            ProgramCreationError(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<ProgramCreationError> for ProgramChooserCreationError {
    fn from(err: ProgramCreationError) -> ProgramChooserCreationError {
        ProgramChooserCreationError::ProgramCreationError(err)
    }
}

/// Error while retrieving the binary representation of a program.
#[derive(Copy, Clone, Debug)]
pub enum GetBinaryError {
    /// The backend doesn't support binary.
    NotSupported,
    /// The backend does not supply any binary formats.
    NoFormats,
}

impl fmt::Display for GetBinaryError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

impl Error for GetBinaryError {
    fn description(&self) -> &str {
        use self::GetBinaryError::*;
        match *self {
            NotSupported => "The backend doesn't support binary",
            NoFormats => "The backend does not supply any binary formats.",
        }
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
