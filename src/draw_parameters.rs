use gl;
use context::Context;
use version::Version;
use version::Api;

use DrawError;
use Rect;
use ToGlEnum;

use std::default::Default;

/// Function that the GPU will use for blending.
///
/// Blending happens at the end of the rendering process, when the GPU wants to write the
/// pixels over pixels that already exist in the framebuffer. The blending function allows
/// you to choose how it should merge the two.
///
/// If you want to add transparent objects one over another, the usual value
/// is `Addition { source: SourceAlpha, destination: OneMinusSourceAlpha }`.
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

    /// Multiply the source or destination component by `1.0` minus the alpha value of the source.
    OneMinusSourceAlpha,

    /// Multiply the source or destination component by the alpha value of the destination.
    DestinationAlpha,

    /// Multiply the source or destination component by `1.0` minus the alpha value of the
    /// destination.
    OneMinusDestinationAlpha,
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

/// Specifies which comparaison the GPU will do to determine whether a sample passes the stencil
/// test.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StencilTest {
    /// The stencil test always passes.
    AlwaysPass,

    /// The stencil test always fails.
    AlwaysFail,

    /// Applies the mask as a bitfield to both the value currently in the stencil buffer and
    /// the reference value (`stencil_reference_value_clockwise` or
    /// `stencil_reference_value_counter_clockwise`), then compares them.
    IfLess { mask: u32 },

    /// Applies the mask as a bitfield to both the value currently in the stencil buffer and
    /// the reference value (`stencil_reference_value_clockwise` or
    /// `stencil_reference_value_counter_clockwise`), then compares them.
    IfLessOrEqual { mask: u32 },

    /// Applies the mask as a bitfield to both the value currently in the stencil buffer and
    /// the reference value (`stencil_reference_value_clockwise` or
    /// `stencil_reference_value_counter_clockwise`), then compares them.
    IfMore { mask: u32 },

    /// Applies the mask as a bitfield to both the value currently in the stencil buffer and
    /// the reference value (`stencil_reference_value_clockwise` or
    /// `stencil_reference_value_counter_clockwise`), then compares them.
    IfMoreOrEqual { mask: u32 },

    /// Applies the mask as a bitfield to both the value currently in the stencil buffer and
    /// the reference value (`stencil_reference_value_clockwise` or
    /// `stencil_reference_value_counter_clockwise`), then compares them.
    IfEqual { mask: u32 },

    /// Applies the mask as a bitfield to both the value currently in the stencil buffer and
    /// the reference value (`stencil_reference_value_clockwise` or
    /// `stencil_reference_value_counter_clockwise`), then compares them.
    IfNotEqual { mask: u32 },
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
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            PolygonMode::Point => gl::POINT,
            PolygonMode::Line => gl::LINE,
            PolygonMode::Fill => gl::FILL,
        }
    }
}

/// Represents the parameters to use when drawing.
///
/// Example:
///
/// ```
/// let params = glium::DrawParameters {
///     depth_test: glium::DepthTest::IfLess,
///     depth_write: true,
///     .. std::default::Default::default()
/// };
/// ```
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawParameters {
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

    /// The function that the GPU will use to merge the existing pixel with the pixel that is
    /// being written.
    ///
    /// `None` means "don't care" (usually when you know that the alpha is always 1).
    pub blending_function: Option<BlendingFunction>,

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
}

impl Default for DrawParameters {
    fn default() -> DrawParameters {
        DrawParameters {
            depth_test: DepthTest::Overwrite,
            depth_write: false,
            depth_range: (0.0, 1.0),
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
            blending_function: Some(BlendingFunction::AlwaysReplace),
            line_width: None,
            point_size: None,
            backface_culling: BackfaceCullingMode::CullingDisabled,
            polygon_mode: PolygonMode::Fill,
            multisampling: true,
            dithering: true,
            viewport: None,
            scissor: None,
            draw_primitives: true,
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
        return Err(DrawError::TransformFeedbackNotSupported);
    }

    Ok(())
}
