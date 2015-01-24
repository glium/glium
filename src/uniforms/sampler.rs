use std::default::Default;
use ToGlEnum;
use gl;

/// Function to use for out-of-bounds samples.
///
/// This is how GL must handle samples that are outside the texture.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SamplerWrapFunction {
    /// Samples at coord `x + 1` map to coord `x`.
    Repeat,

    /// Samples at coord `x + 1` map to coord `1 - x`.
    Mirror,

    /// Samples at coord `x + 1` map to coord `1`.
    Clamp
}

impl ToGlEnum for SamplerWrapFunction {
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            SamplerWrapFunction::Repeat => gl::REPEAT,
            SamplerWrapFunction::Mirror => gl::MIRRORED_REPEAT,
            SamplerWrapFunction::Clamp => gl::CLAMP_TO_EDGE,
        }
    }
}

/// The function that the GPU will use when loading the value of a texel.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum MagnifySamplerFilter {
    /// The nearest texel will be loaded.
    Nearest,

    /// All nearby texels will be loaded and their values will be merged.
    Linear,
}

impl ToGlEnum for MagnifySamplerFilter {
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            MagnifySamplerFilter::Nearest => gl::NEAREST,
            MagnifySamplerFilter::Linear => gl::LINEAR,
        }
    }
}

/// The function that the GPU will use when loading the value of a texel.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum MinifySamplerFilter {
    /// The nearest texel will be loaded.
    ///
    /// Only uses the main texture, mipmaps are totally ignored.
    Nearest,

    /// All nearby texels will be loaded and their values will be merged.
    ///
    /// Only uses the main texture, mipmaps are totally ignored.
    Linear,

    /// The nearest texel of the nearest mipmap will be loaded.
    NearestMipmapNearest,

    /// Takes the nearest texel from the two nearest mipmaps, and merges them.
    LinearMipmapNearest,

    /// Same as `Linear`, but from the nearest mipmap.
    NearestMipmapLinear,

    /// Same as `Linear`, but from the two nearest mipmaps.
    LinearMipmapLinear,
}

impl ToGlEnum for MinifySamplerFilter {
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            MinifySamplerFilter::Nearest => gl::NEAREST,
            MinifySamplerFilter::Linear => gl::LINEAR,
            MinifySamplerFilter::NearestMipmapNearest => gl::NEAREST_MIPMAP_NEAREST,
            MinifySamplerFilter::LinearMipmapNearest => gl::LINEAR_MIPMAP_NEAREST,
            MinifySamplerFilter::NearestMipmapLinear => gl::NEAREST_MIPMAP_LINEAR,
            MinifySamplerFilter::LinearMipmapLinear => gl::LINEAR_MIPMAP_LINEAR,
        }
    }
}

/// A sampler.
pub struct Sampler<'t, T: 't>(pub &'t T, pub SamplerBehavior);

/// Behavior of a sampler.
// TODO: GL_TEXTURE_BORDER_COLOR, GL_TEXTURE_MIN_LOD, GL_TEXTURE_MAX_LOD, GL_TEXTURE_LOD_BIAS,
//       GL_TEXTURE_COMPARE_MODE, GL_TEXTURE_COMPARE_FUNC
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SamplerBehavior {
    /// Functions to use for the X, Y, and Z coordinates.
    pub wrap_function: (SamplerWrapFunction, SamplerWrapFunction, SamplerWrapFunction),
    /// Filter to use when minifying the texture.
    pub minify_filter: MinifySamplerFilter,
    /// Filter to use when magnifying the texture.
    pub magnify_filter: MagnifySamplerFilter,
    /// `1` means no anisotropic filtering, any value above `1` sets the max anisotropy.
    ///
    /// ## Compatibility
    ///
    /// This parameter is always available. However it is ignored on hardware that does
    /// not support anisotropic filtering.
    ///
    /// If you set the value to a value higher than what the hardware supports, it will
    /// be clamped.
    pub max_anisotropy: u16,
}

impl Default for SamplerBehavior {
    fn default() -> SamplerBehavior {
        SamplerBehavior {
            wrap_function: (
                SamplerWrapFunction::Mirror,
                SamplerWrapFunction::Mirror,
                SamplerWrapFunction::Mirror
            ),
            minify_filter: MinifySamplerFilter::Linear,
            magnify_filter: MagnifySamplerFilter::Linear,
            max_anisotropy: 1,
        }
    }
}
