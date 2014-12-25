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
# #![feature(phase)]
#[phase(plugin)]
extern crate glium_macros;

# extern crate glium;
# fn main() {
#[vertex_format]
#[deriving(Copy)]
struct Vertex {
    position: [f32, ..2],
    color: [f32, ..3],
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
let program = glium::Program::new(&display, 
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
# #![feature(phase)]
#[phase(plugin)]
extern crate glium_macros;

# extern crate glium;
# fn main() {
#[uniforms]
struct Uniforms {
    matrix: [[f32, ..4], ..4],
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

#![feature(associated_types)]
#![feature(default_type_params)]
#![feature(globs)]
#![feature(slicing_syntax)]
#![feature(unboxed_closures)]
#![feature(unsafe_destructor)]
#![unstable]
#![warn(missing_docs)]

// TODO: remove these when everything is implemented
#![allow(dead_code)]
#![allow(unused_variables)]

extern crate cgmath;
extern crate glutin;
#[cfg(feature = "image")]
extern crate image;
extern crate libc;
extern crate nalgebra;

pub use index_buffer::IndexBuffer;
pub use vertex_buffer::{VertexBuffer, Vertex, VertexFormat};
pub use program::{Program, ProgramCreationError};
pub use program::ProgramCreationError::{CompilationError, LinkingError, ShaderTypeNotSupported};
pub use texture::{Texture, Texture2d};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub mod debug;
pub mod framebuffer;
pub mod index_buffer;
pub mod uniforms;
pub mod vertex_buffer;
pub mod texture;

mod buffer;
mod context;
mod fbo;
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
#[deriving(Clone, Copy, Show, PartialEq, Eq)]
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

/// Culling mode.
/// 
/// Describes how triangles could be filtered before the fragment part.
#[deriving(Clone, Copy, Show, PartialEq, Eq)]
pub enum BackfaceCullingMode {
    /// All triangles are always drawn.
    CullingDisabled,

    /// Triangles whose vertices are counter-clock-wise won't be drawn.
    CullCounterClockWise,

    /// Triangles whose indices are clock-wise won't be drawn.
    CullClockWise
}

/// The function that the GPU will use to determine whether to write over an existing pixel
///  on the target.
#[deriving(Clone, Copy, Show, PartialEq, Eq)]
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
#[deriving(Clone, Copy, Show, PartialEq, Eq)]
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
#[deriving(Clone, Copy, Show, PartialEq)]
pub struct DrawParameters {
    /// The function that the GPU will use to determine whether to write over an existing pixel
    /// on the target.
    ///
    /// The default is `Overwrite`.
    pub depth_function: DepthFunction,

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
    ///  the camera.
    pub backface_culling: BackfaceCullingMode,

    /// Sets how to render polygons. The default value is `Fill`.
    pub polygon_mode: PolygonMode,

    /// Whether multisample antialiasing (MSAA) should be used. Default value is `true`.
    ///
    /// Note that you will need to set the appropriate option when creating the window.
    /// The recommended way to do is to leave this to `true`, and adjust the option when
    /// creating the window.
    pub multisampling: bool,
}

impl std::default::Default for DrawParameters {
    fn default() -> DrawParameters {
        DrawParameters {
            depth_function: DepthFunction::Overwrite,
            blending_function: Some(BlendingFunction::AlwaysReplace),
            line_width: None,
            backface_culling: BackfaceCullingMode::CullingDisabled,
            polygon_mode: PolygonMode::Fill,
            multisampling: true,
        }
    }
}

impl DrawParameters {
    /// Synchronizes the parmaeters with the current ctxt.state.
    fn sync(&self, ctxt: &mut context::CommandContext) {
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
    }
}

/// Area of a surface in pixels.
///
/// In the OpenGL ecosystem, the (0,0) coordinate is at the bottom-left hand corner of the images.
#[deriving(Show, Clone, Copy, Default)]
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
pub trait Surface {
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
    ///
    fn draw<V, I, ID, U>(&mut self, &VertexBuffer<V>, &I, program: &Program, uniforms: &U,
        draw_parameters: &DrawParameters) where I: index_buffer::ToIndicesSource<ID>, U: uniforms::Uniforms;

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
        fbo::blit(self, target, gl::COLOR_BUFFER_BIT, source_rect, target_rect,
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
    display: Arc<DisplayImpl>,
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
        fbo::clear_color(&self.display, None, red, green, blue, alpha)
    }

    fn clear_depth(&mut self, value: f32) {
        fbo::clear_depth(&self.display, None, value)
    }

    fn clear_stencil(&mut self, value: int) {
        fbo::clear_stencil(&self.display, None, value)
    }

    fn get_dimensions(&self) -> (uint, uint) {
        self.dimensions
    }

    fn get_depth_buffer_bits(&self) -> Option<u16> {
        self.display.context.capabilities().depth_bits
    }

    fn get_stencil_buffer_bits(&self) -> Option<u16> {
        self.display.context.capabilities().stencil_bits
    }

    fn draw<V, I, ID, U>(&mut self, vertex_buffer: &VertexBuffer<V>,
                         index_buffer: &I, program: &Program, uniforms: &U,
                         draw_parameters: &DrawParameters)
                         where I: index_buffer::ToIndicesSource<ID>, U: uniforms::Uniforms,
                         ID: index_buffer::Index
    {
        use index_buffer::ToIndicesSource;

        if draw_parameters.depth_function.requires_depth_buffer() && !self.has_depth_buffer() {
            panic!("Requested a depth function but no depth buffer is attached");
        }

        fbo::draw(&self.display, None, vertex_buffer, &index_buffer.to_indices_source(),
                  program, uniforms, draw_parameters)
    }

    fn get_blit_helper(&self) -> BlitHelper {
        BlitHelper(&self.display, None)
    }
}

#[unsafe_destructor]
impl<'t> Drop for Frame<'t> {
    fn drop(&mut self) {
        self.display.context.swap_buffers();
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
#[deriving(Clone, Show, PartialEq, Eq)]
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
#[deriving(Clone)]
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
            display: self.context.clone(),
            marker: std::kinds::marker::ContravariantLifetime,
            dimensions: self.get_framebuffer_dimensions(),
        }
    }

    /// Returns the maximum value that can be used for anisotropic filtering, or `None`
    /// if the hardware doesn't support it.
    pub fn get_max_anisotropy_support(&self) -> Option<u16> {
        self.context.context.capabilities().max_texture_max_anisotropy.map(|v| v as u16)
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
                let mut value: [gl::types::GLint, ..4] = mem::uninitialized();

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

                tx.send(value);
            }
        });

        rx.recv().map(|v| v as uint * 1024)
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
            let mut cb = self.context.debug_callback.lock();
            *cb = Some(box callback as Box<FnMut(String, debug::Source, debug::MessageType,
                                                 debug::Severity)
                                           + Send + Sync>);
        }

        // this is the C callback
        extern "system" fn callback_wrapper(source: gl::types::GLenum, ty: gl::types::GLenum,
            id: gl::types::GLuint, severity: gl::types::GLenum, _length: gl::types::GLsizei,
            message: *const gl::types::GLchar, user_param: *mut libc::c_void)
        {
            use std::c_str::CString;
            use std::num::FromPrimitive;

            unsafe {
                let user_param = user_param as *mut DisplayImpl;
                let user_param = user_param.as_mut().unwrap();

                let message = CString::new(message, false);
                let message = message.as_str().unwrap_or("<message was not utf-8>");

                let ref mut callback = user_param.debug_callback;
                let mut callback = callback.lock();
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
        let ptr = ptr as *const DisplayImpl;

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
                    ctxt.gl.DebugMessageCallback(callback_wrapper, ptr as *const libc::c_void);
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
        where P: texture::PixelValue + Clone + Send, T: texture::Texture2dData<P>
    {
        let dimensions = self.get_framebuffer_dimensions();
        let pixels_count = dimensions.0 * dimensions.1;

        let (format, gltype) = texture::PixelValue::get_format(None::<P>).to_gl_enum();

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
                let mut data: Vec<P> = Vec::with_capacity(pixels_count);
                ctxt.gl.ReadPixels(0, 0, dimensions.0 as gl::types::GLint,
                    dimensions.1 as gl::types::GLint, format, gltype,
                    data.as_mut_ptr() as *mut libc::c_void);
                data.set_len(pixels_count);
                tx.send(data);
            }
        });

        let data = rx.recv();
        texture::Texture2dData::from_vec(data, dimensions.0 as u32)
    }

    /// Asserts that there are no OpenGL error pending.
    ///
    /// This function is supposed to be used in tests.
    pub fn assert_no_error(&self) {
        let (tx, rx) = channel();

        self.context.context.exec(move |: ctxt| {
            tx.send(get_gl_error(ctxt));
        });

        match rx.recv() {
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
            tx.send(());
        });

        rx.recv();
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
            let mut fbos = self.framebuffer_objects.lock();
            fbos.clear();
        }

        {
            let mut vaos = self.vertex_array_objects.lock();
            vaos.clear();
        }

        {
            let mut samplers = self.samplers.lock();
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
