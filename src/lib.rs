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

# Complete example

The first step is to create the vertex buffer, which contains the list of all the points that
make up our mesh. The elements that we pass to `VertexBuffer::new` must implement the
`glium::vertex::VertexFormat` trait, which can be easily added for any custom struct thanks to the
`implement_vertex!` macro.

See the `vertex` module documentation for more informations.

```no_run
# #[macro_use]
# extern crate glium;
# fn main() {
#[derive(Copy)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

implement_vertex!(Vertex, position, color);

# let display: glium::Display = unsafe { std::mem::uninitialized() };
let vertex = glium::VertexBuffer::new(&display, vec![
    Vertex { position: [-0.5, -0.5], color: [0.0, 1.0, 0.0] },
    Vertex { position: [ 0.0,  0.5], color: [0.0, 0.0, 1.0] },
    Vertex { position: [ 0.5, -0.5], color: [1.0, 0.0, 0.0] },
]);
# }
```

We will also need to tell glium how the vertices must be linked together. We could create an index
buffer, but since we only have a single triangle the simpler solution here is not to use indices.

```no_run
use glium::index;
let indices = index::NoIndices(index::PrimitiveType::TrianglesList);
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

*Note: teaching you the GLSL language is not covered by this guide.*

You may notice that the `attribute` declarations in the vertex shader match the field names and
types of the elements in the vertex buffer. This is required, otherwise drawing will result in
an error.

In the example above, one of our shaders contains `uniform mat4 matrix;`. Uniforms are global
variables in our program whose values are chosen by the application.

```no_run
# #[macro_use]
# extern crate glium;
# fn main() {
let uniforms = uniform! {
    matrix: [
        [ 1.0, 0.0, 0.0, 0.0 ],
        [ 0.0, 1.0, 0.0, 0.0 ],
        [ 0.0, 0.0, 1.0, 0.0 ],
        [ 0.0, 0.0, 0.0, 1.0 ]
    ]
};
# }
```

The value of uniforms can be of any type that implements `glium::uniforms::UniformValue`.
This includes textures and samplers (not covered here). See the `uniforms` module documentation 
for more informations.

Now that everything is initialized, we can finally draw something. The `display.draw()` function
will start drawing a new frame and return a `Frame` object. This `Frame` object has a `draw`
function, which you can use to draw things.

Its arguments are the source of vertices, source of indices, program, uniforms, and an object of
type `DrawParameters` which  contains miscellaneous information specifying how everything should
be rendered (depth test, blending, backface culling, etc.).

```no_run
use glium::Surface;
# let display: glium::Display = unsafe { std::mem::uninitialized() };
# let vertex_buffer: glium::VertexBuffer<u8> = unsafe { std::mem::uninitialized() };
# let indices: glium::IndexBuffer = unsafe { std::mem::uninitialized() };
# let program: glium::Program = unsafe { std::mem::uninitialized() };
# let uniforms = glium::uniforms::EmptyUniforms;
let mut target = display.draw();
target.clear_color(0.0, 0.0, 0.0, 0.0);  // filling the output with the black color
target.draw(&vertex_buffer, &indices, &program, &uniforms,
            &std::default::Default::default()).unwrap();
target.finish();
```

*/
#![feature(core, hash, std_misc, collections)]     // TODO: remove after 1.0 beta

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

pub use context::{PollEventsIter, WaitEventsIter};
pub use draw_parameters::{BlendingFunction, LinearBlendingFactor, BackfaceCullingMode};
pub use draw_parameters::{DepthTest, PolygonMode, DrawParameters};
pub use index::IndexBuffer;
pub use vertex::{VertexBuffer, Vertex, VertexFormat};
pub use program::{Program, ProgramCreationError};
pub use program::ProgramCreationError::{CompilationError, LinkingError, ShaderTypeNotSupported};
pub use sync::{LinearSyncFence, SyncFence};
pub use texture::{Texture, Texture2d};
pub use version::{Api, Version};

use std::default::Default;
use std::collections::hash_state::DefaultState;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLockReadGuard};
use std::sync::mpsc::channel;

pub mod debug;
pub mod framebuffer;
pub mod index;
pub mod pixel_buffer;
pub mod macros;
pub mod program;
pub mod render_buffer;
pub mod uniforms;
pub mod vertex;
pub mod texture;

#[deprecated = "`index_buffer` has been renamed to `index`"]
#[allow(missing_docs)]
pub mod index_buffer {
    pub use index::*;
}

mod buffer;
mod context;
mod draw_parameters;
mod fbo;
mod ops;
mod sampler_object;
mod sync;
mod util;
mod version;
mod vertex_array_object;

mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

/// Trait for objects that are OpenGL objects.
pub trait GlObject {
    type Id;

    /// Returns the id of the object.
    fn get_id(&self) -> Self::Id;
}

/// Handle to a shader or a program.
// TODO: Handle(null()) is equal to Id(0)
#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
enum Handle {
    Id(gl::types::GLuint),
    Handle(gl::types::GLhandleARB),
}

unsafe impl Send for Handle {}

/// Internal trait for enums that can be turned into GLenum.
trait ToGlEnum {
    /// Returns the value.
    fn to_glenum(&self) -> gl::types::GLenum;
}

/// Area of a surface in pixels.
///
/// In the OpenGL ecosystem, the (0,0) coordinate is at the bottom-left hand corner of the images.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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

/// Area of a surface in pixels. Similar to a `Rect` except that dimensions can be negative.
///
/// In the OpenGL ecosystem, the (0,0) coordinate is at the bottom-left hand corner of the images.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BlitTarget {
    /// Number of pixels between the left border of the surface and the left border of
    /// the rectangle.
    pub left: u32,
    /// Number of pixels between the bottom border of the surface and the bottom border
    /// of the rectangle.
    pub bottom: u32,
    /// Width of the area in pixels. Can be negative.
    pub width: i32,
    /// Height of the area in pixels. Can be negative.
    pub height: i32,
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
/// being processed, with the existing depth value. Depending on the value of `depth_test`
/// in the draw parameters, the depth test will either pass, in which case the pipeline
/// continues, or fail, in which case the pixel is discarded. If the value of `depth_write`
/// is true and the test passed, it will then also write the depth value of the pixel on the
/// depth buffer.
///
/// The purpose of this test is to avoid drawing elements that are in the background of the
/// scene over elements that are in the foreground.
///
/// See the documentation of `DepthTest` for more informations.
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
        V: vertex::MultiVerticesSource<'b>, I: index::ToIndicesSource,
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
    #[unstable = "The name will likely change"]
    fn blit_color<S>(&self, source_rect: &Rect, target: &S, target_rect: &BlitTarget,
        filter: uniforms::MagnifySamplerFilter) where S: Surface
    {
        ops::blit(self, target, gl::COLOR_BUFFER_BIT, source_rect, target_rect,
            filter.to_glenum())
    }

    /// Copies the entire surface to a target surface. See `blit_color`.
    #[unstable = "The name will likely change"]
    fn blit_whole_color_to<S>(&self, target: &S, target_rect: &BlitTarget,
        filter: uniforms::MagnifySamplerFilter) where S: Surface
    {
        let src_dim = self.get_dimensions();
        let src_rect = Rect { left: 0, bottom: 0, width: src_dim.0 as u32, height: src_dim.1 as u32 };
        self.blit_color(&src_rect, target, target_rect, filter)
    }

    /// Copies the entire surface to the entire target. See `blit_color`.
    #[unstable = "The name will likely change"]
    fn fill<S>(&self, target: &S, filter: uniforms::MagnifySamplerFilter) where S: Surface {
        let src_dim = self.get_dimensions();
        let src_rect = Rect { left: 0, bottom: 0, width: src_dim.0 as u32, height: src_dim.1 as u32 };
        let target_dim = target.get_dimensions();
        let target_rect = BlitTarget { left: 0, bottom: 0, width: target_dim.0 as i32, height: target_dim.1 as i32 };
        self.blit_color(&src_rect, target, &target_rect, filter)
    }
}

/// Error that can happen while drawing.
#[derive(Clone, Debug)]
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

    /// If you don't use indices, then all vertices sources must have the same size.
    VerticesSourcesLengthMismatch,

    /// You requested not to draw primitives, but this is not supported by the backend.
    TransformFeedbackNotSupported,
}

impl std::fmt::Display for DrawError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            &DrawError::NoDepthBuffer => write!(fmt, "A depth function has been requested but no \
                                                      depth buffer is available."),
            &DrawError::AttributeTypeMismatch => write!(fmt, "The type of a vertex attribute in \
                                                              the vertices source doesn't match \
                                                              what the program requires."),
            &DrawError::AttributeMissing => write!(fmt, "One of the attributes required by the \
                                                         program is missing from the vertex \
                                                         format."),
            &DrawError::ViewportTooLarge => write!(fmt, "The viewport's dimensions are not \
                                                         supported by the backend."),
            &DrawError::InvalidDepthRange => write!(fmt, "The depth range is outside of the \
                                                          `(0, 1)` range."),
            &DrawError::UniformTypeMismatch { ref name, ref expected } => {
                write!(fmt, "The type of a uniform doesn't match what the program requires.")
            },
            &DrawError::UniformBufferToValue { ref name } => write!(fmt, "Tried to bind a uniform \
                                                                          buffer to a single \
                                                                          uniform value."),
            &DrawError::UniformValueToBlock { ref name } => {
                write!(fmt, "Tried to bind a single uniform value to a uniform block.")
            },
            &DrawError::UniformBlockLayoutMismatch { ref name } => {
                write!(fmt, "The layout of the content of the uniform buffer does not match \
                             the layout of the block.")
            },
            &DrawError::UnsupportedVerticesPerPatch => write!(fmt, "The number of vertices per \
                                                                    patch that has been requested \
                                                                    is not supported."),
            &DrawError::TessellationNotSupported => write!(fmt, "Trying to use tessellation, but \
                                                                 this is not supported by the \
                                                                 underlying hardware."),
            &DrawError::TessellationWithoutPatches => write!(fmt, "Using a program which contains \
                                                                   tessellation shaders, but \
                                                                   without submitting patches."),
            &DrawError::SamplersNotSupported => write!(fmt, "Trying to use a sampler, but they are \
                                                             not supported by the backend."),
            &DrawError::InstancesCountMismatch => write!(fmt, "When you use instancing, all \
                                                               vertices sources must have the \
                                                               same size"),
            &DrawError::VerticesSourcesLengthMismatch => write!(fmt, "If you don't use indices, \
                                                                      then all vertices sources \
                                                                      must have the same size."),
            &DrawError::TransformFeedbackNotSupported => write!(fmt, "Requested not to draw \
                                                                      primitves, but this is not \
                                                                      supported by the backend."),
        }
    }
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
                         where I: index::ToIndicesSource, U: uniforms::Uniforms,
                         V: vertex::MultiVerticesSource<'b>
    {
        use index::ToIndicesSource;

        if !self.has_depth_buffer() && (draw_parameters.depth_test.requires_depth_buffer() ||
                draw_parameters.depth_write)
        {
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

        ops::draw(&self.display, None, vertex_buffer, index_buffer.to_indices_source(), program,
                  uniforms, draw_parameters, (self.dimensions.0 as u32, self.dimensions.1 as u32))
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

    /// Changes the settings of an existing `Display`.
    fn rebuild_glium(self, &Display) -> Result<(), GliumCreationError>;
}

/// Error that can happen while creating a glium display.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GliumCreationError {
    /// An error has happened while creating the glutin window or headless renderer.
    GlutinCreationError(glutin::CreationError),

    /// The OpenGL implementation is too old.
    IncompatibleOpenGl(String),
}

impl std::fmt::Display for GliumCreationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let self_error = self as &std::error::Error;
        formatter.write_str(self_error.description())
    }
}

impl std::error::Error for GliumCreationError {
    fn description(&self) -> &str {
        match self {
            &GliumCreationError::GlutinCreationError(_) => "Error while creating glutin window or headless renderer",
            &GliumCreationError::IncompatibleOpenGl(_) => "The OpenGL implementation is too old to work with glium",
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

impl DisplayBuild for glutin::WindowBuilder<'static> {
    fn build_glium(self) -> Result<Display, GliumCreationError> {
        let (context, shared_debug) = try!(context::new_from_window(self));

        let display = Display {
            context: Arc::new(DisplayImpl {
                context: context,
                debug_callback: Mutex::new(None),
                shared_debug_output: shared_debug,
                framebuffer_objects: Some(fbo::FramebuffersContainer::new()),
                vertex_array_objects: Mutex::new(HashMap::with_hash_state(Default::default())),
                samplers: Mutex::new(HashMap::with_hash_state(Default::default())),
            }),
        };

        display.init_debug_callback();
        Ok(display)
    }

    fn rebuild_glium(self, display: &Display) -> Result<(), GliumCreationError> {
        // framebuffer objects and vertex array objects aren't shared, so we have to destroy them
        if let Some(ref fbos) = display.context.framebuffer_objects {
            fbos.purge_all(&display.context.context);
        }

        {
            let mut vaos = display.context.vertex_array_objects.lock().unwrap();
            vaos.clear();
        }

        display.context.context.rebuild(self)
    }
}

#[cfg(feature = "headless")]
impl DisplayBuild for glutin::HeadlessRendererBuilder {
    fn build_glium(self) -> Result<Display, GliumCreationError> {
        let (context, shared_debug) = try!(context::new_from_headless(self));

        let display = Display {
            context: Arc::new(DisplayImpl {
                context: context,
                debug_callback: Mutex::new(None),
                shared_debug_output: shared_debug,
                framebuffer_objects: Some(fbo::FramebuffersContainer::new()),
                vertex_array_objects: Mutex::new(HashMap::with_hash_state(Default::default())),
                samplers: Mutex::new(HashMap::with_hash_state(Default::default())),
            }),
        };

        display.init_debug_callback();
        Ok(display)
    }

    fn rebuild_glium(self, _: &Display) -> Result<(), GliumCreationError> {
        unimplemented!()
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

    // holding the Arc to SharedDebugOutput
    shared_debug_output: Arc<context::SharedDebugOutput>,

    // we maintain a list of FBOs
    // the option is here to destroy the container
    framebuffer_objects: Option<fbo::FramebuffersContainer>,

    // we maintain a list of VAOs for each vertexbuffer-indexbuffer-program association
    // the key is a (buffers-list, program) ; the buffers list must be sorted
    vertex_array_objects: Mutex<HashMap<(Vec<gl::types::GLuint>, Handle),
                                        vertex_array_object::VertexArrayObject, DefaultState<util::FnvHasher>>>,

    // we maintain a list of samplers for each possible behavior
    samplers: Mutex<HashMap<uniforms::SamplerBehavior, sampler_object::SamplerObject, 
                    DefaultState<util::FnvHasher>>>,
}

impl Display {
    /// Reads all events received by the window.
    ///
    /// This iterator polls for events and can be exhausted.
    pub fn poll_events(&self) -> PollEventsIter {
        self.context.context.poll_events()
    }

    /// Reads all events received by the window.
    pub fn wait_events(&self) -> WaitEventsIter {
        self.context.context.wait_events()
    }

    /// Returns the underlying window, or `None` if glium uses a headless context.
    pub fn get_window(&self) -> Option<RwLockReadGuard<glutin::Window>> {
        self.context.context.get_window()
    }

    /// Returns the dimensions of the main framebuffer.
    pub fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        self.context.context.get_framebuffer_dimensions()
    }

    /// Returns the OpenGL version of the current context.
    // TODO: return API as well
    pub fn get_opengl_version(&self) -> Version {
        *self.context.context.get_version()
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
        self.context.context.exec(move |ctxt| {
            unsafe {
                if ctxt.version >= &context::GlVersion(Api::GlEs, 2, 0) ||
                    ctxt.version >= &context::GlVersion(Api::Gl, 4, 1)
                {
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

        self.context.context.exec(move |ctxt| {
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

    // TODO: do this more properly
    fn init_debug_callback(&self) {
        if cfg!(ndebug) {
            return;
        }

        if ::std::env::var("GLIUM_DISABLE_DEBUG_OUTPUT").is_ok() {
            return;
        }

        // this is the C callback
        extern "system" fn callback_wrapper(source: gl::types::GLenum, ty: gl::types::GLenum,
            id: gl::types::GLuint, severity: gl::types::GLenum, _length: gl::types::GLsizei,
            message: *const gl::types::GLchar, user_param: *mut libc::c_void)
        {
            let user_param = user_param as *const context::SharedDebugOutput;
            let user_param = unsafe { user_param.as_ref().unwrap() };

            if (severity == gl::DEBUG_SEVERITY_HIGH || severity == gl::DEBUG_SEVERITY_MEDIUM) && 
               (ty == gl::DEBUG_TYPE_ERROR || ty == gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR ||
                ty == gl::DEBUG_TYPE_PORTABILITY || ty == gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR)
            {
                if user_param.report_errors.load(std::sync::atomic::Ordering::Relaxed) {
                    let message = unsafe {
                        String::from_utf8(std::ffi::c_str_to_bytes(&message).to_vec()).unwrap()
                    };

                    panic!("Debug message with high or medium severity: `{}`.\n\
                            Please report this error: https://github.com/tomaka/glium/issues",
                            message);
                }
            }
        }

        struct SharedDebugOutputPtr(*const context::SharedDebugOutput);
        unsafe impl Send for SharedDebugOutputPtr {}
        let shared_debug_output_ptr = SharedDebugOutputPtr(self.context.shared_debug_output.deref());

        // enabling the callback
        self.context.context.exec(move |ctxt| {
            unsafe {
                if ctxt.version >= &context::GlVersion(Api::Gl, 4,5) || ctxt.extensions.gl_khr_debug ||
                    ctxt.extensions.gl_arb_debug_output
                {
                    if ctxt.state.enabled_debug_output_synchronous != true {
                        ctxt.gl.Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
                        ctxt.state.enabled_debug_output_synchronous = true;
                    }

                    if ctxt.version >= &context::GlVersion(Api::Gl, 4,5) || ctxt.extensions.gl_khr_debug {
                        // TODO: with GLES, the GL_KHR_debug function has a `KHR` suffix
                        //       but with GL only, it doesn't have one
                        ctxt.gl.DebugMessageCallback(callback_wrapper, shared_debug_output_ptr.0
                                                                         as *const libc::c_void);
                        ctxt.gl.DebugMessageControl(gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE, 0,
                                                    std::ptr::null(), gl::TRUE);

                        if ctxt.state.enabled_debug_output != Some(true) {
                            ctxt.gl.Enable(gl::DEBUG_OUTPUT);
                            ctxt.state.enabled_debug_output = Some(true);
                        }

                    } else {
                        ctxt.gl.DebugMessageCallbackARB(callback_wrapper, shared_debug_output_ptr.0
                                                                            as *const libc::c_void);
                        ctxt.gl.DebugMessageControlARB(gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE,
                                                       0, std::ptr::null(), gl::TRUE);

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
                                   T: texture::Texture2dDataSink<Data = P>
    {
        ops::read_from_default_fb(gl::FRONT_LEFT, self)
    }

    /// Execute an arbitrary closure with the OpenGL context active. Useful if another
    /// component needs to directly manipulate OpenGL state.
    ///
    /// **If action manipulates any OpenGL state, it must be restored before action
    /// completes.**
    pub unsafe fn exec_in_context<'a, T, F>(&self, action: F) -> T
                                            where T: Send + 'static,
                                            F: FnOnce() -> T + 'a
    {
        let (tx, rx) = channel();
        self.context.context.exec_maybe_sync(true, move |ctxt| {
            tx.send(action()).ok();
        });

        rx.recv().unwrap()
    }

    /// Asserts that there are no OpenGL errors pending.
    ///
    /// This function should be used in tests.
    pub fn assert_no_error(&self) {
        let (tx, rx) = channel();

        self.context.context.exec(move |mut ctxt| {
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

        self.context.context.exec(move |ctxt| {
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
        // disabling callback
        self.context.exec(move |ctxt| {
            unsafe {
                if ctxt.state.enabled_debug_output != Some(false) {
                    if ctxt.version >= &context::GlVersion(Api::Gl, 4,5) || ctxt.extensions.gl_khr_debug {
                        ctxt.gl.Disable(gl::DEBUG_OUTPUT);
                    } else if ctxt.extensions.gl_arb_debug_output {
                        ctxt.gl.DebugMessageCallbackARB(std::mem::transmute(0usize),
                                                        std::ptr::null());
                    }

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
