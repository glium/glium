/*!
A texture is an image loaded in video memory, which can be sampled in your shaders.

# Texture kinds

One thing that is important to understand when it comes to textures is that the way a texture is
accessed (in other words, its "public API") is disconnected from the internal representation
of the data.

When it comes to accessing a texture, there are six kinds of textures:

 - Floating-point textures.
 - Integral textures (that contain signed integers).
 - Unsigned textures (that contain unsigned integers).
 - Depth textures (that contain depth information).
 - Stencil textures (that contain stencil information).
 - Depth-stencil textures (that contain at the same time depth and stencil information).

Textures have a different API depending on their kind. For example a integral texture can only
be sampled in GLSL through a sampler type which is prefixed with `i`.

The internal format can only be chosen when the texture is created, and then can never be touched
again. Integral and unsigned textures can only contain signed integers and unsigned integers.

Floating-point textures can contain either floating-points or integers. If integers are used,
then the maximum value corresponds to the floating-point value `1.0` and the minimal value to `0.0`.
For example if a texture contains `u8`s and internally contains the value `128`, reading from the
texture will yield the value `0.5`.

# Dimensions

Textures come in nine different dimensions:

 - Textures with one dimension.
 - Textures with two dimensions.
 - Textures with two dimensions and multisampling enabled.
 - Textures with three dimensions.
 - Cube textures, which are arrays of six two-dimensional textures
   corresponding to the six faces of a cube.
 - Arrays of one-dimensional textures.
 - Arrays of two-dimensional textures.
 - Arrays of two-dimensional textures with multisampling enabled.
 - Arrays of cube textures.

The difference between a 3D texture and a 2D textures array (and between a 2D texture and a 1D
textures array) is that texture arrays can only be accessed by individual layers. That is, you can
only access layer 0, or layer 1, or layer 2, and so on. Whereas if you use 3D textures you can
access layer `0.5` for example.

All textures except depth, stencil and depth-stencil textures have **mipmaps**. A mipmap is a
smaller version of the texture whose purpose is to be used during rendering when the texture will
be small on the screen.

# Texture types in glium

In addition to the nine different dimensions types, there are nine kinds of texture formats:

 - The texture contains floating-point data,
   with either the `Compressed` prefix or no prefix at all.
 - The texture contains floating-point data in the sRGB color space, with either the `Compressed`
   prefix or not.
 - The texture contains signed integers, with the `Integral` prefix.
 - The texture contains unsigned integers, with the `Unsigned` prefix.
 - The texture contains depth information, with the `Depth` prefix.
 - The texture contains stencil information, with the `Stencil` prefix.
 - The texture contains depth and stencil information, with the `DepthStencil` prefix.

Each combination of dimensions and format corresponds to a sampler type in GLSL and in glium.
For example, an `IntegralTexture3d` can only be bound to an `isampler3D` uniform in GLSL.

The difference between compressed textures and uncompressed textures is that you can't do
render-to-texture on the former.

The most common types of textures are `CompressedSrgbTexture2d`, `SrgbTexture2d` and `Texture2d`
(the two dimensions being the width and height). These are what you will use most of the time.

# Buffer textures

A `BufferTexture` is a special kind of one-dimensional texture that gets its data from a buffer.
Buffer textures have very limited capabilities (you can't draw to them for example). They are an
alternative to uniform buffers and SSBOs.

See the `buffer_textures` module for more infos.

# About sRGB

For historical reasons, the color data contained in almost all image files are not in RGB but
in sRGB. sRGB colors are slightly brighter than linear RGB in order to compensate for the fact
that screens darken some values that they receive.

When you load image files, you are encouraged to create sRGB textures (with `SrgbTexture2d` instead
of `Texture2d` for example).

By default, glium enables the `GL_FRAMEBUFFER_SRGB` trigger, which expects the output of your
fragment shader to be in linear RGB and then turns it into sRGB before writing in the framebuffer.
Sampling from an sRGB texture will convert the texture colors from sRGB to RGB. If you create a
regular RGB texture and put sRGB data in it, then the result will be too bright.

# Bindless textures

*Bindless textures are a very recent feature that is supported only by recent hardware and
drivers.*

Without bindless textures, using a texture in a shader requires binding the texture to a specific
bind point before drawing. This not only slows down rendering, but may also prevent you from
grouping multiple draw calls into one because of the limitation to the number of available
texture units.

Instead, bindless textures allow you to manually manipulate pointers to textures in video memory.
You can use thousands of textures if you want.

*/
#![allow(unreachable_code)]     // TODO: remove

use std::borrow::Cow;
use std::fmt;
use std::error::Error;

use image_format::FormatNotSupportedError;

pub use image_format::{ClientFormat, TextureFormat};
pub use image_format::{UncompressedFloatFormat, UncompressedIntFormat, UncompressedUintFormat};
pub use image_format::{CompressedFormat, DepthFormat, DepthStencilFormat, StencilFormat};
pub use image_format::{CompressedSrgbFormat, SrgbFormat};
pub use self::any::{TextureAny, TextureAnyMipmap, TextureAnyLayer, TextureAnyLayerMipmap};
pub use self::any::{TextureAnyImage, Dimensions};
pub use self::bindless::{ResidentTexture, TextureHandle, BindlessTexturesNotSupportedError};
pub use self::get_format::{InternalFormat, InternalFormatType, GetFormatError};
pub use self::pixel::PixelValue;
pub use self::ty_support::{is_texture_1d_supported, is_texture_2d_supported};
pub use self::ty_support::{is_texture_3d_supported, is_texture_1d_array_supported};
pub use self::ty_support::{is_texture_2d_array_supported, is_texture_2d_multisample_supported};
pub use self::ty_support::{is_texture_2d_multisample_array_supported, is_cubemaps_supported};
pub use self::ty_support::is_cubemap_arrays_supported;

pub mod bindless;
pub mod buffer_texture;
pub mod pixel_buffer;

mod any;
mod get_format;
mod pixel;
mod ty_support;

include!(concat!(env!("OUT_DIR"), "/textures.rs"));

/// Represents a layer of a cubemap.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]      // TODO:
pub enum CubeLayer {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    PositiveZ,
    NegativeZ,
}

impl CubeLayer {
    /// In some situations whole cubemaps can be bound at once. If this is the case, each layer
    /// of the cubemap has a specific index.
    ///
    /// For example, if you bind a whole cubemap array, then the index `8` will correspond to the
    /// `PositiveY` face of the cubemap whose index is `1` in the array.
    pub fn get_layer_index(&self) -> usize {
        match self {
            &CubeLayer::PositiveX => 0,
            &CubeLayer::NegativeX => 1,
            &CubeLayer::PositiveY => 2,
            &CubeLayer::NegativeY => 3,
            &CubeLayer::PositiveZ => 4,
            &CubeLayer::NegativeZ => 5,
        }
    }
}

/// Represents a kind of texture.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]      // TODO:
pub enum TextureKind {
    Float,
    Integral,
    Unsigned,
    Depth,
    Stencil,
    DepthStencil,
}

/// Describes what to do about mipmaps during texture creation.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MipmapsOption {
    /// No mipmap will be allocated or generated.
    NoMipmap,

    /// Allocates space for all the possible amount of mipmaps given the texture dimensions.
    EmptyMipmaps,

    /// Allocates space for the specified amount of mipmaps (excluding the top level) but does not
    /// generate mipmaps.
    EmptyMipmapsMax(u32),

    /// Allocates and generates mipmaps for all the possible levels given the texture dimensions.
    ///
    /// This does not mean that you will get mipmaps, instead it indicates that mipmaps are *allowed*
    /// to be generated if possible.
    AutoGeneratedMipmaps,

    /// Allocates and generates mipmaps for the specified amount of mipmaps (excluding the top level)
    /// the possible levels given the texture dimensions.
    ///
    /// This does not mean that you will get mipmaps, instead it indicates that mipmaps are *allowed*
    /// to be generated if possible.
    AutoGeneratedMipmapsMax(u32),
}

impl MipmapsOption {
    /// Tells whether mipmaps should be automatically generated.
    #[inline]
    fn should_generate(self) -> bool {
        use self::MipmapsOption::*;
        match self {
            AutoGeneratedMipmaps | AutoGeneratedMipmapsMax(_) => true,
            _ => false,
        }
    }

    /// Number of levels (including the main level).
    fn num_levels(self, width: u32, height: Option<u32>, depth: Option<u32>) -> u32 {
        use self::MipmapsOption::*;
        use std::cmp;
        use std::num::FpCategory;

        match self {
            NoMipmap => 1,
            EmptyMipmaps | AutoGeneratedMipmaps => {
                let max_dimension = cmp::max(width, cmp::max(height.unwrap_or(1),
                                             depth.unwrap_or(1))) as f32;

                if max_dimension.classify() == FpCategory::Zero {
                    1
                } else {
                    1 + max_dimension.log2() as u32
                }
            },
            EmptyMipmapsMax(i) | AutoGeneratedMipmapsMax(i) => {
                let max = EmptyMipmaps.num_levels(width, height, depth) - 1;
                if i > max { // TODO should we perform this check or just clamp the value?
                    panic!("Too many mipmap levels, received {}, maximum for this texture dimension is {}.", i, max);
                }
                1 + i
            },
        }
    }
}

impl From<CompressedMipmapsOption> for MipmapsOption {
    fn from(opt: CompressedMipmapsOption) -> MipmapsOption {
        match opt {
            CompressedMipmapsOption::NoMipmap => MipmapsOption::NoMipmap,
            CompressedMipmapsOption::EmptyMipmaps => MipmapsOption::EmptyMipmaps,
            CompressedMipmapsOption::EmptyMipmapsMax(i) => MipmapsOption::EmptyMipmapsMax(i),
        }
    }
}

/// Describes what to do about mipmaps during compressed texture creation.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CompressedMipmapsOption {
    /// No mipmaps will be allocated or generated.
    NoMipmap,

    /// Allocates space for all the possible amount of mipmaps given the texture dimensions.
    EmptyMipmaps,

    /// Allocates space for the specified amount of mipmaps (excluding the top level) but does not
    /// generate mipmaps.
    EmptyMipmapsMax(u32),
}

/// Trait that describes data for a one-dimensional texture.
pub trait Texture1dDataSource<'a> {
    /// The type of each pixel.
    type Data: Send + Copy + Clone + 'a;

    /// Returns the raw representation of the data.
    fn into_raw(self) -> RawImage1d<'a, Self::Data>;
}

/// Trait that describes types that can be built from one-dimensional texture data.
///
/// The parameter indicates the type of pixels accepted by this sink.
///
/// You are especially encouraged to implement this trait with the parameter `(u8, u8, u8, u8)`,
/// as this is the only format that is guaranteed to be supported by OpenGL when reading pixels.
pub trait Texture1dDataSink<T> {
    /// Builds a new object from raw data.
    fn from_raw(data: Cow<[T]>, width: u32) -> Self where [T]: ToOwned;
}

/// Represents raw data for a two-dimensional image.
pub struct RawImage1d<'a, T: Clone + 'a> {
    /// A contiguous array of pixel data.
    ///
    /// The data must start by the left pixel and progress left-to-right.
    ///
    /// `data.len()` must be equal to `width * format.get_size() / mem::size_of::<T>()`.
    pub data: Cow<'a, [T]>,

    /// Number of pixels per column.
    pub width: u32,

    /// Formats of the pixels.
    pub format: ClientFormat,
}

impl<'a, P: PixelValue> Texture1dDataSource<'a> for Vec<P> where P: Copy + Clone + Send + 'static {
    type Data = P;

    #[inline]
    fn into_raw(self) -> RawImage1d<'a, P> {
        let width = self.len() as u32;

        RawImage1d {
            data: Cow::Owned(self),
            width: width,
            format: <P as PixelValue>::get_format(),
        }
    }
}

impl<'a, P: PixelValue + Clone> Texture1dDataSource<'a> for RawImage1d<'a, P> {
    type Data = P;

    #[inline]
    fn into_raw(self) -> RawImage1d<'a, P> {
        self
    }
}

impl<P> Texture1dDataSink<P> for Vec<P> where P: Copy + Clone + Send {
    #[inline]
    fn from_raw(data: Cow<[P]>, _width: u32) -> Self {
        data.into_owned()
    }
}

impl<'a, P: PixelValue> Texture1dDataSource<'a> for &'a[P] where P: Copy + Clone + Send + 'static {
    type Data = P;

    #[inline]
    fn into_raw(self) -> RawImage1d<'a, P> {
        let width = self.len();

        RawImage1d {
            data: Cow::Borrowed(self),
            width: width as u32,
            format: <P as PixelValue>::get_format(),
        }
    }
}

impl<'a, T: Clone + 'a> RawImage1d<'a, T> {

    /// Builds a raw 1d image from a vector of interleaved RGB values.
    pub fn from_raw_rgb(data: Vec<T>) -> RawImage1d<'a, T>
        where T: ToClientFormat {
        RawImage1d {
            width: (data.len() / 3) as u32,
            data: Cow::Owned(data),
            format: T::rgb_format(),
        }
    }

    /// Builds a raw 1d image from a vector of interleaved RGBA values.
    pub fn from_raw_rgba(data: Vec<T>) -> RawImage1d<'a, T>
        where T: ToClientFormat {
        RawImage1d {
            width: (data.len() / 4) as u32,
            data: Cow::Owned(data),
            format: T::rgba_format(),
        }
    }
}

/// Trait that describes data for a two-dimensional texture.
pub trait Texture2dDataSource<'a> {
    /// The type of each pixel.
    type Data: Send + Copy + Clone + 'a;

    /// Returns the raw representation of the data.
    fn into_raw(self) -> RawImage2d<'a, Self::Data>;
}

/// Trait that describes types that can be built from two-dimensional texture data.
///
/// The parameter indicates the type of pixels accepted by this sink.
///
/// You are especially encouraged to implement this trait with the parameter `(u8, u8, u8, u8)`,
/// as this is the only format that is guaranteed to be supported by OpenGL when reading pixels.
pub trait Texture2dDataSink<T> {
    /// Builds a new object from raw data.
    fn from_raw(data: Cow<[T]>, width: u32, height: u32) -> Self where [T]: ToOwned;
}

/// Represents raw data for a two-dimensional image.
pub struct RawImage2d<'a, T: Clone + 'a> {
    /// A contiguous array of pixel data.
    ///
    /// The data must start by the bottom-left hand corner pixel and progress left-to-right and
    /// bottom-to-top.
    ///
    /// `data.len()` must be equal to `width * height * format.get_size() / mem::size_of::<T>()`.
    pub data: Cow<'a, [T]>,

    /// Number of pixels per column.
    pub width: u32,

    /// Number of pixels per row.
    pub height: u32,

    /// Formats of the pixels.
    pub format: ClientFormat,
}

#[allow(missing_docs)]
pub trait ToClientFormat {
  fn rgb_format() -> ClientFormat;
  fn rgba_format() -> ClientFormat;
}

impl ToClientFormat for u8 {
    fn rgb_format() -> ClientFormat { ClientFormat::U8U8U8 }
    fn rgba_format() -> ClientFormat { ClientFormat::U8U8U8U8 }
}

impl ToClientFormat for i8 {
    fn rgb_format() -> ClientFormat { ClientFormat::I8I8I8 }
    fn rgba_format() -> ClientFormat { ClientFormat::I8I8I8I8 }
}

impl ToClientFormat for u16 {
    fn rgb_format() -> ClientFormat { ClientFormat::U16U16U16 }
    fn rgba_format() -> ClientFormat { ClientFormat::U16U16U16U16 }
}

impl ToClientFormat for i16 {
    fn rgb_format() -> ClientFormat { ClientFormat::I16I16I16 }
    fn rgba_format() -> ClientFormat { ClientFormat::I16I16I16I16 }
}

impl ToClientFormat for u32 {
    fn rgb_format() -> ClientFormat { ClientFormat::U32U32U32 }
    fn rgba_format() -> ClientFormat { ClientFormat::U32U32U32U32 }
}

impl ToClientFormat for i32 {
    fn rgb_format() -> ClientFormat { ClientFormat::I32I32I32 }
    fn rgba_format() -> ClientFormat { ClientFormat::I32I32I32I32 }
}

impl ToClientFormat for f32 {
    fn rgb_format() -> ClientFormat { ClientFormat::F32F32F32 }
    fn rgba_format() -> ClientFormat { ClientFormat::F32F32F32F32 }
}

impl<'a, T: Clone + 'a> RawImage2d<'a, T> {
    /// Builds a raw image from a vector of interleaved RGB values.
    ///
    /// The first pixel is at (0, 0), the last pixel is at (1, 1).
    pub fn from_raw_rgb(data: Vec<T>, dimensions: (u32, u32)) -> RawImage2d<'a, T>
        where T: ToClientFormat {
        RawImage2d {
            data: Cow::Owned(data),
            width: dimensions.0,
            height: dimensions.1,
            format: T::rgb_format(),
        }
    }

    /// Builds a raw image from a vector of interleaved RGBA values.
    ///
    /// The first pixel is at (0, 0), the last pixel is at (1, 1).
    pub fn from_raw_rgba(data: Vec<T>, dimensions: (u32, u32)) -> RawImage2d<'a, T>
        where T: ToClientFormat {
        RawImage2d {
            data: Cow::Owned(data),
            width: dimensions.0,
            height: dimensions.1,
            format: T::rgba_format(),
        }
    }

    /// Builds a raw image from a vector of interleaved RGB values, flipping it vertically.
    ///
    /// The first pixel is at (0, 1), the last pixel is at (1, 0).
    pub fn from_raw_rgb_reversed(data: &[T], dimensions: (u32, u32)) -> RawImage2d<'a, T>
        where T: ToClientFormat {
        let data = data
            .chunks(dimensions.0 as usize * 3)
            .rev()
            .flat_map(|row| row.iter())
            .map(|p| p.clone())
            .collect();

        RawImage2d::from_raw_rgb(data, dimensions)
    }

    /// Builds a raw image from a vector of interleaved RGBA values, flipping it vertically.
    ///
    /// The first pixel is at (0, 1), the last pixel is at (1, 0).
    pub fn from_raw_rgba_reversed(data: &[T], dimensions: (u32, u32)) -> RawImage2d<'a, T>
        where T: ToClientFormat {
        let data = data
            .chunks(dimensions.0 as usize * 4)
            .rev()
            .flat_map(|row| row.iter())
            .map(|p| p.clone())
            .collect();

        RawImage2d::from_raw_rgba(data, dimensions)
    }

    ///Transforms a Vec<RawImage1d> into a RawImage2d
    pub fn from_vec_raw1d(arr: &Vec<RawImage1d<'a, T>>) -> RawImage2d<'a, T> {
        let width   = arr[0].width;
        let height  = arr.len() as u32;
        let format  = arr[0].format;
        let raw_data = {
            let mut vec = Vec::<T>::with_capacity((width * height) as usize);
            for i in arr {
                if width != i.width {
                    panic!("Varying dimensions were found.");
                } else if format != i.format {
                    panic!("Varying formats were found.");
                }
                for j in i.data.iter() {
                    vec.push(j.clone());
                }
            }
            vec
        };
        RawImage2d {
            data: Cow::Owned(raw_data),
            width: width,
            height: height,
            format: format,
        }
    }
}

impl<'a, P: PixelValue + Clone> Texture2dDataSource<'a> for Vec<Vec<P>> {
    type Data = P;

    fn into_raw(self) -> RawImage2d<'a, P> {
        let width = self.iter().next().map(|e| e.len()).unwrap_or(0) as u32;
        let height = self.len() as u32;

        RawImage2d {
            data: Cow::Owned(self.into_iter().flat_map(|e| e.into_iter()).collect()),
            width: width,
            height: height,
            format: <P as PixelValue>::get_format(),
        }
    }
}

impl<'a, P: PixelValue + Clone> Texture2dDataSource<'a> for RawImage2d<'a, P> {
    type Data = P;

    #[inline]
    fn into_raw(self) -> RawImage2d<'a, P> {
        self
    }
}

impl<P> Texture2dDataSink<P> for Vec<Vec<P>> where P: Copy + Clone {
    fn from_raw(data: Cow<[P]>, width: u32, height: u32) -> Self {
        data.chunks(width as usize).map(|e| e.to_vec()).collect()
    }
}

macro_rules! impl_2d_sink_for_raw_image {
    (($t1:ty, $t2:ty, $t3:ty, $t4:ty)) => (
        impl<'a> Texture2dDataSink<($t1, $t2, $t3, $t4)> for RawImage2d<'a, $t1> {
            fn from_raw(data: Cow<[($t1, $t2, $t3, $t4)]>, width: u32, height: u32) -> Self {
                RawImage2d {
                    data: Cow::Owned( {
                        let mut v = Vec::with_capacity(data.len() * 4);
                        for (a, b, c, d) in data.into_owned() {
                            v.push(a);
                            v.push(b);
                            v.push(c);
                            v.push(d);
                        }
                        v
                    } ),
                    width: width,
                    height: height,
                    format: <($t1, $t2, $t3, $t4) as PixelValue>::get_format(),
                }
            }
        }
    );
    (($t1:ty, $t2:ty, $t3:ty)) => (
        impl<'a> Texture2dDataSink<($t1, $t2, $t3)> for RawImage2d<'a, $t1> {
            fn from_raw(data: Cow<[($t1, $t2, $t3)]>, width: u32, height: u32) -> Self {
                RawImage2d {
                    data: Cow::Owned( {
                        let mut v = Vec::with_capacity(data.len() * 3);
                        for (a, b, c) in data.into_owned() {
                            v.push(a);
                            v.push(b);
                            v.push(c);
                        }
                        v
                    } ),
                    width: width,
                    height: height,
                    format: <($t1, $t2, $t3) as PixelValue>::get_format(),
                }
            }
        }
    );
    (($t1:ty, $t2:ty)) => (
        impl<'a> Texture2dDataSink<($t1, $t2)> for RawImage2d<'a, $t1> {
            fn from_raw(data: Cow<[($t1, $t2)]>, width: u32, height: u32) -> Self {
                RawImage2d {
                    data: Cow::Owned( {
                        let mut v = Vec::with_capacity(data.len() * 2);
                        for (a, b) in data.into_owned() {
                            v.push(a);
                            v.push(b);
                        }
                        v
                    } ),
                    width: width,
                    height: height,
                    format: <($t1, $t2) as PixelValue>::get_format(),
                }
            }
        }
    );
    ($t1:ty) => (
        impl<'a> Texture2dDataSink<$t1> for RawImage2d<'a, $t1> {
            fn from_raw(data: Cow<[$t1]>, width: u32, height: u32) -> Self {
                RawImage2d {
                    data: Cow::Owned(data.into_owned()),
                    width: width,
                    height: height,
                    format: <$t1 as PixelValue>::get_format(),
                }
            }
        }
    );
}

impl_2d_sink_for_raw_image!(i8);
impl_2d_sink_for_raw_image!((i8, i8));
impl_2d_sink_for_raw_image!((i8, i8, i8));
impl_2d_sink_for_raw_image!((i8, i8, i8, i8));
impl_2d_sink_for_raw_image!(u8);
impl_2d_sink_for_raw_image!((u8, u8));
impl_2d_sink_for_raw_image!((u8, u8, u8));
impl_2d_sink_for_raw_image!((u8, u8, u8, u8));
impl_2d_sink_for_raw_image!(i16);
impl_2d_sink_for_raw_image!((i16, i16));
impl_2d_sink_for_raw_image!((i16, i16, i16));
impl_2d_sink_for_raw_image!((i16, i16, i16, i16));
impl_2d_sink_for_raw_image!(u16);
impl_2d_sink_for_raw_image!((u16, u16));
impl_2d_sink_for_raw_image!((u16, u16, u16));
impl_2d_sink_for_raw_image!((u16, u16, u16, u16));
impl_2d_sink_for_raw_image!(i32);
impl_2d_sink_for_raw_image!((i32, i32));
impl_2d_sink_for_raw_image!((i32, i32, i32));
impl_2d_sink_for_raw_image!((i32, i32, i32, i32));
impl_2d_sink_for_raw_image!(u32);
impl_2d_sink_for_raw_image!((u32, u32));
impl_2d_sink_for_raw_image!((u32, u32, u32));
impl_2d_sink_for_raw_image!((u32, u32, u32, u32));
impl_2d_sink_for_raw_image!(f32);
impl_2d_sink_for_raw_image!((f32, f32));
impl_2d_sink_for_raw_image!((f32, f32, f32));
impl_2d_sink_for_raw_image!((f32, f32, f32, f32));

/// Trait that describes data for a two-dimensional texture.
pub trait Texture3dDataSource<'a> {
    /// The type of each pixel.
    type Data: Send + Copy + Clone + 'a;

    /// Returns the raw representation of the data.
    fn into_raw(self) -> RawImage3d<'a, Self::Data>;
}

/// Trait that describes types that can be built from one-dimensional texture data.
///
/// The parameter indicates the type of pixels accepted by this sink.
///
/// You are especially encouraged to implement this trait with the parameter `(u8, u8, u8, u8)`,
/// as this is the only format that is guaranteed to be supported by OpenGL when reading pixels.
pub trait Texture3dDataSink<T> {
    /// Builds a new object from raw data.
    fn from_raw(data: Cow<[T]>, width: u32, height: u32, depth: u32) -> Self where [T]: ToOwned;
}

/// Represents raw data for a two-dimensional image.
pub struct RawImage3d<'a, T: Clone + 'a> {
    /// A contiguous array of pixel data.
    ///
    /// `data.len()` must be equal to `width * height * depth * format.get_size() / mem::size_of::<T>()`.
    pub data: Cow<'a, [T]>,

    /// Number of pixels per column.
    pub width: u32,

    /// Number of pixels per row.
    pub height: u32,

    /// Number of pixels per depth.
    pub depth: u32,

    /// Formats of the pixels.
    pub format: ClientFormat,
}

impl<'a, T: Clone + 'a> RawImage3d<'a, T> {
    ///Transforms a Vec<RawImage2d> into a RawImage3d
    pub fn from_vec_raw2d(arr: &Vec<RawImage2d<'a, T>>) -> RawImage3d<'a, T> {
        let depth   = arr.len() as u32;
        let width   = arr[0].width;
        let height  = arr[0].height;
        let format  = arr[0].format;
        let raw_data = {
            let mut vec = Vec::<T>::with_capacity((width * height * depth) as usize);
            for i in arr {
                if width != i.width || height != i.height {
                    panic!("Varying dimensions were found.");
                } else if format != i.format {
                    panic!("Varying formats were found.");
                }
                for j in i.data.iter() {
                    vec.push(j.clone());
                }
            }
            vec
        };
        RawImage3d {
            data: Cow::Owned(raw_data),
            width: width,
            height: height,
            depth: depth,
            format: format,
        }
    }
}

impl<'a, P: PixelValue + Clone> Texture3dDataSource<'a> for Vec<Vec<Vec<P>>> {
    type Data = P;

    fn into_raw(self) -> RawImage3d<'a, P> {
        let width = self.iter().next().and_then(|e| e.iter().next()).map(|e| e.len()).unwrap_or(0)
                    as u32;
        let height = self.iter().next().map(|e| e.len()).unwrap_or(0) as u32;
        let depth = self.len() as u32;

        RawImage3d {
            data: self.into_iter().flat_map(|e| e.into_iter()).flat_map(|e| e.into_iter())
                      .collect(),
            width: width,
            height: height,
            depth: depth,
            format: <P as PixelValue>::get_format(),
        }
    }
}

impl<'a, P: PixelValue + Clone> Texture3dDataSource<'a> for RawImage3d<'a, P> {
    type Data = P;

    #[inline]
    fn into_raw(self) -> RawImage3d<'a, P> {
        self
    }
}

impl<P> Texture3dDataSink<P> for Vec<Vec<Vec<P>>> where P: Copy + Clone {
    #[inline]
    fn from_raw(_data: Cow<[P]>, _width: u32, _height: u32, _depth: u32) -> Self {
        unimplemented!()
    }
}

/// Error that can happen when creating a texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureCreationError {
    /// The requested format is not supported by the backend.
    FormatNotSupported,

    /// The requested texture dimensions are not supported.
    DimensionsNotSupported,

    /// The texture format is not supported by the backend.
    TypeNotSupported,
}

impl fmt::Display for TextureCreationError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

impl Error for TextureCreationError {
    fn description(&self) -> &str {
        use self::TextureCreationError::*;
        match *self {
            FormatNotSupported =>
                "The requested format is not supported by the backend",
            DimensionsNotSupported =>
                "The requested texture dimensions are not supported",
            TypeNotSupported =>
                "The texture format is not supported by the backend",
        }
    }
}

impl From<FormatNotSupportedError> for TextureCreationError {
    #[inline]
    fn from(_: FormatNotSupportedError) -> TextureCreationError {
        TextureCreationError::FormatNotSupported
    }
}
