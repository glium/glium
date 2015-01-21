/*!
Easy-to-use, high-level, OpenGL3+ wrapper.

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
        .with_title(format!("Hello world"))
        .build_glium().unwrap();
}
```

The `display` object is the most important object of this library.

The window you are drawing on will produce events. They can be received by calling
`display.poll_events()`.

## Complete example

We start by creating the vertex buffer, which contains the list of all the points that make up
our mesh. The elements that we pass to `VertexBuffer::new` must implement the
`glium::vertex::VertexFormat` trait. We can easily do this by creating a custom struct
and adding the `#[vertex_format]` attribute to it.

See the `vertex` module documentation for more informations.

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
let vertex = glium::VertexBuffer::new(&display, vec![
    Vertex { position: [-0.5, -0.5], color: [0.0, 1.0, 0.0] },
    Vertex { position: [ 0.0,  0.5], color: [0.0, 0.0, 1.0] },
    Vertex { position: [ 0.5, -0.5], color: [1.0, 0.0, 0.0] },
]);
# }
```

We then create the index buffer, which contains information about the primitives (triangles,
lines, etc.) that compose our mesh.

The last parameter is a list of indices that represent the positions of our points in the
vertex buffer.

```no_run
# let display: glium::Display = unsafe { std::mem::uninitialized() };
let index_buffer = glium::IndexBuffer::new(&display,
    glium::index_buffer::TrianglesList(vec![ 0u16, 1, 2 ]));
```

Next, we create the program, which is composed of a *vertex shader*, a program executed once for
each element in our vertex buffer, and a *fragment shader*, a program executed once for each
pixel before it is written on the final image.

The purpose of a program is to instruct the GPU how to process our mesh, in order to obtain pixels.

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
types of the elements in the vertex buffer. This is required, otherwise drawing will result in an error.

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

Vertex buffers, index buffers, and the program should be stored between draws in order to avoid wasting
time, but objects that implement the `glium::uniforms::Uniforms` trait are usually constructed
every time you draw.

The fields of our `Uniforms` object can be any object that implements `glium::uniforms::UniformValue`.
This includes textures and samplers (not covered here). See the `uniforms` module documentation 
for more informations.

Now that everything is initialized, we can finally draw something. To do so, call `display.draw()`
in order to obtain a `Frame` object. Note that it is also possible to draw on a texture by
calling `texture.as_surface()`, but this is not covered here.

The `Frame` object has a `draw` function, which you can use to draw things. Its arguments are the
vertex buffer, index buffer, program, uniforms, and an object of type `DrawParameters`, which 
contains miscellaneous information specifying how everything should be rendered (depth test, blending,
backface culling, etc.).

```no_run
use glium::Surface;
# let display: glium::Display = unsafe { std::mem::uninitialized() };
# let vertex_buffer: glium::VertexBuffer<u8> = unsafe { std::mem::uninitialized() };
# let index_buffer: glium::IndexBuffer = unsafe { std::mem::uninitialized() };
# let program: glium::Program = unsafe { std::mem::uninitialized() };
# let uniforms = glium::uniforms::EmptyUniforms;
let mut target = display.draw();
target.clear_color(0.0, 0.0, 0.0, 0.0);  // filling the output with the black color
target.draw(&vertex_buffer, &index_buffer, &program, &uniforms,
            &std::default::Default::default()).unwrap();
target.finish();
```

*/
#![feature(slicing_syntax)]
#![feature(unboxed_closures)]
#![feature(unsafe_destructor)]
#![unstable]
#![allow(unstable)]
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
pub use vertex::{VertexBuffer, Vertex, VertexFormat};
pub use program::{Program, ProgramCreationError};
pub use program::ProgramCreationError::{CompilationError, LinkingError, ShaderTypeNotSupported};
pub use sync::{LinearSyncFence, SyncFence};
pub use texture::{Texture, Texture2d};

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;

pub mod debug;
pub mod framebuffer;
pub mod index_buffer;
pub mod pixel_buffer;
pub mod macros;
pub mod program;
pub mod render_buffer;
pub mod uniforms;
pub mod vertex;
pub mod texture;

mod buffer;
mod context;
mod fbo;
mod ops;
mod sampler_object;
mod sync;
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
///
/// Blending happens at the end of the rendering process, when the GPU wants to write the
/// pixels over pixels that already exist in the framebuffer. The blending function allows
/// you to choose how it should merge the two.
///
/// If you want to add transparent objects one over another, the usual value
/// is `Addition { source: Alpha, destination: OneMinusAlpha }`.
#[derive(Clone, Copy, Show, PartialEq, Eq)]
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
#[derive(Clone, Copy, Show, PartialEq, Eq)]
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
/// range (which you can specify in the draw parameters) in order to obtain the depth value in
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

    /// Replace if the z-value of the source is more than, or equal to the destination.
    IfMoreOrEqual,

    /// Replace if the z-value of the source is less than the destination.
    IfLess,

    /// Replace if the z-value of the source is less than, or equal to the destination.
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
    /// See the `DepthFunction` documentation for more details.
    ///
    /// The default is `Overwrite`.
    pub depth_function: DepthFunction,

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
            dithering: true,
            viewport: None,
            scissor: None,
        }
    }
}

impl DrawParameters {
    /// Checks parameters and panics if something is wrong.
    fn validate(&self) -> Result<(), DrawError> {
        if self.depth_range.0 < 0.0 || self.depth_range.0 > 1.0 ||
           self.depth_range.1 < 0.0 || self.depth_range.1 > 1.0
        {
            return Err(DrawError::InvalidDepthRange);
        }

        Ok(())
    }

    /// Synchronizes the parameters with the current ctxt.state.
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
        let blend_factors = match self.blending_function {
            Some(BlendingFunction::AlwaysReplace) => unsafe {
                if ctxt.state.enabled_blend {
                    ctxt.gl.Disable(gl::BLEND);
                    ctxt.state.enabled_blend = false;
                }
                None
            },
            Some(BlendingFunction::Min) => unsafe {
                if ctxt.state.blend_equation != gl::MIN {
                    ctxt.gl.BlendEquation(gl::MIN);
                    ctxt.state.blend_equation = gl::MIN;
                }
                if !ctxt.state.enabled_blend {
                    ctxt.gl.Enable(gl::BLEND);
                    ctxt.state.enabled_blend = true;
                }
                None
            },
            Some(BlendingFunction::Max) => unsafe {
                if ctxt.state.blend_equation != gl::MAX {
                    ctxt.gl.BlendEquation(gl::MAX);
                    ctxt.state.blend_equation = gl::MAX;
                }
                if !ctxt.state.enabled_blend {
                    ctxt.gl.Enable(gl::BLEND);
                    ctxt.state.enabled_blend = true;
                }
                None
            },
            Some(BlendingFunction::Addition { source, destination }) => unsafe {
                if ctxt.state.blend_equation != gl::FUNC_ADD {
                    ctxt.gl.BlendEquation(gl::FUNC_ADD);
                    ctxt.state.blend_equation = gl::FUNC_ADD;
                }
                if !ctxt.state.enabled_blend {
                    ctxt.gl.Enable(gl::BLEND);
                    ctxt.state.enabled_blend = true;
                }
                Some((source, destination))
            },
            Some(BlendingFunction::Subtraction { source, destination }) => unsafe {
                if ctxt.state.blend_equation != gl::FUNC_SUBTRACT {
                    ctxt.gl.BlendEquation(gl::FUNC_SUBTRACT);
                    ctxt.state.blend_equation = gl::FUNC_SUBTRACT;
                }
                if !ctxt.state.enabled_blend {
                    ctxt.gl.Enable(gl::BLEND);
                    ctxt.state.enabled_blend = true;
                }
                Some((source, destination))
            },
            Some(BlendingFunction::ReverseSubtraction { source, destination }) => unsafe {
                if ctxt.state.blend_equation != gl::FUNC_REVERSE_SUBTRACT {
                    ctxt.gl.BlendEquation(gl::FUNC_REVERSE_SUBTRACT);
                    ctxt.state.blend_equation = gl::FUNC_REVERSE_SUBTRACT;
                }
                if !ctxt.state.enabled_blend {
                    ctxt.gl.Enable(gl::BLEND);
                    ctxt.state.enabled_blend = true;
                }
                Some((source, destination))
            },
            _ => None
        };
        if let Some((source, destination)) = blend_factors {
            let source = source.to_glenum();
            let destination = destination.to_glenum();

            if ctxt.state.blend_func != (source, destination) {
                unsafe { ctxt.gl.BlendFunc(source, destination) };
                ctxt.state.blend_func = (source, destination);
            }
        };

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

        // dithering
        if ctxt.state.enabled_dither != self.dithering {
            unsafe {
                if self.dithering {
                    ctxt.gl.Enable(gl::DITHER);
                    ctxt.state.enabled_dither = true;
                } else {
                    ctxt.gl.Disable(gl::DITHER);
                    ctxt.state.enabled_dither = false;
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

        // scissor
        if let Some(scissor) = self.scissor {
            let scissor = (scissor.left as gl::types::GLint, scissor.bottom as gl::types::GLint,
                           scissor.width as gl::types::GLsizei,
                           scissor.height as gl::types::GLsizei);

            unsafe {
                if ctxt.state.scissor != scissor {
                    ctxt.gl.Scissor(scissor.0, scissor.1, scissor.2, scissor.3);
                    ctxt.state.scissor = scissor;
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

/// Object that can be drawn upon.
///
/// # What does the GPU do when you draw?
///
/// This is a summary of everything that happens when you call the `draw` function. Note that
/// this is not necessarly *exactly* what happens. Backends are free to do whatever they want
/// as long as it always matches the expected outcome.
///
/// ## Step 1: Vertex shader
///
/// For each vertex in the vertices source, the GPU invokes the vertex shader that is part
/// of the program, and passes the corresponding vertex's attributes to it.
///
/// The vertex shader *must* write the special `gl_Position` variable in order to indicate
/// the four-dimensions coordinates of the vertex. In order to understand what these coordinates
/// mean, see the "vertex post-processing" step below.
///
/// In addition to the position of the vertex, the vertex shader can also specify the values of
/// various vertex attributes.
///
/// ## Step 2: Tessellation (optional)
///
/// It is possible to use tessellation shaders, but glium does not support them yet.
///
/// ## Step 3: Geometry shader (optional)
///
/// If you specify a geometry shader, then the GPU will invoke it once for each primitive.
///
/// The geometry shader can output multiple primitives.
///
/// ## Step 4: Transform feedback (optional)
///
/// Transform feedback is not supported by glium for the moment.
///
/// ## Step 5: Vertex post-processing
///
/// The vertex shader step told the GPU what the coordinates of each vertex are, but these
/// coordinates have four dimensions, named `x`, `y`, `z` and `w`.
///
/// The GPU then computes the position of the vertex on the 2D surface you are drawing on, and
/// the depth of this vertex:
///
/// ```notrust
/// window_x = viewport_left + viewport_width * ((x / w) + 1.0) / 2.0
/// window_y = viewport_bottom + viewport_height * ((y / w) + 1.0) / 2.0
/// depth = depth_near + (depth_far - depth_near) * ((z / w) + 1.0) / 2.0
/// ```
///
/// *`viewport_left`, `viewport_width`, `viewport_bottom` and `viewport_height` correspond to
/// the `viewport` member of the draw parameters, and `depth_near` and `depth_far` correspond
/// to the `depth_range` member*.
///
/// This means that if `x / w`, `y / w` or `z / w` are equal to `-1.0`, then the result will be
/// `viewport_left`, `viewport_bottom` or `depth_near`. If they are equal to `1.0`, the result
/// will be `viewport_left + viewport_width` (the right of the viewport),
/// `viewport_bottom + viewport_height` (the top of the viewport) or `depth_far`.
///
/// For example if you want to draw a rectangle that covers the whole screen, it should be made
/// of four vertices whose coordinates are `(-1.0, -1.0, 0.0, 1.0)` (bottom-left corner),
/// `(-1.0, 1.0, 0.0, 1.0)` (top-left corner), `(1.0, 1.0, 0.0, 1.0)` (top-right corner) and
/// `(1.0, -1.0, 0.0, 1.0)` (bottom-right corner).
///
/// ## Step 6: Primitive assembly
///
/// The next step consists in building the primitives. Triangle strips, triangle fans and line
/// strips are turned into individual triangles or lines.
///
/// Triangle strips obey certain rules for the order of indices. For example the triangle strip
/// `0, 1, 2, 3, 4, 5` does *not* correspond to `0, 1, 2`, `1, 2, 3`, `2, 3, 4`, `3, 4, 5` as you
/// would expect, but to `0, 1, 2`, `1, 3, 2`, `2, 3, 4`, `3, 5, 4` (some indices are reversed).
/// This is important with regards to the face culling step below.
///
/// Then, if you did specify `PrimitiveMode`, it is used. If you specified `Line`, triangles are
/// turned into lines. If specified `Point`, triangles and lines are turned into points.
///
/// The GPU then looks at the screen coordinates of each primitive, and discards primitives that
/// are entirely outside of the window.
///
/// Note that points whose centers are outside of the viewport are discarded, even if the point
/// width would be big enough for the point to be visible. However this standard behavior is not
/// respected by nVidia drivers, which show the points anyway.
///
/// ## Step 7: Face culling (triangles only)
///
/// This step is purely an optimization step and only concerns triangles.
///
/// If you specify a value for `backface_culling` other than `CullingDisabled`, the GPU will
/// discard triangles depending on the way that the vertices are arranged on the window. You can
/// either discard triangles whose vertices are clockwise or counterclockwise.
///
/// For more information, see the `BackfaceCullingMode` documentation.
///
/// ## Step 8: Rasterization
///
/// Now that the GPU knows where on the window the various triangles, points or lines are, it will
/// determine which pixels of the surface are part of each primitive.
///
/// For points and lines, this step depends on the points width and line width that you specified
/// in the draw parameters.
///
/// <img alt="" src="data:image/png;base64,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" />
///
/// The attributes of each vertex are being interpolated, and the GPU assigns a value for each
/// attribute for each pixel.
///
/// ## Step 9: Fragment shader
///
/// The GPU now executes the fragment shader once for each pixel of each primitive.
///
/// The vertex attributes that were interpolated at the previous step are passed to the fragment
/// shader.
///
/// The fragment shader must return the color to write by setting the value of `gl_FragColor`.
///
/// ## Step 10: Pixel ownership
///
/// This step is mostly an implementation detail. If the window you are drawing on is not on the
/// foreground, or if it is partially obstructed, then the pixels that are not on the
/// foreground will be discarded.
///
/// This is only relevant if you draw to the default framebuffer.
///
/// ## Step 11: Scissor test
///
/// If `scissor` has been specified, then all the pixels that are outside of this rect
/// are discarded.
///
/// ## Step 12: Multisampling
///
/// ## Step 13: Stencil test
///
/// Stencil tests are currently not supported by glium.
///
/// ## Step 14: Depth test
///
/// In addition to the colors, surfaces can also have a depth buffer attached to it. In this
/// situation, just like each pixel has a color, each pixel of the surface also has an associated
/// depth value.
///
/// If a depth buffer is present, the GPU will compare the depth value of the pixel currently
/// being processed, with the existing depth value. Depending on the value of `depth_function`
/// in the draw parameters, the depth test will either pass, in which case the pipeline
/// continues, or fail, in which case the pixel is discarded.
///
/// The purpose of this test is to avoid drawing elements that are in the background of the
/// scene over elements that are in the foreground.
///
/// See the documentation of `DepthFunction` for more informations.
///
/// ## Step 15: Blending
///
/// For each pixel to write, the GPU takes the RGBA color that the fragment shader has returned
/// and the existing RGBA color already written on the surface, and merges the two.
///
/// The way they are merged depends on the value of `blending_function`. This allows you to choose
/// how alpha colors are merged together.
///
/// See the documentation of `BlendingFunction` fore more informations.
///
/// ## Step 16: Dithering (optional)
///
/// ## Step 17: End
///
/// This is finally the step where colors are being written.
///
/// ## Missing steps
///
/// Some steps are missing because they are not supported by glium for the moment: dithering,
/// occlusion query updating, logic operations, sRGB conversion, write masks.
///
/// Instancing and multiple viewports are also missing, as they are not supported.
///
pub trait Surface: Sized {
    /// Clears some attachments of the target.
    fn clear(&mut self, color: Option<(f32, f32, f32, f32)>, depth: Option<f32>,
             stencil: Option<i32>);

    /// Clears the color attachment of the target.
    fn clear_color(&mut self, red: f32, green: f32, blue: f32, alpha: f32) {
        self.clear(Some((red, green, blue, alpha)), None, None);
    }

    /// Clears the depth attachment of the target.
    fn clear_depth(&mut self, value: f32) {
        self.clear(None, Some(value), None);
    }

    /// Clears the stencil attachment of the target.
    fn clear_stencil(&mut self, value: i32) {
        self.clear(None, None, Some(value));
    }

    /// Clears the color and depth attachments of the target.
    fn clear_color_and_depth(&mut self, color: (f32, f32, f32, f32), depth: f32) {
        self.clear(Some(color), Some(depth), None);
    }

    /// Clears the color and stencil attachments of the target.
    fn clear_color_and_stencil(&mut self, color: (f32, f32, f32, f32), stencil: i32) {
        self.clear(Some(color), None, Some(stencil));
    }

    /// Clears the depth and stencil attachments of the target.
    fn clear_depth_and_stencil(&mut self, depth: f32, stencil: i32) {
        self.clear(None, Some(depth), Some(stencil));
    }

    /// Clears the color, depth and stencil attachments of the target.
    fn clear_all(&mut self, color: (f32, f32, f32, f32), depth: f32, stencil: i32) {
        self.clear(Some(color), Some(depth), Some(stencil));
    }

    /// Returns the dimensions in pixels of the target.
    fn get_dimensions(&self) -> (u32, u32);

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
    /// See above for what happens exactly when you draw.
    ///
    /// # Panic
    ///
    /// - Panics if the requested depth function requires a depth buffer and none is attached.
    /// - Panics if the type of some of the vertex source's attributes do not match the program's.
    /// - Panics if a program's attribute is not in the vertex source (does *not* panic if a
    ///   vertex's attribute is not used by the program).
    /// - Panics if the viewport is larger than the dimensions supported by the hardware.
    /// - Panics if the depth range is outside of `(0, 1)`.
    /// - Panics if a value in the uniforms doesn't match the type requested by the program.
    ///
    fn draw<'a, 'b, V, I, U>(&mut self, V, &I, program: &Program, uniforms: U,
        draw_parameters: &DrawParameters) -> Result<(), DrawError> where
        V: vertex::MultiVerticesSource<'b>, I: index_buffer::ToIndicesSource,
        U: uniforms::Uniforms;

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
    /// Note that there is no alpha blending, depth/stencil checking, etc. This function just
    /// copies pixels.
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

/// Error that can happen while drawing.
#[derive(Clone, Show)]
pub enum DrawError {
    /// A depth function has been requested but no depth buffer is available.
    NoDepthBuffer,

    /// The type of a vertex attribute in the vertices source doesn't match what the
    /// program requires.
    AttributeTypeMismatch,

    /// One of the attributes required by the program is missing from the vertex format.
    ///
    /// Note that it is perfectly valid to have an attribute in the vertex format that is
    /// not used by the program.
    AttributeMissing,

    /// The viewport's dimensions are not supported by the backend.
    ViewportTooLarge,

    /// The depth range is outside of the `(0, 1)` range.
    InvalidDepthRange,

    /// The type of a uniform doesn't match what the program requires.
    UniformTypeMismatch {
        /// Name of the uniform you are trying to bind.
        name: String,
        /// The expected type.
        expected: uniforms::UniformType,
    },

    /// Tried to bind a uniform buffer to a single uniform value.
    UniformBufferToValue {
        /// Name of the uniform you are trying to bind.
        name: String,
    },

    /// Tried to bind a single uniform value to a uniform block.
    UniformValueToBlock {
        /// Name of the uniform you are trying to bind.
        name: String,
    },

    /// The layout of the content of the uniform buffer does not match the layout of the block.
    UniformBlockLayoutMismatch {
        /// Name of the block you are trying to bind.
        name: String,
    },

    /// The number of vertices per patch that has been requested is not supported.
    UnsupportedVerticesPerPatch,

    /// Trying to use tessellation, but this is not supported by the underlying hardware.
    TessellationNotSupported,

    /// Using a program which contains tessellation shaders, but without submitting patches.
    TessellationWithoutPatches,

    /// Trying to use a sampler, but they are not supported by the backend.
    SamplersNotSupported,

    /// When you use instancing, all vertices sources must have the same size.
    InstancesCountMismatch,
}

#[doc(hidden)]
pub struct BlitHelper<'a>(&'a Arc<DisplayImpl>, Option<&'a fbo::FramebufferAttachments>);

/// Implementation of `Surface`, targeting the default framebuffer.
///
/// The back- and front-buffers are swapped when the `Frame` is destroyed. This operation is
/// instantaneous, even when vsync is enabled.
pub struct Frame {
    display: Display,
    dimensions: (u32, u32),
}

impl Frame {
    /// Stop drawing, and swap the buffers.
    pub fn finish(self) {
    }
}

impl Surface for Frame {
    fn clear(&mut self, color: Option<(f32, f32, f32, f32)>, depth: Option<f32>,
             stencil: Option<i32>)
    {
        ops::clear(&self.display.context, None, color, depth, stencil);
    }

    fn get_dimensions(&self) -> (u32, u32) {
        self.dimensions
    }

    fn get_depth_buffer_bits(&self) -> Option<u16> {
        self.display.context.context.capabilities().depth_bits
    }

    fn get_stencil_buffer_bits(&self) -> Option<u16> {
        self.display.context.context.capabilities().stencil_bits
    }

    fn draw<'a, 'b, V, I, U>(&mut self, vertex_buffer: V,
                         index_buffer: &I, program: &Program, uniforms: U,
                         draw_parameters: &DrawParameters) -> Result<(), DrawError>
                         where I: index_buffer::ToIndicesSource, U: uniforms::Uniforms,
                         V: vertex::MultiVerticesSource<'b>
    {
        use index_buffer::ToIndicesSource;

        if draw_parameters.depth_function.requires_depth_buffer() && !self.has_depth_buffer() {
            return Err(DrawError::NoDepthBuffer);
        }

        if let Some(viewport) = draw_parameters.viewport {
            if viewport.width > self.display.context.context.capabilities().max_viewport_dims.0
                    as u32
            {
                return Err(DrawError::ViewportTooLarge);
            }
            if viewport.height > self.display.context.context.capabilities().max_viewport_dims.1
                    as u32
            {
                return Err(DrawError::ViewportTooLarge);
            }
        }

        ops::draw(&self.display, None, vertex_buffer.build_vertices_source().as_mut_slice(),
                  index_buffer.to_indices_source(), program, uniforms, draw_parameters,
                  (self.dimensions.0 as u32, self.dimensions.1 as u32))
    }

    fn get_blit_helper(&self) -> BlitHelper {
        BlitHelper(&self.display.context, None)
    }
}

#[unsafe_destructor]
impl Drop for Frame {
    fn drop(&mut self) {
        self.display.context.context.swap_buffers();
    }
}

/// Objects that can build a `Display` object.
pub trait DisplayBuild {
    /// Build a context and a `Display` to draw on it.
    ///
    /// Performs a compatibility check to make sure that all core elements of glium
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
                framebuffer_objects: Some(fbo::FramebuffersContainer::new()),
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
                framebuffer_objects: Some(fbo::FramebuffersContainer::new()),
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
/// your program and between threads.
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
    // the option is here to destroy the container
    framebuffer_objects: Option<fbo::FramebuffersContainer>,

    // we maintain a list of VAOs for each vertexbuffer-indexbuffer-program association
    // the key is a (buffers-list, program) ; the buffers list must be sorted
    vertex_array_objects: Mutex<HashMap<(Vec<gl::types::GLuint>, gl::types::GLuint),
                                        vertex_array_object::VertexArrayObject>>,

    // we maintain a list of samplers for each possible behavior
    samplers: Mutex<HashMap<uniforms::SamplerBehavior, sampler_object::SamplerObject>>,
}

impl Display {
    /// Reads all events received by the window.
    pub fn poll_events(&self) -> Vec<glutin::Event> {
        self.context.context.recv()
    }

    /// Returns the dimensions of the main framebuffer.
    pub fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        self.context.context.get_framebuffer_dimensions()
    }

    /// Start drawing on the backbuffer.
    ///
    /// This function returns a `Frame`, which can be used to draw on it. When the `Frame` is
    /// destroyed, the buffers are swapped.
    ///
    /// Note that destroying a `Frame` is immediate, even if vsync is enabled.
    pub fn draw(&self) -> Frame {
        Frame {
            display: self.clone(),
            dimensions: self.get_framebuffer_dimensions(),
        }
    }

    /// Returns the maximum value that can be used for anisotropic filtering, or `None`
    /// if the hardware doesn't support it.
    pub fn get_max_anisotropy_support(&self) -> Option<u16> {
        self.context.context.capabilities().max_texture_max_anisotropy.map(|v| v as u16)
    }

    /// Returns the maximum dimensions of the viewport.
    ///
    /// Glium will panic if you request a larger viewport than this when drawing.
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
    pub fn get_free_video_memory(&self) -> Option<usize> {
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

        rx.recv().unwrap().map(|v| v as usize * 1024)
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
            *cb = Some(Box::new(callback) as Box<FnMut(String, debug::Source, debug::MessageType,
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

                if let &mut Some(ref mut callback) = callback {
                    callback.call_mut((message.to_string(),
                        FromPrimitive::from_uint(source as usize).unwrap_or(debug::Source::OtherSource),
                        FromPrimitive::from_uint(ty as usize).unwrap_or(debug::MessageType::Other),
                        FromPrimitive::from_uint(severity as usize).unwrap_or(debug::Severity::Notification)));
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
    /// ```no_run
    /// # extern crate glium;
    /// # extern crate glutin;
    /// # fn main() {
    /// # let display: glium::Display = unsafe { ::std::mem::uninitialized() };
    /// let pixels: Vec<Vec<(u8, u8, u8)>> = display.read_front_buffer();
    /// # }
    /// ```
    pub fn read_front_buffer<P, T>(&self) -> T          // TODO: remove Clone for P
                                   where P: texture::PixelValue + Clone + Send,
                                   T: texture::Texture2dData<Data = P>
    {
        ops::read_from_default_fb(gl::FRONT_LEFT, self)
    }

    /// Asserts that there are no OpenGL errors pending.
    ///
    /// This function should be used in tests.
    pub fn assert_no_error(&self) {
        let (tx, rx) = channel();

        self.context.context.exec(move |: mut ctxt| {
            tx.send(get_gl_error(&mut ctxt)).ok();
        });

        match rx.recv().unwrap() {
            Some(msg) => panic!("{}", msg),
            None => ()
        };
    }

    /// Waits until all the previous commands have finished being executed.
    ///
    /// When you execute OpenGL functions, they are not executed immediately. Instead they are
    /// put in a queue. This function waits until all commands have finished being executed, and
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
            let fbos = self.framebuffer_objects.take();
            fbos.unwrap().cleanup(&self.context);
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
fn get_gl_error(ctxt: &mut context::CommandContext) -> Option<&'static str> {
    match unsafe { ctxt.gl.GetError() } {
        gl::NO_ERROR => None,
        gl::INVALID_ENUM => Some("GL_INVALID_ENUM"),
        gl::INVALID_VALUE => Some("GL_INVALID_VALUE"),
        gl::INVALID_OPERATION => Some("GL_INVALID_OPERATION"),
        gl::INVALID_FRAMEBUFFER_OPERATION => Some("GL_INVALID_FRAMEBUFFER_OPERATION"),
        gl::OUT_OF_MEMORY => Some("GL_OUT_OF_MEMORY"),
        gl::STACK_UNDERFLOW => Some("GL_STACK_UNDERFLOW"),
        gl::STACK_OVERFLOW => Some("GL_STACK_OVERFLOW"),
        _ => Some("Unknown glGetError return value")
    }
}
