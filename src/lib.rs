/*!
Easy-to-use high-level OpenGL3+ wrapper.

# Initialization

This library defines the `DisplayBuild` trait which is curently implemented only on
`glutin::WindowBuilder`.

Initialization is done by creating a `WindowBuilder` and calling `build_glium`.

```no_run
extern crate glutin;
extern crate glium;

fn main() {
    use glium::DisplayBuild;

    let display = glutin::WindowBuilder::new()
        .with_dimensions(1024, 768)
        .with_title("Hello world".to_string())
        .build_glium().unwrap();
}
```

The `display` object is the most important object of this library.

The window where you are drawing on will produce events. They can be received by calling
`display.poll_events()`.

## Complete example

We start by creating the vertex buffer, which contains the list of all the points that are part
of our mesh. The elements that we pass to `VertexBuffer::new` must implement the
`glium::vertex_buffer::VertexFormat` trait. We can easily do this by creating a custom struct
and adding the `#[vertex_format]` attribute to it.

You can check the documentation of the `vertex_buffer` module for more informations.

```no_run
# #![feature(plugin)]
#[plugin]
extern crate glium_macros;

# extern crate glium;
# fn main() {
#[vertex_format]
#[derive(Copy)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

# let display: glium::Display = unsafe { std::mem::uninitialized() };
let vertex_buffer = glium::VertexBuffer::new(&display, vec![
    Vertex { position: [-0.5, -0.5], color: [0.0, 1.0, 0.0] },
    Vertex { position: [ 0.0,  0.5], color: [0.0, 0.0, 1.0] },
    Vertex { position: [ 0.5, -0.5], color: [1.0, 0.0, 0.0] },
]);
# }
```

Then we create the index buffer, which contains informations about the primitives (triangles,
lines, etc.) that compose our mesh.

The last parameter is a list of indices that represent the positions of our points in the
vertex buffer.

```no_run
# let display: glium::Display = unsafe { std::mem::uninitialized() };
let index_buffer = glium::IndexBuffer::new(&display,
    glium::index_buffer::TrianglesList(vec![ 0u16, 1, 2 ]));
```

Then we create the program, which is composed of a *vertex shader*, a program executed once for
each element in our vertex buffer, and a *fragment shader*, a program executed once for each
pixel before it is written on the final image.

The purpose of a program is to instruct the GPU how to process our mesh in order to obtain pixels.

```no_run
# let display: glium::Display = unsafe { std::mem::uninitialized() };
let program = glium::Program::from_source(&display,
    // vertex shader
    "   #version 110

        uniform mat4 matrix;

        attribute vec2 position;
        attribute vec3 color;

        varying vec3 v_color;

        void main() {
            gl_Position = vec4(position, 0.0, 1.0) * matrix;
            v_color = color;
        }
    ",

    // fragment shader
    "   #version 110
        varying vec3 v_color;

        void main() {
            gl_FragColor = vec4(v_color, 1.0);
        }
    ",

    // optional geometry shader
    None
).unwrap();
```

*Note: Teaching you the GLSL language is not covered by this guide.*

You may notice that the `attribute` declarations in the vertex shader match the field names and
types of the elements in the vertex buffer. This is required, or drawing will result in an error.

In the example above, you may notice `uniform mat4 matrix;`. This is a *uniform*, in other words
a global variable in our program. We will need to tell glium what the value of `matrix` is by
creating an object that implements the `glium::uniforms::Uniforms` trait.

Similarly to the vertex buffer and vertex format, we can do so by creating a custom struct  and
adding the `#[uniforms]` attribute to it.

```no_run
# #![feature(plugin)]
#[plugin]
extern crate glium_macros;

# extern crate glium;
# fn main() {
#[uniforms]
struct Uniforms {
    matrix: [[f32; 4]; 4],
}

let uniforms = Uniforms {
    matrix: [
        [ 1.0, 0.0, 0.0, 0.0 ],
        [ 0.0, 1.0, 0.0, 0.0 ],
        [ 0.0, 0.0, 1.0, 0.0 ],
        [ 0.0, 0.0, 0.0, 1.0 ]
    ],
};
# }
```

Vertex buffers, index buffers and program should be stored between draws in order to avoid wasting
time, but objects that implement the `glium::uniforms::Uniforms` trait are usually constructed
every time you draw.

Fields of our `Uniforms` object can be any object that implements `glium::uniforms::UniformValue`.
This includes textures and samplers (not covered here). You can check the documentation of the
`uniforms` module for more informations.

Now that everything is initialized, we can finally draw something. To do so, call `display.draw()`
in order to obtain a `Frame` object. Note that it is also possible to draw on a texture by
calling `texture.as_surface()`, but this is not covered here.

The `Frame` object has a `draw` function which you can use to draw things.
Its arguments are the vertex buffer, index buffer, program, uniforms, and an object of type
`DrawParameters` which contains miscellaneous informations about how everything should be rendered
(depth test, blending, backface culling, etc.).

```no_run
use glium::Surface;
# let display: glium::Display = unsafe { std::mem::uninitialized() };
# let vertex_buffer: glium::VertexBuffer<u8> = unsafe { std::mem::uninitialized() };
# let index_buffer: glium::IndexBuffer = unsafe { std::mem::uninitialized() };
# let program: glium::Program = unsafe { std::mem::uninitialized() };
# let uniforms = glium::uniforms::EmptyUniforms;
let mut target = display.draw();
target.clear_color(0.0, 0.0, 0.0, 0.0);  // filling the output with the black color
target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &std::default::Default::default());
target.finish();
```

*/
#![feature(old_orphan_check)]
#![feature(slicing_syntax)]
#![feature(unboxed_closures)]
#![feature(unsafe_destructor)]
#![unstable]
#![warn(missing_docs)]

// TODO: remove these when everything is implemented
#![allow(dead_code)]
#![allow(unused_variables)]

#[cfg(feature = "cgmath")]
extern crate cgmath;
extern crate glutin;
#[cfg(feature = "image")]
extern crate image;
extern crate libc;
#[cfg(feature = "nalgebra")]
extern crate nalgebra;

pub use index_buffer::IndexBuffer;
pub use vertex_buffer::{VertexBuffer, Vertex, VertexFormat};
pub use program::{Program, ProgramCreationError};
pub use program::ProgramCreationError::{CompilationError, LinkingError, ShaderTypeNotSupported};
pub use texture::{Texture, Texture2d};

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;

pub mod debug;
pub mod framebuffer;
pub mod index_buffer;
pub mod render_buffer;
pub mod uniforms;
pub mod vertex_buffer;
pub mod texture;

mod buffer;
mod context;
mod fbo;
mod ops;
mod program;
mod vertex_array_object;

mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

/// Internal trait for objects that are OpenGL objects.
trait GlObject {
    /// Returns the id of the object.
    fn get_id(&self) -> gl::types::GLuint;
}

/// Internal trait for enums that can be turned into GLenum.
trait ToGlEnum {
    /// Returns the value.
    fn to_glenum(&self) -> gl::types::GLenum;
}

/// Function that the GPU will use for blending.
#[derive(Clone, Copy, Show, PartialEq, Eq)]
pub enum BlendingFunction {
    /// Always replace the destination pixel by the source.
    ///
    /// The alpha channels are simply ignored. This is the default mode.
    AlwaysReplace,

    /// Linear interpolation of the source pixel by the source pixel's alpha.
    ///
    /// If the source's alpha is 0, the destination's color will stay the same. If the source's
    ///  alpha is 1, the destination's color will be replaced by the source's. If the source's
    ///  alpha is 0.5, the destination's color is the average between the source's and the
    ///  destination's color.
    ///
    /// This is the mode that you usually use for transparency.
    ///
    /// Means `(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA)` in Openctxt.gl.
    LerpBySourceAlpha,
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
/// - The vertices are arranged in a clockwise way on the screen.
/// - The vertices are arranged in a counterclockwise way on the screen.
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
#[derive(Clone, Copy, Show, PartialEq, Eq)]
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
/// range (that you can specify in the draw parameters) in order to obtain the depth value in
/// in window coordinates. This depth value is always between `0.0` and `1.0`.
///
/// In addition to the buffer where pixel colors are stored, you can also have a buffer
/// which contains the depth value of each pixel. Whenever the GPU tries to write a pixel,
/// it will first compare the depth value of the pixel to be written with the depth value that
/// is stored at this location.
///
/// If you don't have a depth buffer available, you can only pass `Overwrite`. Glium detects if
/// you pass any other value and reports an error.
#[derive(Clone, Copy, Show, PartialEq, Eq)]
pub enum DepthFunction {
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

    /// Replace if the z-value of the source is more or equal to the destination.
    IfMoreOrEqual,

    /// Replace if the z-value of the source is less than the destination.
    IfLess,

    /// Replace if the z-value of the source is less or equal to the destination.
    IfLessOrEqual
}

impl DepthFunction {
    /// Returns true if the function requires a depth buffer to be used.
    pub fn requires_depth_buffer(&self) -> bool {
        match *self {
            DepthFunction::Ignore => true,
            DepthFunction::Overwrite => false,
            DepthFunction::IfEqual => true,
            DepthFunction::IfNotEqual => true,
            DepthFunction::IfMore => true,
            DepthFunction::IfMoreOrEqual => true,
            DepthFunction::IfLess => true,
            DepthFunction::IfLessOrEqual => true,
        }
    }
}

impl ToGlEnum for DepthFunction {
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            DepthFunction::Ignore => gl::NEVER,
            DepthFunction::Overwrite => gl::ALWAYS,
            DepthFunction::IfEqual => gl::EQUAL,
            DepthFunction::IfNotEqual => gl::NOTEQUAL,
            DepthFunction::IfMore => gl::GREATER,
            DepthFunction::IfMoreOrEqual => gl::GEQUAL,
            DepthFunction::IfLess => gl::LESS,
            DepthFunction::IfLessOrEqual => gl::LEQUAL,
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
#[derive(Clone, Copy, Show, PartialEq, Eq)]
pub enum PolygonMode {
    /// Only draw a single point at each vertex.
    ///
    /// All attributes that apply to points are used when using this mode.
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
///     depth_function: glium::DepthFunction::IfLess,
///     .. std::default::Default::default()
/// };
/// ```
///
#[derive(Clone, Copy, Show, PartialEq)]
pub struct DrawParameters {
    /// The function that the GPU will use to determine whether to write over an existing pixel
    /// on the target.
    ///
    /// See the documentation of `DepthFunction` for more details.
    ///
    /// The default is `Overwrite`.
    pub depth_function: DepthFunction,

    /// The range of Z coordinates in surface coordinates.
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

    /// The function that the GPU will use to merge the existing pixel with the pixel that is
    /// being written.
    ///
    /// `None` means "don't care" (usually when you know that the alpha is always 1).
    pub blending_function: Option<BlendingFunction>,

    /// Width in pixels of the lines to draw when drawing lines.
    ///
    /// `None` means "don't care". Use this when you don't draw lines.
    pub line_width: Option<f32>,

    /// Whether or not the GPU should filter out some faces.
    ///
    /// After the vertex shader stage, the GPU will try to remove the faces that aren't facing
    /// the camera.
    ///
    /// See the documentation of `BackfaceCullingMode` for more infos.
    pub backface_culling: BackfaceCullingMode,

    /// Sets how to render polygons. The default value is `Fill`.
    ///
    /// See the documentation of `PolygonMode` for more infos.
    pub polygon_mode: PolygonMode,

    /// Whether multisample antialiasing (MSAA) should be used. Default value is `true`.
    ///
    /// Note that you will need to set the appropriate option when creating the window.
    /// The recommended way to do is to leave this to `true`, and adjust the option when
    /// creating the window.
    pub multisampling: bool,

    /// Specifies the viewport to use when drawing.
    ///
    /// The x and y positions of your vertices are mapped to the viewport so that `(-1, -1)`
    /// corresponds to the lower-left hand corner and `(1, 1)` corresponds to the top-right
    /// hand corner. Any pixel outside of the viewport is discarded.
    ///
    /// You can specify a viewport greater than the target if you want to stretch the image.
    ///
    /// `None` means "use the whole surface".
    pub viewport: Option<Rect>,
}

impl std::default::Default for DrawParameters {
    fn default() -> DrawParameters {
        DrawParameters {
            depth_function: DepthFunction::Overwrite,
            depth_range: (0.0, 1.0),
            blending_function: Some(BlendingFunction::AlwaysReplace),
            line_width: None,
            backface_culling: BackfaceCullingMode::CullingDisabled,
            polygon_mode: PolygonMode::Fill,
            multisampling: true,
            viewport: None,
        }
    }
}

impl DrawParameters {
    /// Checks parameters and panics if something is wrong.
    fn validate(&self) {
        if self.depth_range.0 < 0.0 || self.depth_range.0 > 1.0 ||
           self.depth_range.1 < 0.0 || self.depth_range.1 > 1.0
        {
            panic!("Depth range must be between 0 and 1");
        }
    }

    /// Synchronizes the parmaeters with the current ctxt.state.
    fn sync(&self, ctxt: &mut context::CommandContext, surface_dimensions: (u32, u32)) {
        // depth function
        match self.depth_function {
            DepthFunction::Overwrite => unsafe {
                if ctxt.state.enabled_depth_test {
                    ctxt.gl.Disable(gl::DEPTH_TEST);
                    ctxt.state.enabled_depth_test = false;
                }
            },
            depth_function => unsafe {
                let depth_function = depth_function.to_glenum();
                if ctxt.state.depth_func != depth_function {
                    ctxt.gl.DepthFunc(depth_function);
                    ctxt.state.depth_func = depth_function;
                }
                if !ctxt.state.enabled_depth_test {
                    ctxt.gl.Enable(gl::DEPTH_TEST);
                    ctxt.state.enabled_depth_test = true;
                }
            }
        }

        // depth range
        if self.depth_range != ctxt.state.depth_range {
            unsafe {
                ctxt.gl.DepthRange(self.depth_range.0 as f64, self.depth_range.1 as f64);
            }
            ctxt.state.depth_range = self.depth_range;
        }

        // blending function
        match self.blending_function {
            Some(BlendingFunction::AlwaysReplace) => unsafe {
                if ctxt.state.enabled_blend {
                    ctxt.gl.Disable(gl::BLEND);
                    ctxt.state.enabled_blend = false;
                }
            },
            Some(BlendingFunction::LerpBySourceAlpha) => unsafe {
                if ctxt.state.blend_func != (gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA) {
                    ctxt.gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                    ctxt.state.blend_func = (gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                }
                if !ctxt.state.enabled_blend {
                    ctxt.gl.Enable(gl::BLEND);
                    ctxt.state.enabled_blend = true;
                }
            },
            _ => ()
        }

        // line width
        if let Some(line_width) = self.line_width {
            if ctxt.state.line_width != line_width {
                unsafe {
                    ctxt.gl.LineWidth(line_width);
                    ctxt.state.line_width = line_width;
                }
            }
        }

        // back-face culling
        // note: we never change the value of `glFrontFace`, whose default is GL_CCW
        //  that's why `CullClockWise` uses `GL_BACK` for example
        match self.backface_culling {
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
            let polygon_mode = self.polygon_mode.to_glenum();
            if ctxt.state.polygon_mode != polygon_mode {
                ctxt.gl.PolygonMode(gl::FRONT_AND_BACK, polygon_mode);
                ctxt.state.polygon_mode = polygon_mode;
            }
        }

        // multisampling
        if ctxt.state.enabled_multisample != self.multisampling {
            unsafe {
                if self.multisampling {
                    ctxt.gl.Enable(gl::MULTISAMPLE);
                    ctxt.state.enabled_multisample = true;
                } else {
                    ctxt.gl.Disable(gl::MULTISAMPLE);
                    ctxt.state.enabled_multisample = false;
                }
            }
        }

        // viewport
        if let Some(viewport) = self.viewport {
            assert!(viewport.width <= ctxt.capabilities.max_viewport_dims.0 as u32,
                    "Viewport dimensions are too large");
            assert!(viewport.height <= ctxt.capabilities.max_viewport_dims.1 as u32,
                    "Viewport dimensions are too large");

            let viewport = (viewport.left as gl::types::GLint, viewport.bottom as gl::types::GLint,
                            viewport.width as gl::types::GLsizei,
                            viewport.height as gl::types::GLsizei);

            if ctxt.state.viewport != viewport {
                unsafe { ctxt.gl.Viewport(viewport.0, viewport.1, viewport.2, viewport.3); }
                ctxt.state.viewport = viewport;
            }

        } else {
            assert!(surface_dimensions.0 <= ctxt.capabilities.max_viewport_dims.0 as u32,
                    "Viewport dimensions are too large");
            assert!(surface_dimensions.1 <= ctxt.capabilities.max_viewport_dims.1 as u32,
                    "Viewport dimensions are too large");

            let viewport = (0, 0, surface_dimensions.0 as gl::types::GLsizei,
                            surface_dimensions.1 as gl::types::GLsizei);

            if ctxt.state.viewport != viewport {
                unsafe { ctxt.gl.Viewport(viewport.0, viewport.1, viewport.2, viewport.3); }
                ctxt.state.viewport = viewport;
            }
        }
    }
}

/// Area of a surface in pixels.
///
/// In the OpenGL ecosystem, the (0,0) coordinate is at the bottom-left hand corner of the images.
#[derive(Show, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rect {
    /// Number of pixels between the left border of the surface and the left border of
    /// the rectangle.
    pub left: u32,
    /// Number of pixels between the bottom border of the surface and the bottom border
    /// of the rectangle.
    pub bottom: u32,
    /// Width of the area in pixels.
    pub width: u32,
    /// Height of the area in pixels.
    pub height: u32,
}

/// Object which can be drawn upon.
pub trait Surface: Sized {
    /// Clears the color components of the target.
    fn clear_color(&mut self, red: f32, green: f32, blue: f32, alpha: f32);

    /// Clears the depth component of the target.
    fn clear_depth(&mut self, value: f32);

    /// Clears the stencil component of the target.
    fn clear_stencil(&mut self, value: int);

    /// Returns the dimensions in pixels of the target.
    fn get_dimensions(&self) -> (uint, uint);

    /// Returns the number of bits of each pixel of the depth buffer.
    ///
    /// Returns `None` if there is no depth buffer.
    fn get_depth_buffer_bits(&self) -> Option<u16>;

    /// Returns true if the surface has a depth buffer available.
    fn has_depth_buffer(&self) -> bool {
        self.get_depth_buffer_bits().is_some()
    }

    /// Returns the number of bits of each pixel of the stencil buffer.
    ///
    /// Returns `None` if there is no stencil buffer.
    fn get_stencil_buffer_bits(&self) -> Option<u16>;

    /// Returns true if the surface has a stencil buffer available.
    fn has_stencil_buffer(&self) -> bool {
        self.get_stencil_buffer_bits().is_some()
    }

    /// Draws.
    ///
    /// # Panic
    ///
    /// - Panics if the requested depth function requires a depth buffer and none is attached.
    /// - Panics if the type of some of the vertex source's attributes do not match the program's.
    /// - Panics if a program's attribute is not in the vertex source (does *not* panic if a
    ///   vertex's attribute is not used by the program).
    /// - Panics if the viewport is larger than the dimensions supported by the hardware.
    /// - Panics if the depth range is outside of `(0, 1)`.
    ///
    fn draw<'a, 'b, V, I, ID, U>(&mut self, V, &I, program: &Program, uniforms: U,
        draw_parameters: &DrawParameters) where V: vertex_buffer::IntoVerticesSource<'b>,
        I: index_buffer::ToIndicesSource<ID>, U: uniforms::Uniforms;

    /// Returns an opaque type that is used by the implementation of blit functions.
    fn get_blit_helper(&self) -> BlitHelper;

    /// Copies a rectangle of pixels from this surface to another surface.
    ///
    /// The `source_rect` defines the area of the source (`self`) that will be copied, and the
    /// `target_rect` defines the area where the copied image will be pasted. If the source and
    /// target areas don't have the same dimensions, the image will be resized to match.
    /// The `filter` parameter is relevant only in this situation.
    ///
    /// It is possible for the source and the target to be the same surface. However if the
    /// rectangles overlap, then the behavior is undefined.
    ///
    /// Note that there is no alpha blending, depth/stencil checking, etc. or anything ; this
    /// function just copies pixels.
    #[experimental = "The name will likely change"]
    fn blit_color<S>(&self, source_rect: &Rect, target: &S, target_rect: &Rect,
        filter: uniforms::MagnifySamplerFilter) where S: Surface
    {
        ops::blit(self, target, gl::COLOR_BUFFER_BIT, source_rect, target_rect,
            filter.to_glenum())
    }

    /// Copies the entire surface to a target surface. See `blit_color`.
    #[experimental = "The name will likely change"]
    fn blit_whole_color_to<S>(&self, target: &S, target_rect: &Rect,
        filter: uniforms::MagnifySamplerFilter) where S: Surface
    {
        let src_dim = self.get_dimensions();
        let src_rect = Rect { left: 0, bottom: 0, width: src_dim.0 as u32, height: src_dim.1 as u32 };
        self.blit_color(&src_rect, target, target_rect, filter)
    }

    /// Copies the entire surface to the entire target. See `blit_color`.
    #[experimental = "The name will likely change"]
    fn fill<S>(&self, target: &S, filter: uniforms::MagnifySamplerFilter) where S: Surface {
        let src_dim = self.get_dimensions();
        let src_rect = Rect { left: 0, bottom: 0, width: src_dim.0 as u32, height: src_dim.1 as u32 };
        let target_dim = target.get_dimensions();
        let target_rect = Rect { left: 0, bottom: 0, width: target_dim.0 as u32, height: target_dim.1 as u32 };
        self.blit_color(&src_rect, target, &target_rect, filter)
    }
}

#[doc(hidden)]
pub struct BlitHelper<'a>(&'a Arc<DisplayImpl>, Option<&'a fbo::FramebufferAttachments>);

/// Implementation of `Surface` targetting the default framebuffer.
///
/// The back- and front-buffers are swapped when the `Frame` is destroyed. This operation is
/// instantaneous, even when vsync is enabled.
pub struct Frame<'a> {
    display: Display,
    marker: std::kinds::marker::ContravariantLifetime<'a>,
    dimensions: (uint, uint),
}

impl<'t> Frame<'t> {
    /// Stop drawing and swap the buffers.
    pub fn finish(self) {
    }
}

impl<'t> Surface for Frame<'t> {
    fn clear_color(&mut self, red: f32, green: f32, blue: f32, alpha: f32) {
        ops::clear_color(&self.display.context, None, red, green, blue, alpha)
    }

    fn clear_depth(&mut self, value: f32) {
        ops::clear_depth(&self.display.context, None, value)
    }

    fn clear_stencil(&mut self, value: int) {
        ops::clear_stencil(&self.display.context, None, value)
    }

    fn get_dimensions(&self) -> (uint, uint) {
        self.dimensions
    }

    fn get_depth_buffer_bits(&self) -> Option<u16> {
        self.display.context.context.capabilities().depth_bits
    }

    fn get_stencil_buffer_bits(&self) -> Option<u16> {
        self.display.context.context.capabilities().stencil_bits
    }

    fn draw<'a, 'b, V, I, ID, U>(&mut self, vertex_buffer: V,
                         index_buffer: &I, program: &Program, uniforms: U,
                         draw_parameters: &DrawParameters)
                         where I: index_buffer::ToIndicesSource<ID>, U: uniforms::Uniforms,
                         ID: index_buffer::Index, V: vertex_buffer::IntoVerticesSource<'b>
    {
        use index_buffer::ToIndicesSource;

        draw_parameters.validate();

        if draw_parameters.depth_function.requires_depth_buffer() && !self.has_depth_buffer() {
            panic!("Requested a depth function but no depth buffer is attached");
        }

        if let Some(viewport) = draw_parameters.viewport {
            assert!(viewport.width <= self.display.context.context.capabilities().max_viewport_dims.0
                    as u32, "Viewport dimensions are too large");
            assert!(viewport.height <= self.display.context.context.capabilities().max_viewport_dims.1
                    as u32, "Viewport dimensions are too large");
        }

        ops::draw(&self.display, None, vertex_buffer.into_vertices_source(),
                  &index_buffer.to_indices_source(), program, uniforms, draw_parameters,
                  (self.dimensions.0 as u32, self.dimensions.1 as u32))
    }

    fn get_blit_helper(&self) -> BlitHelper {
        BlitHelper(&self.display.context, None)
    }
}

#[unsafe_destructor]
impl<'t> Drop for Frame<'t> {
    fn drop(&mut self) {
        self.display.context.context.swap_buffers();
    }
}

/// Objects that can build a `Display` object.
pub trait DisplayBuild {
    /// Build a context and a `Display` to draw on it.
    ///
    /// Performances a compatibility check to make sure that all core elements of glium
    /// are supported by the implementation.
    fn build_glium(self) -> Result<Display, GliumCreationError>;
}

/// Error that can happen while creating a glium display.
#[derive(Clone, Show, PartialEq, Eq)]
pub enum GliumCreationError {
    /// An error has happened while creating the glutin window or headless renderer.
    GlutinCreationError(glutin::CreationError),

    /// The OpenGL implementation is too old.
    IncompatibleOpenGl(String),
}

impl std::error::Error for GliumCreationError {
    fn description(&self) -> &str {
        match self {
            &GliumCreationError::GlutinCreationError(_) => "Error while creating glutin window or headless renderer",
            &GliumCreationError::IncompatibleOpenGl(_) => "The OpenGL implementation is too old to work with glium",
        }
    }

    fn detail(&self) -> Option<String> {
        match self {
            &GliumCreationError::GlutinCreationError(_) => None,
            &GliumCreationError::IncompatibleOpenGl(ref e) => Some(e.clone()),
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match self {
            &GliumCreationError::GlutinCreationError(ref err) => Some(err as &std::error::Error),
            &GliumCreationError::IncompatibleOpenGl(_) => None,
        }
    }
}

impl std::error::FromError<glutin::CreationError> for GliumCreationError {
    fn from_error(err: glutin::CreationError) -> GliumCreationError {
        GliumCreationError::GlutinCreationError(err)
    }
}

impl<'a> DisplayBuild for glutin::WindowBuilder<'a> {
    fn build_glium(self) -> Result<Display, GliumCreationError> {
        let context = try!(context::Context::new_from_window(self, None));

        Ok(Display {
            context: Arc::new(DisplayImpl {
                context: context,
                debug_callback: Mutex::new(None),
                framebuffer_objects: Mutex::new(HashMap::new()),
                vertex_array_objects: Mutex::new(HashMap::new()),
                samplers: Mutex::new(HashMap::new()),
            }),
        })
    }
}

#[cfg(feature = "headless")]
impl DisplayBuild for glutin::HeadlessRendererBuilder {
    fn build_glium(self) -> Result<Display, GliumCreationError> {
        let context = try!(context::Context::new_from_headless(self));

        Ok(Display {
            context: Arc::new(DisplayImpl {
                context: context,
                debug_callback: Mutex::new(None),
                framebuffer_objects: Mutex::new(HashMap::new()),
                vertex_array_objects: Mutex::new(HashMap::new()),
                samplers: Mutex::new(HashMap::new()),
            }),
        })
    }
}

/// The main object of this library. Controls the whole display.
///
/// This object contains a smart pointer to the real implementation.
/// Cloning the display allows you to easily share the `Display` object throughout
///  your program and between threads.
#[derive(Clone)]
pub struct Display {
    context: Arc<DisplayImpl>,
}

struct DisplayImpl {
    // contains everything related to the current context and its state
    context: context::Context,

    // the callback used for debug messages
    debug_callback: Mutex<Option<Box<FnMut(String, debug::Source, debug::MessageType, debug::Severity)
                                     + Send + Sync>>>,

    // we maintain a list of FBOs
    // when something requirering a FBO is drawn, we look for an existing one in this hashmap
    framebuffer_objects: Mutex<HashMap<fbo::FramebufferAttachments,
                                       fbo::FrameBufferObject>>,

    // we maintain a list of VAOs for each vertexbuffer-indexbuffer-program association
    // the key is a (vertexbuffer, program)
    vertex_array_objects: Mutex<HashMap<(gl::types::GLuint, gl::types::GLuint, gl::types::GLuint),
                                        vertex_array_object::VertexArrayObject>>,

    // we maintain a list of samplers for each possible behavior
    samplers: Mutex<HashMap<uniforms::SamplerBehavior, uniforms::SamplerObject>>,
}

impl Display {
    /// Reads all events received by the window.
    pub fn poll_events(&self) -> Vec<glutin::Event> {
        self.context.context.recv()
    }

    /// Returns the dimensions of the main framebuffer.
    pub fn get_framebuffer_dimensions(&self) -> (uint, uint) {
        self.context.context.get_framebuffer_dimensions()
    }

    /// Start drawing on the backbuffer.
    ///
    /// This function returns a `Frame` which can be used to draw on it. When the `Frame` is
    /// destroyed, the buffers are swapped.
    ///
    /// Note that destroying a `Frame` is immediate, even if vsync is enabled.
    pub fn draw(&self) -> Frame {
        Frame {
            display: self.clone(),
            marker: std::kinds::marker::ContravariantLifetime,
            dimensions: self.get_framebuffer_dimensions(),
        }
    }

    /// Returns the maximum value that can be used for anisotropic filtering, or `None`
    /// if the hardware doesn't support it.
    pub fn get_max_anisotropy_support(&self) -> Option<u16> {
        self.context.context.capabilities().max_texture_max_anisotropy.map(|v| v as u16)
    }

    /// Returns the maximum dimensions of the viewport that you can pass when drawing.
    ///
    /// Glium will panic if you request a larger viewport.
    pub fn get_max_viewport_dimensions(&self) -> (u32, u32) {
        let d = self.context.context.capabilities().max_viewport_dims;
        (d.0 as u32, d.1 as u32)
    }

    /// Releases the shader compiler, indicating that no new programs will be created for a while.
    ///
    /// # Features
    ///
    /// This method is always available, but is a no-op if it's not available in
    /// the implementation.
    pub fn release_shader_compiler(&self) {
        self.context.context.exec(move |: ctxt| {
            unsafe {
                if ctxt.opengl_es || ctxt.version >= &context::GlVersion(4, 1) {
                    ctxt.gl.ReleaseShaderCompiler();
                }
            }
        });
    }

    /// Returns an estimate of the amount of video memory available in bytes.
    ///
    /// Returns `None` if no estimate is available.
    pub fn get_free_video_memory(&self) -> Option<uint> {
        let (tx, rx) = channel();

        self.context.context.exec(move |: ctxt| {
            unsafe {
                use std::mem;
                let mut value: [gl::types::GLint; 4] = mem::uninitialized();

                let value = if ctxt.extensions.gl_nvx_gpu_memory_info {
                    ctxt.gl.GetIntegerv(gl::GPU_MEMORY_INFO_CURRENT_AVAILABLE_VIDMEM_NVX,
                                   &mut value[0]);
                    Some(value[0])

                } else if ctxt.extensions.gl_ati_meminfo {
                    ctxt.gl.GetIntegerv(gl::TEXTURE_FREE_MEMORY_ATI, &mut value[0]);
                    Some(value[0])

                } else {
                    None
                };

                tx.send(value).ok();
            }
        });

        rx.recv().unwrap().map(|v| v as uint * 1024)
    }

    /// Sets the callback to use when an OpenGL debug message is generated.
    ///
    /// **Important**: some contexts don't support debug output, in which case this function will
    /// act as a no-op. Even if the context does support them, you are not guaranteed to get any.
    /// Debug messages are just a convenience and are not reliable.
    #[experimental = "The API will probably change"]
    pub fn set_debug_callback<F>(&self, callback: F)
        where F: FnMut(String, debug::Source, debug::MessageType, debug::Severity) + Send + Sync
    {
        self.set_debug_callback_impl(callback, false);
    }

    /// Sets the callback to use when an OpenGL debug message is generated.
    ///
    /// Contrary to `set_debug_callback`, the callback is called synchronously.
    #[experimental = "The API will probably change"]
    pub unsafe fn set_debug_callback_sync<F>(&self, callback: F)
        where F: FnMut(String, debug::Source, debug::MessageType, debug::Severity) + Send + Sync
    {
        self.set_debug_callback_impl(callback, true);
    }

    fn set_debug_callback_impl<F>(&self, callback: F, sync: bool)
        where F: FnMut(String, debug::Source, debug::MessageType, debug::Severity) + Send + Sync
    {
        // changing the callback
        {
            let mut cb = self.context.debug_callback.lock().unwrap();
            *cb = Some(box callback as Box<FnMut(String, debug::Source, debug::MessageType,
                                                 debug::Severity)
                                           + Send + Sync>);
        }

        // this is the C callback
        extern "system" fn callback_wrapper(source: gl::types::GLenum, ty: gl::types::GLenum,
            id: gl::types::GLuint, severity: gl::types::GLenum, _length: gl::types::GLsizei,
            message: *const gl::types::GLchar, user_param: *mut libc::c_void)
        {
            use std::num::FromPrimitive;

            unsafe {
                let user_param = user_param as *mut DisplayImpl;
                let user_param = user_param.as_mut().unwrap();

                let message = String::from_utf8(std::ffi::c_str_to_bytes(&message).to_vec())
                                  .unwrap();

                let ref mut callback = user_param.debug_callback;
                let mut callback = callback.lock().unwrap();
                let callback = callback.deref_mut();

                if let &Some(ref mut callback) = callback {
                    callback.call_mut((message.to_string(),
                        FromPrimitive::from_uint(source as uint).unwrap_or(debug::Source::OtherSource),
                        FromPrimitive::from_uint(ty as uint).unwrap_or(debug::MessageType::Other),
                        FromPrimitive::from_uint(severity as uint).unwrap_or(debug::Severity::Notification)));
                }
            }
        }

        // SAFETY NOTICE: we pass a raw pointer to the `DisplayImpl`
        let ptr: &DisplayImpl = self.context.deref();
        let ptr = std::ptr::Unique(ptr as *const DisplayImpl as *mut DisplayImpl);

        // enabling the callback
        self.context.context.exec(move |: ctxt| {
            unsafe {
                if ctxt.version >= &context::GlVersion(4,5) || ctxt.extensions.gl_khr_debug {
                    if ctxt.state.enabled_debug_output_synchronous != sync {
                        if sync {
                            ctxt.gl.Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
                            ctxt.state.enabled_debug_output_synchronous = true;
                        } else {
                            ctxt.gl.Disable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
                            ctxt.state.enabled_debug_output_synchronous = false;
                        }
                    }

                    // TODO: with GLES, the GL_KHR_debug function has a `KHR` suffix
                    //       but with GL only, it doesn't have one
                    ctxt.gl.DebugMessageCallback(callback_wrapper, ptr.0 as *const libc::c_void);
                    ctxt.gl.DebugMessageControl(gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE, 0,
                        std::ptr::null(), gl::TRUE);

                    if ctxt.state.enabled_debug_output != Some(true) {
                        ctxt.gl.Enable(gl::DEBUG_OUTPUT);
                        ctxt.state.enabled_debug_output = Some(true);
                    }
                }
            }
        });
    }

    /// Reads the content of the front buffer.
    ///
    /// You will only see the data that has finished being drawn.
    ///
    /// This function can return any type that implements `Texture2dData`.
    ///
    /// ## Example
    ///
    /// ```noçrun
    /// # extern crate glium;
    /// # extern crate glutin;
    /// # use glium::DisplayBuild;
    /// # fn main() {
    /// # let display: glium::Display = unsafe { ::std::mem::uninitialized() };
    /// let pixels: Vec<Vec<(u8, u8, u8)>> = display.read_front_buffer();
    /// # }
    /// ```
    pub fn read_front_buffer<P, T>(&self) -> T          // TODO: remove Clone for P
        where P: texture::PixelValue + Clone + Send, T: texture::Texture2dData<Data = P>
    {
        use std::mem;

        let dimensions = self.get_framebuffer_dimensions();
        let pixels_count = dimensions.0 * dimensions.1;

        let pixels_size = texture::Texture2dData::get_format(None::<T>).get_size();
        let (format, gltype) = texture::Texture2dData::get_format(None::<T>).to_gl_enum();

        let (tx, rx) = channel();
        self.context.context.exec(move |: ctxt| {
            unsafe {
                // unbinding framebuffers
                if ctxt.state.read_framebuffer != 0 {
                    if ctxt.version >= &context::GlVersion(3, 0) {
                        ctxt.gl.BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
                        ctxt.state.read_framebuffer = 0;
                    } else {
                        ctxt.gl.BindFramebufferEXT(gl::FRAMEBUFFER_EXT, 0);
                        ctxt.state.draw_framebuffer = 0;
                        ctxt.state.read_framebuffer = 0;
                    }
                }

                // adjusting glReadBuffer
                if ctxt.state.default_framebuffer_read != Some(gl::FRONT_LEFT) {
                    ctxt.gl.ReadBuffer(gl::FRONT_LEFT);
                    ctxt.state.default_framebuffer_read = Some(gl::FRONT_LEFT);
                }

                // reading
                let total_data_size = pixels_count * pixels_size / mem::size_of::<P>();
                let mut data: Vec<P> = Vec::with_capacity(total_data_size);
                ctxt.gl.ReadPixels(0, 0, dimensions.0 as gl::types::GLint,
                    dimensions.1 as gl::types::GLint, format, gltype,
                    data.as_mut_ptr() as *mut libc::c_void);
                data.set_len(total_data_size);
                tx.send(data).ok();
            }
        });

        let data = rx.recv().unwrap();
        texture::Texture2dData::from_vec(data, dimensions.0 as u32)
    }

    /// Asserts that there are no OpenGL error pending.
    ///
    /// This function is supposed to be used in tests.
    pub fn assert_no_error(&self) {
        let (tx, rx) = channel();

        self.context.context.exec(move |: ctxt| {
            tx.send(get_gl_error(ctxt)).ok();
        });

        match rx.recv().unwrap() {
            Some(msg) => panic!("{}", msg),
            None => ()
        };
    }

    /// Waits until all the previous commands have finished being executed.
    ///
    /// When you execute OpenGL functions, they are not executed immediatly. Instead they are
    /// put in a queue. This function waits until all commands have finished being executed and
    /// the queue is empty.
    ///
    /// **You don't need to call this function manually, except when running benchmarks.**
    pub fn synchronize(&self) {
        let (tx, rx) = channel();

        self.context.context.exec(move |: ctxt| {
            unsafe { ctxt.gl.Finish(); }
            tx.send(()).ok();
        });

        rx.recv().unwrap();
    }
}

// this destructor is here because objects in `Display` contain an `Arc<DisplayImpl>`,
// which would lead to a leak
impl Drop for DisplayImpl {
    fn drop(&mut self) {
        // disabling callback, to avoid
        self.context.exec(move |: ctxt| {
            unsafe {
                if ctxt.state.enabled_debug_output != Some(false) {
                    ctxt.gl.Disable(gl::DEBUG_OUTPUT);
                    ctxt.state.enabled_debug_output = Some(false);
                    ctxt.gl.Finish();
                }
            }
        });

        {
            let mut fbos = self.framebuffer_objects.lock().unwrap();
            fbos.clear();
        }

        {
            let mut vaos = self.vertex_array_objects.lock().unwrap();
            vaos.clear();
        }

        {
            let mut samplers = self.samplers.lock().unwrap();
            samplers.clear();
        }
    }
}

#[allow(dead_code)]
fn get_gl_error(ctxt: context::CommandContext) -> Option<&'static str> {
    match unsafe { ctxt.gl.GetError() } {
        gl::NO_ERROR => None,
        gl::INVALID_ENUM => Some("GL_INVALID_ENUM"),
        gl::INVALID_VALUE => Some("GL_INVALID_VALUE"),
        gl::INVALID_OPERATION => Some("GL_INVALID_OPERATION"),
        gl::INVALID_FRAMEBUFFER_OPERATION => Some("GL_INVALID_FRAMEBUFFER_OPERATION"),
        gl::OUT_OF_MEMORY => Some("GL_OUT_OF_MEMORY"),
        /*gl::STACK_UNDERFLOW => Some("GL_STACK_UNDERFLOW"),
        gl::STACK_OVERFLOW => Some("GL_STACK_OVERFLOW"),*/
        _ => Some("Unknown glGetError return value")
    }
}
