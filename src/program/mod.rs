//! Items related to creating an OpenGL program.

use std::fmt;
use std::error::Error;
use std::sync::Mutex;
use crate::CapabilitiesSource;

use crate::gl;
use crate::version::Api;
use crate::version::Version;

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
static COMPILER_GLOBAL_LOCK: Mutex<()> = Mutex::new(());

/// Used in ProgramCreationError::CompilationError to explain which shader stage failed compilation
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShaderType {
    /// Vertex shader, maps to gl::VERTEX_SHADER
    Vertex,
    /// Geometry shader, maps to gl::GEOMETRY_SHADER
    Geometry,
    /// Fragment shader, maps to gl::FRAGMENT_SHADER
    Fragment,
    /// Tesselation control shader, maps to gl::TESS_CONTROL_SHADER
    TesselationControl,
    /// Tesselation evaluation shader, maps to gl::TESS_EVALUATION_SHADER
    TesselationEvaluation,
    /// Compute shader, maps to gl::COMPUTE_SHADER
    Compute,
}

impl ShaderType {
    /// Creates an instance of gl::types::GLenum corresponding to the given ShaderType
    pub fn to_opengl_type(self) -> gl::types::GLenum {
        match self {
            ShaderType::Vertex => gl::VERTEX_SHADER,
            ShaderType::Geometry => gl::GEOMETRY_SHADER,
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
            ShaderType::TesselationControl => gl::TESS_CONTROL_SHADER,
            ShaderType::TesselationEvaluation => gl::TESS_EVALUATION_SHADER,
            ShaderType::Compute => gl::COMPUTE_SHADER,
        }
    }
    /// Creates an instance of ShaderType corresponding to the given gl::types::GLenum.
    /// This routine will panic if the given shadertype is not supported by glium.
    pub fn from_opengl_type(gl_type: gl::types::GLenum) -> Self {
        match gl_type {
            gl::VERTEX_SHADER => ShaderType::Vertex,
            gl::GEOMETRY_SHADER => ShaderType::Geometry,
            gl::FRAGMENT_SHADER => ShaderType::Fragment,
            gl::TESS_CONTROL_SHADER => ShaderType::TesselationControl,
            gl::TESS_EVALUATION_SHADER => ShaderType::TesselationEvaluation,
            gl::COMPUTE_SHADER  => ShaderType::Compute,
            _ => {
                panic!("Unsupported shader type")
            }
        }
    }
}

/// Error that can be triggered when creating a `Program`.
#[derive(Clone, Debug)]
pub enum ProgramCreationError {
    /// Error while compiling one of the shaders.
    CompilationError(String, ShaderType),

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
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        use self::ProgramCreationError::*;
        let desc = match *self {
            CompilationError(_,typ) => {
                match typ {
                    ShaderType::Vertex => "Compilation error in vertex shader",
                    ShaderType::Geometry => "Compilation error in geometry shader",
                    ShaderType::Fragment => "Compilation error in fragment shader",
                    ShaderType::TesselationControl => "Compilation error in tesselation control shader",
                    ShaderType::TesselationEvaluation => "Compilation error in tesselation evaluation shader",
                    ShaderType::Compute => "Compilation error in compute shader"
                }
            },
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
        };
        match *self {
            CompilationError(ref s, _) =>
                write!(fmt, "{}: {}", desc, s),
            LinkingError(ref s) =>
                write!(fmt, "{}: {}", desc, s),
            _ =>
                write!(fmt, "{}", desc),
        }
    }
}

impl Error for ProgramCreationError {}

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
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        use self::ProgramChooserCreationError::*;
        match *self {
            ProgramCreationError(ref err) => write!(fmt, "{}", err),
            NoVersion => fmt.write_str("No version of the program has been found for the current OpenGL version."),
        }
    }
}

impl Error for ProgramChooserCreationError {
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
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::GetBinaryError::*;
        let desc = match *self {
            NotSupported => "The backend doesn't support binary",
            NoFormats => "The backend does not supply any binary formats.",
        };
        fmt.write_str(desc)
    }
}

impl Error for GetBinaryError {}

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

        /// Whether the fragment shader outputs colors in `sRGB` or `RGB`. This is true by default,
        /// meaning that the program is responsible for outputting correct `sRGB` values.
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

        /// See [`ProgramCreationInput::SourceCode::outputs_srgb`].
        outputs_srgb: bool,

        /// Whether the shader uses point size.
        uses_point_size: bool,
    },

    /// Use a SPIR-V binary.
    SpirV(SpirvProgram<'a>),
}

/// Represents a SPIR-V program. The shaders can refer to entry points in the same binary.
#[derive(Clone)]
pub struct SpirvProgram<'a> {
    /// The vertex shader.
    pub vertex_shader: SpirvEntryPoint<'a>,

    /// The fragment shader.
    pub fragment_shader: SpirvEntryPoint<'a>,

    /// Optional tessellation control shader.
    pub tessellation_control_shader: Option<SpirvEntryPoint<'a>>,

    /// Optional tessellation evaluation shader.
    pub tessellation_evaluation_shader: Option<SpirvEntryPoint<'a>>,

    /// Optional geometry shader.
    pub geometry_shader: Option<SpirvEntryPoint<'a>>,

    /// The list of variables and mode to use for transform feedback.
    ///
    /// The information specified here will be passed to the OpenGL linker. If you pass
    /// `None`, then you won't be able to use transform feedback.
    pub transform_feedback_varyings: Option<(Vec<String>, TransformFeedbackMode)>,

    /// See [`ProgramCreationInput::SourceCode::outputs_srgb`].
    pub outputs_srgb: bool,

    /// Whether the shader uses point size.
    pub uses_point_size: bool,
}

impl<'a> SpirvProgram<'a> {
    /// Create new `SpirvProgram` from vertex and fragment shaders.
    pub fn from_vs_and_fs(
        vertex_shader: SpirvEntryPoint<'a>,
        fragment_shader: SpirvEntryPoint<'a>,
    ) -> Self {
        Self {
            vertex_shader,
            fragment_shader,
            tessellation_control_shader: None,
            tessellation_evaluation_shader: None,
            geometry_shader: None,
            transform_feedback_varyings: None,
            outputs_srgb: true,
            uses_point_size: false,
        }
    }

    /// Builder method to set `tessellation_control_shader`.
    pub fn tessellation_control_shader(mut self, tessellation_control_shader: Option<SpirvEntryPoint<'a>>) -> Self {
        self.tessellation_control_shader = tessellation_control_shader;
        self
    }

    /// Builder method to set `tessellation_evaluation_shader`.
    pub fn tessellation_evaluation_shader(mut self, tessellation_evaluation_shader: Option<SpirvEntryPoint<'a>>) -> Self {
        self.tessellation_evaluation_shader = tessellation_evaluation_shader;
        self
    }

    /// Builder method to set `geometry_shader`.
    pub fn geometry_shader(mut self, geometry_shader: Option<SpirvEntryPoint<'a>>) -> Self {
        self.geometry_shader = geometry_shader;
        self
    }

    /// Builder method to set `transform_feedback_varyings`.
    pub fn transform_feedback_varyings(mut self, transform_feedback_varyings: Option<(Vec<String>, TransformFeedbackMode)>) -> Self {
        self.transform_feedback_varyings = transform_feedback_varyings;
        self
    }

    /// Builder method to set `outputs_srgb`.
    pub fn outputs_srgb(mut self, outputs_srgb: bool) -> Self {
        self.outputs_srgb = outputs_srgb;
        self
    }

    /// Builder method to set `uses_point_size`.
    pub fn uses_point_size(mut self, uses_point_size: bool) -> Self {
        self.uses_point_size = uses_point_size;
        self
    }
}

/// Represents an entry point of a binary SPIR-V module.
#[derive(Copy, Clone)]
pub struct SpirvEntryPoint<'a> {
    /// The binary module data.
    pub binary: &'a [u8],

    /// The entry point to use, e.g. "main".
    pub entry_point: &'a str,
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
            vertex_shader,
            tessellation_control_shader,
            tessellation_evaluation_shader,
            geometry_shader,
            fragment_shader,
            transform_feedback_varyings: None,
            outputs_srgb: true,
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
            outputs_srgb: true,
            uses_point_size: false,
        }
    }
}
