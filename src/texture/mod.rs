/*!

A texture is an image available for drawing.

To create a texture, you must first create a struct that implements one of `Texture1dData`,
 `Texture2dData` or `Texture3dData`. Then call the appropriate `new` function of the type of
 texture that you desire.

The most common types of textures are `CompressedTexture2d` and `Texture2d` (the two dimensions
being the width and height), it is what you will use most of the time.

*/

use {gl, libc, framebuffer};

#[cfg(feature = "image")]
use image;

use std::{fmt, mem, ptr};
use std::sync::Arc;

use buffer::{mod, Buffer};
use context::GlVersion;
use uniforms::{UniformValue, UniformValueBinder};
use {Surface, GlObject};

pub use self::pixel::PixelValue;
pub use self::render_buffer::RenderBuffer;

mod pixel;
mod render_buffer;

include!(concat!(env!("OUT_DIR"), "/textures.rs"))

/// Trait that describes a texture.
pub trait Texture {
    /// Returns a reference to an opaque type necessary to make things work.
    #[experimental = "May be changed to something totally different"]
    fn get_implementation(&self) -> &TextureImplementation;

    /// Returns the width in pixels of the texture.
    fn get_width(&self) -> u32 {
        self.get_implementation().width
    }

    /// Returns the height in pixels of the texture, or `None` for one dimension textures.
    fn get_height(&self) -> Option<u32> {
        self.get_implementation().height.clone()
    }

    /// Returns the depth in pixels of the texture, or `None` for one or two dimension textures.
    fn get_depth(&self) -> Option<u32> {
        self.get_implementation().depth.clone()
    }

    /// Returns the number of textures in the array, or `None` for non-arrays.
    fn get_array_size(&self) -> Option<u32> {
        self.get_implementation().array_size.clone()
    }
}

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
    fn to_gl_enum_int(&self) -> Option<(gl::types::GLenum, gl::types::GLenum)> {
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
    fn to_gl_enum_uint(&self) -> Option<(gl::types::GLenum, gl::types::GLenum)> {
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
    fn to_default_float_format(&self) -> gl::types::GLenum {
        self.to_float_internal_format()
            .map(|e| e.to_gl_enum())
            .unwrap_or_else(|| self.to_gl_enum().0)
    }

    /// Returns a GLenum corresponding to the default compressed format corresponding
    /// to this client format.
    fn to_default_compressed_format(&self) -> gl::types::GLenum {
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

impl UncompressedFloatFormat {
    fn to_gl_enum(&self) -> gl::types::GLenum {
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

impl UncompressedIntFormat {
    fn to_gl_enum(&self) -> gl::types::GLenum {
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

impl UncompressedUintFormat {
    fn to_gl_enum(&self) -> gl::types::GLenum {
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

impl CompressedFormat {
    fn to_gl_enum(&self) -> gl::types::GLenum {
        match *self {
            CompressedFormat::RGTCFormatU => gl::COMPRESSED_RED_RGTC1,
            CompressedFormat::RGTCFormatI => gl::COMPRESSED_SIGNED_RED_RGTC1,
            CompressedFormat::RGTCFormatUU => gl::COMPRESSED_RG_RGTC2,
            CompressedFormat::RGTCFormatII => gl::COMPRESSED_SIGNED_RG_RGTC2,
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

/// Trait that describes data for a one-dimensional texture.
#[experimental = "Will be rewritten to use an associated type"]
pub trait Texture1dData<T> {
    /// Returns the format of the pixels.
    fn get_format(&self) -> ClientFormat;

    /// Returns a vec where each element is a pixel of the texture.
    fn into_vec(self) -> Vec<T>;

    /// Builds a new object from raw data.
    fn from_vec(Vec<T>) -> Self;
}

impl<P: PixelValue> Texture1dData<P> for Vec<P> {
    fn get_format(&self) -> ClientFormat {
        PixelValue::get_format(None::<P>)
    }

    fn into_vec(self) -> Vec<P> {
        self
    }

    fn from_vec(data: Vec<P>) -> Vec<P> {
        data
    }
}

impl<'a, P: PixelValue + Clone> Texture1dData<P> for &'a [P] {
    fn get_format(&self) -> ClientFormat {
        PixelValue::get_format(None::<P>)
    }

    fn into_vec(self) -> Vec<P> {
        self.to_vec()
    }

    fn from_vec(_: Vec<P>) -> &'a [P] {
        panic!()        // TODO: what to do here?
    }
}

/// Trait that describes data for a two-dimensional texture.
#[experimental = "Will be rewritten to use an associated type"]
pub trait Texture2dData<P> {
    /// Returns the format of the pixels.
    fn get_format(&self) -> ClientFormat;

    /// Returns the dimensions of the texture.
    fn get_dimensions(&self) -> (u32, u32);

    /// Returns a vec where each element is a pixel of the texture.
    fn into_vec(self) -> Vec<P>;

    /// Builds a new object from raw data.
    fn from_vec(Vec<P>, width: u32) -> Self;
}

impl<P: PixelValue + Clone> Texture2dData<P> for Vec<Vec<P>> {      // TODO: remove Clone
    fn get_format(&self) -> ClientFormat {
        PixelValue::get_format(None::<P>)
    }

    fn get_dimensions(&self) -> (u32, u32) {
        (self.iter().next().map(|e| e.len()).unwrap_or(0) as u32, self.len() as u32)
    }

    fn into_vec(self) -> Vec<P> {
        self.into_iter().flat_map(|e| e.into_iter()).collect()
    }

    fn from_vec(data: Vec<P>, width: u32) -> Vec<Vec<P>> {
        data.as_slice().chunks(width as uint).map(|e| e.to_vec()).collect()
    }
}

#[cfg(feature = "image")]
impl<T, P> Texture2dData<T> for image::ImageBuffer<Vec<T>, T, P> where T: image::Primitive + Send,
    P: PixelValue + image::Pixel<T> + Clone + Copy
{
    fn get_format(&self) -> ClientFormat {
        PixelValue::get_format(None::<P>)
    }

    fn get_dimensions(&self) -> (u32, u32) {
        use image::GenericImage;
        self.dimensions()
    }

    fn into_vec(self) -> Vec<T> {
        use image::GenericImage;
        let (width, _) = self.dimensions();

        let raw_data = self.into_vec();

        raw_data
            .as_slice()
            .chunks(width as uint * image::Pixel::channel_count(None::<&P>) as uint)
            .rev()
            .flat_map(|row| row.iter())
            .map(|p| p.clone())
            .collect()
    }

    fn from_vec(_: Vec<T>, _: u32) -> image::ImageBuffer<Vec<T>, T, P> {
        unimplemented!()        // TODO: 
    }
}

#[cfg(feature = "image")]
impl Texture2dData<u8> for image::DynamicImage {
    fn get_format(&self) -> ClientFormat {
        ClientFormat::U8U8U8U8
    }

    fn get_dimensions(&self) -> (u32, u32) {
        use image::GenericImage;
        self.dimensions()
    }

    fn into_vec(self) -> Vec<u8> {
        Texture2dData::into_vec(self.to_rgba())
    }

    fn from_vec(_: Vec<u8>, _: u32) -> image::DynamicImage {
        unimplemented!()        // TODO: 
    }
}

/// Trait that describes data for a three-dimensional texture.
#[experimental = "Will be rewritten to use an associated type"]
pub trait Texture3dData<P> {
    /// Returns the format of the pixels.
    fn get_format(&self) -> ClientFormat;

    /// Returns the dimensions of the texture.
    fn get_dimensions(&self) -> (u32, u32, u32);

    /// Returns a vec where each element is a pixel of the texture.
    fn into_vec(self) -> Vec<P>;

    /// Builds a new object from raw data.
    fn from_vec(Vec<P>, width: u32, height: u32) -> Self;
}

impl<P: PixelValue> Texture3dData<P> for Vec<Vec<Vec<P>>> {
    fn get_format(&self) -> ClientFormat {
        PixelValue::get_format(None::<P>)
    }

    fn get_dimensions(&self) -> (u32, u32, u32) {
        (self.iter().next().and_then(|e| e.iter().next()).map(|e| e.len()).unwrap_or(0) as u32,
            self.iter().next().map(|e| e.len()).unwrap_or(0) as u32, self.len() as u32)
    }

    fn into_vec(self) -> Vec<P> {
        self.into_iter().flat_map(|e| e.into_iter()).flat_map(|e| e.into_iter()).collect()
    }

    fn from_vec(data: Vec<P>, width: u32, height: u32) -> Vec<Vec<Vec<P>>> {
        unimplemented!()        // TODO: 
    }
}

/// Buffer that stores the content of a texture.
///
/// The generic type represents the type of pixels that the buffer contains.
///
/// **Note**: pixel buffers are unusable for the moment (they are not yet implemented).
pub struct PixelBuffer<T> {
    buffer: Buffer,
}

impl<T> PixelBuffer<T> where T: PixelValue {
    /// Builds a new buffer with an uninitialized content.
    pub fn new_empty(display: &super::Display, capacity: uint) -> PixelBuffer<T> {
        PixelBuffer {
            buffer: Buffer::new_empty::<buffer::PixelUnpackBuffer>(display, 1, capacity,
                                                                   gl::DYNAMIC_READ),
        }
    }

    /// Turns a `PixelBuffer<T>` into a `PixelBuffer<U>` without any check.
    pub unsafe fn transmute<U>(self) -> PixelBuffer<U> where U: PixelValue {
        PixelBuffer { buffer: self.buffer }
    }
}

/// Opaque type that is used to make things work.
pub struct TextureImplementation {
    display: Arc<super::DisplayImpl>,
    id: gl::types::GLuint,
    bind_point: gl::types::GLenum,
    width: u32,
    height: Option<u32>,
    depth: Option<u32>,
    array_size: Option<u32>,
}

impl TextureImplementation {
    /// Builds a new texture.
    fn new<P>(display: &super::Display, format: gl::types::GLenum, data: Option<Vec<P>>,
        client_format: gl::types::GLenum, client_type: gl::types::GLenum, width: u32,
        height: Option<u32>, depth: Option<u32>, array_size: Option<u32>) -> TextureImplementation
        where P: Send
    {
        if let Some(ref data) = data {
            if width as uint * height.unwrap_or(1) as uint * depth.unwrap_or(1) as uint *
                array_size.unwrap_or(1) as uint != data.len() &&
               width as uint * height.unwrap_or(1) as uint * depth.unwrap_or(1) as uint *
                array_size.unwrap_or(1) as uint * 2 != data.len() &&
               width as uint * height.unwrap_or(1) as uint * depth.unwrap_or(1) as uint *
                array_size.unwrap_or(1) as uint * 3 != data.len() &&
               width as uint * height.unwrap_or(1) as uint * depth.unwrap_or(1) as uint *
                array_size.unwrap_or(1) as uint * 4 != data.len()
            {
                panic!("Texture data size mismatch");
            }
        }

        let texture_type = if height.is_none() && depth.is_none() {
            if array_size.is_none() { gl::TEXTURE_1D } else { gl::TEXTURE_1D_ARRAY }
        } else if depth.is_none() {
            if array_size.is_none() { gl::TEXTURE_2D } else { gl::TEXTURE_2D_ARRAY }
        } else {
            gl::TEXTURE_3D
        };

        let (tx, rx) = channel();
        display.context.context.exec(move |: ctxt| {
            unsafe {
                let data = data;
                let data_raw = if let Some(ref data) = data {
                    data.as_ptr() as *const libc::c_void
                } else {
                    ptr::null()
                };

                ctxt.gl.PixelStorei(gl::UNPACK_ALIGNMENT, 1);

                if ctxt.state.pixel_unpack_buffer_binding != 0 {
                    ctxt.state.pixel_unpack_buffer_binding = 0;
                    ctxt.gl.BindBuffer(gl::PIXEL_UNPACK_BUFFER, 0);
                }

                let id: gl::types::GLuint = mem::uninitialized();
                ctxt.gl.GenTextures(1, mem::transmute(&id));

                ctxt.gl.BindTexture(texture_type, id);

                ctxt.gl.TexParameteri(texture_type, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
                if height.is_some() || depth.is_some() || array_size.is_some() {
                    ctxt.gl.TexParameteri(texture_type, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
                }
                if depth.is_some() || array_size.is_some() {
                    ctxt.gl.TexParameteri(texture_type, gl::TEXTURE_WRAP_R, gl::REPEAT as i32);
                }
                ctxt.gl.TexParameteri(texture_type, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                ctxt.gl.TexParameteri(texture_type, gl::TEXTURE_MIN_FILTER,
                    gl::LINEAR_MIPMAP_LINEAR as i32);

                if texture_type == gl::TEXTURE_3D || texture_type == gl::TEXTURE_2D_ARRAY {
                    ctxt.gl.TexImage3D(texture_type, 0, format as i32, width as i32,
                        height.unwrap() as i32,
                        if let Some(d) = depth { d } else { array_size.unwrap_or(1) } as i32, 0,
                        client_format as u32, client_type, data_raw);

                } else if texture_type == gl::TEXTURE_2D || texture_type == gl::TEXTURE_1D_ARRAY {
                    ctxt.gl.TexImage2D(texture_type, 0, format as i32, width as i32,
                        height.unwrap() as i32, 0, client_format as u32, client_type, data_raw);
                } else {
                    ctxt.gl.TexImage1D(texture_type, 0, format as i32, width as i32, 0,
                        client_format as u32, client_type, data_raw);
                }

                if ctxt.version >= &GlVersion(3, 0) {
                    ctxt.gl.GenerateMipmap(texture_type);
                } else {
                    ctxt.gl.GenerateMipmapEXT(texture_type);
                }

                tx.send(id);
            }
        });

        TextureImplementation {
            display: display.context.clone(),
            id: rx.recv(),
            bind_point: texture_type,
            width: width,
            height: height,
            depth: depth,
            array_size: array_size,
        }
    }

    /// Reads the content of a mipmap level of the texture.
    // TODO: this function only works for level 0 right now
    //       width/height need adjustements
    #[cfg(feature = "gl_extensions")]
    fn read<P>(&self, level: u32) -> Vec<P> where P: PixelValue {
        assert_eq!(level, 0);   // TODO: 

        let pixels_count = (self.width * self.height.unwrap_or(1) * self.depth.unwrap_or(1))
                            as uint;

        let (format, gltype) = PixelValue::get_format(None::<P>).to_gl_enum();
        let my_id = self.id;

        let (tx, rx) = channel();
        self.display.context.exec(move |: ctxt| {
            unsafe {
                let mut data: Vec<P> = Vec::with_capacity(pixels_count);

                ctxt.gl.PixelStorei(gl::PACK_ALIGNMENT, 1);

                if ctxt.version >= &GlVersion(4, 5) {
                    ctxt.gl.GetTextureImage(my_id, level as gl::types::GLint, format, gltype,
                        (pixels_count * mem::size_of::<P>()) as gl::types::GLsizei,
                        data.as_mut_ptr() as *mut libc::c_void);

                } else if ctxt.extensions.gl_ext_direct_state_access {
                    ctxt.gl.GetTextureImageEXT(my_id, gl::TEXTURE_2D, level as gl::types::GLint,
                        format, gltype, data.as_mut_ptr() as *mut libc::c_void);

                } else {
                    ctxt.gl.BindTexture(gl::TEXTURE_2D, my_id);
                    ctxt.gl.GetTexImage(gl::TEXTURE_2D, level as gl::types::GLint, format, gltype,
                        data.as_mut_ptr() as *mut libc::c_void);
                }

                data.set_len(pixels_count);
                tx.send(data);
            }
        });

        rx.recv()
    }
}

impl GlObject for TextureImplementation {
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl fmt::Show for TextureImplementation {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        (format!("Texture #{} (dimensions: {}x{}x{})", self.id,
            self.width, self.height, self.depth)).fmt(formatter)
    }
}

impl Drop for TextureImplementation {
    fn drop(&mut self) {
        // removing FBOs which contain this texture
        {
            let mut fbos = self.display.framebuffer_objects.lock();
            let to_delete = fbos.keys().filter(|b| b.colors.iter().find(|&id| id == &self.id).is_some())
                .map(|k| k.clone()).collect::<Vec<_>>();
            for k in to_delete.into_iter() {
                fbos.remove(&k);
            }
        }

        let id = self.id.clone();
        self.display.context.exec(move |: ctxt| {
            unsafe { ctxt.gl.DeleteTextures(1, [ id ].as_ptr()); }
        });
    }
}

/// Struct that allows you to draw on a texture.
///
/// To obtain such an object, call `texture.as_surface()`.
pub struct TextureSurface<'a>(framebuffer::FrameBuffer<'a>);

impl<'a> Surface for TextureSurface<'a> {
    fn clear_color(&mut self, red: f32, green: f32, blue: f32, alpha: f32) {
        self.0.clear_color(red, green, blue, alpha)
    }

    fn clear_depth(&mut self, value: f32) {
        self.0.clear_depth(value)
    }

    fn clear_stencil(&mut self, value: int) {
        self.0.clear_stencil(value)
    }

    fn get_dimensions(&self) -> (uint, uint) {
        self.0.get_dimensions()
    }

    fn get_depth_buffer_bits(&self) -> Option<u16> {
        self.0.get_depth_buffer_bits()
    }

    fn get_stencil_buffer_bits(&self) -> Option<u16> {
        self.0.get_stencil_buffer_bits()
    }

    fn draw<V, I, U>(&mut self, vb: &::VertexBuffer<V>, ib: &I, program: &::Program,
        uniforms: &U, draw_parameters: &::DrawParameters) where I: ::IndicesSource,
        U: ::uniforms::Uniforms
    {
        self.0.draw(vb, ib, program, uniforms, draw_parameters)
    }

    fn get_blit_helper(&self) -> ::BlitHelper {
        self.0.get_blit_helper()
    }
}
