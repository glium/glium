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

The `Target` object has a `draw` function which takes an object that implements the
`glium::DrawCommand` trait. The only command provided by glium is `BasicDraw`.

The arguments are the vertex buffer, index buffer, program, uniforms, and an object of type
`DrawParameters` which contains miscellaneous informations about how everything should be rendered
(depth test, blending, backface culling, etc.).

```no_run
# let display: glium::Display = unsafe { std::mem::uninitialized() };
# let vertex_buffer: glium::VertexBuffer<u8> = unsafe { std::mem::uninitialized() };
# let index_buffer: glium::IndexBuffer = unsafe { std::mem::uninitialized() };
# let program: glium::Program = unsafe { std::mem::uninitialized() };
# let uniforms = glium::uniforms::EmptyUniforms;
let mut target = display.draw();
target.clear_color(0.0, 0.0, 0.0, 0.0);  // filling the output with the black color
target.draw(glium::BasicDraw(&vertex_buffer, &index_buffer, &program, &uniforms, &std::default::Default::default()));
target.finish();
```

*/

#![feature(if_let)]
#![feature(phase)]
#![feature(slicing_syntax)]
#![feature(tuple_indexing)]
#![feature(unsafe_destructor)]
#![unstable]
#![deny(missing_doc)]

#[phase(plugin)]
extern crate compile_msg;

#[phase(plugin)]
extern crate gl_generator;

extern crate cgmath;
extern crate glutin;
extern crate libc;
//extern crate nalgebra;
extern crate native;
extern crate time;

#[doc(hidden)]
pub use data_types::GLDataTuple;

pub use index_buffer::IndexBuffer;
pub use vertex_buffer::{VertexBuffer, VertexBindings, VertexFormat};
pub use program::{Program, ProgramCreationError};
pub use texture::{Texture, Texture2D};

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

pub mod uniforms;
/// Contains everything related to vertex buffers.
pub mod vertex_buffer;
pub mod texture;

mod context;
mod data_types;
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
            "GL_EXT_framebuffer_object"
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

#[cfg(all(not(target_os = "windows"), not(target_os = "linux"), not(target_os = "macos"), not(target_os = "android")))]
compile_error!("This platform is not supported")

/// A command that asks to draw something.
///
/// You can implement this for your own type by redirecting the call to another command.
pub trait DrawCommand {
    /// Draws the object on the specified target.
    fn draw(self, &mut Target);
}

/// Types of primitives.
#[allow(missing_doc)]
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
            _ => fail!("Not supported by GLES")
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
}

impl std::default::Default for DrawParameters {
    fn default() -> DrawParameters {
        DrawParameters {
            depth_function: None,
            blending_function: Some(AlwaysReplace),
            line_width: None,
            backface_culling: CullingDisabled,
        }
    }
}

impl DrawParameters {
    /// Synchronizes the parmaeters with the current state.
    fn sync(&self, gl: &gl::Gl, state: &mut context::GLState) {
        // depth function
        match self.depth_function {
            Some(Overwrite) => {
                if state.enabled_depth_test {
                    gl.Disable(gl::DEPTH_TEST);
                    state.enabled_depth_test = false;
                }
            },
            Some(depth_function) => {
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
            Some(AlwaysReplace) => {
                if state.enabled_blend {
                    gl.Disable(gl::BLEND);
                    state.enabled_blend = false;
                }
            },
            Some(LerpBySourceAlpha) => {
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
                gl.LineWidth(line_width);
                state.line_width = line_width;
            }
        }

        // back-face culling
        // note: we never change the value of `glFrontFace`, whose default is GL_CCW
        //  that's why `CullClockWise` uses `GL_BACK` for example
        match self.backface_culling {
            CullingDisabled => {
                if state.enabled_cull_face {
                    gl.Disable(gl::CULL_FACE);
                    state.enabled_cull_face = false;
                }
            },
            CullCounterClockWise => {
                if !state.enabled_cull_face {
                    gl.Enable(gl::CULL_FACE);
                    state.enabled_cull_face = true;
                }
                if state.cull_face != gl::FRONT {
                    gl.CullFace(gl::FRONT);
                    state.cull_face = gl::FRONT;
                }
            },
            CullClockWise => {
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
    }
}

/// A target where things can be drawn.
pub struct Target<'t> {
    display: Arc<DisplayImpl>,
    display_hold: Option<&'t Display>,
    texture: Option<&'t mut texture::TextureImplementation>,
    framebuffer: Option<FrameBufferObject>,
    execute_end: Option<proc(&DisplayImpl):Send>,
    dimensions: (uint, uint),
}

impl<'t> Target<'t> {
    /// Clears the color components of the target.
    pub fn clear_color(&mut self, red: f32, green: f32, blue: f32, alpha: f32) {
        let (red, green, blue, alpha) = (
            red as gl::types::GLclampf,
            green as gl::types::GLclampf,
            blue as gl::types::GLclampf,
            alpha as gl::types::GLclampf
        );

        self.display.context.exec(proc(gl, state) {
            if state.clear_color != (red, green, blue, alpha) {
                gl.ClearColor(red, green, blue, alpha);
                state.clear_color = (red, green, blue, alpha);
            }

            gl.Clear(gl::COLOR_BUFFER_BIT);
        });
    }

    /// Clears the depth component of the target.
    pub fn clear_depth(&mut self, value: f32) {
        let value = value as gl::types::GLclampf;

        self.display.context.exec(proc(gl, state) {
            if state.clear_depth != value {
                gl.ClearDepth(value as f64);        // TODO: find out why this needs "as"
                state.clear_depth = value;
            }

            gl.Clear(gl::DEPTH_BUFFER_BIT);
        });
    }

    /// Clears the stencil component of the target.
    pub fn clear_stencil(&mut self, value: int) {
        let value = value as gl::types::GLint;

        self.display.context.exec(proc(gl, state) {
            if state.clear_stencil != value {
                gl.ClearStencil(value);
                state.clear_stencil = value;
            }

            gl.Clear(gl::STENCIL_BUFFER_BIT);
        });
    }

    /// Returns the dimensions in pixels of the target.
    pub fn get_dimensions(&self) -> (uint, uint) {
        self.dimensions
    }

    /// Stop drawing on the target.
    pub fn finish(self) {
    }

    /// Draws.
    pub fn draw<D: DrawCommand>(&mut self, object: D) {
        object.draw(self);
    }
}

/// Basic draw command.
pub struct BasicDraw<'a, 'b, 'c, 'd, 'e, V, U: 'd>(pub &'a VertexBuffer<V>, pub &'b IndexBuffer,
    pub &'c Program, pub &'d U, pub &'e DrawParameters);

impl<'a, 'b, 'c, 'd, 'e, V, U: uniforms::Uniforms>
    DrawCommand for BasicDraw<'a, 'b, 'c, 'd, 'e, V, U>
{
    fn draw(self, target: &mut Target) {
        let BasicDraw(vertex_buffer, index_buffer, program, uniforms, draw_parameters) = self;

        let fbo_id = target.framebuffer.as_ref().map(|f| f.id);
        let (vb_id, vb_elementssize, vb_bindingsclone) = vertex_buffer::get_clone(vertex_buffer);
        let (ib_id, ib_elemcounts, ib_datatype, ib_primitives) = index_buffer::get_clone(index_buffer);
        let program_id = program::get_program_id(program);
        let uniforms = uniforms.to_binder();
        let uniforms_locations = program::get_uniforms_locations(program);
        let draw_parameters = draw_parameters.clone();

        let (tx, rx) = channel();

        target.display.context.exec(proc(gl, state) {
            unsafe {
                if gl.BindFramebuffer.is_loaded() {
                    gl.BindFramebuffer(gl::FRAMEBUFFER, fbo_id.unwrap_or(0));
                } else {
                    gl.BindFramebufferEXT(gl::FRAMEBUFFER_EXT, fbo_id.unwrap_or(0));
                }

                // binding program
                if state.program != program_id {
                    gl.UseProgram(program_id);
                    state.program = program_id;
                }

                // binding program uniforms
                uniforms.0(gl, |name| {
                    uniforms_locations
                        .find_equiv(&name)
                        .map(|val| val.0)
                });

                // binding vertex buffer
                if state.array_buffer_binding != Some(vb_id) {
                    gl.BindBuffer(gl::ARRAY_BUFFER, vb_id);
                    state.array_buffer_binding = Some(vb_id);
                }

                // binding index buffer
                if state.element_array_buffer_binding != Some(ib_id) {
                    gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ib_id);
                    state.element_array_buffer_binding = Some(ib_id);
                }

                // binding vertex buffer
                let mut locations = Vec::new();
                for &(ref name, vertex_buffer::VertexAttrib { offset, data_type, elements_count })
                    in vb_bindingsclone.iter()
                {
                    let loc = gl.GetAttribLocation(program_id, name.to_c_str().unwrap());
                    locations.push(loc);

                    if loc != -1 {
                        match data_type {
                            gl::BYTE | gl::UNSIGNED_BYTE | gl::SHORT | gl::UNSIGNED_SHORT | gl::INT | gl::UNSIGNED_INT
                                => fail!("Not supported"), // TODO: gl.VertexAttribIPointer(loc as u32, elements_count, data_type, vb_elementssize as i32, offset as *const libc::c_void),
                            _ => gl.VertexAttribPointer(loc as u32, elements_count as gl::types::GLint, data_type, 0, vb_elementssize as i32, offset as *const libc::c_void)
                        }
                        
                        gl.EnableVertexAttribArray(loc as u32);
                    }
                }

                // sync-ing parameters
                draw_parameters.sync(gl, state);
                
                // drawing
                gl.DrawElements(ib_primitives, ib_elemcounts as i32, ib_datatype, std::ptr::null());

                // disable vertex attrib array
                for l in locations.iter() {
                    gl.DisableVertexAttribArray(l.clone() as u32);
                }
            }

            tx.send(());
        });

        rx.recv();
    }
}

#[unsafe_destructor]
impl<'t> Drop for Target<'t> {
    fn drop(&mut self) {
        match self.execute_end.take() {
            Some(f) => f(&*self.display),
            None => ()
        }
    }
}

/// Frame buffer.
struct FrameBufferObject {
    display: Arc<DisplayImpl>,
    id: gl::types::GLuint,
}

impl FrameBufferObject {
    /// Builds a new FBO.
    fn new(display: Arc<DisplayImpl>) -> FrameBufferObject {
        let (tx, rx) = channel();

        display.context.exec(proc(gl, _state) {
            unsafe {
                let id: gl::types::GLuint = std::mem::uninitialized();
                gl.GenFramebuffers(1, std::mem::transmute(&id));
                tx.send(id);
            }
        });

        FrameBufferObject {
            display: display,
            id: rx.recv(),
        }
    }
}

impl Drop for FrameBufferObject {
    fn drop(&mut self) {
        let id = self.id.clone();
        self.display.context.exec(proc(gl, _state) {
            unsafe { gl.DeleteFramebuffers(1, [ id ].as_ptr()); }
        });
    }
}

/// Render buffer.
#[allow(dead_code)]     // TODO: remove
struct RenderBuffer {
    display: Arc<DisplayImpl>,
    id: gl::types::GLuint,
}

impl RenderBuffer {
    /// Builds a new render buffer.
    fn new(display: Arc<DisplayImpl>) -> RenderBuffer {
        let (tx, rx) = channel();

        display.context.exec(proc(gl, _state) {
            unsafe {
                let id: gl::types::GLuint = std::mem::uninitialized();
                gl.GenRenderbuffers(1, std::mem::transmute(&id));
                tx.send(id);
            }
        });

        RenderBuffer {
            display: display,
            id: rx.recv(),
        }
    }
}

impl Drop for RenderBuffer {
    fn drop(&mut self) {
        let id = self.id.clone();
        self.display.context.exec(proc(gl, _state) {
            unsafe { gl.DeleteRenderbuffers(1, [ id ].as_ptr()); }
        });
    }
}

/// Objects that can build a `Display` object.
pub trait DisplayBuild {
    /// Build a context and a `Display` to draw on it.
    fn build_glium(self) -> Result<Display, String>;
}

impl DisplayBuild for glutin::WindowBuilder {
    fn build_glium(self) -> Result<Display, String> {
        let window = try!(self.build());
        let context = context::Context::new_from_window(window);

        Ok(Display {
            context: Arc::new(DisplayImpl {
                context: context,
                gl_version: (0, 0),
            }),
        })
    }
}

impl DisplayBuild for glutin::HeadlessRendererBuilder {
    fn build_glium(self) -> Result<Display, String> {
        let window = try!(self.build());
        let context = context::Context::new_from_headless(window);

        Ok(Display {
            context: Arc::new(DisplayImpl {
                context: context,
                gl_version: (0, 0),
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
    context: context::Context,
    gl_version: (gl::types::GLint, gl::types::GLint),
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

    /// 
    pub fn draw(&self) -> Target {
        Target {
            display: self.context.clone(),
            display_hold: Some(self),
            dimensions: self.get_framebuffer_dimensions(),
            texture: None,
            framebuffer: None,
            execute_end: Some(proc(context: &DisplayImpl) {
                context.context.swap_buffers();
            }),
        }
    }

    /// Releases the shader compiler, indicating that no new programs will be created for a while.
    pub fn release_shader_compiler(&self) {
        self.context.context.exec(proc(gl, _) {
            if gl.ReleaseShaderCompiler.is_loaded() {
                gl.ReleaseShaderCompiler();
            }
        });
    }
}
