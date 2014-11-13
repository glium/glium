/*!

A texture is an image available for drawing.

To create a texture, you must first create a struct that implements one of `Texture1dData`,
 `Texture2dData` or `Texture3dData`. Then call the appropriate `new` function of the type of
 texture that you desire.

The most common type of texture is a `Texture2d` (the two dimensions being the width and height),
 it is what you will use most of the time.

**Note**: `TextureCube` does not yet exist.

*/

use {gl, libc, framebuffer};

#[cfg(feature = "image")]
use image;

use buffer::{mod, Buffer};
use context::GlVersion;
use Surface;

use std::{fmt, mem, ptr};
use std::sync::Arc;

pub use self::pixel::PixelValue;
pub use self::render_buffer::RenderBuffer;
pub use self::textures::{Texture1d, Texture2d, Texture3d};
pub use self::textures::{Texture1dArray, Texture2dArray};
pub use self::textures::{CompressedTexture2d};

mod pixel;
mod render_buffer;
mod textures;

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
pub enum ClientFormat {
    ClientFormatU8,
    ClientFormatU8U8,
    ClientFormatU8U8U8,
    ClientFormatU8U8U8U8,
    ClientFormatI8,
    ClientFormatI8I8,
    ClientFormatI8I8I8,
    ClientFormatI8I8I8I8,
    ClientFormatU16,
    ClientFormatU16U16,
    ClientFormatU16U16U16,
    ClientFormatU16U16U16U16,
    ClientFormatI16,
    ClientFormatI16I16,
    ClientFormatI16I16I16,
    ClientFormatI16I16I16I16,
    ClientFormatU32,
    ClientFormatU32U32,
    ClientFormatU32U32U32,
    ClientFormatU32U32U32U32,
    ClientFormatI32,
    ClientFormatI32I32,
    ClientFormatI32I32I32,
    ClientFormatI32I32I32I32,
    ClientFormatU3U3U2,
    ClientFormatU5U6U5,
    ClientFormatU4U4U4U4,
    ClientFormatU5U5U5U1,
    ClientFormatU10U10U10U2,
    ClientFormatF16,
    ClientFormatF16F16,
    ClientFormatF16F16F16,
    ClientFormatF16F16F16F16,
    ClientFormatF32,
    ClientFormatF32F32,
    ClientFormatF32F32F32,
    ClientFormatF32F32F32F32,
}

impl ClientFormat {
    /// Returns a (format, type) tuple.
    #[doc(hidden)]      // TODO: shouldn't be pub
    pub fn to_gl_enum(&self) -> (gl::types::GLenum, gl::types::GLenum) {
        match *self {
            ClientFormatU8 => (gl::RED, gl::UNSIGNED_BYTE),
            ClientFormatU8U8 => (gl::RG, gl::UNSIGNED_BYTE),
            ClientFormatU8U8U8 => (gl::RGB, gl::UNSIGNED_BYTE),
            ClientFormatU8U8U8U8 => (gl::RGBA, gl::UNSIGNED_BYTE),
            ClientFormatI8 => (gl::RED, gl::BYTE),
            ClientFormatI8I8 => (gl::RG, gl::BYTE),
            ClientFormatI8I8I8 => (gl::RGB, gl::BYTE),
            ClientFormatI8I8I8I8 => (gl::RGBA, gl::BYTE),
            ClientFormatU16 => (gl::RED, gl::UNSIGNED_SHORT),
            ClientFormatU16U16 => (gl::RG, gl::UNSIGNED_SHORT),
            ClientFormatU16U16U16 => (gl::RGB, gl::UNSIGNED_SHORT),
            ClientFormatU16U16U16U16 => (gl::RGBA, gl::UNSIGNED_SHORT),
            ClientFormatI16 => (gl::RED, gl::SHORT),
            ClientFormatI16I16 => (gl::RG, gl::SHORT),
            ClientFormatI16I16I16 => (gl::RGB, gl::SHORT),
            ClientFormatI16I16I16I16 => (gl::RGBA, gl::SHORT),
            ClientFormatU32 => (gl::RED, gl::UNSIGNED_INT),
            ClientFormatU32U32 => (gl::RG, gl::UNSIGNED_INT),
            ClientFormatU32U32U32 => (gl::RGB, gl::UNSIGNED_INT),
            ClientFormatU32U32U32U32 => (gl::RGBA, gl::UNSIGNED_INT),
            ClientFormatI32 => (gl::RED, gl::INT),
            ClientFormatI32I32 => (gl::RG, gl::INT),
            ClientFormatI32I32I32 => (gl::RGB, gl::INT),
            ClientFormatI32I32I32I32 => (gl::RGBA, gl::INT),
            ClientFormatU3U3U2 => (gl::RGB, gl::UNSIGNED_BYTE_3_3_2),
            ClientFormatU5U6U5 => (gl::RGB, gl::UNSIGNED_SHORT_5_6_5),
            ClientFormatU4U4U4U4 => (gl::RGBA, gl::UNSIGNED_SHORT_4_4_4_4),
            ClientFormatU5U5U5U1 => (gl::RGBA, gl::UNSIGNED_SHORT_5_5_5_1),
            ClientFormatU10U10U10U2 => (gl::RGBA, gl::UNSIGNED_INT_10_10_10_2),
            ClientFormatF16 => (gl::RED, gl::HALF_FLOAT),
            ClientFormatF16F16 => (gl::RG, gl::HALF_FLOAT),
            ClientFormatF16F16F16 => (gl::RGB, gl::HALF_FLOAT),
            ClientFormatF16F16F16F16 => (gl::RGBA, gl::HALF_FLOAT),
            ClientFormatF32 => (gl::RED, gl::FLOAT),
            ClientFormatF32F32 => (gl::RG, gl::FLOAT),
            ClientFormatF32F32F32 => (gl::RGB, gl::FLOAT),
            ClientFormatF32F32F32F32 => (gl::RGBA, gl::FLOAT),
        }
    }

    /// Returns the default corresponding floating-point-like internal format.
    pub fn to_float_internal_format(&self) -> Option<UncompressedFloatFormat> {
        match *self {
            ClientFormatU8 => Some(FloatFormatU8),
            ClientFormatU8U8 => Some(FloatFormatU8U8),
            ClientFormatU8U8U8 => Some(FloatFormatU8U8U8),
            ClientFormatU8U8U8U8 => Some(FloatFormatU8U8U8U8),
            ClientFormatI8 => Some(FloatFormatI8),
            ClientFormatI8I8 => Some(FloatFormatI8I8),
            ClientFormatI8I8I8 => Some(FloatFormatI8I8I8),
            ClientFormatI8I8I8I8 => Some(FloatFormatI8I8I8I8),
            ClientFormatU16 => Some(FloatFormatU16),
            ClientFormatU16U16 => Some(FloatFormatU16U16),
            ClientFormatU16U16U16 => None,
            ClientFormatU16U16U16U16 => Some(FloatFormatU16U16U16U16),
            ClientFormatI16 => Some(FloatFormatI16),
            ClientFormatI16I16 => Some(FloatFormatI16I16),
            ClientFormatI16I16I16 => Some(FloatFormatI16I16I16),
            ClientFormatI16I16I16I16 => None,
            ClientFormatU32 => None,
            ClientFormatU32U32 => None,
            ClientFormatU32U32U32 => None,
            ClientFormatU32U32U32U32 => None,
            ClientFormatI32 => None,
            ClientFormatI32I32 => None,
            ClientFormatI32I32I32 => None,
            ClientFormatI32I32I32I32 => None,
            ClientFormatU3U3U2 => None,
            ClientFormatU5U6U5 => None,
            ClientFormatU4U4U4U4 => Some(FloatFormatU4U4U4U4),
            ClientFormatU5U5U5U1 => Some(FloatFormatU5U5U5U1),
            ClientFormatU10U10U10U2 => Some(FloatFormatU10U10U10U2),
            ClientFormatF16 => Some(FloatFormatF16),
            ClientFormatF16F16 => Some(FloatFormatF16F16),
            ClientFormatF16F16F16 => Some(FloatFormatF16F16F16),
            ClientFormatF16F16F16F16 => Some(FloatFormatF16F16F16F16),
            ClientFormatF32 => Some(FloatFormatF32),
            ClientFormatF32F32 => Some(FloatFormatF32F32),
            ClientFormatF32F32F32 => Some(FloatFormatF32F32F32),
            ClientFormatF32F32F32F32 => Some(FloatFormatF32F32F32F32),
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
            ClientFormatU8 => gl::COMPRESSED_RED,
            ClientFormatU8U8 => gl::COMPRESSED_RG,
            ClientFormatU8U8U8 => gl::COMPRESSED_RGB,
            ClientFormatU8U8U8U8 => gl::COMPRESSED_RGBA,
            ClientFormatI8 => gl::COMPRESSED_RED,
            ClientFormatI8I8 => gl::COMPRESSED_RG,
            ClientFormatI8I8I8 => gl::COMPRESSED_RGB,
            ClientFormatI8I8I8I8 => gl::COMPRESSED_RGBA,
            ClientFormatU16 => gl::COMPRESSED_RED,
            ClientFormatU16U16 => gl::COMPRESSED_RG,
            ClientFormatU16U16U16 => gl::COMPRESSED_RGB,
            ClientFormatU16U16U16U16 => gl::COMPRESSED_RGBA,
            ClientFormatI16 => gl::COMPRESSED_RED,
            ClientFormatI16I16 => gl::COMPRESSED_RG,
            ClientFormatI16I16I16 => gl::COMPRESSED_RGB,
            ClientFormatI16I16I16I16 => gl::COMPRESSED_RGBA,
            ClientFormatU32 => gl::COMPRESSED_RED,
            ClientFormatU32U32 => gl::COMPRESSED_RG,
            ClientFormatU32U32U32 => gl::COMPRESSED_RGB,
            ClientFormatU32U32U32U32 => gl::COMPRESSED_RGBA,
            ClientFormatI32 => gl::COMPRESSED_RED,
            ClientFormatI32I32 => gl::COMPRESSED_RG,
            ClientFormatI32I32I32 => gl::COMPRESSED_RGB,
            ClientFormatI32I32I32I32 => gl::COMPRESSED_RGBA,
            ClientFormatU3U3U2 => gl::COMPRESSED_RGB,
            ClientFormatU5U6U5 => gl::COMPRESSED_RGB,
            ClientFormatU4U4U4U4 => gl::COMPRESSED_RGBA,
            ClientFormatU5U5U5U1 => gl::COMPRESSED_RGBA,
            ClientFormatU10U10U10U2 => gl::COMPRESSED_RGBA,
            ClientFormatF16 => gl::COMPRESSED_RED,
            ClientFormatF16F16 => gl::COMPRESSED_RG,
            ClientFormatF16F16F16 => gl::COMPRESSED_RGB,
            ClientFormatF16F16F16F16 => gl::COMPRESSED_RGBA,
            ClientFormatF32 => gl::COMPRESSED_RED,
            ClientFormatF32F32 => gl::COMPRESSED_RG,
            ClientFormatF32F32F32 => gl::COMPRESSED_RGB,
            ClientFormatF32F32F32F32 => gl::COMPRESSED_RGBA,
        }
    }
}

/// List of uncompressed pixel formats that contain floating points-like data.
///
/// Some formats are marked as "guaranteed to be supported". What this means is that you are
/// certain that the backend will use exactly these formats. If you try to use a format that
/// is not supported by the backend, it will automatically fall back to a larger format.
pub enum UncompressedFloatFormat {
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    FloatFormatU8,
    /// 
    ///
    /// Guaranteed to be supported for textures.
    FloatFormatI8,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    FloatFormatU16,
    /// 
    ///
    /// Guaranteed to be supported for textures.
    FloatFormatI16,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    FloatFormatU8U8,
    /// 
    ///
    /// Guaranteed to be supported for textures.
    FloatFormatI8I8,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    FloatFormatU16U16,
    /// 
    ///
    /// Guaranteed to be supported for textures.
    FloatFormatI16I16,
    /// 
    FloatFormatU3U32U,
    /// 
    FloatFormatU4U4U4,
    /// 
    FloatFormatU5U5U5,
    /// 
    ///
    /// Guaranteed to be supported for textures.
    FloatFormatU8U8U8,
    /// 
    ///
    /// Guaranteed to be supported for textures.
    FloatFormatI8I8I8,
    /// 
    FloatFormatU10U10U10,
    /// 
    FloatFormatU12U12U12,
    /// 
    ///
    /// Guaranteed to be supported for textures.
    FloatFormatI16I16I16,
    /// 
    FloatFormatU2U2U2U2,
    /// 
    FloatFormatU4U4U4U4,
    /// 
    FloatFormatU5U5U5U1,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    FloatFormatU8U8U8U8,
    /// 
    ///
    /// Guaranteed to be supported for textures.
    FloatFormatI8I8I8I8,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    FloatFormatU10U10U10U2,
    /// 
    FloatFormatU12U12U12U12,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    FloatFormatU16U16U16U16,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    FloatFormatF16,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    FloatFormatF16F16,
    /// 
    ///
    /// Guaranteed to be supported for textures.
    FloatFormatF16F16F16,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    FloatFormatF16F16F16F16,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    FloatFormatF32,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    FloatFormatF32F32,
    /// 
    ///
    /// Guaranteed to be supported for textures.
    FloatFormatF32F32F32,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    FloatFormatF32F32F32F32,
    /// 
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    FloatFormatF11F11F10,
    /// Uses three components of 9 bits of precision that all share the same exponent.
    ///
    /// Use this format only if all the components are approximately equal.
    ///
    /// Guaranteed to be supported for textures.
    FloatFormatF9F9F9,
}

impl UncompressedFloatFormat {
    fn to_gl_enum(&self) -> gl::types::GLenum {
        match *self {
            FloatFormatU8 => gl::R8,
            FloatFormatI8 => gl::R8_SNORM,
            FloatFormatU16 => gl::R16,
            FloatFormatI16 => gl::R16_SNORM,
            FloatFormatU8U8 => gl::RG8,
            FloatFormatI8I8 => gl::RG8_SNORM,
            FloatFormatU16U16 => gl::RG16,
            FloatFormatI16I16 => gl::RG16_SNORM,
            FloatFormatU3U32U => gl::R3_G3_B2,
            FloatFormatU4U4U4 => gl::RGB4,
            FloatFormatU5U5U5 => gl::RGB5,
            FloatFormatU8U8U8 => gl::RGB8,
            FloatFormatI8I8I8 => gl::RGB8_SNORM,
            FloatFormatU10U10U10 => gl::RGB10,
            FloatFormatU12U12U12 => gl::RGB12,
            FloatFormatI16I16I16 => gl::RGB16_SNORM,
            FloatFormatU2U2U2U2 => gl::RGBA2,
            FloatFormatU4U4U4U4 => gl::RGBA4,
            FloatFormatU5U5U5U1 => gl::RGB5_A1,
            FloatFormatU8U8U8U8 => gl::RGBA8,
            FloatFormatI8I8I8I8 => gl::RGBA8_SNORM,
            FloatFormatU10U10U10U2 => gl::RGB10_A2,
            FloatFormatU12U12U12U12 => gl::RGBA12,
            FloatFormatU16U16U16U16 => gl::RGBA16,
            FloatFormatF16 => gl::R16F,
            FloatFormatF16F16 => gl::RG16F,
            FloatFormatF16F16F16 => gl::RGB16F,
            FloatFormatF16F16F16F16 => gl::RGBA16F,
            FloatFormatF32 => gl::R32F,
            FloatFormatF32F32 => gl::RG32F,
            FloatFormatF32F32F32 => gl::RGB32F,
            FloatFormatF32F32F32F32 => gl::RGBA32F,
            FloatFormatF11F11F10 => gl::R11F_G11F_B10F,
            FloatFormatF9F9F9 => gl::RGB9_E5,
        }
    }
}

/// List of uncompressed pixel formats that contain integral data.
#[allow(missing_docs)]
pub enum UncompressedIntegralFormat {
    IntegralFormatI8,
    IntegralFormatU8,
    IntegralFormatI16,
    IntegralFormatU16,
    IntegralFormatI32,
    IntegralFormatU32,
    IntegralFormatI8I8,
    IntegralFormatU8U8,
    IntegralFormatI16I16,
    IntegralFormatU16U16,
    IntegralFormatI32I32,
    IntegralFormatU32U32,
    IntegralFormatI8I8I8,
    IntegralFormatU8U8U8,
    /// May not be supported by renderbuffers.
    IntegralFormatI16I16I16,
    /// May not be supported by renderbuffers.
    IntegralFormatU16U16U16,
    /// May not be supported by renderbuffers.
    IntegralFormatI32I32I32,
    /// May not be supported by renderbuffers.
    IntegralFormatU32U32U32,
    /// May not be supported by renderbuffers.
    IntegralFormatI8I8I8I8,
    /// May not be supported by renderbuffers.
    IntegralFormatU8U8U8U8,
    IntegralFormatI16I16I16I16,
    IntegralFormatU16U16U16U16,
    IntegralFormatI32I32I32I32,
    IntegralFormatU32U32U32U32,
    IntegralFormatU10U10U10U2,
}

impl UncompressedIntegralFormat {
    fn to_gl_enum(&self) -> gl::types::GLenum {
        match *self {
            IntegralFormatI8 => gl::R8I,
            IntegralFormatU8 => gl::R8UI,
            IntegralFormatI16 => gl::R16I,
            IntegralFormatU16 => gl::R16UI,
            IntegralFormatI32 => gl::R32I,
            IntegralFormatU32 => gl::R32UI,
            IntegralFormatI8I8 => gl::RG8I,
            IntegralFormatU8U8 => gl::RG8UI,
            IntegralFormatI16I16 => gl::RG16I,
            IntegralFormatU16U16 => gl::RG16UI,
            IntegralFormatI32I32 => gl::RG32I,
            IntegralFormatU32U32 => gl::RG32UI,
            IntegralFormatI8I8I8 => gl::RGB8I,
            IntegralFormatU8U8U8 => gl::RGB8UI,
            IntegralFormatI16I16I16 => gl::RGB16I,
            IntegralFormatU16U16U16 => gl::RGB16UI,
            IntegralFormatI32I32I32 => gl::RGB32I,
            IntegralFormatU32U32U32 => gl::RGB32UI,
            IntegralFormatI8I8I8I8 => gl::RGBA8I,
            IntegralFormatU8U8U8U8 => gl::RGBA8UI,
            IntegralFormatI16I16I16I16 => gl::RGBA16I,
            IntegralFormatU16U16U16U16 => gl::RGBA16UI,
            IntegralFormatI32I32I32I32 => gl::RGBA32I,
            IntegralFormatU32U32U32U32 => gl::RGBA32UI,
            IntegralFormatU10U10U10U2 => gl::RGB10_A2UI,
        }
    }
}

/// List of compressed texture formats.
///
/// TODO: many formats are missing
pub enum CompressedFormat {
    /// Red/green compressed texture with one unsigned component.
    CompressedRGTCFormatU,
    /// Red/green compressed texture with one signed component.
    CompressedRGTCFormatI,
    /// Red/green compressed texture with two unsigned components.
    CompressedRGTCFormatUU,
    /// Red/green compressed texture with two signed components.
    CompressedRGTCFormatII,
}

impl CompressedFormat {
    fn to_gl_enum(&self) -> gl::types::GLenum {
        match *self {
            CompressedRGTCFormatU => gl::COMPRESSED_RED_RGTC1,
            CompressedRGTCFormatI => gl::COMPRESSED_SIGNED_RED_RGTC1,
            CompressedRGTCFormatUU => gl::COMPRESSED_RG_RGTC2,
            CompressedRGTCFormatII => gl::COMPRESSED_SIGNED_RG_RGTC2,
        }
    }
}

/// Format of the internal representation of a texture.
pub enum TextureFormat {
    /// 
    UncompressedFloat(UncompressedFloatFormat),
    /// 
    UncompressedIntegral(UncompressedIntegralFormat),
}

/// Trait that describes data for a one-dimensional texture.
#[experimental = "Will be rewritten to use an associated type"]
pub trait Texture1dData<P> {
    /// Returns a vec where each element is a pixel of the texture.
    fn into_vec(self) -> Vec<P>;
}

impl<P: PixelValue> Texture1dData<P> for Vec<P> {
    fn into_vec(self) -> Vec<P> {
        self
    }
}

impl<'a, P: PixelValue + Clone> Texture1dData<P> for &'a [P] {
    fn into_vec(self) -> Vec<P> {
        self.to_vec()
    }
}

/// Trait that describes data for a two-dimensional texture.
#[experimental = "Will be rewritten to use an associated type"]
pub trait Texture2dData<P> {
    /// Returns the dimensions of the texture.
    fn get_dimensions(&self) -> (u32, u32);

    /// Returns a vec where each element is a pixel of the texture.
    fn into_vec(self) -> Vec<P>;

    /// Builds a new object from raw data.
    fn from_vec(Vec<P>, width: u32) -> Self;
}

impl<P: PixelValue + Clone> Texture2dData<P> for Vec<Vec<P>> {      // TODO: remove Clone
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
impl<T, P> Texture2dData<P> for image::ImageBuf<P> where T: Primitive, P: PixelValue +
    image::Pixel<T> + Clone + Copy
{
    fn get_dimensions(&self) -> (u32, u32) {
        use image::GenericImage;
        self.dimensions()
    }

    fn into_vec(self) -> Vec<P> {
        use image::GenericImage;
        let (width, _) = self.dimensions();

        let raw_data = self.into_vec();

        raw_data.as_slice().chunks(width as uint).rev().flat_map(|l| l.iter())
            .map(|l| l.clone()).collect()
    }

    fn from_vec(_: Vec<P>, _: u32) -> image::ImageBuf<P> {
        unimplemented!()
    }
}

#[cfg(feature = "image")]
impl Texture2dData<image::Rgba<u8>> for image::DynamicImage {
    fn get_dimensions(&self) -> (u32, u32) {
        use image::GenericImage;
        self.dimensions()
    }

    fn into_vec(self) -> Vec<image::Rgba<u8>> {
        self.to_rgba().into_vec()
    }

    fn from_vec(_: Vec<image::Rgba<u8>>, _: u32) -> image::DynamicImage {
        unimplemented!()
    }
}

/// Trait that describes data for a three-dimensional texture.
#[experimental = "Will be rewritten to use an associated type"]
pub trait Texture3dData<P> {
    /// Returns the dimensions of the texture.
    fn get_dimensions(&self) -> (u32, u32, u32);

    /// Returns a vec where each element is a pixel of the texture.
    fn into_vec(self) -> Vec<P>;
}

impl<P: PixelValue> Texture3dData<P> for Vec<Vec<Vec<P>>> {
    fn get_dimensions(&self) -> (u32, u32, u32) {
        (self.iter().next().and_then(|e| e.iter().next()).map(|e| e.len()).unwrap_or(0) as u32,
            self.iter().next().map(|e| e.len()).unwrap_or(0) as u32, self.len() as u32)
    }

    fn into_vec(self) -> Vec<P> {
        self.into_iter().flat_map(|e| e.into_iter()).flat_map(|e| e.into_iter()).collect()
    }
}

/// Buffer that stores the content of a texture.
///
/// The generic type represents the texture type that the buffer contains.
///
/// **Note**: pixel buffers are unusable for the moment (they are not yet implemented).
pub struct PixelBuffer<T> {
    buffer: Buffer,
}

impl<T> PixelBuffer<T> {
    /// Builds a new buffer with an uninitialized content.
    pub fn new_empty(display: &super::Display, capacity: uint) -> PixelBuffer<()> {
        PixelBuffer {
            buffer: Buffer::new_empty::<buffer::PixelUnpackBuffer>(display, 1, capacity,
                gl::DYNAMIC_READ),
        }
    }

    /// Turns a `PixelBuffer<T>` into a `PixelBuffer<U>` without any check.
    pub unsafe fn transmute<U>(self) -> PixelBuffer<U> {
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

/// This function is not visible outside of `glium`.
#[doc(hidden)]
pub fn get_id(texture: &TextureImplementation) -> gl::types::GLuint {
    texture.id
}

impl TextureImplementation {
    /// Builds a new texture.
    fn new<P: PixelValue>(display: &super::Display, format: gl::types::GLenum,
        data: Option<Vec<P>>, width: u32, height: Option<u32>, depth: Option<u32>,
        array_size: Option<u32>) -> TextureImplementation
    {
        if let Some(ref data) = data {
            if width as uint * height.unwrap_or(1) as uint * depth.unwrap_or(1) as uint *
                array_size.unwrap_or(1) as uint != data.len()
            {
                panic!("Texture data has different size from \
                        width * height * depth * array_size * elemLen");
            }
        }

        let texture_type = if height.is_none() && depth.is_none() {
            if array_size.is_none() { gl::TEXTURE_1D } else { gl::TEXTURE_1D_ARRAY }
        } else if depth.is_none() {
            if array_size.is_none() { gl::TEXTURE_2D } else { gl::TEXTURE_2D_ARRAY }
        } else {
            gl::TEXTURE_3D
        };

        let (client_format, client_type) = PixelValue::get_format(None::<P>).to_gl_enum();

        let (tx, rx) = channel();
        display.context.context.exec(proc(gl, state, version, _) {
            unsafe {
                let data = data;
                let data_raw: *const libc::c_void = match data {
                    Some(data) => mem::transmute(data.as_slice().as_ptr()),
                    None => ptr::null(),
                };

                gl.PixelStorei(gl::UNPACK_ALIGNMENT, if width % 4 == 0 {
                    4
                } else if height.unwrap_or(1) % 2 == 0 {
                    2
                } else {
                    1
                });

                if state.pixel_unpack_buffer_binding.is_some() {
                    state.pixel_unpack_buffer_binding = None;
                    gl.BindBuffer(gl::PIXEL_UNPACK_BUFFER, 0);
                }

                let id: gl::types::GLuint = mem::uninitialized();
                gl.GenTextures(1, mem::transmute(&id));

                gl.BindTexture(texture_type, id);

                gl.TexParameteri(texture_type, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
                if height.is_some() || depth.is_some() || array_size.is_some() {
                    gl.TexParameteri(texture_type, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
                }
                if depth.is_some() || array_size.is_some() {
                    gl.TexParameteri(texture_type, gl::TEXTURE_WRAP_R, gl::REPEAT as i32);
                }
                gl.TexParameteri(texture_type, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                gl.TexParameteri(texture_type, gl::TEXTURE_MIN_FILTER,
                    gl::LINEAR_MIPMAP_LINEAR as i32);

                if texture_type == gl::TEXTURE_3D || texture_type == gl::TEXTURE_2D_ARRAY {
                    gl.TexImage3D(texture_type, 0, format as i32, width as i32,
                        height.unwrap() as i32,
                        if let Some(d) = depth { d } else { array_size.unwrap_or(1) } as i32, 0,
                        client_format as u32, client_type, data_raw);

                } else if texture_type == gl::TEXTURE_2D || texture_type == gl::TEXTURE_1D_ARRAY {
                    gl.TexImage2D(texture_type, 0, format as i32, width as i32,
                        height.unwrap() as i32, 0, client_format as u32, client_type, data_raw);
                } else {
                    gl.TexImage1D(texture_type, 0, format as i32, width as i32, 0,
                        client_format as u32, client_type, data_raw);
                }

                if version >= &GlVersion(3, 0) {
                    gl.GenerateMipmap(texture_type);
                } else {
                    gl.GenerateMipmapEXT(texture_type);
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
    fn read<P>(&self, level: u32) -> Vec<P> where P: PixelValue {
        assert_eq!(level, 0);   // TODO: 

        let pixels_count = (self.width * self.height.unwrap_or(1) * self.depth.unwrap_or(1))
                            as uint;

        let (format, gltype) = PixelValue::get_format(None::<P>).to_gl_enum();
        let my_id = self.id;

        let (tx, rx) = channel();
        self.display.context.exec(proc(gl, state, version, extensions) {
            unsafe {
                let mut data: Vec<P> = Vec::with_capacity(pixels_count);

                gl.PixelStorei(gl::PACK_ALIGNMENT, 1);

                if version >= &GlVersion(4, 5) {
                    gl.GetTextureImage(my_id, level as gl::types::GLint, format, gltype,
                        (pixels_count * mem::size_of::<P>()) as gl::types::GLsizei,
                        data.as_mut_ptr() as *mut libc::c_void);

                } else if extensions.gl_ext_direct_state_access {
                    gl.GetTextureImageEXT(my_id, gl::TEXTURE_2D, level as gl::types::GLint,
                        format, gltype, data.as_mut_ptr() as *mut libc::c_void);

                } else {
                    gl.BindTexture(gl::TEXTURE_2D, my_id);
                    gl.GetTexImage(gl::TEXTURE_2D, level as gl::types::GLint, format, gltype,
                        data.as_mut_ptr() as *mut libc::c_void);
                }

                data.set_len(pixels_count);
                tx.send(data);
            }
        });

        rx.recv()
    }
}

impl fmt::Show for TextureImplementation {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::FormatError> {
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
        self.display.context.exec(proc(gl, _state, _, _) {
            unsafe { gl.DeleteTextures(1, [ id ].as_ptr()); }
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
