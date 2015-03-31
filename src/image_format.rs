/*!
This private module handles the various image formats in OpenGL.

*/
use gl;
use ToGlEnum;
use context::Context;

use version::{Api, Version};

/// Error that is returned if the format is not supported by OpenGL.
#[derive(Copy, Debug)]
pub struct FormatNotSupportedError;

/// Texture format request.
#[derive(Copy, Clone, Debug)]
pub enum TextureFormatRequest {
    /// Request a specific format.
    Specific(TextureFormat),

    /// Request any floating-point format, normalized or not.
    AnyFloatingPoint,

    // TODO: 
    // /// Request any floating-point format represented with floats.
    //AnyFloatingPointFloat,

    /// Request any compressed format.
    AnyCompressed,

    /// Request any integral format.
    AnyIntegral,

    /// Request any unsigned format.
    AnyUnsigned,

    /// Request any depth format.
    AnyDepth,

    /// Request any stencil format.
    AnyStencil,

    /// Request any depth-stencil format.
    AnyDepthStencil,
}

/// List of client-side pixel formats.
///
/// These are all the possible formats of input data when uploading to a texture.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientFormat {
    U8,
    U8U8,
    U8U8U8,
    U8U8U8U8,
    I8,
    I8I8,
    I8I8I8,
    I8I8I8I8,
    U16,
    U16U16,
    U16U16U16,
    U16U16U16U16,
    I16,
    I16I16,
    I16I16I16,
    I16I16I16I16,
    U32,
    U32U32,
    U32U32U32,
    U32U32U32U32,
    I32,
    I32I32,
    I32I32I32,
    I32I32I32I32,
    U3U3U2,
    U5U6U5,
    U4U4U4U4,
    U5U5U5U1,
    U10U10U10U2,
    F16,
    F16F16,
    F16F16F16,
    F16F16F16F16,
    F32,
    F32F32,
    F32F32F32,
    F32F32F32F32,
}

impl ClientFormat {
    /// Returns the size in bytes of a pixel of this type.
    pub fn get_size(&self) -> usize {
        use std::mem;

        match *self {
            ClientFormat::U8 => 1 * mem::size_of::<u8>(),
            ClientFormat::U8U8 => 2 * mem::size_of::<u8>(),
            ClientFormat::U8U8U8 => 3 * mem::size_of::<u8>(),
            ClientFormat::U8U8U8U8 => 4 * mem::size_of::<u8>(),
            ClientFormat::I8 => 1 * mem::size_of::<i8>(),
            ClientFormat::I8I8 => 2 * mem::size_of::<i8>(),
            ClientFormat::I8I8I8 => 3 * mem::size_of::<i8>(),
            ClientFormat::I8I8I8I8 => 4 * mem::size_of::<i8>(),
            ClientFormat::U16 => 1 * mem::size_of::<u16>(),
            ClientFormat::U16U16 => 2 * mem::size_of::<u16>(),
            ClientFormat::U16U16U16 => 3 * mem::size_of::<u16>(),
            ClientFormat::U16U16U16U16 => 4 * mem::size_of::<u16>(),
            ClientFormat::I16 => 1 * mem::size_of::<i16>(),
            ClientFormat::I16I16 => 2 * mem::size_of::<i16>(),
            ClientFormat::I16I16I16 => 3 * mem::size_of::<i16>(),
            ClientFormat::I16I16I16I16 => 4 * mem::size_of::<i16>(),
            ClientFormat::U32 => 1 * mem::size_of::<u32>(),
            ClientFormat::U32U32 => 2 * mem::size_of::<u32>(),
            ClientFormat::U32U32U32 => 3 * mem::size_of::<u32>(),
            ClientFormat::U32U32U32U32 => 4 * mem::size_of::<u32>(),
            ClientFormat::I32 => 1 * mem::size_of::<i32>(),
            ClientFormat::I32I32 => 2 * mem::size_of::<i32>(),
            ClientFormat::I32I32I32 => 3 * mem::size_of::<i32>(),
            ClientFormat::I32I32I32I32 => 4 * mem::size_of::<i32>(),
            ClientFormat::U3U3U2 => (3 + 3 + 2) / 8,
            ClientFormat::U5U6U5 => (5 + 6 + 5) / 8,
            ClientFormat::U4U4U4U4 => (4 + 4 + 4 + 4) / 8,
            ClientFormat::U5U5U5U1 => (5 + 5 + 5 + 1) / 8,
            ClientFormat::U10U10U10U2 => (10 + 10 + 10 + 2) / 8,
            ClientFormat::F16 => 16 / 8,
            ClientFormat::F16F16 => (16 + 16) / 8,
            ClientFormat::F16F16F16 => (16 + 16 + 16) / 8,
            ClientFormat::F16F16F16F16 => (16 + 16 + 16 + 16) / 8,
            ClientFormat::F32 => 1 * mem::size_of::<f32>(),
            ClientFormat::F32F32 => 2 * mem::size_of::<f32>(),
            ClientFormat::F32F32F32 => 3 * mem::size_of::<f32>(),
            ClientFormat::F32F32F32F32 => 4 * mem::size_of::<f32>(),
        }
    }

    /// Returns the number of components of this client format.
    pub fn get_num_components(&self) -> u8 {
        match *self {
            ClientFormat::U8 => 1,
            ClientFormat::U8U8 => 2,
            ClientFormat::U8U8U8 => 3,
            ClientFormat::U8U8U8U8 => 4,
            ClientFormat::I8 => 1,
            ClientFormat::I8I8 => 2,
            ClientFormat::I8I8I8 => 3,
            ClientFormat::I8I8I8I8 => 4,
            ClientFormat::U16 => 1,
            ClientFormat::U16U16 => 2,
            ClientFormat::U16U16U16 => 3,
            ClientFormat::U16U16U16U16 => 4,
            ClientFormat::I16 => 1,
            ClientFormat::I16I16 => 2,
            ClientFormat::I16I16I16 => 3,
            ClientFormat::I16I16I16I16 => 4,
            ClientFormat::U32 => 1,
            ClientFormat::U32U32 => 2,
            ClientFormat::U32U32U32 => 3,
            ClientFormat::U32U32U32U32 => 4,
            ClientFormat::I32 => 1,
            ClientFormat::I32I32 => 2,
            ClientFormat::I32I32I32 => 3,
            ClientFormat::I32I32I32I32 => 4,
            ClientFormat::U3U3U2 => 3,
            ClientFormat::U5U6U5 => 3,
            ClientFormat::U4U4U4U4 => 4,
            ClientFormat::U5U5U5U1 => 4,
            ClientFormat::U10U10U10U2 => 4,
            ClientFormat::F16 => 1,
            ClientFormat::F16F16 => 2,
            ClientFormat::F16F16F16 => 3,
            ClientFormat::F16F16F16F16 => 4,
            ClientFormat::F32 => 1,
            ClientFormat::F32F32 => 2,
            ClientFormat::F32F32F32 => 3,
            ClientFormat::F32F32F32F32 => 4,
        }
    }
}

/// List of uncompressed pixel formats that contain floating-point-like data.
///
/// Some formats are marked as "guaranteed to be supported". What this means is that you are
/// certain that the backend will use exactly these formats. If you try to use a format that
/// is not supported by the backend, it will automatically fall back to a larger format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UncompressedFloatFormat {
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    U8,
    /// 
    ///
    /// Guaranteed to be supported for textures.
    I8,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    U16,
    /// 
    ///
    /// Guaranteed to be supported for textures.
    I16,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    U8U8,
    /// 
    ///
    /// Guaranteed to be supported for textures.
    I8I8,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    U16U16,
    /// 
    ///
    /// Guaranteed to be supported for textures.
    I16I16,
    /// 
    U3U32U,
    /// 
    U4U4U4,
    /// 
    U5U5U5,
    /// 
    ///
    /// Guaranteed to be supported for textures.
    U8U8U8,
    /// 
    ///
    /// Guaranteed to be supported for textures.
    I8I8I8,
    /// 
    U10U10U10,
    /// 
    U12U12U12,
    /// 
    ///
    /// Guaranteed to be supported for textures.
    I16I16I16,
    /// 
    U2U2U2U2,
    /// 
    U4U4U4U4,
    /// 
    U5U5U5U1,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    U8U8U8U8,
    /// 
    ///
    /// Guaranteed to be supported for textures.
    I8I8I8I8,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    U10U10U10U2,
    /// 
    U12U12U12U12,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    U16U16U16U16,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    F16,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    F16F16,
    /// 
    ///
    /// Guaranteed to be supported for textures.
    F16F16F16,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    F16F16F16F16,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    F32,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    F32F32,
    /// 
    ///
    /// Guaranteed to be supported for textures.
    F32F32F32,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    F32F32F32F32,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    F11F11F10,
    /// Uses three components of 9 bits of precision that all share the same exponent.
    ///
    /// Use this format only if all the components are approximately equal.
    ///
    /// Guaranteed to be supported for textures.
    F9F9F9,
}

impl UncompressedFloatFormat {
    /// Turns this format into a more generic `TextureFormat`.
    pub fn to_texture_format(self) -> TextureFormat {
        TextureFormat::UncompressedFloat(self)
    }
}

impl ToGlEnum for UncompressedFloatFormat {
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            UncompressedFloatFormat::U8 => gl::R8,
            UncompressedFloatFormat::I8 => gl::R8_SNORM,
            UncompressedFloatFormat::U16 => gl::R16,
            UncompressedFloatFormat::I16 => gl::R16_SNORM,
            UncompressedFloatFormat::U8U8 => gl::RG8,
            UncompressedFloatFormat::I8I8 => gl::RG8_SNORM,
            UncompressedFloatFormat::U16U16 => gl::RG16,
            UncompressedFloatFormat::I16I16 => gl::RG16_SNORM,
            UncompressedFloatFormat::U3U32U => gl::R3_G3_B2,
            UncompressedFloatFormat::U4U4U4 => gl::RGB4,
            UncompressedFloatFormat::U5U5U5 => gl::RGB5,
            UncompressedFloatFormat::U8U8U8 => gl::RGB8,
            UncompressedFloatFormat::I8I8I8 => gl::RGB8_SNORM,
            UncompressedFloatFormat::U10U10U10 => gl::RGB10,
            UncompressedFloatFormat::U12U12U12 => gl::RGB12,
            UncompressedFloatFormat::I16I16I16 => gl::RGB16_SNORM,
            UncompressedFloatFormat::U2U2U2U2 => gl::RGBA2,
            UncompressedFloatFormat::U4U4U4U4 => gl::RGBA4,
            UncompressedFloatFormat::U5U5U5U1 => gl::RGB5_A1,
            UncompressedFloatFormat::U8U8U8U8 => gl::RGBA8,
            UncompressedFloatFormat::I8I8I8I8 => gl::RGBA8_SNORM,
            UncompressedFloatFormat::U10U10U10U2 => gl::RGB10_A2,
            UncompressedFloatFormat::U12U12U12U12 => gl::RGBA12,
            UncompressedFloatFormat::U16U16U16U16 => gl::RGBA16,
            UncompressedFloatFormat::F16 => gl::R16F,
            UncompressedFloatFormat::F16F16 => gl::RG16F,
            UncompressedFloatFormat::F16F16F16 => gl::RGB16F,
            UncompressedFloatFormat::F16F16F16F16 => gl::RGBA16F,
            UncompressedFloatFormat::F32 => gl::R32F,
            UncompressedFloatFormat::F32F32 => gl::RG32F,
            UncompressedFloatFormat::F32F32F32 => gl::RGB32F,
            UncompressedFloatFormat::F32F32F32F32 => gl::RGBA32F,
            UncompressedFloatFormat::F11F11F10 => gl::R11F_G11F_B10F,
            UncompressedFloatFormat::F9F9F9 => gl::RGB9_E5,
        }
    }
}

/// List of uncompressed pixel formats that contain signed integral data.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UncompressedIntFormat {
    I8,
    I16,
    I32,
    I8I8,
    I16I16,
    I32I32,
    I8I8I8,
    /// May not be supported by renderbuffers.
    I16I16I16,
    /// May not be supported by renderbuffers.
    I32I32I32,
    /// May not be supported by renderbuffers.
    I8I8I8I8,
    I16I16I16I16,
    I32I32I32I32,
}

impl UncompressedIntFormat {
    /// Turns this format into a more generic `TextureFormat`.
    pub fn to_texture_format(self) -> TextureFormat {
        TextureFormat::UncompressedIntegral(self)
    }
}

impl ToGlEnum for UncompressedIntFormat {
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            UncompressedIntFormat::I8 => gl::R8I,
            UncompressedIntFormat::I16 => gl::R16I,
            UncompressedIntFormat::I32 => gl::R32I,
            UncompressedIntFormat::I8I8 => gl::RG8I,
            UncompressedIntFormat::I16I16 => gl::RG16I,
            UncompressedIntFormat::I32I32 => gl::RG32I,
            UncompressedIntFormat::I8I8I8 => gl::RGB8I,
            UncompressedIntFormat::I16I16I16 => gl::RGB16I,
            UncompressedIntFormat::I32I32I32 => gl::RGB32I,
            UncompressedIntFormat::I8I8I8I8 => gl::RGBA8I,
            UncompressedIntFormat::I16I16I16I16 => gl::RGBA16I,
            UncompressedIntFormat::I32I32I32I32 => gl::RGBA32I,
        }
    }
}

/// List of uncompressed pixel formats that contain unsigned integral data.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UncompressedUintFormat {
    U8,
    U16,
    U32,
    U8U8,
    U16U16,
    U32U32,
    U8U8U8,
    /// May not be supported by renderbuffers.
    U16U16U16,
    /// May not be supported by renderbuffers.
    U32U32U32,
    /// May not be supported by renderbuffers.
    U8U8U8U8,
    U16U16U16U16,
    U32U32U32U32,
    U10U10U10U2,
}

impl UncompressedUintFormat {
    /// Turns this format into a more generic `TextureFormat`.
    pub fn to_texture_format(self) -> TextureFormat {
        TextureFormat::UncompressedUnsigned(self)
    }
}

impl ToGlEnum for UncompressedUintFormat {
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            UncompressedUintFormat::U8 => gl::R8UI,
            UncompressedUintFormat::U16 => gl::R16UI,
            UncompressedUintFormat::U32 => gl::R32UI,
            UncompressedUintFormat::U8U8 => gl::RG8UI,
            UncompressedUintFormat::U16U16 => gl::RG16UI,
            UncompressedUintFormat::U32U32 => gl::RG32UI,
            UncompressedUintFormat::U8U8U8 => gl::RGB8UI,
            UncompressedUintFormat::U16U16U16 => gl::RGB16UI,
            UncompressedUintFormat::U32U32U32 => gl::RGB32UI,
            UncompressedUintFormat::U8U8U8U8 => gl::RGBA8UI,
            UncompressedUintFormat::U16U16U16U16 => gl::RGBA16UI,
            UncompressedUintFormat::U32U32U32U32 => gl::RGBA32UI,
            UncompressedUintFormat::U10U10U10U2 => gl::RGB10_A2UI,
        }
    }
}

/// List of compressed texture formats.
///
/// TODO: many formats are missing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressedFormat {
    /// Red/green compressed texture with one unsigned component.
    RGTCFormatU,
    /// Red/green compressed texture with one signed component.
    RGTCFormatI,
    /// Red/green compressed texture with two unsigned components.
    RGTCFormatUU,
    /// Red/green compressed texture with two signed components.
    RGTCFormatII,
}

impl CompressedFormat {
    /// Turns this format into a more generic `TextureFormat`.
    pub fn to_texture_format(self) -> TextureFormat {
        TextureFormat::CompressedFormat(self)
    }
}

impl ToGlEnum for CompressedFormat {
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            CompressedFormat::RGTCFormatU => gl::COMPRESSED_RED_RGTC1,
            CompressedFormat::RGTCFormatI => gl::COMPRESSED_SIGNED_RED_RGTC1,
            CompressedFormat::RGTCFormatUU => gl::COMPRESSED_RG_RGTC2,
            CompressedFormat::RGTCFormatII => gl::COMPRESSED_SIGNED_RG_RGTC2,
        }
    }
}

/// List of formats available for depth textures.
///
/// `I16`, `I24` and `I32` are still treated as if they were floating points.
/// Only the internal representation is integral.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthFormat {
    I16,
    I24,
    /// May not be supported by all hardware.
    I32,
    F32,
}

impl DepthFormat {
    /// Turns this format into a more generic `TextureFormat`.
    pub fn to_texture_format(self) -> TextureFormat {
        TextureFormat::DepthFormat(self)
    }
}

impl ToGlEnum for DepthFormat {
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            DepthFormat::I16 => gl::DEPTH_COMPONENT16,
            DepthFormat::I24 => gl::DEPTH_COMPONENT24,
            DepthFormat::I32 => gl::DEPTH_COMPONENT32,
            DepthFormat::F32 => gl::DEPTH_COMPONENT32F,
        }
    }
}

/// List of formats available for depth-stencil textures.
// TODO: If OpenGL 4.3 or ARB_stencil_texturing is not available, then depth/stencil
//       textures are treated by samplers exactly like depth-only textures
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthStencilFormat {
    I24I8,
    F32I8,
}

impl DepthStencilFormat {
    /// Turns this format into a more generic `TextureFormat`.
    pub fn to_texture_format(self) -> TextureFormat {
        TextureFormat::DepthStencilFormat(self)
    }
}

impl ToGlEnum for DepthStencilFormat {
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            DepthStencilFormat::I24I8 => gl::DEPTH24_STENCIL8,
            DepthStencilFormat::F32I8 => gl::DEPTH32F_STENCIL8,
        }
    }
}

/// List of formats available for stencil textures.
///
/// You are strongly advised to only use `I8`.
// TODO: Stencil only formats cannot be used for Textures, unless OpenGL 4.4 or
//       ARB_texture_stencil8 is available.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StencilFormat {
    I1,
    I4,
    I8,
    I16,
}

impl StencilFormat {
    /// Turns this format into a more generic `TextureFormat`.
    pub fn to_texture_format(self) -> TextureFormat {
        TextureFormat::StencilFormat(self)
    }
}

impl ToGlEnum for StencilFormat {
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            StencilFormat::I1 => gl::STENCIL_INDEX1,
            StencilFormat::I4 => gl::STENCIL_INDEX4,
            StencilFormat::I8 => gl::STENCIL_INDEX8,
            StencilFormat::I16 => gl::STENCIL_INDEX16,
        }
    }
}

/// Format of the internal representation of a texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum TextureFormat {
    UncompressedFloat(UncompressedFloatFormat),
    UncompressedIntegral(UncompressedIntFormat),
    UncompressedUnsigned(UncompressedUintFormat),
    CompressedFormat(CompressedFormat),
    DepthFormat(DepthFormat),
    StencilFormat(StencilFormat),
    DepthStencilFormat(DepthStencilFormat),
}

/// Checks that the texture format is supported and compatible with the client format.
///
/// Returns two `GLenum`s. The first one can be unsized and is suitable for the internal format
/// of `glTexImage#D`. The second one is always sized and is suitable for `glTexStorage*D` or
/// `glRenderbufferStorage`.
pub fn format_request_to_glenum(context: &Context, client: Option<ClientFormat>,
                                format: TextureFormatRequest)
                                -> Result<(gl::types::GLenum, Option<gl::types::GLenum>),
                                          FormatNotSupportedError>
{
    let version = context.get_version();
    let extensions = context.get_extensions();

    Ok(match format {
        TextureFormatRequest::AnyFloatingPoint => {
            let size = client.map(|c| c.get_num_components());

            if version >= &Version(Api::Gl, 3, 0) {
                match size {
                    Some(1) => (gl::RED, Some(gl::R8)),
                    Some(2) => (gl::RG, Some(gl::RG8)),
                    Some(3) => (gl::RGB, Some(gl::RGB8)),
                    Some(4) => (gl::RGBA, Some(gl::RGBA8)),
                    None => (gl::RGBA, Some(gl::RGBA8)),
                    _ => unreachable!(),
                }

            } else if version >= &Version(Api::Gl, 1, 1) {
                match size {
                    Some(1) => if extensions.gl_arb_texture_rg {
                        (gl::RED, Some(gl::R8))
                    } else {
                        (gl::RED, None)
                    },
                    Some(2) => if extensions.gl_arb_texture_rg {
                        (gl::RG, Some(gl::RG8))
                    } else {
                        (gl::RG, None)
                    },
                    Some(3) => (gl::RGB, Some(gl::RGB8)),
                    Some(4) => (gl::RGBA, Some(gl::RGBA8)),
                    None => (gl::RGBA, Some(gl::RGBA8)),
                    _ => unreachable!(),
                }

            } else {
                (size.unwrap_or(4) as gl::types::GLenum, None)
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(f)) => {
            let value = f.to_glenum();
            match value {
                gl::RGB | gl::RGBA => {
                    (value, None)
                },
                gl::RGB4 | gl::RGB5 | gl::RGB8 | gl::RGB10 | gl::RGB12 | gl::RGB16 |
                gl::RGBA2 | gl::RGBA4 | gl::RGB5_A1 | gl::RGBA8 | gl::RGB10_A2 |
                gl::RGBA12 | gl::RGBA16 | gl::R3_G3_B2 => {
                    (value, Some(value))
                },
                _ => {
                    if version >= &Version(Api::Gl, 3, 0) {
                        (value, Some(value))
                    } else {
                        return Err(FormatNotSupportedError);
                    }
                }
            }
        },

        TextureFormatRequest::Specific(TextureFormat::CompressedFormat(f)) => {
            if version >= &Version(Api::Gl, 3, 0) {    // FIXME: 
                (f.to_glenum(), Some(f.to_glenum()))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedIntegral(f)) => {
            if version >= &Version(Api::Gl, 3, 0) {    // FIXME: 
                (f.to_glenum(), Some(f.to_glenum()))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedUnsigned(f)) => {
            if version >= &Version(Api::Gl, 3, 0) {    // FIXME: 
                (f.to_glenum(), Some(f.to_glenum()))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::DepthFormat(f)) => {
            if version >= &Version(Api::Gl, 3, 0) {    // FIXME: 
                (f.to_glenum(), Some(f.to_glenum()))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::StencilFormat(f)) => {
            if version >= &Version(Api::Gl, 3, 0) {    // FIXME: 
                (f.to_glenum(), Some(f.to_glenum()))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::DepthStencilFormat(f)) => {
            if version >= &Version(Api::Gl, 3, 0) {    // FIXME: 
                (f.to_glenum(), Some(f.to_glenum()))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::AnyCompressed => {
            let size = client.map(|c| c.get_num_components());

            if version >= &Version(Api::Gl, 1, 1) {
                match size {
                    Some(1) => if version >= &Version(Api::Gl, 3, 0) || extensions.gl_arb_texture_rg {
                        (gl::COMPRESSED_RED, None)
                    } else {
                        // format not supported
                        (1, None)
                    },
                    Some(2) => if version >= &Version(Api::Gl, 3, 0) || extensions.gl_arb_texture_rg {
                        (gl::COMPRESSED_RG, None)
                    } else {
                        // format not supported
                        (2, None)
                    },
                    Some(3) => (gl::COMPRESSED_RGB, None),
                    Some(4) => (gl::COMPRESSED_RGBA, None),
                    None => (gl::COMPRESSED_RGBA, None),
                    _ => unreachable!(),
                }

            } else {
                // OpenGL 1.0 doesn't support compressed textures, so we use a
                // regular float format instead
                (size.unwrap_or(4) as gl::types::GLenum, None)
            }
        },

        TextureFormatRequest::AnyIntegral => {
            let size = client.map(|c| c.get_num_components());

            if version >= &Version(Api::Gl, 3, 0) {
                match size {  // FIXME: choose between 8, 16 and 32 depending on the client format
                    Some(1) => (gl::R32I, Some(gl::R32I)),
                    Some(2) => (gl::RG32I, Some(gl::RG32I)),
                    Some(3) => (gl::RGB32I, Some(gl::RGB32I)),
                    Some(4) => (gl::RGBA32I, Some(gl::RGBA32I)),
                    None => (gl::RGBA32I, Some(gl::RGBA32I)),
                    _ => unreachable!(),
                }

            } else {
                if !extensions.gl_ext_texture_integer {
                    return Err(FormatNotSupportedError);
                }

                match size {  // FIXME: choose between 8, 16 and 32 depending on the client format
                    Some(1) => if extensions.gl_arb_texture_rg {
                        (gl::R32I, Some(gl::R32I))
                    } else {
                        return Err(FormatNotSupportedError);
                    },
                    Some(2) => if extensions.gl_arb_texture_rg {
                        (gl::RG32I, Some(gl::RG32I))
                    } else {
                        return Err(FormatNotSupportedError);
                    },
                    Some(3) => (gl::RGB32I_EXT, Some(gl::RGB32I_EXT)),
                    Some(4) => (gl::RGBA32I_EXT, Some(gl::RGBA32I_EXT)),
                    None => (gl::RGBA32I_EXT, Some(gl::RGBA32I_EXT)),
                    _ => unreachable!(),
                }
            }
        },

        TextureFormatRequest::AnyUnsigned => {
            let size = client.map(|c| c.get_num_components());

            if version >= &Version(Api::Gl, 3, 0) {
                match size {  // FIXME: choose between 8, 16 and 32 depending on the client format
                    Some(1) => (gl::R32UI, Some(gl::R32I)),
                    Some(2) => (gl::RG32UI, Some(gl::RG32UI)),
                    Some(3) => (gl::RGB32UI, Some(gl::RGB32UI)),
                    Some(4) => (gl::RGBA32UI, Some(gl::RGBA32UI)),
                    None => (gl::RGBA32UI, Some(gl::RGBA32UI)),
                    _ => unreachable!(),
                }

            } else {
                if !extensions.gl_ext_texture_integer {
                    return Err(FormatNotSupportedError);
                }

                match size {  // FIXME: choose between 8, 16 and 32 depending on the client format
                    Some(1) => if extensions.gl_arb_texture_rg {
                        (gl::R32UI, Some(gl::R32UI))
                    } else {
                        return Err(FormatNotSupportedError);
                    },
                    Some(2) => if extensions.gl_arb_texture_rg {
                        (gl::RG32UI, Some(gl::RG32UI))
                    } else {
                        return Err(FormatNotSupportedError);
                    },
                    Some(3) => (gl::RGB32UI_EXT, Some(gl::RGB32UI_EXT)),
                    Some(4) => (gl::RGBA32UI_EXT, Some(gl::RGBA32UI_EXT)),
                    None => (gl::RGBA32UI_EXT, Some(gl::RGBA32UI_EXT)),
                    _ => unreachable!(),
                }
            }
        },

        TextureFormatRequest::AnyDepth => {
            if version >= &Version(Api::Gl, 1, 4) {
                (gl::DEPTH_COMPONENT, None)
            } else if extensions.gl_arb_depth_texture {
                (gl::DEPTH_COMPONENT, None)
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::AnyStencil => {
            if version < &Version(Api::Gl, 3, 0) {
                return Err(FormatNotSupportedError);
            }

            // TODO: we just request I8, but this could be more flexible
            return format_request_to_glenum(context, client,
                                     TextureFormatRequest::Specific(
                                        TextureFormat::UncompressedIntegral(
                                            UncompressedIntFormat::I8)));
        },

        TextureFormatRequest::AnyDepthStencil => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::DEPTH_STENCIL, None)
            } else if extensions.gl_ext_packed_depth_stencil {
                (gl::DEPTH_STENCIL_EXT, None)
            } else {
                return Err(FormatNotSupportedError);
            }
        },
    })
}

/// Checks that the client texture format is supported.
///
/// Returns two GLenums suitable for `glTexImage#D` and `glTexSubImage#D`.
pub fn client_format_to_glenum(_context: &Context, client: ClientFormat, format: TextureFormatRequest)
                               -> (gl::types::GLenum, gl::types::GLenum)
{
    match format {
        TextureFormatRequest::AnyFloatingPoint | TextureFormatRequest::AnyCompressed |
        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(_)) |
        TextureFormatRequest::Specific(TextureFormat::CompressedFormat(_)) =>
        {
            client_format_to_gl_enum(&client)
        },

        TextureFormatRequest::AnyIntegral |
        TextureFormatRequest::Specific(TextureFormat::UncompressedIntegral(_)) =>
        {
            client_format_to_gl_enum_int(&client).expect("Client format must \
                                                          have an integral format")
        },

        TextureFormatRequest::AnyUnsigned |
        TextureFormatRequest::Specific(TextureFormat::UncompressedUnsigned(_)) =>
        {
            client_format_to_gl_enum_uint(&client).expect("Client format must \
                                                           have an integral format")
        },

        TextureFormatRequest::AnyDepth |
        TextureFormatRequest::Specific(TextureFormat::DepthFormat(_)) =>
        {
            if client != ClientFormat::F32 {
                panic!("Only ClientFormat::F32 can be used to upload on a depth texture");
            }

            (gl::DEPTH_COMPONENT, gl::FLOAT)
        }

        TextureFormatRequest::AnyStencil |
        TextureFormatRequest::Specific(TextureFormat::StencilFormat(_)) =>
        {
            let (format, ty) = client_format_to_gl_enum_int(&client).expect("Client format must \
                                                                             have an integral \
                                                                             format");

            if format != gl::RED_INTEGER {
                panic!("Can only have one component when uploading a stencil texture");
            }

            (gl::STENCIL_INDEX, ty)
        }

        TextureFormatRequest::AnyDepthStencil |
        TextureFormatRequest::Specific(TextureFormat::DepthStencilFormat(_)) =>
        {
            unimplemented!();
        },
    }
}

/// Returns the two `GLenum`s corresponding to this client format.
fn client_format_to_gl_enum(format: &ClientFormat) -> (gl::types::GLenum, gl::types::GLenum) {
    match *format {
        ClientFormat::U8 => (gl::RED, gl::UNSIGNED_BYTE),
        ClientFormat::U8U8 => (gl::RG, gl::UNSIGNED_BYTE),
        ClientFormat::U8U8U8 => (gl::RGB, gl::UNSIGNED_BYTE),
        ClientFormat::U8U8U8U8 => (gl::RGBA, gl::UNSIGNED_BYTE),
        ClientFormat::I8 => (gl::RED, gl::BYTE),
        ClientFormat::I8I8 => (gl::RG, gl::BYTE),
        ClientFormat::I8I8I8 => (gl::RGB, gl::BYTE),
        ClientFormat::I8I8I8I8 => (gl::RGBA, gl::BYTE),
        ClientFormat::U16 => (gl::RED, gl::UNSIGNED_SHORT),
        ClientFormat::U16U16 => (gl::RG, gl::UNSIGNED_SHORT),
        ClientFormat::U16U16U16 => (gl::RGB, gl::UNSIGNED_SHORT),
        ClientFormat::U16U16U16U16 => (gl::RGBA, gl::UNSIGNED_SHORT),
        ClientFormat::I16 => (gl::RED, gl::SHORT),
        ClientFormat::I16I16 => (gl::RG, gl::SHORT),
        ClientFormat::I16I16I16 => (gl::RGB, gl::SHORT),
        ClientFormat::I16I16I16I16 => (gl::RGBA, gl::SHORT),
        ClientFormat::U32 => (gl::RED, gl::UNSIGNED_INT),
        ClientFormat::U32U32 => (gl::RG, gl::UNSIGNED_INT),
        ClientFormat::U32U32U32 => (gl::RGB, gl::UNSIGNED_INT),
        ClientFormat::U32U32U32U32 => (gl::RGBA, gl::UNSIGNED_INT),
        ClientFormat::I32 => (gl::RED, gl::INT),
        ClientFormat::I32I32 => (gl::RG, gl::INT),
        ClientFormat::I32I32I32 => (gl::RGB, gl::INT),
        ClientFormat::I32I32I32I32 => (gl::RGBA, gl::INT),
        ClientFormat::U3U3U2 => (gl::RGB, gl::UNSIGNED_BYTE_3_3_2),
        ClientFormat::U5U6U5 => (gl::RGB, gl::UNSIGNED_SHORT_5_6_5),
        ClientFormat::U4U4U4U4 => (gl::RGBA, gl::UNSIGNED_SHORT_4_4_4_4),
        ClientFormat::U5U5U5U1 => (gl::RGBA, gl::UNSIGNED_SHORT_5_5_5_1),
        ClientFormat::U10U10U10U2 => (gl::RGBA, gl::UNSIGNED_INT_10_10_10_2),
        ClientFormat::F16 => (gl::RED, gl::HALF_FLOAT),
        ClientFormat::F16F16 => (gl::RG, gl::HALF_FLOAT),
        ClientFormat::F16F16F16 => (gl::RGB, gl::HALF_FLOAT),
        ClientFormat::F16F16F16F16 => (gl::RGBA, gl::HALF_FLOAT),
        ClientFormat::F32 => (gl::RED, gl::FLOAT),
        ClientFormat::F32F32 => (gl::RG, gl::FLOAT),
        ClientFormat::F32F32F32 => (gl::RGB, gl::FLOAT),
        ClientFormat::F32F32F32F32 => (gl::RGBA, gl::FLOAT),
    }
}

/// Returns the two `GLenum`s corresponding to this client format for the "signed integer" format,
/// if possible
fn client_format_to_gl_enum_int(format: &ClientFormat)
                                -> Option<(gl::types::GLenum, gl::types::GLenum)>
{
    let (components, format) = client_format_to_gl_enum(format);

    let components = match components {
        gl::RED => gl::RED_INTEGER,
        gl::RG => gl::RG_INTEGER,
        gl::RGB => gl::RGB_INTEGER,
        gl::RGBA => gl::RGBA_INTEGER,
        _ => return None
    };

    match format {
        gl::BYTE => (),
        gl::SHORT => (),
        gl::INT => (),
        _ => return None
    };

    Some((components, format))
}

/// Returns the two `GLenum`s corresponding to this client format for the "unsigned integer" format,
/// if possible
fn client_format_to_gl_enum_uint(format: &ClientFormat)
                                 -> Option<(gl::types::GLenum, gl::types::GLenum)>
{
    let (components, format) = client_format_to_gl_enum(format);

    let components = match components {
        gl::RED => gl::RED_INTEGER,
        gl::RG => gl::RG_INTEGER,
        gl::RGB => gl::RGB_INTEGER,
        gl::RGBA => gl::RGBA_INTEGER,
        _ => return None
    };

    match format {
        gl::UNSIGNED_BYTE => (),
        gl::UNSIGNED_SHORT => (),
        gl::UNSIGNED_INT => (),
        gl::UNSIGNED_BYTE_3_3_2 => (),
        gl::UNSIGNED_SHORT_5_6_5 => (),
        gl::UNSIGNED_SHORT_4_4_4_4 => (),
        gl::UNSIGNED_SHORT_5_5_5_1 => (),
        gl::UNSIGNED_INT_10_10_10_2 => (),
        _ => return None
    };

    Some((components, format))
}

/// Returns the `UncompressedFloatFormat` most suitable for the `ClientFormat`.
fn to_float_internal_format(format: &ClientFormat) -> Option<UncompressedFloatFormat> {
    match *format {
        ClientFormat::U8 => Some(UncompressedFloatFormat::U8),
        ClientFormat::U8U8 => Some(UncompressedFloatFormat::U8U8),
        ClientFormat::U8U8U8 => Some(UncompressedFloatFormat::U8U8U8),
        ClientFormat::U8U8U8U8 => Some(UncompressedFloatFormat::U8U8U8U8),
        ClientFormat::I8 => Some(UncompressedFloatFormat::I8),
        ClientFormat::I8I8 => Some(UncompressedFloatFormat::I8I8),
        ClientFormat::I8I8I8 => Some(UncompressedFloatFormat::I8I8I8),
        ClientFormat::I8I8I8I8 => Some(UncompressedFloatFormat::I8I8I8I8),
        ClientFormat::U16 => Some(UncompressedFloatFormat::U16),
        ClientFormat::U16U16 => Some(UncompressedFloatFormat::U16U16),
        ClientFormat::U16U16U16 => None,
        ClientFormat::U16U16U16U16 => Some(UncompressedFloatFormat::U16U16U16U16),
        ClientFormat::I16 => Some(UncompressedFloatFormat::I16),
        ClientFormat::I16I16 => Some(UncompressedFloatFormat::I16I16),
        ClientFormat::I16I16I16 => Some(UncompressedFloatFormat::I16I16I16),
        ClientFormat::I16I16I16I16 => None,
        ClientFormat::U32 => None,
        ClientFormat::U32U32 => None,
        ClientFormat::U32U32U32 => None,
        ClientFormat::U32U32U32U32 => None,
        ClientFormat::I32 => None,
        ClientFormat::I32I32 => None,
        ClientFormat::I32I32I32 => None,
        ClientFormat::I32I32I32I32 => None,
        ClientFormat::U3U3U2 => None,
        ClientFormat::U5U6U5 => None,
        ClientFormat::U4U4U4U4 => Some(UncompressedFloatFormat::U4U4U4U4),
        ClientFormat::U5U5U5U1 => Some(UncompressedFloatFormat::U5U5U5U1),
        ClientFormat::U10U10U10U2 => Some(UncompressedFloatFormat::U10U10U10U2),
        ClientFormat::F16 => Some(UncompressedFloatFormat::F16),
        ClientFormat::F16F16 => Some(UncompressedFloatFormat::F16F16),
        ClientFormat::F16F16F16 => Some(UncompressedFloatFormat::F16F16F16),
        ClientFormat::F16F16F16F16 => Some(UncompressedFloatFormat::F16F16F16F16),
        ClientFormat::F32 => Some(UncompressedFloatFormat::F32),
        ClientFormat::F32F32 => Some(UncompressedFloatFormat::F32F32),
        ClientFormat::F32F32F32 => Some(UncompressedFloatFormat::F32F32F32),
        ClientFormat::F32F32F32F32 => Some(UncompressedFloatFormat::F32F32F32F32),
    }
}
