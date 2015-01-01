use gl;

use GlObject;
use ToGlEnum;

use Display;

/// Function to use for out-of-bounds samples.
///
/// This is how GL must handle samples that are outside the texture.
#[deriving(Show, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SamplerWrapFunction {
    /// Samples at coord `x + 1` are mapped to coord `x`.
    Repeat,

    /// Samples at coord `x + 1` are mapped to coord `1 - x`.
    Mirror,

    /// Samples at coord `x + 1` are mapped to coord `1`.
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
#[deriving(Show, Clone, Copy, Hash, PartialEq, Eq)]
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
#[deriving(Show, Clone, Copy, Hash, PartialEq, Eq)]
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

    /// Same as `Linear` but from the nearest mipmap.
    NearestMipmapLinear,

    /// 
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
#[deriving(Show, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SamplerBehavior {
    /// Functions to use for the X, Y, and Z coordinates.
    pub wrap_function: (SamplerWrapFunction, SamplerWrapFunction, SamplerWrapFunction),
    /// Filter to use when mignifying the texture.
    pub minify_filter: MinifySamplerFilter,
    /// Filter to use when magnifying the texture.
    pub magnify_filter: MagnifySamplerFilter,
    /// `1` means no anisotropic filtering, any value superior to `1` does.
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

impl ::std::default::Default for SamplerBehavior {
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

/// An OpenGL sampler object.
#[doc(hidden)]      // TODO: hack
pub struct SamplerObject {
    display: Display,
    id: gl::types::GLuint,
}

impl SamplerObject {
    #[doc(hidden)]
    pub fn new(display: &Display, behavior: &SamplerBehavior) -> SamplerObject {
        let (tx, rx) = channel();

        let behavior = behavior.clone();
        display.context.context.exec(move |: ctxt| {
            let sampler = unsafe {
                use std::mem;
                let mut sampler: gl::types::GLuint = mem::uninitialized();
                ctxt.gl.GenSamplers(1, &mut sampler);
                sampler
            };

            unsafe {
                ctxt.gl.SamplerParameteri(sampler, gl::TEXTURE_WRAP_S,
                    behavior.wrap_function.0.to_glenum() as gl::types::GLint);
                ctxt.gl.SamplerParameteri(sampler, gl::TEXTURE_WRAP_T,
                    behavior.wrap_function.1.to_glenum() as gl::types::GLint);
                ctxt.gl.SamplerParameteri(sampler, gl::TEXTURE_WRAP_R,
                    behavior.wrap_function.2.to_glenum() as gl::types::GLint);
                ctxt.gl.SamplerParameteri(sampler, gl::TEXTURE_MIN_FILTER,
                    behavior.minify_filter.to_glenum() as gl::types::GLint);
                ctxt.gl.SamplerParameteri(sampler, gl::TEXTURE_MAG_FILTER,
                    behavior.magnify_filter.to_glenum() as gl::types::GLint);

                if let Some(max_value) = ctxt.capabilities.max_texture_max_anisotropy {
                    let value = if behavior.max_anisotropy as f32 > max_value {
                        max_value
                    } else {
                        behavior.max_anisotropy as f32
                    };

                    ctxt.gl.SamplerParameterf(sampler, gl::TEXTURE_MAX_ANISOTROPY_EXT, value);
                }
            }

            tx.send(sampler);
        });

        SamplerObject {
            display: display.clone(),
            id: rx.recv(),
        }
    }
}

impl GlObject for SamplerObject {
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for SamplerObject {
    fn drop(&mut self) {
        let id = self.id;
        self.display.context.context.exec(move |: ctxt| {
            unsafe {
                ctxt.gl.DeleteSamplers(1, [id].as_ptr());
            }
        });
    }
}

#[doc(hidden)]      // TODO: hack
pub fn get_sampler(display: &::Display, behavior: &SamplerBehavior) -> gl::types::GLuint {
    match display.context.samplers.lock().unwrap().get(behavior) {
        Some(obj) => return obj.get_id(),
        None => ()
    };

    let sampler = SamplerObject::new(display, behavior);
    let id = sampler.get_id();
    display.context.samplers.lock().unwrap().insert(behavior.clone(), sampler);
    id
}
