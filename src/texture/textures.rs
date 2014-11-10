use super::{TextureSurface, Texture1dData, Texture2dData, Texture3dData, TextureImplementation};
use super::Texture;
use super::{UncompressedFloatFormat, CompressedFormat};
use super::PixelValue;

use framebuffer;

use Display;

/// A one-dimensional texture.
pub struct Texture1d(TextureImplementation);

impl Texture1d {
    /// Creates a one-dimensional texture.
    pub fn new<P: PixelValue, T: Texture1dData<P>>(display: &Display, data: T) -> Texture1d {
        let data = data.into_vec();
        let width = data.len() as u32;
        let format = PixelValue::get_format(None::<P>).to_default_float_format();
        Texture1d(TextureImplementation::new(display, format, Some(data), width, None, None, None))
    }
}

impl Texture for Texture1d {
    fn get_implementation(&self) -> &TextureImplementation {
        &self.0
    }
}

/// An array of one-dimensional textures.
pub struct Texture1dArray(TextureImplementation);

impl Texture1dArray {
    /// Creates an array of one-dimensional textures.
    ///
    /// # Panic
    ///
    /// Panics if all the elements don't have the same dimensions.
    pub fn new<P: PixelValue, T: Texture1dData<P>>(display: &Display, data: Vec<T>)
        -> Texture1dArray
    {
        let array_size = data.len();
        let mut width = 0;
        let data = data.into_iter().flat_map(|t| {
            let d = t.into_vec(); width = d.len(); d.into_iter()
        }).collect();

        let format = PixelValue::get_format(None::<P>).to_default_float_format();

        Texture1dArray(TextureImplementation::new(display, format, Some(data), width as u32, None,
            None, Some(array_size as u32)))
    }
}

impl Texture for Texture1dArray {
    fn get_implementation(&self) -> &TextureImplementation {
        &self.0
    }
}

/// A two-dimensional texture. This is usually the texture that you want to use.
pub struct Texture2d(TextureImplementation);

impl Texture2d {
    /// Creates a two-dimensional texture.
    pub fn new<P: PixelValue, T: Texture2dData<P>>(display: &Display, data: T) -> Texture2d {
        let format = PixelValue::get_format(None::<P>).to_default_float_format();
        let dimensions = data.get_dimensions();
        let data = data.into_vec();

        Texture2d(TextureImplementation::new(display, format, Some(data), dimensions.0,
            Some(dimensions.1), None, None))
    }

    /// Creates an empty two-dimensional textures.
    ///
    /// The texture will contain undefined data.
    ///
    /// **Note**: you will need to pass a generic parameter.
    ///
    /// # Example 
    ///
    /// ```
    /// # extern crate glium;
    /// # extern crate glutin;
    /// # use glium::DisplayBuild;
    /// # fn main() {
    /// use glium::texture::FloatFormatU8U8U8U8;
    /// # let display: glium::Display = glutin::HeadlessRendererBuilder::new(1024, 768)
    /// #   .build_glium().unwrap();
    /// let texture = glium::Texture2d::new_empty(&display, FloatFormatU8U8U8U8, 512, 512);
    /// # }
    /// ```
    ///
    pub fn new_empty(display: &Display, format: UncompressedFloatFormat, width: u32,
        height: u32) -> Texture2d
    {
        let format = format.to_gl_enum();
        Texture2d(TextureImplementation::new::<u8>(display, format, None, width, Some(height),
            None, None))
    }

    /// Starts drawing on the texture.
    ///
    /// All the function calls to the `TextureSurface` will draw on the texture instead of the
    /// screen.
    ///
    /// ## Low-level informations
    ///
    /// The first time that this function is called, a FrameBuffer Object will be created and
    /// cached. The following calls to `as_surface` will load the existing FBO and re-use it.
    /// When the texture is destroyed, the FBO is destroyed too.
    ///
    pub fn as_surface<'a>(&'a self) -> TextureSurface<'a> {
        // TODO: hacky, shouldn't recreate a Display
        TextureSurface(framebuffer::FrameBuffer::new(&::Display { context: self.0.display.clone() })
            .with_color_texture(self))
    }

    /// Reads the content of the texture into a `Texture2DData`.
    pub fn read<P, T>(&self) -> T where P: PixelValue, T: Texture2dData<P> {
        let data = self.0.read::<P>(0);
        Texture2dData::from_vec(data, self.get_width() as u32)
    }
}

impl Texture for Texture2d {
    fn get_implementation(&self) -> &TextureImplementation {
        &self.0
    }
}

/// A compressed two-dimensional texture.
/// 
/// This is usually the texture that you want to use if you don't need to render to this texture.
///
/// A `CompressedTexture2d` uses less memory than a `Texture2d`, but can't be used as surfaces.
pub struct CompressedTexture2d(TextureImplementation);

impl CompressedTexture2d {
    /// Creates a two-dimensional texture.
    pub fn new<P: PixelValue, T: Texture2dData<P>>(display: &Display, data: T)
        -> CompressedTexture2d
    {
        let format = PixelValue::get_format(None::<P>).to_default_compressed_format();
        let dimensions = data.get_dimensions();
        let data = data.into_vec();

        CompressedTexture2d(TextureImplementation::new(display, format, Some(data), dimensions.0,
            Some(dimensions.1), None, None))
    }

    /// Creates a two-dimensional texture with a specific format.
    pub fn with_format<P: PixelValue, T: Texture2dData<P>>(display: &Display,
        format: CompressedFormat, data: T) -> CompressedTexture2d
    {
        let format = format.to_gl_enum();
        let dimensions = data.get_dimensions();
        let data = data.into_vec();

        CompressedTexture2d(TextureImplementation::new(display, format, Some(data), dimensions.0,
            Some(dimensions.1), None, None))
    }

    /// Reads the content of the texture into a `Texture2dData`.
    pub fn read<P, T>(&self) -> T where P: PixelValue, T: Texture2dData<P> {
        let data = self.0.read::<P>(0);
        Texture2dData::from_vec(data, self.get_width() as u32)
    }
}

impl Texture for CompressedTexture2d {
    fn get_implementation(&self) -> &TextureImplementation {
        &self.0
    }
}

/// An array of two-dimensional textures.
pub struct Texture2dArray(TextureImplementation);

impl Texture2dArray {
    /// Creates an array of two-dimensional textures.
    ///
    /// # Panic
    ///
    /// Panics if all the elements don't have the same dimensions.
    pub fn new<P: PixelValue, T: Texture2dData<P>>(display: &Display, data: Vec<T>)
        -> Texture2dArray
    {
        let array_size = data.len();
        let mut dimensions = (0, 0);
        let data = data.into_iter().flat_map(|t| {
            dimensions = t.get_dimensions(); t.into_vec().into_iter()
        }).collect();

        let format = PixelValue::get_format(None::<P>).to_default_float_format();

        Texture2dArray(TextureImplementation::new(display, format, Some(data), dimensions.0,
            Some(dimensions.1), None, Some(array_size as u32)))
    }
}

impl Texture for Texture2dArray {
    fn get_implementation(&self) -> &TextureImplementation {
        &self.0
    }
}

/// A three-dimensional texture.
pub struct Texture3d(TextureImplementation);

impl Texture3d {
    /// Creates a three-dimensional texture.
    pub fn new<P: PixelValue, T: Texture3dData<P>>(display: &Display, data: T) -> Texture3d {
        let dimensions = data.get_dimensions();
        let data = data.into_vec();
        let format = PixelValue::get_format(None::<P>).to_default_float_format();
        Texture3d(TextureImplementation::new(display, format, Some(data), dimensions.0,
            Some(dimensions.1), Some(dimensions.2), None))
    }
}

impl Texture for Texture3d {
    fn get_implementation(&self) -> &TextureImplementation {
        &self.0
    }
}
