/*!
This private module handles the various image formats in OpenGL.

*/
use gl;
use ContextExt;
use context::Context;

use version::{Api, Version};

/// Error that is returned if the format is not supported by OpenGL.
#[derive(Copy, Clone, Debug)]
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

    /// Request any sRGB format.
    AnySrgb,

    /// Request any compressed sRGB format.
    AnyCompressedSrgb,

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

/// List of uncompressed pixel formats that contain floating-point data in the sRGB color space.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SrgbFormat {
    U8U8U8,
    U8U8U8U8,
}

impl SrgbFormat {
    /// Turns this format into a more generic `TextureFormat`.
    pub fn to_texture_format(self) -> TextureFormat {
        TextureFormat::Srgb(self)
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

/// List of compressed texture formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressedFormat {
    /// Red/green compressed texture with one unsigned component.
    RgtcFormatU,
    /// Red/green compressed texture with one signed component.
    RgtcFormatI,
    /// Red/green compressed texture with two unsigned components.
    RgtcFormatUU,
    /// Red/green compressed texture with two signed components.
    RgtcFormatII,

    /// BPTC format with four components represented as integers.
    BptcUnorm4,
    /// BPTC format with three components (no alpha) represented as signed floats.
    BptcSignedFloat3,
    /// BPTC format with three components (no alpha) represented as unsigned floats.
    BptcUnsignedFloat3,

    /// S3TC DXT1 without alpha, see https://www.opengl.org/wiki/S3_Texture_Compression.
    S3tcDxt1NoAlpha,
    /// S3TC DXT1 with 1-bit alpha, see https://www.opengl.org/wiki/S3_Texture_Compression.
    S3tcDxt1Alpha,
    /// S3TC DXT3, see https://www.opengl.org/wiki/S3_Texture_Compression.
    S3tcDxt3Alpha,
    /// S3TC DXT5, see https://www.opengl.org/wiki/S3_Texture_Compression.
    S3tcDxt5Alpha,
}

impl CompressedFormat {
    /// Turns this format into a more generic `TextureFormat`.
    pub fn to_texture_format(self) -> TextureFormat {
        TextureFormat::CompressedFormat(self)
    }
}

/// List of compressed pixel formats in the sRGB color space.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressedSrgbFormat {
    /// BPTC format. sRGB with alpha. Also called `BC7` by DirectX.
    Bptc,
    S3tcDxt1NoAlpha,
    S3tcDxt1Alpha,
    S3tcDxt3Alpha,
    S3tcDxt5Alpha,
}

impl CompressedSrgbFormat {
    /// Turns this format into a more generic `TextureFormat`.
    pub fn to_texture_format(self) -> TextureFormat {
        TextureFormat::CompressedSrgbFormat(self)
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

/// Format of the internal representation of a texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum TextureFormat {
    UncompressedFloat(UncompressedFloatFormat),
    UncompressedIntegral(UncompressedIntFormat),
    UncompressedUnsigned(UncompressedUintFormat),
    Srgb(SrgbFormat),
    CompressedFormat(CompressedFormat),
    CompressedSrgbFormat(CompressedSrgbFormat),
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
        /*******************************************************************/
        /*                           REGULAR                               */
        /*******************************************************************/
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

            } else if version >= &Version(Api::GlEs, 2, 0) {
                // TODO: better dispatch with versions and extensions
                match size {
                    Some(3) => (gl::RGB, Some(gl::RGB8)),
                    Some(4) => (gl::RGBA, Some(gl::RGBA8)),
                    None => (gl::RGBA, Some(gl::RGBA8)),
                    _ => return Err(FormatNotSupportedError)
                }

            } else {
                (size.unwrap_or(4) as gl::types::GLenum, None)
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::U8)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::R8, Some(gl::R8))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::I8)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::R8_SNORM, Some(gl::R8_SNORM))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::U16)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::R16, Some(gl::R16))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::I16)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::R16_SNORM, Some(gl::R16_SNORM))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::U8U8)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::RG8, Some(gl::RG8))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::I8I8)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::RG8_SNORM, Some(gl::RG8_SNORM))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::U16U16)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::RG16, Some(gl::RG16))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::I16I16)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::RG16_SNORM, Some(gl::RG16_SNORM))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::U3U32U)) => {
            if version >= &Version(Api::Gl, 1, 1) {
                (gl::R3_G3_B2, Some(gl::R3_G3_B2))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::U4U4U4)) => {
            if version >= &Version(Api::Gl, 1, 1) {
                (gl::RGB4, Some(gl::RGB4))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::U5U5U5)) => {
            if version >= &Version(Api::Gl, 1, 1) {
                (gl::RGB5, Some(gl::RGB5))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::U8U8U8)) => {
            if version >= &Version(Api::Gl, 1, 1) {
                (gl::RGB8, Some(gl::RGB8))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::I8I8I8)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::RGB8_SNORM, Some(gl::RGB8_SNORM))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::U10U10U10)) => {
            if version >= &Version(Api::Gl, 1, 1) {
                (gl::RGB10, Some(gl::RGB10))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::U12U12U12)) => {
            if version >= &Version(Api::Gl, 1, 1) {
                (gl::RGB12, Some(gl::RGB12))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::I16I16I16)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::RGB16_SNORM, Some(gl::RGB16_SNORM))
            } else if version >= &Version(Api::Gl, 1, 1) {
                (gl::RGB16, Some(gl::RGB16))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::U2U2U2U2)) => {
            if version >= &Version(Api::Gl, 1, 1) {
                (gl::RGBA2, Some(gl::RGBA2))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::U4U4U4U4)) => {
            if version >= &Version(Api::Gl, 1, 1) {
                (gl::RGBA4, Some(gl::RGBA4))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::U5U5U5U1)) => {
            if version >= &Version(Api::Gl, 1, 1) {
                (gl::RGB5_A1, Some(gl::RGB5_A1))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::U8U8U8U8)) => {
            if version >= &Version(Api::Gl, 1, 1) {
                (gl::RGBA8, Some(gl::RGBA8))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::I8I8I8I8)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::RGBA8_SNORM, Some(gl::RGBA8_SNORM))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::U10U10U10U2)) => {
            if version >= &Version(Api::Gl, 1, 1) {
                (gl::RGB10_A2, Some(gl::RGB10_A2))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::U12U12U12U12)) => {
            if version >= &Version(Api::Gl, 1, 1) {
                (gl::RGBA12, Some(gl::RGBA12))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::U16U16U16U16)) => {
            if version >= &Version(Api::Gl, 1, 1) {
                (gl::RGBA16, Some(gl::RGBA16))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::F16)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::R16F, Some(gl::R16F))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::F16F16)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::RG16F, Some(gl::RG16F))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::F16F16F16)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::RGB16F, Some(gl::RGB16F))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::F16F16F16F16)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::RGBA16F, Some(gl::RGBA16F))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::F32)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::R32F, Some(gl::R32F))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::F32F32)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::RG32F, Some(gl::RG32F))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::F32F32F32)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::RGB32F, Some(gl::RGB32F))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::F32F32F32F32)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::RGBA32F, Some(gl::RGBA32F))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::F11F11F10)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::R11F_G11F_B10F, Some(gl::R11F_G11F_B10F))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(UncompressedFloatFormat::F9F9F9)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::RGB9_E5, Some(gl::RGB9_E5))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        /*******************************************************************/
        /*                         COMPRESSED                              */
        /*******************************************************************/
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

        TextureFormatRequest::Specific(TextureFormat::CompressedFormat(CompressedFormat::RgtcFormatU)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::COMPRESSED_RED_RGTC1, Some(gl::COMPRESSED_RED_RGTC1))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::CompressedFormat(CompressedFormat::RgtcFormatI)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::COMPRESSED_SIGNED_RED_RGTC1, Some(gl::COMPRESSED_SIGNED_RED_RGTC1))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::CompressedFormat(CompressedFormat::RgtcFormatUU)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::COMPRESSED_RG_RGTC2, Some(gl::COMPRESSED_RG_RGTC2))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::CompressedFormat(CompressedFormat::RgtcFormatII)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::COMPRESSED_SIGNED_RG_RGTC2, Some(gl::COMPRESSED_SIGNED_RG_RGTC2))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::CompressedFormat(CompressedFormat::BptcUnorm4)) => {
            if version >= &Version(Api::Gl, 4, 2) || extensions.gl_arb_texture_compression_bptc {
                (gl::COMPRESSED_RGBA_BPTC_UNORM, Some(gl::COMPRESSED_RGBA_BPTC_UNORM))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::CompressedFormat(CompressedFormat::BptcSignedFloat3)) => {
            if version >= &Version(Api::Gl, 4, 2) || extensions.gl_arb_texture_compression_bptc {
                (gl::COMPRESSED_RGB_BPTC_SIGNED_FLOAT, Some(gl::COMPRESSED_RGB_BPTC_SIGNED_FLOAT))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::CompressedFormat(CompressedFormat::BptcUnsignedFloat3)) => {
            if version >= &Version(Api::Gl, 4, 2) || extensions.gl_arb_texture_compression_bptc {
                (gl::COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT, Some(gl::COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::CompressedFormat(CompressedFormat::S3tcDxt1NoAlpha)) => {
            if extensions.gl_ext_texture_compression_s3tc {
                (gl::COMPRESSED_RGB_S3TC_DXT1_EXT, Some(gl::COMPRESSED_RGB_S3TC_DXT1_EXT))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::CompressedFormat(CompressedFormat::S3tcDxt1Alpha)) => {
            if extensions.gl_ext_texture_compression_s3tc {
                (gl::COMPRESSED_RGBA_S3TC_DXT1_EXT, Some(gl::COMPRESSED_RGBA_S3TC_DXT1_EXT))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::CompressedFormat(CompressedFormat::S3tcDxt3Alpha)) => {
            if extensions.gl_ext_texture_compression_s3tc {
                (gl::COMPRESSED_RGBA_S3TC_DXT3_EXT, Some(gl::COMPRESSED_RGBA_S3TC_DXT3_EXT))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::CompressedFormat(CompressedFormat::S3tcDxt5Alpha)) => {
            if extensions.gl_ext_texture_compression_s3tc {
                (gl::COMPRESSED_RGBA_S3TC_DXT5_EXT, Some(gl::COMPRESSED_RGBA_S3TC_DXT5_EXT))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        /*******************************************************************/
        /*                             SRGB                                */
        /*******************************************************************/
        TextureFormatRequest::AnySrgb => {
            let size = client.map(|c| c.get_num_components());

            if version >= &Version(Api::Gl, 2, 1) || version >= &Version(Api::GlEs, 3, 0) ||
               extensions.gl_ext_texture_srgb
            {
                match size {
                    Some(1 ... 3) => (gl::SRGB8, Some(gl::SRGB8)),
                    Some(4) => (gl::SRGB8_ALPHA8, Some(gl::SRGB8_ALPHA8)),
                    None => (gl::SRGB8, Some(gl::SRGB8_ALPHA8)),
                    _ => unreachable!(),
                }

            } else {
                // no support for sRGB
                try!(format_request_to_glenum(context, client,
                                              TextureFormatRequest::AnyFloatingPoint))
            }
        },

        TextureFormatRequest::Specific(TextureFormat::Srgb(SrgbFormat::U8U8U8)) => {
            if version >= &Version(Api::Gl, 2, 1) || version >= &Version(Api::GlEs, 3, 0) ||
               extensions.gl_ext_texture_srgb
            {
                (gl::SRGB8, Some(gl::SRGB8))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::Srgb(SrgbFormat::U8U8U8U8)) => {
            if version >= &Version(Api::Gl, 2, 1) || version >= &Version(Api::GlEs, 3, 0) ||
               extensions.gl_ext_texture_srgb
            {
                (gl::SRGB8_ALPHA8, Some(gl::SRGB8_ALPHA8))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        /*******************************************************************/
        /*                        COMPRESSED SRGB                          */
        /*******************************************************************/
        TextureFormatRequest::AnyCompressedSrgb => {
            let size = client.map(|c| c.get_num_components());

            if version >= &Version(Api::Gl, 4, 0) || extensions.gl_ext_texture_srgb {
                match size {
                    Some(1 ... 3) => (gl::COMPRESSED_SRGB, None),
                    Some(4) => (gl::COMPRESSED_SRGB_ALPHA, None),
                    None => (gl::COMPRESSED_SRGB_ALPHA, None),
                    _ => unreachable!(),
                }

            } else {
                // no support for compressed srgb textures
                try!(format_request_to_glenum(context, client, TextureFormatRequest::AnySrgb))
            }
        },

        TextureFormatRequest::Specific(TextureFormat::CompressedSrgbFormat(CompressedSrgbFormat::Bptc)) => {
            if version >= &Version(Api::Gl, 4, 2) || extensions.gl_arb_texture_compression_bptc {
                (gl::COMPRESSED_SRGB_ALPHA_BPTC_UNORM, Some(gl::COMPRESSED_SRGB_ALPHA_BPTC_UNORM))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::CompressedSrgbFormat(CompressedSrgbFormat::S3tcDxt1NoAlpha)) => {
            if extensions.gl_ext_texture_compression_s3tc && extensions.gl_ext_texture_srgb {
                (gl::COMPRESSED_SRGB_S3TC_DXT1_EXT, Some(gl::COMPRESSED_SRGB_S3TC_DXT1_EXT))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::CompressedSrgbFormat(CompressedSrgbFormat::S3tcDxt1Alpha)) => {
            if extensions.gl_ext_texture_compression_s3tc && extensions.gl_ext_texture_srgb {
                (gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT, Some(gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::CompressedSrgbFormat(CompressedSrgbFormat::S3tcDxt3Alpha)) => {
            if extensions.gl_ext_texture_compression_s3tc && extensions.gl_ext_texture_srgb {
                (gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT, Some(gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::CompressedSrgbFormat(CompressedSrgbFormat::S3tcDxt5Alpha)) => {
            if extensions.gl_ext_texture_compression_s3tc && extensions.gl_ext_texture_srgb {
                (gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT, Some(gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        /*******************************************************************/
        /*                          INTEGRAL                               */
        /*******************************************************************/
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

        TextureFormatRequest::Specific(TextureFormat::UncompressedIntegral(UncompressedIntFormat::I8)) => {
            if version >= &Version(Api::Gl, 3, 0) || (extensions.gl_ext_texture_integer &&
                                                      extensions.gl_arb_texture_rg)
            {
                (gl::R8I, Some(gl::R8I))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedIntegral(UncompressedIntFormat::I16)) => {
            if version >= &Version(Api::Gl, 3, 0) || (extensions.gl_ext_texture_integer &&
                                                      extensions.gl_arb_texture_rg)
            {
                (gl::R16I, Some(gl::R16I))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedIntegral(UncompressedIntFormat::I32)) => {
            if version >= &Version(Api::Gl, 3, 0) || (extensions.gl_ext_texture_integer &&
                                                      extensions.gl_arb_texture_rg)
            {
                (gl::R32I, Some(gl::R32I))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedIntegral(UncompressedIntFormat::I8I8)) => {
            if version >= &Version(Api::Gl, 3, 0) || (extensions.gl_ext_texture_integer &&
                                                      extensions.gl_arb_texture_rg)
            {
                (gl::RG8I, Some(gl::RG8I))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedIntegral(UncompressedIntFormat::I16I16)) => {
            if version >= &Version(Api::Gl, 3, 0) || (extensions.gl_ext_texture_integer &&
                                                      extensions.gl_arb_texture_rg)
            {
                (gl::RG16I, Some(gl::RG16I))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedIntegral(UncompressedIntFormat::I32I32)) => {
            if version >= &Version(Api::Gl, 3, 0) || (extensions.gl_ext_texture_integer &&
                                                      extensions.gl_arb_texture_rg)
            {
                (gl::RG32I, Some(gl::RG32I))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedIntegral(UncompressedIntFormat::I8I8I8)) => {
            if version >= &Version(Api::Gl, 3, 0) || extensions.gl_ext_texture_integer {
                (gl::RGB8I, Some(gl::RGB8I))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedIntegral(UncompressedIntFormat::I16I16I16)) => {
            if version >= &Version(Api::Gl, 3, 0) || extensions.gl_ext_texture_integer {
                (gl::RGB16I, Some(gl::RGB16I))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedIntegral(UncompressedIntFormat::I32I32I32)) => {
            if version >= &Version(Api::Gl, 3, 0) || extensions.gl_ext_texture_integer {
                (gl::RGB32I, Some(gl::RGB32I))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedIntegral(UncompressedIntFormat::I8I8I8I8)) => {
            if version >= &Version(Api::Gl, 3, 0) || extensions.gl_ext_texture_integer {
                (gl::RGBA8I, Some(gl::RGBA8I))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedIntegral(UncompressedIntFormat::I16I16I16I16)) => {
            if version >= &Version(Api::Gl, 3, 0) || extensions.gl_ext_texture_integer {
                (gl::RGBA16I, Some(gl::RGBA16I))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedIntegral(UncompressedIntFormat::I32I32I32I32)) => {
            if version >= &Version(Api::Gl, 3, 0) || extensions.gl_ext_texture_integer {
                (gl::RGBA32I, Some(gl::RGBA32I))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        /*******************************************************************/
        /*                          UNSIGNED                               */
        /*******************************************************************/
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


        TextureFormatRequest::Specific(TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U8)) => {
            if version >= &Version(Api::Gl, 3, 0) || (extensions.gl_ext_texture_integer &&
                                                      extensions.gl_arb_texture_rg)
            {
                (gl::R8UI, Some(gl::R8UI))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U16)) => {
            if version >= &Version(Api::Gl, 3, 0) || (extensions.gl_ext_texture_integer &&
                                                      extensions.gl_arb_texture_rg)
            {
                (gl::R16UI, Some(gl::R16UI))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U32)) => {
            if version >= &Version(Api::Gl, 3, 0) || (extensions.gl_ext_texture_integer &&
                                                      extensions.gl_arb_texture_rg)
            {
                (gl::R32UI, Some(gl::R32UI))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U8U8)) => {
            if version >= &Version(Api::Gl, 3, 0) || (extensions.gl_ext_texture_integer &&
                                                      extensions.gl_arb_texture_rg)
            {
                (gl::RG8UI, Some(gl::RG8UI))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U16U16)) => {
            if version >= &Version(Api::Gl, 3, 0) || (extensions.gl_ext_texture_integer &&
                                                      extensions.gl_arb_texture_rg)
            {
                (gl::RG16UI, Some(gl::RG16UI))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U32U32)) => {
            if version >= &Version(Api::Gl, 3, 0) || (extensions.gl_ext_texture_integer &&
                                                      extensions.gl_arb_texture_rg)
            {
                (gl::RG32UI, Some(gl::RG32UI))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U8U8U8)) => {
            if version >= &Version(Api::Gl, 3, 0) || extensions.gl_ext_texture_integer {
                (gl::RGB8UI, Some(gl::RGB8UI))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U16U16U16)) => {
            if version >= &Version(Api::Gl, 3, 0) || extensions.gl_ext_texture_integer {
                (gl::RGB16UI, Some(gl::RGB16UI))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U32U32U32)) => {
            if version >= &Version(Api::Gl, 3, 0) || extensions.gl_ext_texture_integer {
                (gl::RGB32UI, Some(gl::RGB32UI))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U8U8U8U8)) => {
            if version >= &Version(Api::Gl, 3, 0) || extensions.gl_ext_texture_integer {
                (gl::RGBA8UI, Some(gl::RGBA8UI))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U16U16U16U16)) => {
            if version >= &Version(Api::Gl, 3, 0) || extensions.gl_ext_texture_integer {
                (gl::RGBA16UI, Some(gl::RGBA16UI))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U32U32U32U32)) => {
            if version >= &Version(Api::Gl, 3, 0) || extensions.gl_ext_texture_integer {
                (gl::RGBA32UI, Some(gl::RGBA32UI))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U10U10U10U2)) => {
            if version >= &Version(Api::Gl, 3, 0) || extensions.gl_ext_texture_integer {
                (gl::RGB10_A2UI, Some(gl::RGB10_A2UI))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        /*******************************************************************/
        /*                            DEPTH                                */
        /*******************************************************************/
        TextureFormatRequest::AnyDepth => {
            if version >= &Version(Api::Gl, 2, 0) {
                (gl::DEPTH_COMPONENT, Some(gl::DEPTH_COMPONENT24))
            } else if version >= &Version(Api::Gl, 1, 4) || extensions.gl_arb_depth_texture ||
                      extensions.gl_oes_depth_texture
            {
                (gl::DEPTH_COMPONENT, None)     // TODO: sized format?
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::DepthFormat(DepthFormat::I16)) => {
            if version >= &Version(Api::Gl, 3, 0) || extensions.gl_arb_depth_texture {
                (gl::DEPTH_COMPONENT16, Some(gl::DEPTH_COMPONENT16))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::DepthFormat(DepthFormat::I24)) => {
            if version >= &Version(Api::Gl, 3, 0) || extensions.gl_arb_depth_texture {
                (gl::DEPTH_COMPONENT24, Some(gl::DEPTH_COMPONENT24))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::DepthFormat(DepthFormat::I32)) => {
            if version >= &Version(Api::Gl, 3, 0) || extensions.gl_arb_depth_texture {
                (gl::DEPTH_COMPONENT32, Some(gl::DEPTH_COMPONENT32))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::DepthFormat(DepthFormat::F32)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::DEPTH_COMPONENT32F, Some(gl::DEPTH_COMPONENT32F))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        /*******************************************************************/
        /*                           STENCIL                               */
        /*******************************************************************/
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

        TextureFormatRequest::Specific(TextureFormat::StencilFormat(_)) => {
            unimplemented!();
        },

        /*******************************************************************/
        /*                        DEPTH-STENCIL                            */
        /*******************************************************************/
        TextureFormatRequest::AnyDepthStencil => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::DEPTH_STENCIL, Some(gl::DEPTH24_STENCIL8))
            } else if extensions.gl_ext_packed_depth_stencil {
                (gl::DEPTH_STENCIL_EXT, Some(gl::DEPTH24_STENCIL8_EXT))
            } else if extensions.gl_oes_packed_depth_stencil {
                (gl::DEPTH_STENCIL_OES, Some(gl::DEPTH24_STENCIL8_OES))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::DepthStencilFormat(DepthStencilFormat::I24I8)) => {
            if version >= &Version(Api::Gl, 3, 0) || extensions.gl_ext_packed_depth_stencil ||
               extensions.gl_oes_packed_depth_stencil
            {
                (gl::DEPTH24_STENCIL8, Some(gl::DEPTH24_STENCIL8))
            } else {
                return Err(FormatNotSupportedError);
            }
        },

        TextureFormatRequest::Specific(TextureFormat::DepthStencilFormat(DepthStencilFormat::F32I8)) => {
            if version >= &Version(Api::Gl, 3, 0) {
                (gl::DEPTH32F_STENCIL8, Some(gl::DEPTH32F_STENCIL8))
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
        TextureFormatRequest::AnySrgb | TextureFormatRequest::AnyCompressedSrgb |
        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(_)) |
        TextureFormatRequest::Specific(TextureFormat::Srgb(_)) |
        TextureFormatRequest::Specific(TextureFormat::CompressedFormat(_)) |
        TextureFormatRequest::Specific(TextureFormat::CompressedSrgbFormat(_)) =>
        {
            match client {
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
        },

        TextureFormatRequest::AnyIntegral | TextureFormatRequest::AnyUnsigned |
        TextureFormatRequest::Specific(TextureFormat::UncompressedIntegral(_)) |
        TextureFormatRequest::Specific(TextureFormat::UncompressedUnsigned(_)) =>
        {
            match client {
                ClientFormat::U8 => (gl::RED_INTEGER, gl::UNSIGNED_BYTE),
                ClientFormat::U8U8 => (gl::RG_INTEGER, gl::UNSIGNED_BYTE),
                ClientFormat::U8U8U8 => (gl::RGB_INTEGER, gl::UNSIGNED_BYTE),
                ClientFormat::U8U8U8U8 => (gl::RGBA_INTEGER, gl::UNSIGNED_BYTE),
                ClientFormat::I8 => (gl::RED_INTEGER, gl::BYTE),
                ClientFormat::I8I8 => (gl::RG_INTEGER, gl::BYTE),
                ClientFormat::I8I8I8 => (gl::RGB_INTEGER, gl::BYTE),
                ClientFormat::I8I8I8I8 => (gl::RGBA_INTEGER, gl::BYTE),
                ClientFormat::U16 => (gl::RED_INTEGER, gl::UNSIGNED_SHORT),
                ClientFormat::U16U16 => (gl::RG_INTEGER, gl::UNSIGNED_SHORT),
                ClientFormat::U16U16U16 => (gl::RGB_INTEGER, gl::UNSIGNED_SHORT),
                ClientFormat::U16U16U16U16 => (gl::RGBA_INTEGER, gl::UNSIGNED_SHORT),
                ClientFormat::I16 => (gl::RED_INTEGER, gl::SHORT),
                ClientFormat::I16I16 => (gl::RG_INTEGER, gl::SHORT),
                ClientFormat::I16I16I16 => (gl::RGB_INTEGER, gl::SHORT),
                ClientFormat::I16I16I16I16 => (gl::RGBA_INTEGER, gl::SHORT),
                ClientFormat::U32 => (gl::RED_INTEGER, gl::UNSIGNED_INT),
                ClientFormat::U32U32 => (gl::RG_INTEGER, gl::UNSIGNED_INT),
                ClientFormat::U32U32U32 => (gl::RGB_INTEGER, gl::UNSIGNED_INT),
                ClientFormat::U32U32U32U32 => (gl::RGBA_INTEGER, gl::UNSIGNED_INT),
                ClientFormat::I32 => (gl::RED_INTEGER, gl::INT),
                ClientFormat::I32I32 => (gl::RG_INTEGER, gl::INT),
                ClientFormat::I32I32I32 => (gl::RGB_INTEGER, gl::INT),
                ClientFormat::I32I32I32I32 => (gl::RGBA_INTEGER, gl::INT),
                ClientFormat::U3U3U2 => (gl::RGB_INTEGER, gl::UNSIGNED_BYTE_3_3_2),
                ClientFormat::U5U6U5 => (gl::RGB_INTEGER, gl::UNSIGNED_SHORT_5_6_5),
                ClientFormat::U4U4U4U4 => (gl::RGBA_INTEGER, gl::UNSIGNED_SHORT_4_4_4_4),
                ClientFormat::U5U5U5U1 => (gl::RGBA_INTEGER, gl::UNSIGNED_SHORT_5_5_5_1),
                ClientFormat::U10U10U10U2 => (gl::RGBA_INTEGER, gl::UNSIGNED_INT_10_10_10_2),
                ClientFormat::F16 => (gl::RED_INTEGER, gl::HALF_FLOAT),
                ClientFormat::F16F16 => (gl::RG_INTEGER, gl::HALF_FLOAT),
                ClientFormat::F16F16F16 => (gl::RGB_INTEGER, gl::HALF_FLOAT),
                ClientFormat::F16F16F16F16 => (gl::RGBA_INTEGER, gl::HALF_FLOAT),
                ClientFormat::F32 => (gl::RED_INTEGER, gl::FLOAT),
                ClientFormat::F32F32 => (gl::RG_INTEGER, gl::FLOAT),
                ClientFormat::F32F32F32 => (gl::RGB_INTEGER, gl::FLOAT),
                ClientFormat::F32F32F32F32 => (gl::RGBA_INTEGER, gl::FLOAT),
            }
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
            match client {
                ClientFormat::U8 => (gl::RED_INTEGER, gl::UNSIGNED_BYTE),
                ClientFormat::I8 => (gl::RED_INTEGER, gl::BYTE),
                ClientFormat::U16 => (gl::RED_INTEGER, gl::UNSIGNED_SHORT),
                ClientFormat::I16 => (gl::RED_INTEGER, gl::SHORT),
                ClientFormat::U32 => (gl::RED_INTEGER, gl::UNSIGNED_INT),
                ClientFormat::I32 => (gl::RED_INTEGER, gl::INT),
                ClientFormat::F16 => (gl::RED_INTEGER, gl::HALF_FLOAT),
                ClientFormat::F32 => (gl::RED_INTEGER, gl::FLOAT),
                _ => panic!("Can't upload to a stencil texture with more than one channel")
            }
        }

        TextureFormatRequest::AnyDepthStencil |
        TextureFormatRequest::Specific(TextureFormat::DepthStencilFormat(_)) =>
        {
            unimplemented!();
        },
    }
}
