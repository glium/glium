use gl;

use ToGlEnum;

/// List of client-side pixel formats.
///
/// These are all the possible formats of data when uploading to a texture.
#[allow(missing_docs)]
#[deriving(Show, Clone, Copy, PartialEq, Eq)]
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
    pub fn get_size(&self) -> uint {
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

    /// Returns a (format, type) tuple.
    #[doc(hidden)]      // TODO: shouldn't be pub
    pub fn to_gl_enum(&self) -> (gl::types::GLenum, gl::types::GLenum) {
        match *self {
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

    /// Returns a (format, type) tuple corresponding to the "signed integer" format, if possible.
    #[doc(hidden)]      // TODO: shouldn't be pub
    pub fn to_gl_enum_int(&self) -> Option<(gl::types::GLenum, gl::types::GLenum)> {
        let (components, format) = self.to_gl_enum();

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

    /// Returns a (format, type) tuple corresponding to the "unsigned integer" format, if possible.
    #[doc(hidden)]      // TODO: shouldn't be pub
    pub fn to_gl_enum_uint(&self) -> Option<(gl::types::GLenum, gl::types::GLenum)> {
        let (components, format) = self.to_gl_enum();

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

    /// Returns the default corresponding floating-point-like internal format.
    pub fn to_float_internal_format(&self) -> Option<UncompressedFloatFormat> {
        match *self {
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

    /// Returns a GLenum corresponding to the default floating-point-like format corresponding
    /// to this client format.
    #[doc(hidden)]      // TODO: shouldn't be pub
    pub fn to_default_float_format(&self) -> gl::types::GLenum {
        self.to_float_internal_format()
            .map(|e| e.to_glenum())
            .unwrap_or_else(|| self.to_gl_enum().0)
    }

    /// Returns a GLenum corresponding to the default compressed format corresponding
    /// to this client format.
    #[doc(hidden)]      // TODO: shouldn't be pub
    pub fn to_default_compressed_format(&self) -> gl::types::GLenum {
        match *self {
            ClientFormat::U8 => gl::COMPRESSED_RED,
            ClientFormat::U8U8 => gl::COMPRESSED_RG,
            ClientFormat::U8U8U8 => gl::COMPRESSED_RGB,
            ClientFormat::U8U8U8U8 => gl::COMPRESSED_RGBA,
            ClientFormat::I8 => gl::COMPRESSED_RED,
            ClientFormat::I8I8 => gl::COMPRESSED_RG,
            ClientFormat::I8I8I8 => gl::COMPRESSED_RGB,
            ClientFormat::I8I8I8I8 => gl::COMPRESSED_RGBA,
            ClientFormat::U16 => gl::COMPRESSED_RED,
            ClientFormat::U16U16 => gl::COMPRESSED_RG,
            ClientFormat::U16U16U16 => gl::COMPRESSED_RGB,
            ClientFormat::U16U16U16U16 => gl::COMPRESSED_RGBA,
            ClientFormat::I16 => gl::COMPRESSED_RED,
            ClientFormat::I16I16 => gl::COMPRESSED_RG,
            ClientFormat::I16I16I16 => gl::COMPRESSED_RGB,
            ClientFormat::I16I16I16I16 => gl::COMPRESSED_RGBA,
            ClientFormat::U32 => gl::COMPRESSED_RED,
            ClientFormat::U32U32 => gl::COMPRESSED_RG,
            ClientFormat::U32U32U32 => gl::COMPRESSED_RGB,
            ClientFormat::U32U32U32U32 => gl::COMPRESSED_RGBA,
            ClientFormat::I32 => gl::COMPRESSED_RED,
            ClientFormat::I32I32 => gl::COMPRESSED_RG,
            ClientFormat::I32I32I32 => gl::COMPRESSED_RGB,
            ClientFormat::I32I32I32I32 => gl::COMPRESSED_RGBA,
            ClientFormat::U3U3U2 => gl::COMPRESSED_RGB,
            ClientFormat::U5U6U5 => gl::COMPRESSED_RGB,
            ClientFormat::U4U4U4U4 => gl::COMPRESSED_RGBA,
            ClientFormat::U5U5U5U1 => gl::COMPRESSED_RGBA,
            ClientFormat::U10U10U10U2 => gl::COMPRESSED_RGBA,
            ClientFormat::F16 => gl::COMPRESSED_RED,
            ClientFormat::F16F16 => gl::COMPRESSED_RG,
            ClientFormat::F16F16F16 => gl::COMPRESSED_RGB,
            ClientFormat::F16F16F16F16 => gl::COMPRESSED_RGBA,
            ClientFormat::F32 => gl::COMPRESSED_RED,
            ClientFormat::F32F32 => gl::COMPRESSED_RG,
            ClientFormat::F32F32F32 => gl::COMPRESSED_RGB,
            ClientFormat::F32F32F32F32 => gl::COMPRESSED_RGBA,
        }
    }
}

/// List of uncompressed pixel formats that contain floating points-like data.
///
/// Some formats are marked as "guaranteed to be supported". What this means is that you are
/// certain that the backend will use exactly these formats. If you try to use a format that
/// is not supported by the backend, it will automatically fall back to a larger format.
#[deriving(Show, Clone, Copy, PartialEq, Eq)]
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
#[deriving(Show, Clone, Copy, PartialEq, Eq)]
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
#[deriving(Show, Clone, Copy, PartialEq, Eq)]
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
#[deriving(Show, Clone, Copy, PartialEq, Eq)]
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
#[deriving(Show, Clone, Copy, PartialEq, Eq)]
pub enum DepthFormat {
    I16,
    I24,
    /// May not be supported by all hardwares.
    I32,
    F32,
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
#[deriving(Show, Clone, Copy, PartialEq, Eq)]
pub enum DepthStencilFormat {
    I24I8,
    F32I8,
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
#[deriving(Show, Clone, Copy, PartialEq, Eq)]
pub enum StencilFormat {
    I1,
    I4,
    I8,
    I16,
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
#[deriving(Show, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    /// 
    UncompressedFloat(UncompressedFloatFormat),
    /// 
    UncompressedIntegral(UncompressedIntFormat),
}
