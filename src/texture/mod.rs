/*!
A texture is an image loaded in video memory, which can be sampled in your shaders.

Textures come in ten different dimensions:

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
 - Buffer textures, which are one-dimensional textures that are mapped to a buffer.

In addition to this, there are six kinds of texture formats:

 - The texture contains floating-point data,
   with either the `Compressed` prefix or no prefix at all.
 - The texture contains signed integers, with the `Integral` prefix.
 - The texture contains unsigned integers, with the `Unsigned` prefix.
 - The texture contains depth information, with the `Depth` prefix.
 - The texture contains stencil information, with the `Stencil` prefix.
 - The texture contains depth and stencil information, with the `DepthStencil` prefix.

Each combination of dimensions and format corresponds to a sampler type in GLSL. For example,
an `IntegralTexture3d` can only be bound to an `isampler3D` uniform in GLSL. Some combinations
don't exist, like `DepthBufferTexture`.

The difference between compressed textures and uncompressed textures is that you can't do
render-to-texture on the former.

The most common types of textures are `CompressedTexture2d` and `Texture2d` (the two dimensions
being the width and height). These are what you will use most of the time.

*/
#![allow(unreachable_code)]     // TODO: remove

use std::borrow::Cow;

use {gl, framebuffer};

#[cfg(feature = "image")]
use image;

use pixel_buffer::PixelBuffer;
use uniforms::{UniformValue, IntoUniformValue, Sampler};
use {Surface, GlObject, Rect};

use self::tex_impl::{TextureImplementation, TextureFormatRequest};

pub use self::format::{ClientFormat, TextureFormat};
pub use self::format::{UncompressedFloatFormat, UncompressedIntFormat, UncompressedUintFormat};
pub use self::format::{CompressedFormat, DepthFormat, DepthStencilFormat, StencilFormat};
pub use self::pixel::PixelValue;

mod format;
mod pixel;
mod tex_impl;

include!(concat!(env!("OUT_DIR"), "/textures.rs"));

/// Trait that describes a texture.
pub trait Texture {
    /// Returns the width in pixels of the texture.
    fn get_width(&self) -> u32;

    /// Returns the height in pixels of the texture, or `None` for one dimensional textures.
    fn get_height(&self) -> Option<u32>;

    /// Returns the depth in pixels of the texture, or `None` for one or two dimensional textures.
    fn get_depth(&self) -> Option<u32>;

    /// Returns the number of textures in the array, or `None` for non-arrays.
    fn get_array_size(&self) -> Option<u32>;
}

/// Trait that describes data for a one-dimensional texture.
pub trait Texture1dDataSource<'a> {
    type Data: Send + Copy + Clone + 'a;

    /// Returns the raw representation of the data.
    fn into_raw(self) -> RawImage1d<'a, Self::Data>;
}

/// Trait that describes data for a one-dimensional texture.
pub trait Texture1dDataSink {
    type Data: Send + Copy + 'static;

    /// Returns the list of accepted formats.
    fn get_preferred_formats() -> Vec<ClientFormat>;

    /// Builds a new object from raw data.
    fn from_raw(RawImage1d<Self::Data>) -> Self;
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

    fn into_raw(self) -> RawImage1d<'a, P> {
        let width = self.len() as u32;

        RawImage1d {
            data: Cow::Owned(self),
            width: width,
            format: <P as PixelValue>::get_format(),
        }
    }
}

impl<P: PixelValue> Texture1dDataSink for Vec<P> where P: Copy + Clone + Send {
    type Data = P;

    fn get_preferred_formats() -> Vec<ClientFormat> {
        vec![<P as PixelValue>::get_format()]
    }

    fn from_raw(data: RawImage1d<P>) -> Self {
        assert_eq!(data.format, <P as PixelValue>::get_format());
        data.data.into_owned()
    }
}

impl<'a, P: PixelValue> Texture1dDataSource<'a> for &'a[P] where P: Copy + Clone + Send + 'static {
    type Data = P;

    fn into_raw(self) -> RawImage1d<'a, P> {
        let width = self.len();

        RawImage1d {
            data: Cow::Borrowed(self),
            width: width as u32,
            format: <P as PixelValue>::get_format(),
        }
    }
}

/// Trait that describes data for a two-dimensional texture.
pub trait Texture2dDataSource<'a> {
    type Data: Send + Copy + Clone + 'a;

    /// Returns the raw representation of the data.
    fn into_raw(self) -> RawImage2d<'a, Self::Data>;
}

/// Trait that describes data for a two-dimensional texture.
pub trait Texture2dDataSink {
    type Data: Send + Copy + Clone + 'static;

    /// Returns the list of accepted formats.
    fn get_preferred_formats() -> Vec<ClientFormat>;

    /// Builds a new object from raw data.
    fn from_raw(RawImage2d<Self::Data>) -> Self;
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

impl<P: PixelValue> Texture2dDataSink for Vec<Vec<P>> where P: Copy + Clone + Send {
    type Data = P;

    fn get_preferred_formats() -> Vec<ClientFormat> {
        vec![<P as PixelValue>::get_format()]
    }

    fn from_raw(data: RawImage2d<P>) -> Self {
        assert_eq!(data.format, <P as PixelValue>::get_format());
        let width = data.width;
        data.data.as_slice().chunks(width as usize).map(|e| e.to_vec()).collect()
    }
}

#[cfg(feature = "image")]
impl<'a, T, P> Texture2dDataSource<'a> for image::ImageBuffer<P, Vec<T>>
                                       where T: image::Primitive + Send + 'static,
                                             P: PixelValue + image::Pixel<Subpixel = T> + Clone + Copy
{
    type Data = T;

    fn into_raw(self) -> RawImage2d<'a, T> {
        use image::GenericImage;

        let (width, height) = self.dimensions();

        // the image library gives us rows from bottom to top, so we need to flip them
        let data = self.into_raw()
            .as_slice()
            .chunks(width as usize * <P as image::Pixel>::channel_count() as usize)
            .rev()
            .flat_map(|row| row.iter())
            .map(|p| p.clone())
            .collect();

        RawImage2d {
            data: data,
            width: width,
            height: height,
            format: <P as PixelValue>::get_format(),
        }
    }
}

#[cfg(feature = "image")]
impl<T, P> Texture2dDataSink for image::ImageBuffer<P, Vec<T>>
                                 where T: image::Primitive + Send + 'static,
                                       P: PixelValue + image::Pixel<Subpixel = T> + Clone + Copy
{
    type Data = T;

    fn get_preferred_formats() -> Vec<ClientFormat> {
        vec![<P as PixelValue>::get_format()]
    }

    fn from_raw(data: RawImage2d<T>) -> Self {
        let pixels_size = <P as image::Pixel>::channel_count();
        let width = data.width;
        let height = data.height;

        // opengl gives us rows from bottom to top, so we need to flip them
        let data = data.data
            .as_slice()
            .chunks(width as usize * <P as image::Pixel>::channel_count() as usize)
            .rev()
            .flat_map(|row| row.iter())
            .map(|p| p.clone())
            .collect();

        image::ImageBuffer::from_raw(width, height, data).unwrap()
    }
}

#[cfg(feature = "image")]
impl<'a> Texture2dDataSource<'a> for image::DynamicImage {
    type Data = u8;

    fn into_raw(self) -> RawImage2d<'a, u8> {
        Texture2dDataSource::into_raw(self.to_rgba())
    }
}

#[cfg(feature = "image")]
impl Texture2dDataSink for image::DynamicImage {
    type Data = u8;

    fn get_preferred_formats() -> Vec<ClientFormat> {
        vec![ClientFormat::U8U8U8U8]
    }

    fn from_raw(data: RawImage2d<u8>) -> image::DynamicImage {
        image::DynamicImage::ImageRgba8(Texture2dDataSink::from_raw(data))
    }
}

/// Trait that describes data for a two-dimensional texture.
pub trait Texture3dDataSource<'a> {
    type Data: Send + Copy + Clone + 'a;

    /// Returns the raw representation of the data.
    fn into_raw(self) -> RawImage3d<'a, Self::Data>;
}

/// Trait that describes data for a two-dimensional texture.
pub trait Texture3dDataSink {
    type Data: Send + Copy + 'static;

    /// Returns the list of accepted formats.
    fn get_preferred_formats() -> Vec<ClientFormat>;

    /// Builds a new object from raw data.
    fn from_raw(RawImage3d<Self::Data>) -> Self;
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

impl<P> Texture3dDataSink for Vec<Vec<Vec<P>>> where P: PixelValue + Clone {
    type Data = P;

    fn get_preferred_formats() -> Vec<ClientFormat> {
        vec![<P as PixelValue>::get_format()]
    }

    fn from_raw(data: RawImage3d<P>) -> Self {
        assert_eq!(data.format, <P as PixelValue>::get_format());
        unimplemented!()
    }
}

/// Struct that allows you to draw on a texture.
///
/// To obtain such an object, call `texture.as_surface()`.
pub struct TextureSurface<'a>(framebuffer::SimpleFrameBuffer<'a>);

impl<'a> Surface for TextureSurface<'a> {
    fn clear(&mut self, color: Option<(f32, f32, f32, f32)>, depth: Option<f32>,
             stencil: Option<i32>)
    {
        self.0.clear(color, depth, stencil)
    }

    fn get_dimensions(&self) -> (u32, u32) {
        self.0.get_dimensions()
    }

    fn get_depth_buffer_bits(&self) -> Option<u16> {
        self.0.get_depth_buffer_bits()
    }

    fn get_stencil_buffer_bits(&self) -> Option<u16> {
        self.0.get_stencil_buffer_bits()
    }

    fn draw<'b, 'v, V, I, U>(&mut self, vb: V, ib: &I, program: &::Program,
        uniforms: U, draw_parameters: &::DrawParameters) -> Result<(), ::DrawError>
        where I: ::index::ToIndicesSource,
        U: ::uniforms::Uniforms, V: ::vertex::MultiVerticesSource<'v>
    {
        self.0.draw(vb, ib, program, uniforms, draw_parameters)
    }

    fn get_blit_helper(&self) -> ::BlitHelper {
        self.0.get_blit_helper()
    }
}

/// Error that can happen when creating a texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureCreationError {
    /// The requested format is not supported by the backend.
    UnsupportedFormat,

    /// The requested texture dimensions are not supported.
    DimensionsNotSupported,
}

/// Error that can happen when creating a texture which we don't know whether it is supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureMaybeSupportedCreationError {
    /// The texture type is supported, but a `TextureCreationError` happened.
    CreationError(TextureCreationError),

    /// The texture type is not supported by the backend.
    NotSupported,
}
