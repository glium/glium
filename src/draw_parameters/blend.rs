use context::CommandContext;
use version::Api;
use version::Version;

use DrawError;
use gl;

/// Blend effect that the GPU will use for blending.
///
/// Blending happens at the end of the rendering process, when the GPU wants to write the
/// pixels over pixels that already exist in the framebuffer. The blending function allows
/// you to choose how it should merge the two.
///
/// If you want to add transparent objects one over another, use
/// `Blend::alpha_blending()`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Blend {
    /// The blending function for color channels.
    pub color: BlendingFunction,
    /// The blending function for alpha channels.
    pub alpha: BlendingFunction,
    /// A constant color that can be used in the blending functions.
    pub constant_value: (f32, f32, f32, f32),
}

impl Blend {
    /// Returns a blend effect to add transparent objects over others.
    pub fn alpha_blending() -> Blend {
        Blend {
            color: BlendingFunction::Addition {
                source: LinearBlendingFactor::SourceAlpha,
                destination: LinearBlendingFactor::OneMinusSourceAlpha,
            },
            alpha: BlendingFunction::Addition {
                source: LinearBlendingFactor::SourceAlpha,
                destination: LinearBlendingFactor::OneMinusSourceAlpha
            },
            constant_value: (0.0, 0.0, 0.0, 0.0)
        }
    }
}

impl Default for Blend {
    fn default() -> Blend {
        Blend {
            color: BlendingFunction::AlwaysReplace,
            alpha: BlendingFunction::AlwaysReplace,
            constant_value: (1.0, 1.0, 1.0, 1.0),
        }
    }
}

/// Function that the GPU will use for blending.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendingFunction {
    /// Simply overwrite the destination pixel with the source pixel.
    ///
    /// The alpha channels are simply ignored. This is the default mode.
    ///
    /// For example writing `(0.5, 0.9, 0.4, 0.2)` over `(0.9, 0.1, 0.4, 0.3)` will
    /// result in `(0.5, 0.9, 0.4, 0.2)`.
    AlwaysReplace,

    /// For each individual component (red, green, blue, and alpha), the minimum value is chosen
    /// between the source and the destination.
    ///
    /// For example writing `(0.5, 0.9, 0.4, 0.2)` over `(0.9, 0.1, 0.4, 0.3)` will
    /// result in `(0.5, 0.1, 0.4, 0.2)`.
    Min,

    /// For each individual component (red, green, blue, and alpha), the maximum value is chosen
    /// between the source and the destination.
    ///
    /// For example writing `(0.5, 0.9, 0.4, 0.2)` over `(0.9, 0.1, 0.4, 0.3)` will
    /// result in `(0.9, 0.9, 0.4, 0.3)`.
    Max,

    /// For each individual component (red, green, blue, and alpha), a weighted addition
    /// between the source and the destination.
    ///
    /// The result is equal to `source_component * source_factor + dest_component * dest_factor`,
    /// where `source_factor` and `dest_factor` are the values of `source` and `destination` of
    /// this enum.
    Addition {
        /// The factor to apply to the source pixel.
        source: LinearBlendingFactor,

        /// The factor to apply to the destination pixel.
        destination: LinearBlendingFactor,
    },

    /// For each individual component (red, green, blue, and alpha), a weighted subtraction
    /// of the source by the destination.
    ///
    /// The result is equal to `source_component * source_factor - dest_component * dest_factor`,
    /// where `source_factor` and `dest_factor` are the values of `source` and `destination` of
    /// this enum.
    Subtraction {
        /// The factor to apply to the source pixel.
        source: LinearBlendingFactor,

        /// The factor to apply to the destination pixel.
        destination: LinearBlendingFactor,
    },

    /// For each individual component (red, green, blue, and alpha), a weighted subtraction
    /// of the destination by the source.
    ///
    /// The result is equal to `-source_component * source_factor + dest_component * dest_factor`,
    /// where `source_factor` and `dest_factor` are the values of `source` and `destination` of
    /// this enum.
    ReverseSubtraction {
        /// The factor to apply to the source pixel.
        source: LinearBlendingFactor,

        /// The factor to apply to the destination pixel.
        destination: LinearBlendingFactor,
    },
}

/// Indicates which value to multiply each component with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinearBlendingFactor {
    /// Multiply the source or destination component by zero, which always
    /// gives `0.0`.
    Zero,

    /// Multiply the source or destination component by one, which always
    /// gives you the original value.
    One,

    /// Multiply the source or destination component by its corresponding value
    /// in the source.
    ///
    /// If you apply this to the source components, you get the values squared.
    SourceColor,

    /// Equivalent to `1 - SourceColor`.
    OneMinusSourceColor,

    /// Multiply the source or destination component by its corresponding value
    /// in the destination.
    ///
    /// If you apply this to the destination components, you get the values squared.
    DestinationColor,

    /// Equivalent to `1 - DestinationColor`.
    OneMinusDestinationColor,

    /// Multiply the source or destination component by the alpha value of the source.
    SourceAlpha,

    /// Multiply the source or destination component by the smallest value of
    /// `SourceAlpha` and `1 - DestinationAlpha`.
    SourceAlphaSaturate,

    /// Multiply the source or destination component by `1.0` minus the alpha value of the source.
    OneMinusSourceAlpha,

    /// Multiply the source or destination component by the alpha value of the destination.
    DestinationAlpha,

    /// Multiply the source or destination component by `1.0` minus the alpha value of the
    /// destination.
    OneMinusDestinationAlpha,

    /// Multiply the source or destination component by the corresponding value
    /// in `Blend::const_value`.
    ConstantColor,

    /// Multiply the source or destination component by `1.0` minus the corresponding
    /// value in `Blend::const_value`.
    OneMinusConstantColor,

    /// Multiply the source or destination component by the alpha value of `Blend::const_value`.
    ConstantAlpha,

    /// Multiply the source or destination component by `1.0` minus the alpha value of
    /// `Blend::const_value`.
    OneMinusConstantAlpha,
}

impl LinearBlendingFactor {
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            LinearBlendingFactor::Zero => gl::ZERO,
            LinearBlendingFactor::One => gl::ONE,
            LinearBlendingFactor::SourceColor => gl::SRC_COLOR,
            LinearBlendingFactor::OneMinusSourceColor => gl::ONE_MINUS_SRC_COLOR,
            LinearBlendingFactor::DestinationColor => gl::DST_COLOR,
            LinearBlendingFactor::OneMinusDestinationColor => gl::ONE_MINUS_DST_COLOR,
            LinearBlendingFactor::SourceAlpha => gl::SRC_ALPHA,
            LinearBlendingFactor::OneMinusSourceAlpha => gl::ONE_MINUS_SRC_ALPHA,
            LinearBlendingFactor::DestinationAlpha => gl::DST_ALPHA,
            LinearBlendingFactor::OneMinusDestinationAlpha => gl::ONE_MINUS_DST_ALPHA,
            LinearBlendingFactor::SourceAlphaSaturate => gl::SRC_ALPHA_SATURATE,
            LinearBlendingFactor::ConstantColor => gl::CONSTANT_COLOR,
            LinearBlendingFactor::OneMinusConstantColor => gl::ONE_MINUS_CONSTANT_COLOR,
            LinearBlendingFactor::ConstantAlpha => gl::CONSTANT_ALPHA,
            LinearBlendingFactor::OneMinusConstantAlpha => gl::ONE_MINUS_CONSTANT_ALPHA,
        }
    }
}

pub fn sync_blending(ctxt: &mut CommandContext, blend: Blend) -> Result<(), DrawError> {
    #[inline(always)]
    fn blend_eq(ctxt: &mut CommandContext, blending_function: BlendingFunction)
                -> Result<gl::types::GLenum, DrawError>
    {
        match blending_function {
            BlendingFunction::AlwaysReplace |
            BlendingFunction::Addition { .. } => Ok(gl::FUNC_ADD),
            BlendingFunction::Subtraction { .. } => Ok(gl::FUNC_SUBTRACT),
            BlendingFunction::ReverseSubtraction { .. } => Ok(gl::FUNC_REVERSE_SUBTRACT),

            BlendingFunction::Min => {
                if ctxt.version <= &Version(Api::GlEs, 2, 0) &&
                   !ctxt.extensions.gl_ext_blend_minmax
                {
                    Err(DrawError::BlendingParameterNotSupported)
                } else {
                    Ok(gl::MIN)
                }
            },

            BlendingFunction::Max => {
                if ctxt.version <= &Version(Api::GlEs, 2, 0) &&
                   !ctxt.extensions.gl_ext_blend_minmax
                {
                    Err(DrawError::BlendingParameterNotSupported)
                } else {
                    Ok(gl::MAX)
                }
            },
        }
    }

    #[inline(always)]
    fn blending_factors(blending_function: BlendingFunction)
                        -> Option<(LinearBlendingFactor, LinearBlendingFactor)>
    {
        match blending_function {
            BlendingFunction::AlwaysReplace |
            BlendingFunction::Min |
            BlendingFunction::Max => None,
            BlendingFunction::Addition { source, destination } =>
                Some((source, destination)),
            BlendingFunction::Subtraction { source, destination } =>
                Some((source, destination)),
            BlendingFunction::ReverseSubtraction { source, destination } =>
                Some((source, destination)),
        }
    }

    if let (BlendingFunction::AlwaysReplace, BlendingFunction::AlwaysReplace) =
           (blend.color, blend.alpha)
    {
        // Both color and alpha always replace. This equals no blending.
        if ctxt.state.enabled_blend {
            unsafe { ctxt.gl.Disable(gl::BLEND); }
            ctxt.state.enabled_blend = false;
        }

    } else {
        if !ctxt.state.enabled_blend {
            unsafe { ctxt.gl.Enable(gl::BLEND); }
            ctxt.state.enabled_blend = true;
        }

        let (color_eq, alpha_eq) = (blend_eq(ctxt, blend.color)?,
                                    blend_eq(ctxt, blend.alpha)?);
        if ctxt.state.blend_equation != (color_eq, alpha_eq) {
            unsafe { ctxt.gl.BlendEquationSeparate(color_eq, alpha_eq); }
            ctxt.state.blend_equation = (color_eq, alpha_eq);
        }

        // Map to dummy factors if the blending equation does not use the factors.
        let (color_factor_src, color_factor_dst) = blending_factors(blend.color)
            .unwrap_or((LinearBlendingFactor::One, LinearBlendingFactor::Zero));
        let (alpha_factor_src, alpha_factor_dst) = blending_factors(blend.alpha)
            .unwrap_or((LinearBlendingFactor::One, LinearBlendingFactor::Zero));

        // Updating the blending color if necessary.
        if color_factor_src == LinearBlendingFactor::ConstantColor ||
           color_factor_src == LinearBlendingFactor::OneMinusConstantColor ||
           color_factor_dst == LinearBlendingFactor::ConstantColor ||
           color_factor_dst == LinearBlendingFactor::OneMinusConstantColor ||
           alpha_factor_src == LinearBlendingFactor::ConstantColor ||
           alpha_factor_src == LinearBlendingFactor::OneMinusConstantColor ||
           alpha_factor_dst == LinearBlendingFactor::ConstantColor ||
           alpha_factor_dst == LinearBlendingFactor::OneMinusConstantColor ||
           color_factor_src == LinearBlendingFactor::ConstantAlpha ||
           color_factor_src == LinearBlendingFactor::OneMinusConstantAlpha ||
           color_factor_dst == LinearBlendingFactor::ConstantAlpha ||
           color_factor_dst == LinearBlendingFactor::OneMinusConstantAlpha ||
           alpha_factor_src == LinearBlendingFactor::ConstantAlpha ||
           alpha_factor_src == LinearBlendingFactor::OneMinusConstantAlpha ||
           alpha_factor_dst == LinearBlendingFactor::ConstantAlpha ||
           alpha_factor_dst == LinearBlendingFactor::OneMinusConstantAlpha
        {
            if ctxt.state.blend_color != blend.constant_value {
                let (r, g, b, a) = blend.constant_value;
                unsafe { ctxt.gl.BlendColor(r, g, b, a); }
                ctxt.state.blend_color = blend.constant_value;
            }
        }

        // Updating the blending function if necessary.
        let color_factor_src = color_factor_src.to_glenum();
        let color_factor_dst = color_factor_dst.to_glenum();
        let alpha_factor_src = alpha_factor_src.to_glenum();
        let alpha_factor_dst = alpha_factor_dst.to_glenum();
        if ctxt.state.blend_func != (color_factor_src, color_factor_dst,
                                     alpha_factor_src, alpha_factor_dst)
        {
            unsafe {
                ctxt.gl.BlendFuncSeparate(color_factor_src, color_factor_dst,
                                          alpha_factor_src, alpha_factor_dst);
            }

            ctxt.state.blend_func = (color_factor_src, color_factor_dst,
                                     alpha_factor_src, alpha_factor_dst);
        }
    }

    Ok(())
}
