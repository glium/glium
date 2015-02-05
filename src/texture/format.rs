use gl;

use ToGlEnum;

/// List of client-side pixel formats.
///
/// These are all the possible formats of data when uploading to a texture.
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
            ClientFormat::U10U10U10U2 => (10 + 10 + 10 + 2) / 2,
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
