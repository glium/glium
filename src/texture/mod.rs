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
use std::convert::From;

use {gl, framebuffer};

#[cfg(feature = "image")]
use image;

use backend::Facade;

use pixel_buffer::PixelBuffer;
use uniforms::{UniformValue, AsUniformValue, UniformType, Sampler};
use {Surface, GlObject};

use FboAttachments;
use fbo::ValidatedAttachments;
use Rect;
use BlitTarget;
use uniforms;

use image_format::{TextureFormatRequest, FormatNotSupportedError};

pub use image_format::{ClientFormat, TextureFormat};
pub use image_format::{UncompressedFloatFormat, UncompressedIntFormat, UncompressedUintFormat};
pub use image_format::{CompressedFormat, DepthFormat, DepthStencilFormat, StencilFormat};
pub use image_format::{CompressedSrgbFormat, SrgbFormat};
pub use self::any::{TextureAny, TextureAnyMipmap, TextureType};
pub use self::get_format::{InternalFormat, InternalFormatType};
pub use self::pixel::PixelValue;

mod any;
mod get_format;
mod pixel;

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
    fn from_raw(data: Cow<[T]>, width: u32) -> Self;
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

impl<'a, P: PixelValue + Clone> Texture1dDataSource<'a> for RawImage1d<'a, P> {
    type Data = P;

    fn into_raw(self) -> RawImage1d<'a, P> {
        self
    }
}

impl<P> Texture1dDataSink<P> for Vec<P> where P: Copy + Clone + Send {
    fn from_raw(data: Cow<[P]>, _width: u32) -> Self {
        data.into_owned()
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
    fn from_raw(data: Cow<[T]>, width: u32, height: u32) -> Self;
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

impl<'a, T: Clone + 'a> RawImage2d<'a, T> {
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

    fn into_raw(self) -> RawImage2d<'a, P> {
        self
    }
}

impl<P> Texture2dDataSink<P> for Vec<Vec<P>> where P: Copy + Clone {
    fn from_raw(data: Cow<[P]>, width: u32, height: u32) -> Self {
        data.chunks(width as usize).map(|e| e.to_vec()).collect()
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

        // the image library gives us rows from top to bottom, so we need to flip them
        let data = self.into_raw()
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
impl<T, P> Texture2dDataSink<P> for image::ImageBuffer<P, Vec<T>>
                                    where T: image::Primitive + Send + 'static,
                                          P: PixelValue + image::Pixel<Subpixel = T> + Clone + Copy
{
    fn from_raw(data: Cow<[P]>, width: u32, height: u32) -> Self {
        // opengl gives us rows from bottom to top, so we need to flip them
        let data = data
            .chunks(width as usize)
            .rev()
            .flat_map(|row| row.iter())
            .flat_map(|pixel| pixel.channels().iter())
            .cloned()
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
impl Texture2dDataSink<(u8, u8, u8, u8)> for image::DynamicImage {
    fn from_raw(data: Cow<[(u8, u8, u8, u8)]>, w: u32, h: u32) -> image::DynamicImage {
        let data = unsafe { ::std::mem::transmute(data) };     // FIXME: <-
        image::DynamicImage::ImageRgba8(Texture2dDataSink::from_raw(data, w, h))
    }
}

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
    fn from_raw(data: Cow<[T]>, width: u32, height: u32, depth: u32) -> Self;
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

    fn into_raw(self) -> RawImage3d<'a, P> {
        self
    }
}

impl<P> Texture3dDataSink<P> for Vec<Vec<Vec<P>>> where P: Copy + Clone {
    fn from_raw(_data: Cow<[P]>, _width: u32, _height: u32, _depth: u32) -> Self {
        unimplemented!()
    }
}

/// Struct that allows you to draw on a texture.
///
/// To obtain such an object, call `texture.as_surface()`.
pub struct TextureSurface<'a>(framebuffer::SimpleFrameBuffer<'a>);

impl<'a> Surface for TextureSurface<'a> {
    fn clear(&mut self, rect: Option<&Rect>, color: Option<(f32, f32, f32, f32)>,
             depth: Option<f32>, stencil: Option<i32>)
    {
        self.0.clear(rect, color, depth, stencil)
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

    fn draw<'b, 'v, V, I, U>(&mut self, vb: V, ib: I, program: &::Program,
        uniforms: &U, draw_parameters: &::DrawParameters) -> Result<(), ::DrawError>
        where I: Into<::index::IndicesSource<'b>>,
        U: ::uniforms::Uniforms, V: ::vertex::MultiVerticesSource<'v>
    {
        self.0.draw(vb, ib, program, uniforms, draw_parameters)
    }

    fn blit_color<S>(&self, source_rect: &Rect, target: &S, target_rect: &BlitTarget,
                     filter: uniforms::MagnifySamplerFilter) where S: Surface
    {
        target.blit_from_simple_framebuffer(&self.0, source_rect, target_rect, filter)
    }

    fn blit_from_frame(&self, source_rect: &Rect, target_rect: &BlitTarget,
                       filter: uniforms::MagnifySamplerFilter)
    {
        self.0.blit_from_frame(source_rect, target_rect, filter)
    }

    fn blit_from_simple_framebuffer(&self, source: &framebuffer::SimpleFrameBuffer,
                                    source_rect: &Rect, target_rect: &BlitTarget,
                                    filter: uniforms::MagnifySamplerFilter)
    {
        self.0.blit_from_simple_framebuffer(source, source_rect, target_rect, filter)
    }

    fn blit_from_multioutput_framebuffer(&self, source: &framebuffer::MultiOutputFrameBuffer,
                                         source_rect: &Rect, target_rect: &BlitTarget,
                                         filter: uniforms::MagnifySamplerFilter)
    {
        self.0.blit_from_multioutput_framebuffer(source, source_rect, target_rect, filter)
    }
}

impl<'a> FboAttachments for TextureSurface<'a> {
    fn get_attachments(&self) -> Option<&ValidatedAttachments> {
        self.0.get_attachments()
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

impl From<FormatNotSupportedError> for TextureMaybeSupportedCreationError {
    fn from(_: FormatNotSupportedError) -> TextureMaybeSupportedCreationError {
        TextureMaybeSupportedCreationError::NotSupported
    }
}
