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

# Drawing

Drawing something requires three elements:

 - A vertex buffer, which contains the vertices of the shape that you wish to draw.
 - An index buffer, which contains the shapes which connect the vertices.
 - A program that the GPU will execute.

## Vertex buffer

To create a vertex buffer, you must create a struct and add the `#[vertex_format]` attribute to
it. Then simply call `VertexBuffer::new` with a `Vec` of this type.

```no_run
# #![feature(phase)]
# #[phase(plugin)]
# extern crate glium_macros;
# extern crate glium;
# fn main() {
#[vertex_format]
#[allow(non_snake_case)]
struct Vertex {
    iPosition: [f32, ..2],
    iTexCoords: [f32, ..2],
}

# let display: glium::Display = unsafe { std::mem::uninitialized() };
let vertex_buffer = glium::VertexBuffer::new(&display, vec![
    Vertex { iPosition: [-1.0, -1.0], iTexCoords: [0.0, 1.0] },
    Vertex { iPosition: [-1.0,  1.0], iTexCoords: [0.0, 0.0] },
    Vertex { iPosition: [ 1.0,  1.0], iTexCoords: [1.0, 0.0] },
    Vertex { iPosition: [ 1.0, -1.0], iTexCoords: [1.0, 1.0] }
]);
# }
```

## Index buffer

Creating an index buffer is done by calling `build_index_buffer` with an array containing
the indices from the vertex buffer.

```no_run
# let display: glium::Display = unsafe { std::mem::uninitialized() };
let index_buffer = glium::IndexBuffer::new(&display, glium::TrianglesList,
    &[0u8, 1, 2, 0, 2, 3]);
```

## Program

```no_run
static VERTEX_SRC: &'static str = "
    #version 110

    uniform mat4 uMatrix;

    attribute vec2 iPosition;
    attribute vec2 iTexCoords;

    varying vec2 vTexCoords;

    void main() {
        gl_Position = vec4(iPosition, 0.0, 1.0) * uMatrix;
        vTexCoords = iTexCoords;
    }
";

static FRAGMENT_SRC: &'static str = "
    #version 110
    varying vec2 vTexCoords;

    void main() {
        gl_FragColor = vec4(vTexCoords.x, vTexCoords.y, 0.0, 1.0);
    }
";

# let display: glium::Display = unsafe { std::mem::uninitialized() };
let program = glium::Program::new(&display, VERTEX_SRC, FRAGMENT_SRC, None).unwrap();
```

The `attribute`s or `in` variables in the vertex shader must match the names of the elements
of the `#[vertex_format]` structure.

The `Result` returned by `Program::new` will report any compilation or linking error.

## Uniforms

The last step is to build the list of uniforms for the program.

```no_run
# #![feature(phase)]
# #[phase(plugin)]
# extern crate glium_macros;
# extern crate glium;
# fn main() {
#[uniforms]
#[allow(non_snake_case)]
struct Uniforms<'a> {
    uTexture: &'a glium::Texture,
    uMatrix: [[f32, ..4], ..4],
}

# let display: glium::Display = unsafe { std::mem::uninitialized() };
# let tex = unsafe { std::mem::uninitialized() };
# let matrix = unsafe { std::mem::uninitialized() };
let uniforms = Uniforms {
    uTexture: tex,
    uMatrix: matrix,
};
# }
```

## Drawing

Draw by calling `display.draw()`. This function call will return a `Target` object which can
be used to draw things.

Buffers are automatically swapped when the `Target` is destroyed.

Once you are done drawing, you can `target.finish()` or let it go out of the scope.

```no_run
# let display: glium::Display = unsafe { std::mem::uninitialized() };
# let vertex_buffer: glium::VertexBuffer<u8> = unsafe { std::mem::uninitialized() };
# let index_buffer: glium::IndexBuffer = unsafe { std::mem::uninitialized() };
# let program: glium::Program = unsafe { std::mem::uninitialized() };
# let uniforms = glium::uniforms::EmptyUniforms;
let mut target = display.draw();
target.clear_color(0.0, 0.0, 0.0, 0.0);
target.draw(glium::BasicDraw(&vertex_buffer, &index_buffer, &program, &uniforms, &std::default::Default::default()));
target.finish();
```

*/

#![feature(if_let)]
#![feature(phase)]
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
extern crate nalgebra;
extern crate native;
extern crate time;

#[doc(hidden)]
pub use data_types::GLDataTuple;

pub use index_buffer::IndexBuffer;
pub use vertex_buffer::{VertexBuffer, VertexBindings, VertexFormat};
pub use program::{Program, ProgramCreationError};
pub use texture::Texture;

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

pub mod uniforms;
/// Contains everything related to vertex buffers.
pub mod vertex_buffer;

mod context;
mod data_types;
mod index_buffer;
mod program;
mod texture;

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
mod gl {
    generate_gl_bindings!("gl", "core", "4.5", "struct", [
        "GL_EXT_direct_state_access",
        "GL_EXT_framebuffer_object"
    ])
}

#[cfg(target_os = "android")]
mod gl {
    pub use self::Gles2 as Gl;
    generate_gl_bindings!("gles2", "core", "2.0", "struct")
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
}

impl std::default::Default for DrawParameters {
    fn default() -> DrawParameters {
        DrawParameters {
            depth_function: None,
            blending_function: Some(AlwaysReplace),
            line_width: None,
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
    }
}

/// A target where things can be drawn.
pub struct Target<'t> {
    display: Arc<DisplayImpl>,
    display_hold: Option<&'t Display>,
    texture: Option<&'t mut Texture>,
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
                for (name, &(data_type, data_size, data_offset)) in vb_bindingsclone.iter() {
                    let loc = gl.GetAttribLocation(program_id, name.to_c_str().unwrap());
                    locations.push(loc);

                    if loc != -1 {
                        match data_type {
                            gl::BYTE | gl::UNSIGNED_BYTE | gl::SHORT | gl::UNSIGNED_SHORT | gl::INT | gl::UNSIGNED_INT
                                => fail!("Not supported"), // TODO: gl.VertexAttribIPointer(loc as u32, data_size, data_type, vb_elementssize as i32, data_offset as *const libc::c_void),
                            _ => gl.VertexAttribPointer(loc as u32, data_size, data_type, 0, vb_elementssize as i32, data_offset as *const libc::c_void)
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
    fn build_glium(self) -> Result<Display, ()>;
}

impl DisplayBuild for glutin::WindowBuilder {
    fn build_glium(self) -> Result<Display, ()> {
        let window = try!(self.build().map_err(|_| ()));
        let context = context::Context::new(window);

        let gl_version = {
            let (tx, rx) = channel();
            context.exec(proc(gl, _state) {
                // TODO: not supported by GLES
                tx.send((0, 0));
                /*unsafe {
                    use std::mem;

                    let mut major_version: gl::types::GLint = mem::uninitialized();
                    let mut minor_version: gl::types::GLint = mem::uninitialized();

                    gl.GetIntegerv(gl::MAJOR_VERSION, &mut major_version);
                    gl.GetIntegerv(gl::MINOR_VERSION, &mut minor_version);

                    (major_version, minor_version)
                }*/
            });
            rx.recv()
        };

        Ok(Display {
            context: Arc::new(DisplayImpl {
                context: context,
                gl_version: gl_version,
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
