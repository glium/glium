//! Image units, views into specific planes of textures
use ToGlEnum;
use gl;

/// How we bind a texture to an image unit
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ImageUnitBehavior {
    /// The mip level to bind
    pub level: usize,
    pub(crate) layer: Option<usize>,
    /// How the shader will access the image unit
    pub access: ImageUnitAccess,
    /// How the shader should interpret the image
    pub format: ImageUnitFormat,
}

impl Default for ImageUnitBehavior {
    #[inline]
    fn default() -> ImageUnitBehavior {
        ImageUnitBehavior {
            level: 0,
            layer: None,
            access: ImageUnitAccess::Read,
            format: ImageUnitFormat::R32I,
        }
    }
}

/// An image unit uniform marker
pub struct ImageUnit<'t, T: 't>(pub &'t T, pub ImageUnitBehavior);

impl<'t, T: 't> ImageUnit<'t, T> {
    /// Create a new marker
    pub fn new(texture: &'t T) -> ImageUnit<'t, T> {
        ImageUnit(texture, Default::default())
    }

    /// Set the mip level that will be bound
    pub fn set_level(mut self, level: usize) -> Self {
        self.1.level = level;
        self
    }

    /// Sets the layer of the texture to bind, or None to disable layer binding
    /// TODO: only implement this for texture types where layering makes sense
    pub fn set_layer(mut self, layer: Option<usize>) -> Self {
        self.1.layer = layer;
        self
    }

    /// State how the shader will access the image unit
    pub fn set_access(mut self, access: ImageUnitAccess) -> Self {
        self.1.access = access;
        self
    }

    /// State how the shader should interpret the image data
    pub fn set_format(mut self, format: ImageUnitFormat) -> Self {
        self.1.format = format;
        self
    }
}

/// States how the shader will access the image unit
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ImageUnitAccess {
    /// The shader will only read from the image unit
    Read,
    /// The shader will only write to the image unit
    Write,
    /// The shader will perform both reads and writes to the image unit
    ReadWrite,
}

impl ToGlEnum for ImageUnitAccess {
    #[inline]
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            ImageUnitAccess::Read => gl::READ_ONLY,
            ImageUnitAccess::Write => gl::WRITE_ONLY,
            ImageUnitAccess::ReadWrite => gl::READ_WRITE,
        }
    }
}

/// How the shader should interpret the data in the image
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ImageUnitFormat {
    /// The image elements are 4-component 32 bit floating point
    RGBA32F,
    /// The image elements are 4-component 16 bit floating point
    RGBA16F,
    /// The image elements are 2-component 32 bit floating point
    RG32F,
    /// The image elements are 4-component 16 bit floating point
    RG16F,
    /// The image elements are 2 11-bit floats and 1 10-bit float
    R11FG11FB10F,
    /// The image elements are 1-component 32 bit floating point
    R32F,
    /// The image elements are 4-component 16 bit floating point
    R16F,

    /// The image elements are 4-component 32 bit unsigned integer
    RGBA32UI,
    /// The image elements are 4-component 16 bit unsigned integer
    RGBA16UI,
    /// The image elements have 3 10-bit unsigned integer components and 1 2-bit alpha component
    RGB10A2UI,
    /// The image elements are 4-component 8 bit unsigned integer
    RGBA8UI,
    /// The image elements are 2-component 32 bit unsigned integer
    RG32UI,
    /// The image elements are 2-component 16 bit unsigned integer
    RG16UI,
    /// The image elements are 2-component 8 bit unsigned integer
    RG8UI,
    /// The image elements are 1-component 32 bit unsigned integer
    R32UI,
    /// The image elements are 1-component 16 bit unsigned integer
    R16UI,
    /// The image elements are 1-component 8 bit unsigned integer
    R8UI,

    /// The image elements are 4-component 32 bit signed integer
    RGBA32I,
    /// The image elements are 4-component 16 bit signed integer
    RGBA16I,
    /// The image elements are 4-component 8 bit signed integer
    RGBA8I,
    /// The image elements are 2-component 32 bit signed integer
    RG32I,
    /// The image elements are 2-component 16 bit signed integer
    RG16I,
    /// The image elements are 2-component 8 bit signed integer
    RG8I,
    /// The image elements are 1-component 32 bit signed integer
    R32I,
    /// The image elements are 1-component 16 bit signed integer
    R16I,
    /// The image elements are 1-component 8 bit signed integer
    R8I,

    /// The image elements are 4-component 16 bit floating point
    RGBA16,
    /// The image elements are 3-component 10 bit floating point with 2 alpha bits
    RGB10A2,
    /// The image elements are 4-component 8 bit floating point
    RGBA8,
    /// The image elements are 2-component 16 bit floating point
    RG16,
    /// The image elements are 2-component 8 bit floating point
    RG8,
    /// The image elements are 1-component 16 bit floating point
    R16,
    /// The image elements are 1-component 8 bit floating point
    R8,

    /// The image elements are 4-component 16 bit floating point, normalized to the -1.0 to 1.0 range
    RGBA16snorm,
    /// The image elements are 4-component 8 bit floating point, normalized to the -1.0 to 1.0 range
    RGBA8snorm,
    /// The image elements are 2-component 16 bit floating point, normalized to the -1.0 to 1.0 range
    RG16snorm,
    /// The image elements are 2-component 8 bit floating point, normalized to the -1.0 to 1.0 range
    RG8snorm,
    /// The image elements are 1-component 16 bit floating point, normalized to the -1.0 to 1.0 range
    R16snorm,
    /// The image elements are 1-component 8 bit floating point, normalized to the -1.0 to 1.0 range
    R8snorm,
}

impl ToGlEnum for ImageUnitFormat {
    #[inline]
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            ImageUnitFormat::RGBA32F => gl::RGBA32F,
            ImageUnitFormat::RGBA16F => gl::RGBA16F,
            ImageUnitFormat::RG32F => gl::RG32F,
            ImageUnitFormat::RG16F => gl::RG16F,
            ImageUnitFormat::R11FG11FB10F => gl::R11F_G11F_B10F,
            ImageUnitFormat::R32F => gl::R32F,
            ImageUnitFormat::R16F => gl::R16F,

            ImageUnitFormat::RGBA32UI => gl::RGBA32UI,
            ImageUnitFormat::RGBA16UI => gl::RGBA16UI,
            ImageUnitFormat::RGB10A2UI => gl::RGB10_A2UI,
            ImageUnitFormat::RGBA8UI => gl::RGBA8UI,
            ImageUnitFormat::RG32UI => gl::RG32UI,
            ImageUnitFormat::RG16UI => gl::RG16UI,
            ImageUnitFormat::RG8UI => gl::RG8UI,
            ImageUnitFormat::R32UI => gl::R32UI,
            ImageUnitFormat::R16UI => gl::R16UI,
            ImageUnitFormat::R8UI => gl::R8UI,

            ImageUnitFormat::RGBA32I => gl::RGBA32I,
            ImageUnitFormat::RGBA16I => gl::RGBA16I,
            ImageUnitFormat::RGBA8I => gl::RGBA8I,
            ImageUnitFormat::RG32I => gl::RG32I,
            ImageUnitFormat::RG16I => gl::RG16I,
            ImageUnitFormat::RG8I => gl::RG8I,
            ImageUnitFormat::R32I => gl::R32I,
            ImageUnitFormat::R16I => gl::R16I,
            ImageUnitFormat::R8I => gl::R8I,

            ImageUnitFormat::RGBA16 => gl::RGBA16,
            ImageUnitFormat::RGB10A2 => gl::RGB10_A2,
            ImageUnitFormat::RGBA8 => gl::RGBA8,
            ImageUnitFormat::RG16 => gl::RG16,
            ImageUnitFormat::RG8 => gl::RG8,
            ImageUnitFormat::R16 => gl::R16,
            ImageUnitFormat::R8 => gl::R8,

            ImageUnitFormat::RGBA16snorm => gl::RGBA16_SNORM,
            ImageUnitFormat::RGBA8snorm => gl::RGBA8_SNORM,
            ImageUnitFormat::RG16snorm => gl::RG16_SNORM,
            ImageUnitFormat::RG8snorm => gl::RG8_SNORM,
            ImageUnitFormat::R16snorm => gl::R16_SNORM,
            ImageUnitFormat::R8snorm => gl::R8_SNORM,
        }
    }
}
