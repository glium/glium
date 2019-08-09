//! Describes miscellaneous parameters to be used when drawing.
//!
//! # Example
//!
//! ```rust
//! let params = glium::DrawParameters {
//!     depth: glium::Depth {
//!         test: glium::draw_parameters::DepthTest::IfLess,
//!         write: true,
//!         .. Default::default()
//!     },
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
//! After drawing with these parameters, you can retrieve the value inside the query:
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

pub use self::blend::{Blend, BlendingFunction, LinearBlendingFactor};
pub use self::depth::{Depth, DepthTest, DepthClamp};
pub use self::query::{QueryCreationError};
pub use self::query::{SamplesPassedQuery, TimeElapsedQuery, PrimitivesGeneratedQuery};
pub use self::query::{AnySamplesPassedQuery, TransformFeedbackPrimitivesWrittenQuery};
pub use self::stencil::{StencilTest, StencilOperation, Stencil};

mod blend;
mod depth;
mod query;
mod stencil;

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
    CullCounterClockwise,

    /// Triangles whose vertices are clockwise won't be drawn.
    CullClockwise
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
///     depth: glium::Depth {
///         test: glium::DepthTest::IfLess,
///         write: true,
///         .. Default::default()
///     },
///     .. Default::default()
/// };
/// ```
///
#[derive(Clone, Debug)]
pub struct DrawParameters<'a> {
    /// How the fragment will interact with the depth buffer.
    pub depth: Depth,

    /// How the fragment will interact with the stencil buffer.
    pub stencil: Stencil,

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

    /// If the bit corresponding to 2^i is 1 in the bitmask, then GL_CLIP_DISTANCEi is enabled.
    ///
    /// The most common value for GL_MAX_CLIP_DISTANCES is 8, so 32 bits in the mask is plenty.
    ///
    /// See `https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/gl_ClipDistance.xhtml`.
    pub clip_planes_bitmask: u32,

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
    /// for the GPU to know where on the screen the primitive is. This field specifies the
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
    
    /// If enabled, will split the index buffer (if any is used in the draw call) 
    /// at the MAX value of the IndexType (u8::MAX, u16::MAX or u32::MAX) and start a new primitive
    /// of the same type ("primitive restarting"). Supported on > OpenGL 3.1 or OpenGL ES 3.0. 
    /// If the backend does not support GL_PRIMITIVE_RESTART_FIXED_INDEX, an Error 
    /// of type `FixedIndexRestartingNotSupported` will be returned.
    pub primitive_restart_index: bool,
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
            depth: Depth::default(),
            stencil: Default::default(),
            blend: Default::default(),
            color_mask: (true, true, true, true),
            line_width: None,
            point_size: None,
            backface_culling: BackfaceCullingMode::CullingDisabled,
            polygon_mode: PolygonMode::Fill,
            clip_planes_bitmask: 0,
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
            primitive_restart_index: false,
        }
    }
}

/// DEPRECATED. Checks parameters and returns an error if something is wrong.
pub fn validate(context: &Context, params: &DrawParameters) -> Result<(), DrawError> {
    if params.depth.range.0 < 0.0 || params.depth.range.0 > 1.0 ||
       params.depth.range.1 < 0.0 || params.depth.range.1 > 1.0
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
    depth::sync_depth(ctxt, &draw_parameters.depth)?;
    stencil::sync_stencil(ctxt, &draw_parameters.stencil);
    blend::sync_blending(ctxt, draw_parameters.blend)?;
    sync_color_mask(ctxt, draw_parameters.color_mask);
    sync_line_width(ctxt, draw_parameters.line_width);
    sync_point_size(ctxt, draw_parameters.point_size);
    sync_polygon_mode(ctxt, draw_parameters.backface_culling, draw_parameters.polygon_mode);
    sync_clip_planes_bitmask(ctxt, draw_parameters.clip_planes_bitmask)?;
    sync_multisampling(ctxt, draw_parameters.multisampling);
    sync_dithering(ctxt, draw_parameters.dithering);
    sync_viewport_scissor(ctxt, draw_parameters.viewport, draw_parameters.scissor,
                          dimensions);
    sync_rasterizer_discard(ctxt, draw_parameters.draw_primitives)?;
    sync_queries(ctxt, draw_parameters.samples_passed_query,
                      draw_parameters.time_elapsed_query,
                      draw_parameters.primitives_generated_query,
                      draw_parameters.transform_feedback_primitives_written_query)?;
    sync_conditional_render(ctxt, draw_parameters.condition);
    sync_smooth(ctxt, draw_parameters.smooth, primitives_types)?;
    sync_provoking_vertex(ctxt, draw_parameters.provoking_vertex)?;
    sync_primitive_bounding_box(ctxt, &draw_parameters.primitive_bounding_box);
    sync_primitive_restart_index(ctxt, draw_parameters.primitive_restart_index)?;

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
    //  that's why `CullClockwise` uses `GL_BACK` for example
    match backface_culling {
        BackfaceCullingMode::CullingDisabled => unsafe {
            if ctxt.state.enabled_cull_face {
                ctxt.gl.Disable(gl::CULL_FACE);
                ctxt.state.enabled_cull_face = false;
            }
        },
        BackfaceCullingMode::CullCounterClockwise => unsafe {
            if !ctxt.state.enabled_cull_face {
                ctxt.gl.Enable(gl::CULL_FACE);
                ctxt.state.enabled_cull_face = true;
            }
            if ctxt.state.cull_face != gl::FRONT {
                ctxt.gl.CullFace(gl::FRONT);
                ctxt.state.cull_face = gl::FRONT;
            }
        },
        BackfaceCullingMode::CullClockwise => unsafe {
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

fn sync_clip_planes_bitmask(ctxt: &mut context::CommandContext, clip_planes_bitmask: u32)
                            -> Result<(), DrawError> {
    unsafe {
        let mut max_clip_planes: gl::types::GLint = 0;
        ctxt.gl.GetIntegerv(gl::MAX_CLIP_DISTANCES, &mut max_clip_planes);
        for i in 0..32 {
            if clip_planes_bitmask & (1 << i) != 0 {
                if i < max_clip_planes {
                    ctxt.gl.Enable(gl::CLIP_DISTANCE0 + i as u32);
                } else {
                    return Err(DrawError::ClipPlaneIndexOutOfBounds);
                }
            } else {
                if i < max_clip_planes {
                    ctxt.gl.Disable(gl::CLIP_DISTANCE0 + i as u32);
                }
            }
        }
        Ok(())
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
        q.begin_query(ctxt)?;
    } else if let Some(SamplesQueryParam::AnySamplesPassedQuery(q)) = samples_passed_query {
        q.begin_query(ctxt)?;
    } else {
        TimeElapsedQuery::end_samples_passed_query(ctxt);
    }

    if let Some(time_elapsed_query) = time_elapsed_query {
        time_elapsed_query.begin_query(ctxt)?;
    } else {
        TimeElapsedQuery::end_time_elapsed_query(ctxt);
    }

    if let Some(primitives_generated_query) = primitives_generated_query {
        primitives_generated_query.begin_query(ctxt)?;
    } else {
        TimeElapsedQuery::end_primitives_generated_query(ctxt);
    }

    if let Some(tfq) = transform_feedback_primitives_written_query {
        tfq.begin_query(ctxt)?;
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

fn sync_primitive_restart_index(ctxt: &mut context::CommandContext,
                                enabled: bool)
                                -> Result<(), DrawError>
{
    // TODO: use GL_PRIMITIVE_RESTART (if possible) if 
    // GL_PRIMITIVE_RESTART_FIXED_INDEX is not supported
    if ctxt.version >= &Version(Api::Gl, 3, 1)   || ctxt.version >= &Version(Api::GlEs, 3, 0) ||
    ctxt.extensions.gl_arb_es3_compatibility
    {
        if enabled {
            unsafe { ctxt.gl.Enable(gl::PRIMITIVE_RESTART_FIXED_INDEX); }
            ctxt.state.enabled_primitive_fixed_restart = true;
        } else {
            unsafe { ctxt.gl.Disable(gl::PRIMITIVE_RESTART_FIXED_INDEX); }
            ctxt.state.enabled_primitive_fixed_restart = false;
        }

    } else {
        if enabled {
            return Err(DrawError::FixedIndexRestartingNotSupported);
        }
    }
    

    Ok(())
}
