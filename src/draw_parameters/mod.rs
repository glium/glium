//! Describes miscellaneous parameters to be used when drawing.
//!
//! # Example
//!
//! ```rust
//! let params = glium::DrawParameters {
//!     depth_test: glium::draw_parameters::DepthTest::IfLess,
//!     depth_write: true,
//!     scissor: Some(glium::Rect { bottom: 0, left: 100, width: 100, height: 200 }),
//!     .. Default::default()
//! };
//! ```
//!
//! # Queries
//!
//! Query objects allow you to obtain information about the rendering process. For example, a
//! `SamplesPassedQuery` allows you to know the number of samples that have been drawn.
//!
//! ```no_run
//! # let display: glium::Display = unsafe { ::std::mem::uninitialized() };
//! let query = glium::draw_parameters::SamplesPassedQuery::new(&display).unwrap();
//! let params = glium::DrawParameters {
//!     samples_passed_query: Some((&query).into()),
//!     .. Default::default()
//! };
//! ```
//!
//! After drawing with these parameters, you can retreive the value inside the query:
//!
//! ```no_run
//! # let query: glium::draw_parameters::SamplesPassedQuery = unsafe { std::mem::uninitialized() };
//! let value = query.get();
//! ```
//!
//! This operation will consume the query and block until the GPU has finished drawing. Instead,
//! you can also use the query as a condition for drawing:
//!
//! ```no_run
//! # let query: glium::draw_parameters::SamplesPassedQuery = unsafe { std::mem::uninitialized() };
//! let params = glium::DrawParameters {
//!     condition: Some(glium::draw_parameters::ConditionalRendering {
//!         query: (&query).into(),
//!         wait: true,
//!         per_region: true,
//!     }),
//!     .. Default::default()
//! };
//! ```
//!
//! If you use conditional rendering, glium will submit the draw command but the GPU will execute
//! it only if the query contains a value different from 0.
//!
//! ## WrongQueryOperation errors
//!
//! OpenGL puts some restrictions about the usage of queries. If you draw one or several times
//! with a query, then draw *without* that query, then the query cannot be used again. Trying
//! to draw with it results in a `WrongQueryOperation` error returned by the `draw` function.
//!
//! For the same reasons, as soon as you call `is_ready` on a query it will stop being usable.
//!
use gl;
use context;
use context::Context;
use version::Version;
use version::Api;

use index::PrimitiveType;

use QueryExt;
use CapabilitiesSource;
use DrawError;
use Rect;
use ToGlEnum;
use vertex::TransformFeedbackSession;

use std::ops::Range;

pub use self::query::{QueryCreationError};
pub use self::query::{SamplesPassedQuery, TimeElapsedQuery, PrimitivesGeneratedQuery};
pub use self::query::{AnySamplesPassedQuery, TransformFeedbackPrimitivesWrittenQuery};

mod query;

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

    /// For each individual component (red, green, blue, and alpha), a weighted substraction
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

    /// For each individual component (red, green, blue, and alpha), a weighted substraction
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

    /// Multiply the source or destination compoent by `1.0` minus the corresponding
    /// value in `Blend::const_value`.
    OneMinusConstantColor,

    /// Multiply the source or destination component by the alpha value of `Blend::const_value`.
    ConstantAlpha,

    /// Multiply the source or destination componet by `1.0` minus the alpha value of
    /// `Blend::const_value`.
    OneMinusConstantAlpha,
}

impl ToGlEnum for LinearBlendingFactor {
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

/// Describes how triangles should be filtered before the fragment processing. Backface culling
/// is purely an optimization. If you don't know what this does, just use `CullingDisabled`.
///
/// # Backface culling
///
/// After the vertex shader stage, the GPU knows the 2D coordinates of each vertex of
/// each triangle.
///
/// For a given triangle, there are only two situations:
///
/// - The vertices are arranged in a clockwise direction on the screen.
/// - The vertices are arranged in a counterclockwise direction on the screen.
///
/// If you wish so, you can ask the GPU to discard all the primitives that belong to one
/// of these two categories.
///
/// ## Example
///
/// The vertices of this triangle are counter-clock-wise.
///
/// <svg width="556.84381" height="509.69049" version="1.1">
///   <g transform="translate(-95.156215,-320.37201)">
///     <path style="fill:none;stroke:#000000;stroke-width:4;stroke-miterlimit:4;stroke-opacity:1;stroke-dasharray:none" d="M 324.25897,418.99654 539.42145,726.08292 212.13204,741.23521 z" />
///     <text style="font-size:40px;font-style:normal;font-weight:normal;line-height:125%;letter-spacing:0px;word-spacing:0px;fill:#000000;fill-opacity:1;stroke:none;font-family:Sans" x="296.98483" y="400.81378"><tspan x="296.98483" y="400.81378">1</tspan></text>
///     <text style="font-size:40px;font-style:normal;font-weight:normal;line-height:125%;letter-spacing:0px;word-spacing:0px;fill:#000000;fill-opacity:1;stroke:none;font-family:Sans" x="175.22902" y="774.8031"><tspan x="175.22902" y="774.8031">2</tspan></text>
///     <text style="font-size:40px;font-style:normal;font-weight:normal;line-height:125%;letter-spacing:0px;word-spacing:0px;fill:#000000;fill-opacity:1;stroke:none;font-family:Sans" x="555.58386" y="748.30627"><tspan x="555.58386" y="748.30627">3</tspan></text>
///   </g>
/// </svg>
///
/// # Usage
///
/// The trick is that if you make a 180° rotation of a shape, all triangles that were
/// clockwise become counterclockwise and vice versa.
///
/// Therefore you can arrange your model so that the triangles that are facing the screen
/// are all either clockwise or counterclockwise, and all the triangle are *not* facing
/// the screen are the other one.
///
/// By doing so you can use backface culling to discard all the triangles that are not
/// facing the screen, and increase your framerate.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackfaceCullingMode {
    /// All triangles are always drawn.
    CullingDisabled,

    /// Triangles whose vertices are counterclockwise won't be drawn.
    CullCounterClockWise,

    /// Triangles whose vertices are clockwise won't be drawn.
    CullClockWise
}

/// The function that the GPU will use to determine whether to write over an existing pixel
/// on the target.
///
/// # Depth buffers
///
/// After the fragment shader has been run, the GPU maps the output Z coordinates to the depth
/// range (which you can specify in the draw parameters) in order to obtain the depth value in
/// in window coordinates. This depth value is always between `0.0` and `1.0`.
///
/// In addition to the buffer where pixel colors are stored, you can also have a buffer
/// which contains the depth value of each pixel. Whenever the GPU tries to write a pixel,
/// it will first compare the depth value of the pixel to be written with the depth value that
/// is stored at this location. If `depth_write` is set to `true` in the draw parameters, it will
/// then write the depth value in the buffer.
///
/// The most common value for depth testing is to set `depth_test` to `IfLess`, and `depth_write`
/// to `true`.
///
/// If you don't have a depth buffer available, you can only pass `Overwrite`. Glium detects if
/// you pass any other value and reports an error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepthTest {
    /// Never replace the target pixel.
    ///
    /// This option doesn't really make sense, but is here for completeness.
    Ignore,

    /// Always replace the target pixel.
    ///
    /// This is the default mode.
    Overwrite,

    /// Replace if the z-value of the source is equal to the destination.
    IfEqual,

    /// Replace if the z-value of the source is different than the destination.
    IfNotEqual,

    /// Replace if the z-value of the source is more than the destination.
    IfMore,

    /// Replace if the z-value of the source is more than, or equal to the destination.
    IfMoreOrEqual,

    /// Replace if the z-value of the source is less than the destination.
    IfLess,

    /// Replace if the z-value of the source is less than, or equal to the destination.
    IfLessOrEqual
}

impl DepthTest {
    /// Returns true if the function requires a depth buffer to be used.
    #[inline]
    pub fn requires_depth_buffer(&self) -> bool {
        match *self {
            DepthTest::Ignore => true,
            DepthTest::Overwrite => false,
            DepthTest::IfEqual => true,
            DepthTest::IfNotEqual => true,
            DepthTest::IfMore => true,
            DepthTest::IfMoreOrEqual => true,
            DepthTest::IfLess => true,
            DepthTest::IfLessOrEqual => true,
        }
    }
}

impl ToGlEnum for DepthTest {
    #[inline]
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            DepthTest::Ignore => gl::NEVER,
            DepthTest::Overwrite => gl::ALWAYS,
            DepthTest::IfEqual => gl::EQUAL,
            DepthTest::IfNotEqual => gl::NOTEQUAL,
            DepthTest::IfMore => gl::GREATER,
            DepthTest::IfMoreOrEqual => gl::GEQUAL,
            DepthTest::IfLess => gl::LESS,
            DepthTest::IfLessOrEqual => gl::LEQUAL,
        }
    }
}

/// Specifies whether the depth value of samples should be clamped to `0.0` or `1.0`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepthClamp {
    /// Do not clamp. Samples with values outside of the `[0.0, 1.0]` range will be discarded.
    ///
    /// This is the default value and is supported everywhere.
    NoClamp,

    /// Clamp the depth values. All samples will always be drawn.
    ///
    /// This value is only supported on OpenGL.
    Clamp,

    /// Depth values inferior to `0.0` will be clamped to `0.0`.
    ///
    /// **This option is supported only by very few OpenGL devices**.
    ClampNear,

    /// Depth values superior to `1.0` will be clamped to `1.0`.
    ///
    /// **This option is supported only by very few OpenGL devices**.
    ClampFar,
}

/// Specifies which comparison the GPU will do to determine whether a sample passes the stencil
/// test. The general equation is `(ref & mask) CMP (stencil & mask)`, where `ref` is the reference
/// value (`stencil_reference_value_clockwise` or `stencil_reference_value_counter_clockwise`),
/// `CMP` is the comparison chosen, and `stencil` is the current value in the stencil buffer.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StencilTest {
    /// The stencil test always passes.
    AlwaysPass,

    /// The stencil test always fails.
    AlwaysFail,

    /// `(ref & mask) < (stencil & mask)`
    IfLess {
        /// The mask that is and'ed with the reference value and stencil buffer.
        mask: u32
    },

    /// `(ref & mask) <= (stencil & mask)`
    IfLessOrEqual {
        /// The mask that is and'ed with the reference value and stencil buffer.
        mask: u32,
    },

    /// `(ref & mask) > (stencil & mask)`
    IfMore {
        /// The mask that is and'ed with the reference value and stencil buffer.
        mask: u32,
    },

    /// `(ref & mask) >= (stencil & mask)`
    IfMoreOrEqual {
        /// The mask that is and'ed with the reference value and stencil buffer.
        mask: u32,
    },

    /// `(ref & mask) == (stencil & mask)`
    IfEqual {
        /// The mask that is and'ed with the reference value and stencil buffer.
        mask: u32,
    },

    /// `(ref & mask) != (stencil & mask)`
    IfNotEqual {
        /// The mask that is and'ed with the reference value and stencil buffer.
        mask: u32,
    },
}

/// Specificies which operation the GPU will do depending on the result of the stencil test.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StencilOperation {
    /// Keeps the value currently in the stencil buffer.
    Keep,

    /// Writes zero in the stencil buffer.
    Zero,

    /// Writes the reference value (`stencil_reference_value_clockwise` or
    /// `stencil_reference_value_counter_clockwise`) in the stencil buffer.
    Replace,

    /// Increments the value currently in the stencil buffer. If the value is the
    /// maximum, don't do anything.
    Increment,

    /// Increments the value currently in the stencil buffer. If the value is the
    /// maximum, wrap to `0`.
    IncrementWrap,

    /// Decrements the value currently in the stencil buffer. If the value is `0`,
    /// don't do anything.
    Decrement,

    /// Decrements the value currently in the stencil buffer. If the value is `0`,
    /// wrap to `-1`.
    DecrementWrap,

    /// Inverts each bit of the value.
    Invert,
}

impl ToGlEnum for StencilOperation {
    #[inline]
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            StencilOperation::Keep => gl::KEEP,
            StencilOperation::Zero => gl::ZERO,
            StencilOperation::Replace => gl::REPLACE,
            StencilOperation::Increment => gl::INCR,
            StencilOperation::IncrementWrap => gl::INCR_WRAP,
            StencilOperation::Decrement => gl::DECR,
            StencilOperation::DecrementWrap => gl::DECR_WRAP,
            StencilOperation::Invert => gl::INVERT,
        }
    }
}

/// Defines how the device should render polygons.
///
/// The usual value is `Fill`, which fills the content of polygon with the color. However other
/// values are sometimes useful, especially for debugging purposes.
///
/// # Example
///
/// The same triangle drawn respectively with `Fill`, `Line` and `Point` (barely visible).
///
/// <svg width="890.26135" height="282.59375" version="1.1">
///  <g transform="translate(0,-769.9375)">
///     <path style="fill:#ff0000;fill-opacity:1;stroke:none" d="M 124.24877,771.03979 258.59906,1051.8622 0,1003.3749 z" />
///     <path style="fill:none;fill-opacity:1;stroke:#ff0000;stroke-opacity:1" d="M 444.46713,771.03979 578.81742,1051.8622 320.21836,1003.3749 z" />
///     <path style="fill:#ff0000;fill-opacity:1;stroke:none" d="m 814.91074,385.7662 c 0,0.0185 -0.015,0.0335 -0.0335,0.0335 -0.0185,0 -0.0335,-0.015 -0.0335,-0.0335 0,-0.0185 0.015,-0.0335 0.0335,-0.0335 0.0185,0 0.0335,0.015 0.0335,0.0335 z" transform="matrix(18.833333,0,0,18.833333,-14715.306,-6262.0056)" />
///     <path style="fill:#ff0000;fill-opacity:1;stroke:none" d="m 814.91074,385.7662 c 0,0.0185 -0.015,0.0335 -0.0335,0.0335 -0.0185,0 -0.0335,-0.015 -0.0335,-0.0335 0,-0.0185 0.015,-0.0335 0.0335,-0.0335 0.0185,0 0.0335,0.015 0.0335,0.0335 z" transform="matrix(18.833333,0,0,18.833333,-14591.26,-6493.994)" />
///     <path style="fill:#ff0000;fill-opacity:1;stroke:none" d="m 814.91074,385.7662 c 0,0.0185 -0.015,0.0335 -0.0335,0.0335 -0.0185,0 -0.0335,-0.015 -0.0335,-0.0335 0,-0.0185 0.015,-0.0335 0.0335,-0.0335 0.0185,0 0.0335,0.015 0.0335,0.0335 z" transform="matrix(18.833333,0,0,18.833333,-14457.224,-6213.6135)" />
///  </g>
/// </svg>
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolygonMode {
    /// Only draw a single point at each vertex.
    ///
    /// All attributes that apply to points (`point_size`) are used when using this mode.
    Point,

    /// Only draw a line in the boundaries of each polygon.
    ///
    /// All attributes that apply to lines (`line_width`) are used when using this mode.
    Line,

    /// Fill the content of the polygon. This is the default mode.
    Fill,
}

impl ToGlEnum for PolygonMode {
    #[inline]
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            PolygonMode::Point => gl::POINT,
            PolygonMode::Line => gl::LINE,
            PolygonMode::Fill => gl::FILL,
        }
    }
}

/// Specifies a hint for the smoothing.
///
/// Note that this is just a hint and the driver may disregard it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Smooth {
    /// The most efficient option should be chosen.
    Fastest,

    /// The most correct, or highest quality, option should be chosen.
    Nicest,

    /// No preference.
    DontCare,
}

impl ToGlEnum for Smooth {
    #[inline]
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            Smooth::Fastest => gl::FASTEST,
            Smooth::Nicest => gl::NICEST,
            Smooth::DontCare => gl::DONT_CARE,
        }
    }
}

/// The vertex to use for flat shading.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProvokingVertex {
    /// Use the last vertex of each primitive.
    LastVertex,

    /// Use the first vertex of each primitive.
    ///
    /// Note that for triangle fans, this is not the first vertex but the second vertex.
    FirstVertex,
}

/// Represents the parameters to use when drawing.
///
/// Example:
///
/// ```
/// let params = glium::DrawParameters {
///     depth_test: glium::DepthTest::IfLess,
///     depth_write: true,
///     .. Default::default()
/// };
/// ```
///
#[derive(Clone, Debug)]
pub struct DrawParameters<'a> {
    /// The function that the GPU will use to determine whether to write over an existing pixel
    /// on the target. Don't forget to set `depth_write` appropriately if you use a depth test.
    ///
    /// See the `DepthTest` documentation for more details.
    ///
    /// The default is `Overwrite`.
    pub depth_test: DepthTest,

    /// Sets whether the GPU will write the depth values on the depth buffer if they pass the
    /// depth test.
    ///
    /// The default is `false`. You most likely want `true` if you're doing depth testing.
    ///
    /// If you pass `true` but don't have a depth buffer available, drawing will produce
    /// a `NoDepthBuffer` error.
    pub depth_write: bool,

    /// The range of possible Z values in surface coordinates.
    ///
    /// Just like OpenGL turns X and Y coordinates between `-1.0` and `1.0` into surface
    /// coordinates, it will also map your Z coordinates to a certain range which you can
    /// specify here.
    ///
    /// The two values must be between `0.0` and `1.0`, anything outside this range will result
    /// in a panic. By default the depth range is `(0.0, 1.0)`.
    ///
    /// The first value of the tuple must be the "near" value, where `-1.0` will be mapped.
    /// The second value must be the "far" value, where `1.0` will be mapped.
    /// It is possible for the "near" value to be greater than the "far" value.
    pub depth_range: (f32, f32),

    /// Sets whether the depth values of samples should be clamped to `0.0` and `1.0`.
    ///
    /// The default value is `NoClamp`.
    pub depth_clamp: DepthClamp,

    /// A comparaison against the existing value in the stencil buffer.
    ///
    /// Only relevant for faces that are clockwise on the target surface. Other faces, points and
    /// lines use `stencil_test_counter_clockwise` instead.
    ///
    /// The default value is `AlwaysPass`.
    pub stencil_test_clockwise: StencilTest,

    /// Reference value that is used by `stencil_test_clockwise`, `stencil_fail_operation_clockwise`,
    /// `stencil_pass_depth_fail_operation_clockwise` and `stencil_depth_pass_operation_clockwise`.
    pub stencil_reference_value_clockwise: i32,

    /// Allows specifying a mask when writing data on the stencil buffer.
    ///
    /// Only relevant for faces that are clockwise on the target surface. Other faces, points and
    /// lines use `stencil_write_mask_counter_clockwise` instead.
    ///
    /// The default value is `0xffffffff`.
    pub stencil_write_mask_clockwise: u32,

    /// Specifies the operation to do when a fragment fails the stencil test.
    ///
    /// The stencil test is the test specified by `stencil_test_clockwise`.
    ///
    /// Only relevant for faces that are clockwise on the target surface. Other faces, points and
    /// lines use `stencil_fail_operation_counter_clockwise` instead.
    ///
    /// The default value is `Keep`.
    pub stencil_fail_operation_clockwise: StencilOperation,

    /// Specifies the operation to do when a fragment passes the stencil test but fails
    /// the depth test.
    ///
    /// The stencil test is the test specified by `stencil_test_clockwise`.
    ///
    /// Only relevant for faces that are clockwise on the target surface. Other faces, points and
    /// lines use `stencil_pass_depth_fail_operation_counter_clockwise` instead.
    ///
    /// The default value is `Keep`.
    pub stencil_pass_depth_fail_operation_clockwise: StencilOperation,

    /// Specifies the operation to do when a fragment passes both the stencil and depth tests.
    ///
    /// The stencil test is the test specified by `stencil_test_clockwise`.
    ///
    /// Only relevant for faces that are clockwise on the target surface. Other faces, points and
    /// lines use `stencil_depth_pass_operation_counter_clockwise` instead.
    ///
    /// The default value is `Keep`.
    pub stencil_depth_pass_operation_clockwise: StencilOperation,

    /// A comparaison against the existing value in the stencil buffer.
    ///
    /// Only relevant for points, lines and faces that are counter-clockwise on the target surface.
    /// Other faces use `stencil_test_counter_clockwise` instead.
    ///
    /// The default value is `AlwaysPass`.
    pub stencil_test_counter_clockwise: StencilTest,

    /// Reference value that is used by `stencil_test_counter_clockwise`,
    /// `stencil_fail_operation_counter_clockwise`,
    /// `stencil_pass_depth_fail_operation_counter_clockwise` and
    /// `stencil_depth_pass_operation_counter_clockwise`.
    pub stencil_reference_value_counter_clockwise: i32,

    /// Allows specifying a mask when writing data on the stencil buffer.
    ///
    /// Only relevant for points, lines and faces that are counter-clockwise on the target surface.
    /// Other faces use `stencil_write_mask_clockwise` instead.
    ///
    /// The default value is `0xffffffff`.
    pub stencil_write_mask_counter_clockwise: u32,

    /// Specifies the operation to do when a fragment fails the stencil test.
    ///
    /// The stencil test is the test specified by `stencil_test_counter_clockwise`.
    ///
    /// Only relevant for faces that are counter-clockwise on the target surface. Other faces
    /// use `stencil_fail_operation_clockwise` instead.
    ///
    /// The default value is `Keep`.
    pub stencil_fail_operation_counter_clockwise: StencilOperation,

    /// Specifies the operation to do when a fragment passes the stencil test but fails
    /// the depth test.
    ///
    /// The stencil test is the test specified by `stencil_test_counter_clockwise`.
    ///
    /// Only relevant for faces that are counter-clockwise on the target surface. Other faces
    /// use `stencil_pass_depth_fail_operation_clockwise` instead.
    ///
    /// The default value is `Keep`.
    pub stencil_pass_depth_fail_operation_counter_clockwise: StencilOperation,

    /// Specifies the operation to do when a fragment passes both the stencil and depth tests.
    ///
    /// The stencil test is the test specified by `stencil_test_counter_clockwise`.
    ///
    /// Only relevant for faces that are counter-clockwise on the target surface. Other faces
    /// use `stencil_depth_pass_operation_clockwise` instead.
    ///
    /// The default value is `Keep`.
    pub stencil_depth_pass_operation_counter_clockwise: StencilOperation,

    /// The effect that the GPU will use to merge the existing pixel with the pixel that is
    /// being written.
    pub blend: Blend,

    /// Allows you to disable some color components.
    ///
    /// This affects all attachments to the framebuffer. It's at the same level as the
    /// blending function.
    ///
    /// The parameters are in order: red, green, blue, alpha. `true` means that the given
    /// component will be written, `false` means that it will be ignored. The default value
    /// is `(true, true, true, true)`.
    pub color_mask: (bool, bool, bool, bool),

    /// Width in pixels of the lines to draw when drawing lines.
    ///
    /// `None` means "don't care". Use this when you don't draw lines.
    pub line_width: Option<f32>,

    /// Diameter in pixels of the points to draw when drawing points.
    ///
    /// `None` means "don't care". Use this when you don't draw points.
    pub point_size: Option<f32>,

    /// Whether or not the GPU should filter out some faces.
    ///
    /// After the vertex shader stage, the GPU will try to remove the faces that aren't facing
    /// the camera.
    ///
    /// See the `BackfaceCullingMode` documentation for more infos.
    pub backface_culling: BackfaceCullingMode,

    /// How to render polygons. The default value is `Fill`.
    ///
    /// See the documentation of `PolygonMode` for more infos.
    pub polygon_mode: PolygonMode,

    /// Whether multisample antialiasing (MSAA) should be used. Default value is `true`.
    ///
    /// Note that you will need to set the appropriate option when creating the window.
    /// The recommended way to do is to leave this to `true`, and adjust the option when
    /// creating the window.
    pub multisampling: bool,

    /// Whether dithering is activated. Default value is `true`.
    ///
    /// Dithering will smoothen the transition between colors in your color buffer.
    pub dithering: bool,

    /// The viewport to use when drawing.
    ///
    /// The X and Y positions of your vertices are mapped to the viewport so that `(-1, -1)`
    /// corresponds to the lower-left hand corner and `(1, 1)` corresponds to the top-right
    /// hand corner. Any pixel outside of the viewport is discarded.
    ///
    /// You can specify a viewport greater than the target if you want to stretch the image.
    ///
    /// `None` means "use the whole surface".
    pub viewport: Option<Rect>,

    /// If specified, only pixels in this rect will be displayed. Default is `None`.
    ///
    /// This is different from a viewport. The image will stretch to fill the viewport, but
    /// not the scissor box.
    pub scissor: Option<Rect>,

    /// If `false`, the pipeline will stop after the primitives generation stage. The default
    /// value is `true`.
    ///
    /// If `false`, the fragment shader of your program won't be executed.
    ///
    /// If `false`, drawing may return `TransformFeedbackNotSupported` if the backend doesn't
    /// support this feature.
    ///
    /// This parameter may seem pointless, but it can be useful when you use transform
    /// feedback or if you just use your shaders to write to a buffer.
    pub draw_primitives: bool,

    /// If set, each sample (ie. usually each pixel) written to the output adds one to the
    /// counter of the `SamplesPassedQuery`.
    pub samples_passed_query: Option<SamplesQueryParam<'a>>,

    /// If set, the time it took for the GPU to execute this draw command is added to the total
    /// stored inside the `TimeElapsedQuery`.
    pub time_elapsed_query: Option<&'a TimeElapsedQuery>,

    /// If set, the number of primitives generated is added to the total stored inside the query.
    pub primitives_generated_query: Option<&'a PrimitivesGeneratedQuery>,

    /// If set, the number of vertices written by transform feedback.
    pub transform_feedback_primitives_written_query:
                                    Option<&'a TransformFeedbackPrimitivesWrittenQuery>,

    /// If set, the commands will only be executed if the specified query contains `true` or
    /// a number different than 0.
    pub condition: Option<ConditionalRendering<'a>>,

    /// If set, then the generated primitives will be written back to a buffer.
    pub transform_feedback: Option<&'a TransformFeedbackSession<'a>>,

    /// If set, then the generated primitives will be smoothed.
    ///
    /// Note that blending needs to be enabled for this to work.
    pub smooth: Option<Smooth>,

    /// In your vertex shader or geometry shader, you have the possibility to mark some output
    /// varyings as `flat`. If this is the case, the value of one of the vertices will be used
    /// for the whole primitive. This variable allows you to specify which vertex.
    ///
    /// The default value is `LastVertex`, as this is the default in OpenGL. Any other value can
    /// potentially trigger a `ProvokingVertexNotSupported` error. Most notably OpenGL ES doesn't
    /// support anything else but `LastVertex`.
    pub provoking_vertex: ProvokingVertex,

    /// Hint for the GPU of the bounding box of the geometry.
    ///
    /// If you're using geometry shaders or tessellation shaders, it can be extremely advantageous
    /// for the GPU to know where on the screen the primitive is. This field specificies the
    /// bounding box (`x`, `y`, `z`, `w`) of the primitive and serves as a hint to the GPU.
    ///
    /// The GPU is free not to draw samples outside of the bounding box. Whether the samples are
    /// drawn is implementation-specific.
    ///
    /// This field is useless if you're not using a geometry shader or tessellation shader.
    ///
    /// Since this is purely an optimization, this parameter is ignored if the backend doesn't
    /// support it.
    pub primitive_bounding_box: (Range<f32>, Range<f32>, Range<f32>, Range<f32>),
}

/// Condition whether to render or not.
#[derive(Debug, Copy, Clone)]
pub struct ConditionalRendering<'a> {
    /// The query to use.
    pub query: SamplesQueryParam<'a>,

    /// If true, the GPU will wait until the query result has been obtained. If false, the GPU
    /// is free to ignore the query and draw anyway.
    pub wait: bool,

    /// If true, only samples that match those that were written with the query active will
    /// be drawn.
    pub per_region: bool,
}

/// The query to use for samples counting.
#[derive(Debug, Copy, Clone)]
pub enum SamplesQueryParam<'a> {
    /// A `SamplesPassedQuery`.
    SamplesPassedQuery(&'a SamplesPassedQuery),
    /// A `AnySamplesPassedQuery`.
    AnySamplesPassedQuery(&'a AnySamplesPassedQuery),
}

impl<'a> From<&'a SamplesPassedQuery> for SamplesQueryParam<'a> {
    #[inline]
    fn from(r: &'a SamplesPassedQuery) -> SamplesQueryParam<'a> {
        SamplesQueryParam::SamplesPassedQuery(r)
    }
}

impl<'a> From<&'a AnySamplesPassedQuery> for SamplesQueryParam<'a> {
    #[inline]
    fn from(r: &'a AnySamplesPassedQuery) -> SamplesQueryParam<'a> {
        SamplesQueryParam::AnySamplesPassedQuery(r)
    }
}

impl<'a> Default for DrawParameters<'a> {
    fn default() -> DrawParameters<'a> {
        DrawParameters {
            depth_test: DepthTest::Overwrite,
            depth_write: false,
            depth_range: (0.0, 1.0),
            depth_clamp: DepthClamp::NoClamp,
            stencil_test_clockwise: StencilTest::AlwaysPass,
            stencil_reference_value_clockwise: 0,
            stencil_write_mask_clockwise: 0xffffffff,
            stencil_fail_operation_clockwise: StencilOperation::Keep,
            stencil_pass_depth_fail_operation_clockwise: StencilOperation::Keep,
            stencil_depth_pass_operation_clockwise: StencilOperation::Keep,
            stencil_test_counter_clockwise: StencilTest::AlwaysPass,
            stencil_reference_value_counter_clockwise: 0,
            stencil_write_mask_counter_clockwise: 0xffffffff,
            stencil_fail_operation_counter_clockwise: StencilOperation::Keep,
            stencil_pass_depth_fail_operation_counter_clockwise: StencilOperation::Keep,
            stencil_depth_pass_operation_counter_clockwise: StencilOperation::Keep,
            blend: Blend {
                color: BlendingFunction::AlwaysReplace,
                alpha: BlendingFunction::AlwaysReplace,
                constant_value: (1.0, 1.0, 1.0, 1.0)
            },
            color_mask: (true, true, true, true),
            line_width: None,
            point_size: None,
            backface_culling: BackfaceCullingMode::CullingDisabled,
            polygon_mode: PolygonMode::Fill,
            multisampling: true,
            dithering: true,
            viewport: None,
            scissor: None,
            draw_primitives: true,
            samples_passed_query: None,
            time_elapsed_query: None,
            primitives_generated_query: None,
            transform_feedback_primitives_written_query: None,
            condition: None,
            transform_feedback: None,
            smooth: None,
            provoking_vertex: ProvokingVertex::LastVertex,
            primitive_bounding_box: (-1.0 .. 1.0, -1.0 .. 1.0, -1.0 .. 1.0, -1.0 .. 1.0),
        }
    }
}

/// Checks parameters and panics if something is wrong.
pub fn validate(context: &Context, params: &DrawParameters) -> Result<(), DrawError> {
    if params.depth_range.0 < 0.0 || params.depth_range.0 > 1.0 ||
       params.depth_range.1 < 0.0 || params.depth_range.1 > 1.0
    {
        return Err(DrawError::InvalidDepthRange);
    }

    if !params.draw_primitives && context.get_version() < &Version(Api::Gl, 3, 0) &&
        !context.get_extensions().gl_ext_transform_feedback
    {
        return Err(DrawError::RasterizerDiscardNotSupported);
    }

    Ok(())
}

#[doc(hidden)]
pub fn sync(ctxt: &mut context::CommandContext, draw_parameters: &DrawParameters,
            dimensions: (u32, u32), primitives_types: PrimitiveType) -> Result<(), DrawError>
{
    try!(sync_depth(ctxt, draw_parameters.depth_test, draw_parameters.depth_write,
                    draw_parameters.depth_range, draw_parameters.depth_clamp));
    sync_stencil(ctxt, &draw_parameters);
    try!(sync_blending(ctxt, draw_parameters.blend));
    sync_color_mask(ctxt, draw_parameters.color_mask);
    sync_line_width(ctxt, draw_parameters.line_width);
    sync_point_size(ctxt, draw_parameters.point_size);
    sync_polygon_mode(ctxt, draw_parameters.backface_culling, draw_parameters.polygon_mode);
    sync_multisampling(ctxt, draw_parameters.multisampling);
    sync_dithering(ctxt, draw_parameters.dithering);
    sync_viewport_scissor(ctxt, draw_parameters.viewport, draw_parameters.scissor,
                          dimensions);
    try!(sync_rasterizer_discard(ctxt, draw_parameters.draw_primitives));
    try!(sync_queries(ctxt, draw_parameters.samples_passed_query,
                      draw_parameters.time_elapsed_query,
                      draw_parameters.primitives_generated_query,
                      draw_parameters.transform_feedback_primitives_written_query));
    sync_conditional_render(ctxt, draw_parameters.condition);
    try!(sync_smooth(ctxt, draw_parameters.smooth, primitives_types));
    try!(sync_provoking_vertex(ctxt, draw_parameters.provoking_vertex));
    sync_primitive_bounding_box(ctxt, &draw_parameters.primitive_bounding_box);

    Ok(())
}

fn sync_depth(ctxt: &mut context::CommandContext, depth_test: DepthTest, depth_write: bool,
              depth_range: (f32, f32), depth_clamp: DepthClamp) -> Result<(), DrawError>
{
    // depth clamp
    {
        let state = &mut *ctxt.state;
        match (depth_clamp, &mut state.enabled_depth_clamp_near,
               &mut state.enabled_depth_clamp_far)
        {
            (DepthClamp::NoClamp, &mut false, &mut false) => (),
            (DepthClamp::Clamp, &mut true, &mut true) => (),

            (DepthClamp::NoClamp, near, far) => {
                if ctxt.version >= &Version(Api::Gl, 3, 0) || ctxt.extensions.gl_arb_depth_clamp ||
                   ctxt.extensions.gl_nv_depth_clamp
                {
                    unsafe { ctxt.gl.Disable(gl::DEPTH_CLAMP) };
                    *near = false;
                    *far = false;
                } else {
                    return Err(DrawError::DepthClampNotSupported);
                }
            },

            (DepthClamp::Clamp, near, far) => {
                if ctxt.version >= &Version(Api::Gl, 3, 0) || ctxt.extensions.gl_arb_depth_clamp ||
                   ctxt.extensions.gl_nv_depth_clamp
                {
                    unsafe { ctxt.gl.Enable(gl::DEPTH_CLAMP) };
                    *near = true;
                    *far = true;
                } else {
                    return Err(DrawError::DepthClampNotSupported);
                }
            },

            (DepthClamp::ClampNear, &mut true, &mut false) => (),
            (DepthClamp::ClampFar, &mut false, &mut true) => (),

            (DepthClamp::ClampNear, &mut true, far) => {
                if ctxt.extensions.gl_amd_depth_clamp_separate {
                    unsafe { ctxt.gl.Disable(gl::DEPTH_CLAMP_FAR_AMD) };
                    *far = false;
                } else {
                    return Err(DrawError::DepthClampNotSupported);
                }

            },

            (DepthClamp::ClampNear, near @ &mut false, far) => {
                if ctxt.extensions.gl_amd_depth_clamp_separate {
                    unsafe { ctxt.gl.Enable(gl::DEPTH_CLAMP_NEAR_AMD) };
                    if *far { unsafe { ctxt.gl.Disable(gl::DEPTH_CLAMP_FAR_AMD); } }
                    *near = true;
                    *far = false;
                } else {
                    return Err(DrawError::DepthClampNotSupported);
                }
            },

            (DepthClamp::ClampFar, near, &mut true) => {
                if ctxt.extensions.gl_amd_depth_clamp_separate {
                    unsafe { ctxt.gl.Disable(gl::DEPTH_CLAMP_NEAR_AMD) };
                    *near = false;
                } else {
                    return Err(DrawError::DepthClampNotSupported);
                }
            },

            (DepthClamp::ClampFar, near, far @ &mut false) => {
                if ctxt.extensions.gl_amd_depth_clamp_separate {
                    unsafe { ctxt.gl.Enable(gl::DEPTH_CLAMP_FAR_AMD) };
                    if *near { unsafe { ctxt.gl.Disable(gl::DEPTH_CLAMP_NEAR_AMD); } }
                    *near = false;
                    *far = true;
                } else {
                    return Err(DrawError::DepthClampNotSupported);
                }
            },
        }
    }

    // depth range
    if depth_range != ctxt.state.depth_range {
        unsafe {
            ctxt.gl.DepthRange(depth_range.0 as f64, depth_range.1 as f64);
        }
        ctxt.state.depth_range = depth_range;
    }

    if depth_test == DepthTest::Overwrite && !depth_write {
        // simply disabling GL_DEPTH_TEST
        if ctxt.state.enabled_depth_test {
            unsafe { ctxt.gl.Disable(gl::DEPTH_TEST) };
            ctxt.state.enabled_depth_test = false;
        }
        return Ok(());

    } else {
        if !ctxt.state.enabled_depth_test {
            unsafe { ctxt.gl.Enable(gl::DEPTH_TEST) };
            ctxt.state.enabled_depth_test = true;
        }
    }

    // depth test
    unsafe {
        let depth_test = depth_test.to_glenum();
        if ctxt.state.depth_func != depth_test {
            ctxt.gl.DepthFunc(depth_test);
            ctxt.state.depth_func = depth_test;
        }
    }

    // depth mask
    if depth_write != ctxt.state.depth_mask {
        unsafe {
            ctxt.gl.DepthMask(if depth_write { gl::TRUE } else { gl::FALSE });
        }
        ctxt.state.depth_mask = depth_write;
    }

    Ok(())
}

fn sync_stencil(ctxt: &mut context::CommandContext, params: &DrawParameters) {
    // TODO: optimize me

    let (test_cw, read_mask_cw) = match params.stencil_test_clockwise {
        StencilTest::AlwaysPass => (gl::ALWAYS, 0),
        StencilTest::AlwaysFail => (gl::NEVER, 0),
        StencilTest::IfLess { mask } => (gl::LESS, mask),
        StencilTest::IfLessOrEqual { mask } => (gl::LEQUAL, mask),
        StencilTest::IfMore { mask } => (gl::GREATER, mask),
        StencilTest::IfMoreOrEqual { mask } => (gl::GEQUAL, mask),
        StencilTest::IfEqual { mask } => (gl::EQUAL, mask),
        StencilTest::IfNotEqual { mask } => (gl::NOTEQUAL, mask),
    };

    let (test_ccw, read_mask_ccw) = match params.stencil_test_counter_clockwise {
        StencilTest::AlwaysPass => (gl::ALWAYS, 0),
        StencilTest::AlwaysFail => (gl::NEVER, 0),
        StencilTest::IfLess { mask } => (gl::LESS, mask),
        StencilTest::IfLessOrEqual { mask } => (gl::LEQUAL, mask),
        StencilTest::IfMore { mask } => (gl::GREATER, mask),
        StencilTest::IfMoreOrEqual { mask } => (gl::GEQUAL, mask),
        StencilTest::IfEqual { mask } => (gl::EQUAL, mask),
        StencilTest::IfNotEqual { mask } => (gl::NOTEQUAL, mask),
    };

    if ctxt.state.stencil_func_back != (test_cw, params.stencil_reference_value_clockwise, read_mask_cw) {
        unsafe { ctxt.gl.StencilFuncSeparate(gl::BACK, test_cw, params.stencil_reference_value_clockwise, read_mask_cw) };
        ctxt.state.stencil_func_back = (test_cw, params.stencil_reference_value_clockwise, read_mask_cw);
    }

    if ctxt.state.stencil_func_front != (test_ccw, params.stencil_reference_value_counter_clockwise, read_mask_ccw) {
        unsafe { ctxt.gl.StencilFuncSeparate(gl::FRONT, test_ccw, params.stencil_reference_value_counter_clockwise, read_mask_ccw) };
        ctxt.state.stencil_func_front = (test_ccw, params.stencil_reference_value_counter_clockwise, read_mask_ccw);
    }

    if ctxt.state.stencil_mask_back != params.stencil_write_mask_clockwise {
        unsafe { ctxt.gl.StencilMaskSeparate(gl::BACK, params.stencil_write_mask_clockwise) };
        ctxt.state.stencil_mask_back = params.stencil_write_mask_clockwise;
    }

    if ctxt.state.stencil_mask_front != params.stencil_write_mask_clockwise {
        unsafe { ctxt.gl.StencilMaskSeparate(gl::FRONT, params.stencil_write_mask_clockwise) };
        ctxt.state.stencil_mask_front = params.stencil_write_mask_clockwise;
    }

    let op_back = (params.stencil_fail_operation_clockwise.to_glenum(),
                   params.stencil_pass_depth_fail_operation_clockwise.to_glenum(),
                   params.stencil_depth_pass_operation_clockwise.to_glenum());
    if ctxt.state.stencil_op_back != op_back {
        unsafe { ctxt.gl.StencilOpSeparate(gl::BACK, op_back.0, op_back.1, op_back.2) };
        ctxt.state.stencil_op_back = op_back;
    }

    let op_front = (params.stencil_fail_operation_counter_clockwise.to_glenum(),
                    params.stencil_pass_depth_fail_operation_counter_clockwise.to_glenum(),
                    params.stencil_depth_pass_operation_counter_clockwise.to_glenum());
    if ctxt.state.stencil_op_front != op_front {
        unsafe { ctxt.gl.StencilOpSeparate(gl::FRONT, op_front.0, op_front.1, op_front.2) };
        ctxt.state.stencil_op_front = op_front;
    }

    let enable_stencil = test_cw != gl::ALWAYS || test_ccw != gl::ALWAYS ||
                         op_back.0 != gl::KEEP || op_front.0 != gl::KEEP ||
                         op_back.1 != gl::KEEP || op_front.1 != gl::KEEP ||
                         op_back.2 != gl::KEEP || op_front.2 != gl::KEEP;
    if ctxt.state.enabled_stencil_test != enable_stencil {
        if enable_stencil {
            unsafe { ctxt.gl.Enable(gl::STENCIL_TEST) };
        } else {
            unsafe { ctxt.gl.Disable(gl::STENCIL_TEST) };
        }

        ctxt.state.enabled_stencil_test = enable_stencil;
    }
}

fn sync_blending(ctxt: &mut context::CommandContext, blend: Blend) -> Result<(), DrawError> {
    #[inline(always)]
    fn blend_eq(ctxt: &mut context::CommandContext, blending_function: BlendingFunction)
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

        let (color_eq, alpha_eq) = (try!(blend_eq(ctxt, blend.color)),
                                    try!(blend_eq(ctxt, blend.alpha)));
        if ctxt.state.blend_equation != (color_eq, alpha_eq) {
            unsafe { ctxt.gl.BlendEquationSeparate(color_eq, alpha_eq); }
            ctxt.state.blend_equation = (color_eq, alpha_eq);
        }

        // Map to dummy factors if the blending equation does not use the factors.
        let (color_factor_src, color_factor_dst) = blending_factors(blend.color)
            .unwrap_or((LinearBlendingFactor::One, LinearBlendingFactor::Zero));
        let (alpha_factor_src, alpha_factor_dst) = blending_factors(blend.alpha)
            .unwrap_or((LinearBlendingFactor::One, LinearBlendingFactor::Zero));
        let color_factor_src = color_factor_src.to_glenum();
        let color_factor_dst = color_factor_dst.to_glenum();
        let alpha_factor_src = alpha_factor_src.to_glenum();
        let alpha_factor_dst = alpha_factor_dst.to_glenum();
        if ctxt.state.blend_func != (color_factor_src, color_factor_dst,
            alpha_factor_src, alpha_factor_dst) {
            unsafe {
                ctxt.gl.BlendFuncSeparate(color_factor_src, color_factor_dst,
                    alpha_factor_src, alpha_factor_dst);
            }
            ctxt.state.blend_func = (color_factor_src, color_factor_dst,
                alpha_factor_src, alpha_factor_dst);
        }

        // Update blend color.
        if ctxt.state.blend_color != blend.constant_value {
            let (r, g, b, a) = blend.constant_value;
            unsafe { ctxt.gl.BlendColor(r, g, b, a); }
            ctxt.state.blend_color = blend.constant_value;
        }
    }

    Ok(())
}

fn sync_color_mask(ctxt: &mut context::CommandContext, mask: (bool, bool, bool, bool)) {
    let mask = (
        if mask.0 { 1 } else { 0 },
        if mask.1 { 1 } else { 0 },
        if mask.2 { 1 } else { 0 },
        if mask.3 { 1 } else { 0 },
    );

    if ctxt.state.color_mask != mask {
        unsafe {
            ctxt.gl.ColorMask(mask.0, mask.1, mask.2, mask.3);
        }

        ctxt.state.color_mask = mask;
    }
}

fn sync_line_width(ctxt: &mut context::CommandContext, line_width: Option<f32>) {
    if let Some(line_width) = line_width {
        if ctxt.state.line_width != line_width {
            unsafe {
                ctxt.gl.LineWidth(line_width);
                ctxt.state.line_width = line_width;
            }
        }
    }
}

fn sync_point_size(ctxt: &mut context::CommandContext, point_size: Option<f32>) {
    if let Some(point_size) = point_size {
        if ctxt.state.point_size != point_size {
            unsafe {
                ctxt.gl.PointSize(point_size);
                ctxt.state.point_size = point_size;
            }
        }
    }
}

fn sync_polygon_mode(ctxt: &mut context::CommandContext, backface_culling: BackfaceCullingMode,
                     polygon_mode: PolygonMode)
{
    // back-face culling
    // note: we never change the value of `glFrontFace`, whose default is GL_CCW
    //  that's why `CullClockWise` uses `GL_BACK` for example
    match backface_culling {
        BackfaceCullingMode::CullingDisabled => unsafe {
            if ctxt.state.enabled_cull_face {
                ctxt.gl.Disable(gl::CULL_FACE);
                ctxt.state.enabled_cull_face = false;
            }
        },
        BackfaceCullingMode::CullCounterClockWise => unsafe {
            if !ctxt.state.enabled_cull_face {
                ctxt.gl.Enable(gl::CULL_FACE);
                ctxt.state.enabled_cull_face = true;
            }
            if ctxt.state.cull_face != gl::FRONT {
                ctxt.gl.CullFace(gl::FRONT);
                ctxt.state.cull_face = gl::FRONT;
            }
        },
        BackfaceCullingMode::CullClockWise => unsafe {
            if !ctxt.state.enabled_cull_face {
                ctxt.gl.Enable(gl::CULL_FACE);
                ctxt.state.enabled_cull_face = true;
            }
            if ctxt.state.cull_face != gl::BACK {
                ctxt.gl.CullFace(gl::BACK);
                ctxt.state.cull_face = gl::BACK;
            }
        },
    }

    // polygon mode
    unsafe {
        let polygon_mode = polygon_mode.to_glenum();
        if ctxt.state.polygon_mode != polygon_mode {
            ctxt.gl.PolygonMode(gl::FRONT_AND_BACK, polygon_mode);
            ctxt.state.polygon_mode = polygon_mode;
        }
    }
}

fn sync_multisampling(ctxt: &mut context::CommandContext, multisampling: bool) {
    if ctxt.state.enabled_multisample != multisampling {
        unsafe {
            if multisampling {
                ctxt.gl.Enable(gl::MULTISAMPLE);
                ctxt.state.enabled_multisample = true;
            } else {
                ctxt.gl.Disable(gl::MULTISAMPLE);
                ctxt.state.enabled_multisample = false;
            }
        }
    }
}

fn sync_dithering(ctxt: &mut context::CommandContext, dithering: bool) {
    if ctxt.state.enabled_dither != dithering {
        unsafe {
            if dithering {
                ctxt.gl.Enable(gl::DITHER);
                ctxt.state.enabled_dither = true;
            } else {
                ctxt.gl.Disable(gl::DITHER);
                ctxt.state.enabled_dither = false;
            }
        }
    }
}

fn sync_viewport_scissor(ctxt: &mut context::CommandContext, viewport: Option<Rect>,
                         scissor: Option<Rect>, surface_dimensions: (u32, u32))
{
    // viewport
    if let Some(viewport) = viewport {
        assert!(viewport.width <= ctxt.capabilities.max_viewport_dims.0 as u32,
                "Viewport dimensions are too large");
        assert!(viewport.height <= ctxt.capabilities.max_viewport_dims.1 as u32,
                "Viewport dimensions are too large");

        let viewport = (viewport.left as gl::types::GLint, viewport.bottom as gl::types::GLint,
                        viewport.width as gl::types::GLsizei,
                        viewport.height as gl::types::GLsizei);

        if ctxt.state.viewport != Some(viewport) {
            unsafe { ctxt.gl.Viewport(viewport.0, viewport.1, viewport.2, viewport.3); }
            ctxt.state.viewport = Some(viewport);
        }

    } else {
        assert!(surface_dimensions.0 <= ctxt.capabilities.max_viewport_dims.0 as u32,
                "Viewport dimensions are too large");
        assert!(surface_dimensions.1 <= ctxt.capabilities.max_viewport_dims.1 as u32,
                "Viewport dimensions are too large");

        let viewport = (0, 0, surface_dimensions.0 as gl::types::GLsizei,
                        surface_dimensions.1 as gl::types::GLsizei);

        if ctxt.state.viewport != Some(viewport) {
            unsafe { ctxt.gl.Viewport(viewport.0, viewport.1, viewport.2, viewport.3); }
            ctxt.state.viewport = Some(viewport);
        }
    }

    // scissor
    if let Some(scissor) = scissor {
        let scissor = (scissor.left as gl::types::GLint, scissor.bottom as gl::types::GLint,
                       scissor.width as gl::types::GLsizei,
                       scissor.height as gl::types::GLsizei);

        unsafe {
            if ctxt.state.scissor != Some(scissor) {
                ctxt.gl.Scissor(scissor.0, scissor.1, scissor.2, scissor.3);
                ctxt.state.scissor = Some(scissor);
            }

            if !ctxt.state.enabled_scissor_test {
                ctxt.gl.Enable(gl::SCISSOR_TEST);
                ctxt.state.enabled_scissor_test = true;
            }
        }
    } else {
        unsafe {
            if ctxt.state.enabled_scissor_test {
                ctxt.gl.Disable(gl::SCISSOR_TEST);
                ctxt.state.enabled_scissor_test = false;
            }
        }
    }
}

fn sync_rasterizer_discard(ctxt: &mut context::CommandContext, draw_primitives: bool)
                           -> Result<(), DrawError>
{
    if ctxt.state.enabled_rasterizer_discard == draw_primitives {
        if ctxt.version >= &Version(Api::Gl, 3, 0) {
            if draw_primitives {
                unsafe { ctxt.gl.Disable(gl::RASTERIZER_DISCARD); }
                ctxt.state.enabled_rasterizer_discard = false;
            } else {
                unsafe { ctxt.gl.Enable(gl::RASTERIZER_DISCARD); }
                ctxt.state.enabled_rasterizer_discard = true;
            }

        } else if ctxt.extensions.gl_ext_transform_feedback {
            if draw_primitives {
                unsafe { ctxt.gl.Disable(gl::RASTERIZER_DISCARD_EXT); }
                ctxt.state.enabled_rasterizer_discard = false;
            } else {
                unsafe { ctxt.gl.Enable(gl::RASTERIZER_DISCARD_EXT); }
                ctxt.state.enabled_rasterizer_discard = true;
            }

        } else {
            return Err(DrawError::RasterizerDiscardNotSupported);
        }
    }

    Ok(())
}

fn sync_queries(ctxt: &mut context::CommandContext,
                samples_passed_query: Option<SamplesQueryParam>,
                time_elapsed_query: Option<&TimeElapsedQuery>,
                primitives_generated_query: Option<&PrimitivesGeneratedQuery>,
                transform_feedback_primitives_written_query:
                                            Option<&TransformFeedbackPrimitivesWrittenQuery>)
                -> Result<(), DrawError>
{
    if let Some(SamplesQueryParam::SamplesPassedQuery(q)) = samples_passed_query {
        try!(q.begin_query(ctxt));
    } else if let Some(SamplesQueryParam::AnySamplesPassedQuery(q)) = samples_passed_query {
        try!(q.begin_query(ctxt));
    } else {
        TimeElapsedQuery::end_samples_passed_query(ctxt);
    }

    if let Some(time_elapsed_query) = time_elapsed_query {
        try!(time_elapsed_query.begin_query(ctxt));
    } else {
        TimeElapsedQuery::end_time_elapsed_query(ctxt);
    }

    if let Some(primitives_generated_query) = primitives_generated_query {
        try!(primitives_generated_query.begin_query(ctxt));
    } else {
        TimeElapsedQuery::end_primitives_generated_query(ctxt);
    }

    if let Some(tfq) = transform_feedback_primitives_written_query {
        try!(tfq.begin_query(ctxt));
    } else {
        TimeElapsedQuery::end_transform_feedback_primitives_written_query(ctxt);
    }

    Ok(())
}

fn sync_conditional_render(ctxt: &mut context::CommandContext,
                           condition: Option<ConditionalRendering>)
{
    if let Some(ConditionalRendering { query, wait, per_region }) = condition {
        match query {
            SamplesQueryParam::SamplesPassedQuery(ref q) => {
                q.begin_conditional_render(ctxt, wait, per_region);
            },
            SamplesQueryParam::AnySamplesPassedQuery(ref q) => {
                q.begin_conditional_render(ctxt, wait, per_region);
            },
        }

    } else {
        TimeElapsedQuery::end_conditional_render(ctxt);
    }
}

fn sync_smooth(ctxt: &mut context::CommandContext,
               smooth: Option<Smooth>,
               primitive_type: PrimitiveType) -> Result<(), DrawError> {

    if let Some(smooth) = smooth {
        // check if smoothing is supported, it isn't on OpenGL ES
        if !(ctxt.version >= &Version(Api::Gl, 1, 0)) {
            return Err(DrawError::SmoothingNotSupported);
        }

        let hint = smooth.to_glenum();

        match primitive_type {
            // point
            PrimitiveType::Points =>
                return Err(DrawError::SmoothingNotSupported),

            // line
            PrimitiveType::LinesList | PrimitiveType::LinesListAdjacency |
            PrimitiveType::LineStrip | PrimitiveType::LineStripAdjacency |
            PrimitiveType::LineLoop => unsafe {
                if !ctxt.state.enabled_line_smooth {
                    ctxt.state.enabled_line_smooth = true;
                    ctxt.gl.Enable(gl::LINE_SMOOTH);
                }

                if ctxt.state.smooth.0 != hint {
                    ctxt.state.smooth.0 = hint;
                    ctxt.gl.Hint(gl::LINE_SMOOTH_HINT, hint);
                }
            },

            // polygon
            _ => unsafe {
                if !ctxt.state.enabled_polygon_smooth {
                    ctxt.state.enabled_polygon_smooth = true;
                    ctxt.gl.Enable(gl::POLYGON_SMOOTH);
                }

                if ctxt.state.smooth.1 != hint {
                    ctxt.state.smooth.1 = hint;
                    ctxt.gl.Hint(gl::POLYGON_SMOOTH_HINT, hint);
                }
            }
          }
        }
        else {
          match primitive_type {
            // point
            PrimitiveType::Points => (),

            // line
            PrimitiveType::LinesList | PrimitiveType::LinesListAdjacency |
            PrimitiveType::LineStrip | PrimitiveType::LineStripAdjacency |
            PrimitiveType::LineLoop => unsafe {
                if ctxt.state.enabled_line_smooth {
                    ctxt.state.enabled_line_smooth = false;
                    ctxt.gl.Disable(gl::LINE_SMOOTH);
                }
            },

            // polygon
            _ => unsafe {
                if ctxt.state.enabled_polygon_smooth {
                    ctxt.state.enabled_polygon_smooth = false;
                    ctxt.gl.Disable(gl::POLYGON_SMOOTH);
                }
            }
        }
    }

    Ok(())
}

fn sync_provoking_vertex(ctxt: &mut context::CommandContext, value: ProvokingVertex)
                         -> Result<(), DrawError>
{
    let value = match value {
        ProvokingVertex::LastVertex => gl::LAST_VERTEX_CONVENTION,
        ProvokingVertex::FirstVertex => gl::FIRST_VERTEX_CONVENTION,
    };

    if ctxt.state.provoking_vertex == value {
        return Ok(());
    }

    if ctxt.version >= &Version(Api::Gl, 3, 2) || ctxt.extensions.gl_arb_provoking_vertex {
        unsafe { ctxt.gl.ProvokingVertex(value); }
        ctxt.state.provoking_vertex = value;

    } else if ctxt.extensions.gl_ext_provoking_vertex {
        unsafe { ctxt.gl.ProvokingVertexEXT(value); }
        ctxt.state.provoking_vertex = value;

    } else {
        return Err(DrawError::ProvokingVertexNotSupported);
    }

    Ok(())
}

fn sync_primitive_bounding_box(ctxt: &mut context::CommandContext,
                               bb: &(Range<f32>, Range<f32>, Range<f32>, Range<f32>))
{
    let value = (bb.0.start, bb.1.start, bb.2.start, bb.3.start,
                 bb.0.end, bb.1.end, bb.2.end, bb.3.end);

    if ctxt.state.primitive_bounding_box == value {
        return;
    }

    if ctxt.version >= &Version(Api::GlEs, 3, 2) {
        unsafe { ctxt.gl.PrimitiveBoundingBox(value.0, value.1, value.2, value.3,
                                              value.4, value.5, value.6, value.7); }
        ctxt.state.primitive_bounding_box = value;

    } else if ctxt.extensions.gl_arb_es3_2_compatibility {
        unsafe { ctxt.gl.PrimitiveBoundingBoxARB(value.0, value.1, value.2, value.3,
                                                 value.4, value.5, value.6, value.7); }
        ctxt.state.primitive_bounding_box = value;

    } else if ctxt.extensions.gl_oes_primitive_bounding_box {
        unsafe { ctxt.gl.PrimitiveBoundingBoxOES(value.0, value.1, value.2, value.3,
                                                 value.4, value.5, value.6, value.7); }
        ctxt.state.primitive_bounding_box = value;

    } else if ctxt.extensions.gl_ext_primitive_bounding_box {
        unsafe { ctxt.gl.PrimitiveBoundingBoxEXT(value.0, value.1, value.2, value.3,
                                                 value.4, value.5, value.6, value.7); }
        ctxt.state.primitive_bounding_box = value;
    }
}
