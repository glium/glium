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

# Fundamentals

In order to draw something on the window, you must first call `display.draw()`. This function
call returns a `Target` object which you can manipulate to draw things. Once the object
is destroyed, the result will be presented to the user by swapping the front and back buffers.

You can easily fill the window with one color by calling `clear_colors` on the target. Drawing
something more complex, however, requires two elements:

 - A mesh, which describes the shape and the characteristics of the object that you want to draw,
 and is composed of a `VertexBuffer` object and an `IndexBuffer` object.
 - A program that is going to be executed by the GPU and is the result of compiling and linking
 GLSL code, alongside with the values of its global variables which are called *uniforms*.

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
let index_buffer = glium::IndexBuffer::new(&display, glium::TrianglesList, &[ 0u16, 1, 2 ]);
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
in order to obtain a `Target` object. Note that it is also possible to draw on a texture by
calling `texture.draw()`, but this is not covered here.

The `Target` object has a `draw` function which you can use to draw things.
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

#![feature(if_let)]
#![feature(phase)]
#![feature(slicing_syntax)]
#![feature(tuple_indexing)]
#![feature(unsafe_destructor)]
#![unstable]
#![deny(missing_docs)]

#[phase(plugin)]
extern crate compile_msg;

#[phase(plugin)]
extern crate gl_generator;

extern crate cgmath;
extern crate glutin;
#[cfg(feature = "image")]
extern crate image;
extern crate libc;
//extern crate nalgebra;
extern crate native;
extern crate time;

#[doc(hidden)]
pub use data_types::GLDataTuple;

pub use index_buffer::IndexBuffer;
pub use framebuffer::FrameBuffer;
pub use vertex_buffer::{VertexBuffer, VertexBindings, VertexFormat};
pub use program::{Program, ProgramCreationError};
pub use program::{CompilationError, LinkingError, ProgramCreationFailure, ShaderTypeNotSupported};
pub use texture::{Texture, Texture2d};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub mod uniforms;
/// Contains everything related to vertex buffers.
pub mod vertex_buffer;
pub mod texture;

mod buffer;
mod context;
mod data_types;
mod framebuffer;
mod index_buffer;
mod program;

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
mod gl {
    generate_gl_bindings! {
        api: "gl",
        profile: "core",
        version: "4.5",
        generator: "struct",
        extensions: [
            "GL_EXT_direct_state_access",
            "GL_EXT_framebuffer_object",
            "GL_EXT_framebuffer_blit",
        ]
    }
}

#[cfg(target_os = "android")]
mod gl {
    pub use self::Gles2 as Gl;
    generate_gl_bindings! {
        api: "gles2",
        profile: "core",
        version: "2.0",
        generator: "struct",
        extensions: []
    }
}

#[cfg(all(
    not(target_os = "windows"),
    not(target_os = "linux"),
    not(target_os = "macos"),
    not(target_os = "android")
))]
compile_error!("This platform is not supported")

/// A command that asks to draw something.
///
/// You can implement this for your own type by redirecting the call to another command.
pub trait DrawCommand {
    /// Draws the object on the specified target.
    fn draw(self, &mut Target);
}

/// Types of primitives.
#[allow(missing_docs)]
#[experimental = "Will be replaced soon"]
pub enum PrimitiveType {
    PointsList,
    LinesList,
    LinesListAdjacency,
    LineStrip,
    LineStripAdjacency,
    TrianglesList,
    TrianglesListAdjacency,
    TriangleStrip,
    TriangleStripAdjacency,
    TriangleFan
}

impl PrimitiveType {
    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
    fn get_gl_enum(&self) -> gl::types::GLenum {
        match *self {
            PointsList => gl::POINTS,
            LinesList => gl::LINES,
            LinesListAdjacency => gl::LINES_ADJACENCY,
            LineStrip => gl::LINE_STRIP,
            LineStripAdjacency => gl::LINE_STRIP_ADJACENCY,
            TrianglesList => gl::TRIANGLES,
            TrianglesListAdjacency => gl::TRIANGLES_ADJACENCY,
            TriangleStrip => gl::TRIANGLE_STRIP,
            TriangleStripAdjacency => gl::TRIANGLE_STRIP_ADJACENCY,
            TriangleFan => gl::TRIANGLE_FAN
        }
    }

    #[cfg(target_os = "android")]
    fn get_gl_enum(&self) -> gl::types::GLenum {
        match *self {
            PointsList => gl::POINTS,
            LinesList => gl::LINES,
            LineStrip => gl::LINE_STRIP,
            TrianglesList => gl::TRIANGLES,
            TriangleStrip => gl::TRIANGLE_STRIP,
            TriangleFan => gl::TRIANGLE_FAN,
            _ => panic!("Not supported by GLES")
        }
    }
}

/// Function that the GPU will use for blending.
#[deriving(Clone, Show, PartialEq, Eq)]
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
    /// Means `(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA)` in OpenGL.
    LerpBySourceAlpha,
}

/// Culling mode.
/// 
/// Describes how triangles could be filtered before the fragment part.
#[deriving(Clone, Show, PartialEq, Eq)]
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
#[deriving(Clone, Show, PartialEq, Eq)]
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
    /// Turns the `DepthFunction` into the corresponding GLenum.
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            Ignore => gl::NEVER,
            Overwrite => gl::ALWAYS,
            IfEqual => gl::EQUAL,
            IfNotEqual => gl::NOTEQUAL,
            IfMore => gl::GREATER,
            IfMoreOrEqual => gl::GEQUAL,
            IfLess => gl::LESS,
            IfLessOrEqual => gl::LEQUAL,
        }
    }
}

/// Defines how the device should render polygons.
///
/// The usual value is `Fill`, which fills the content of polygon with the color. However other
/// values are sometimes useful, especially for debugging purposes.
#[deriving(Clone, Show, PartialEq, Eq)]
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

impl PolygonMode {
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            Point => gl::POINT,
            Line => gl::LINE,
            Fill => gl::FILL,
        }
    }
}

/// Represents the parameters to use when drawing.
///
/// Example:
/// 
/// ```
/// let params = glium::DrawParameters {
///     depth_function: Some(glium::IfLess),
///     .. std::default::Default::default()
/// };
/// ```
///
#[deriving(Clone, Show, PartialEq)]
pub struct DrawParameters {
    /// The function that the GPU will use to determine whether to write over an existing pixel
    ///  on the target. 
    /// `None` means "don't care".
    pub depth_function: Option<DepthFunction>,

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
}

impl std::default::Default for DrawParameters {
    fn default() -> DrawParameters {
        DrawParameters {
            depth_function: None,
            blending_function: Some(AlwaysReplace),
            line_width: None,
            backface_culling: CullingDisabled,
            polygon_mode: Fill,
        }
    }
}

impl DrawParameters {
    /// Synchronizes the parmaeters with the current state.
    fn sync(&self, gl: &gl::Gl, state: &mut context::GLState) {
        // depth function
        match self.depth_function {
            Some(Overwrite) => unsafe {
                if state.enabled_depth_test {
                    gl.Disable(gl::DEPTH_TEST);
                    state.enabled_depth_test = false;
                }
            },
            Some(depth_function) => unsafe {
                let depth_function = depth_function.to_glenum();
                if state.depth_func != depth_function {
                    gl.DepthFunc(depth_function);
                    state.depth_func = depth_function;
                }
                if !state.enabled_depth_test {
                    gl.Enable(gl::DEPTH_TEST);
                    state.enabled_depth_test = true;
                }
            },
            _ => ()
        }

        // blending function
        match self.blending_function {
            Some(AlwaysReplace) => unsafe {
                if state.enabled_blend {
                    gl.Disable(gl::BLEND);
                    state.enabled_blend = false;
                }
            },
            Some(LerpBySourceAlpha) => unsafe {
                if state.blend_func != (gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA) {
                    gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                    state.blend_func = (gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                }
                if !state.enabled_blend {
                    gl.Enable(gl::BLEND);
                    state.enabled_blend = true;
                }
            },
            _ => ()
        }

        // line width
        if let Some(line_width) = self.line_width {
            if state.line_width != line_width {
                unsafe {
                    gl.LineWidth(line_width);
                    state.line_width = line_width;
                }
            }
        }

        // back-face culling
        // note: we never change the value of `glFrontFace`, whose default is GL_CCW
        //  that's why `CullClockWise` uses `GL_BACK` for example
        match self.backface_culling {
            CullingDisabled => unsafe {
                if state.enabled_cull_face {
                    gl.Disable(gl::CULL_FACE);
                    state.enabled_cull_face = false;
                }
            },
            CullCounterClockWise => unsafe {
                if !state.enabled_cull_face {
                    gl.Enable(gl::CULL_FACE);
                    state.enabled_cull_face = true;
                }
                if state.cull_face != gl::FRONT {
                    gl.CullFace(gl::FRONT);
                    state.cull_face = gl::FRONT;
                }
            },
            CullClockWise => unsafe {
                if !state.enabled_cull_face {
                    gl.Enable(gl::CULL_FACE);
                    state.enabled_cull_face = true;
                }
                if state.cull_face != gl::BACK {
                    gl.CullFace(gl::BACK);
                    state.cull_face = gl::BACK;
                }
            },
        }

        // polygon mode
        unsafe {
            let polygon_mode = self.polygon_mode.to_glenum();
            if state.polygon_mode != polygon_mode {
                gl.PolygonMode(gl::FRONT_AND_BACK, polygon_mode);
                state.polygon_mode = polygon_mode;
            }
        }
    }
}

/// Area of a surface in pixels.
///
/// In the OpenGL ecosystem, the (0,0) coordinate is at the bottom-left hand corner of the images.
#[deriving(Show, Clone, Default)]
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

    /// Draws.
    fn draw<V, U>(&mut self, &VertexBuffer<V>, &IndexBuffer, program: &Program, uniforms: &U,
        draw_parameters: &DrawParameters) where U: uniforms::Uniforms;

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
        filter: uniforms::SamplerFilter) where S: Surface
    {
        framebuffer::blit(self, target, gl::COLOR_BUFFER_BIT, source_rect, target_rect,
            filter.to_glenum())
    }

    /// Copies the entire surface to a target surface. See `blit_color`.
    #[experimental = "The name will likely change"]
    fn blit_whole_color_to<S>(&self, target: &S, target_rect: &Rect,
        filter: uniforms::SamplerFilter) where S: Surface
    {
        let src_dim = self.get_dimensions();
        let src_rect = Rect { left: 0, bottom: 0, width: src_dim.0 as u32, height: src_dim.1 as u32 };
        self.blit_color(&src_rect, target, target_rect, filter)
    }

    /// Copies the entire surface to the entire target. See `blit_color`.
    #[experimental = "The name will likely change"]
    fn fill<S>(&self, target: &S, filter: uniforms::SamplerFilter) where S: Surface {
        let src_dim = self.get_dimensions();
        let src_rect = Rect { left: 0, bottom: 0, width: src_dim.0 as u32, height: src_dim.1 as u32 };
        let target_dim = target.get_dimensions();
        let target_rect = Rect { left: 0, bottom: 0, width: target_dim.0 as u32, height: target_dim.1 as u32 };
        self.blit_color(&src_rect, target, &target_rect, filter)
    }
}

#[doc(hidden)]
pub struct BlitHelper<'a>(&'a Arc<DisplayImpl>, Option<&'a framebuffer::FramebufferAttachments>);

/// Implementation of `Surface` targetting the default framebuffer.
///
/// The back- and front-buffers are swapped when the `Target` is destroyed. This operation is
/// instantaneous, even when vsync is enabled.
pub struct Target<'a> {
    display: Arc<DisplayImpl>,
    marker: std::kinds::marker::ContravariantLifetime<'a>,
    dimensions: (uint, uint),
}

impl<'t> Target<'t> {
    /// Stop drawing and swap the buffers.
    pub fn finish(self) {
    }
}

impl<'t> Surface for Target<'t> {
    fn clear_color(&mut self, red: f32, green: f32, blue: f32, alpha: f32) {
        framebuffer::clear_color(&self.display, None, red, green, blue, alpha)
    }

    fn clear_depth(&mut self, value: f32) {
        framebuffer::clear_depth(&self.display, None, value)
    }

    fn clear_stencil(&mut self, value: int) {
        framebuffer::clear_stencil(&self.display, None, value)
    }

    fn get_dimensions(&self) -> (uint, uint) {
        self.dimensions
    }

    fn draw<V, U: uniforms::Uniforms>(&mut self, vertex_buffer: &VertexBuffer<V>,
        index_buffer: &IndexBuffer, program: &Program, uniforms: &U,
        draw_parameters: &DrawParameters)
    {
        framebuffer::draw(&self.display, None, vertex_buffer, index_buffer, program, uniforms,
            draw_parameters)
    }

    fn get_blit_helper(&self) -> BlitHelper {
        BlitHelper(&self.display, None)
    }
}

#[unsafe_destructor]
impl<'t> Drop for Target<'t> {
    fn drop(&mut self) {
        self.display.context.swap_buffers();
    }
}

/// Objects that can build a `Display` object.
pub trait DisplayBuild {
    /// Build a context and a `Display` to draw on it.
    fn build_glium(self) -> Result<Display, GliumCreationError>;
}

/// Error that can happen while creating a glium display.
#[deriving(Clone, Show, PartialEq, Eq)]
pub enum GliumCreationError {
    /// An error has happened while creating the glutin window or headless renderer.
    GlutinCreationError(glutin::CreationError),
}

impl std::error::Error for GliumCreationError {
    fn description(&self) -> &str {
        match self {
            &GlutinCreationError(_) => "Error while creating glutin window or headless renderer",
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match self {
            &GlutinCreationError(ref err) => Some(err as &std::error::Error),
        }
    }
}

impl std::error::FromError<glutin::CreationError> for GliumCreationError {
    fn from_error(err: glutin::CreationError) -> GliumCreationError {
        GlutinCreationError(err)
    }
}

impl DisplayBuild for glutin::WindowBuilder {
    fn build_glium(self) -> Result<Display, GliumCreationError> {
        let window = try!(self.build());
        let context = context::Context::new_from_window(window);

        Ok(Display {
            context: Arc::new(DisplayImpl {
                context: context,
                framebuffer_objects: Mutex::new(HashMap::new()),
            }),
        })
    }
}

impl DisplayBuild for glutin::HeadlessRendererBuilder {
    fn build_glium(self) -> Result<Display, GliumCreationError> {
        let window = try!(self.build());
        let context = context::Context::new_from_headless(window);

        Ok(Display {
            context: Arc::new(DisplayImpl {
                context: context,
                framebuffer_objects: Mutex::new(HashMap::new()),
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

    // we maintain a list of FBOs
    // when something requirering a FBO is drawn, we look for an existing one in this hashmap
    framebuffer_objects: Mutex<HashMap<framebuffer::FramebufferAttachments,
                                       framebuffer::FrameBufferObject>>,
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
    /// This function returns a `Target` which can be used to draw on it. When the `Target` is
    /// destroyed, the buffers are swapped.
    ///
    /// Note that destroying a `Target` is immediate, even if vsync is enabled.
    pub fn draw(&self) -> Target {
        Target {
            display: self.context.clone(),
            marker: std::kinds::marker::ContravariantLifetime,
            dimensions: self.get_framebuffer_dimensions(),
        }
    }

    /// Releases the shader compiler, indicating that no new programs will be created for a while.
    pub fn release_shader_compiler(&self) {
        self.context.context.exec(proc(gl, _, version, _) {
            unsafe {
                if version >= &context::GlVersion(4, 1) {
                    gl.ReleaseShaderCompiler();
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
    /// ```
    /// # extern crate glium;
    /// # extern crate glutin;
    /// # use glium::DisplayBuild;
    /// # fn main() {
    /// # let display = glutin::HeadlessRendererBuilder::new(1024, 768).build_glium().unwrap();
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
        self.context.context.exec(proc(gl, state, version, _) {
            unsafe {
                // unbinding framebuffers
                if state.read_framebuffer.is_some() {
                    if version >= &context::GlVersion(3, 0) {
                        gl.BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
                        state.read_framebuffer = None;
                    } else {
                        gl.BindFramebufferEXT(gl::FRAMEBUFFER_EXT, 0);
                        state.draw_framebuffer = None;
                        state.read_framebuffer = None;
                    }
                }

                // reading
                let mut data: Vec<P> = Vec::with_capacity(pixels_count);
                gl.ReadPixels(0, 0, dimensions.0 as gl::types::GLint,
                    dimensions.1 as gl::types::GLint, format, gltype,
                    data.as_mut_ptr() as *mut libc::c_void);
                data.set_len(pixels_count);
                tx.send(data);
            }
        });

        let data = rx.recv();
        texture::Texture2dData::from_vec(data, dimensions.0 as u32)
    }
}

// this destructor is here because framebuffers contain an `Arc<DisplayImpl>`, which would lead
// to a leak
impl Drop for DisplayImpl {
    fn drop(&mut self) {
        let mut fbos = self.framebuffer_objects.lock();
        fbos.clear();
    }
}

#[allow(dead_code)]
fn get_gl_error(gl: &gl::Gl) -> &'static str {
    match unsafe { gl.GetError() } {
        gl::NO_ERROR => "GL_NO_ERROR",
        gl::INVALID_ENUM => "GL_INVALID_ENUM",
        gl::INVALID_VALUE => "GL_INVALID_VALUE",
        gl::INVALID_OPERATION => "GL_INVALID_OPERATION",
        gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
        gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
        /*gl::STACK_UNDERFLOW => "GL_STACK_UNDERFLOW",
        gl::STACK_OVERFLOW => "GL_STACK_OVERFLOW",*/
        _ => "Unknown glGetError return value"
    }
}
