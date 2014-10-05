#![feature(phase)]
#![feature(tuple_indexing)]
#![feature(unsafe_destructor)]
#![unstable]
#![deny(missing_doc)]

/*!
Easy-to-use high-level OpenGL3+ wrapper.

# Initialization

This library defines the `DisplayBuild` trait which is curently implemented only on
`glutin::WindowBuilder`.

Initialization is done by creating a `WindowBuilder` and calling `build_glium_core`.

```no_run
extern crate glutin;
extern crate glium_core;

fn main() {
    use glium_core::DisplayBuild;

    let display = glutin::WindowBuilder::new()
        .with_dimensions(1024, 768)
        .with_title("Hello world".to_string())
        .build_glium_core().unwrap();
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
# extern crate glium_core_macros;
# extern crate glium_core;
# fn main() {
#[vertex_format]
#[allow(non_snake_case)]
struct Vertex {
    iPosition: [f32, ..2],
    iTexCoords: [f32, ..2],
}

# let display: glium_core::Display = unsafe { std::mem::uninitialized() };
let vertex_buffer = glium_core::VertexBuffer::new(&display, vec![
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
# let display: glium_core::Display = unsafe { std::mem::uninitialized() };
let index_buffer = glium_core::IndexBuffer::new(&display, glium_core::TrianglesList,
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

# let display: glium_core::Display = unsafe { std::mem::uninitialized() };
let program = glium_core::Program::new(&display, VERTEX_SRC, FRAGMENT_SRC, None).unwrap();
```

The `attribute`s or `in` variables in the vertex shader must match the names of the elements
of the `#[vertex_format]` structure.

The `Result` returned by `build_program` will report any compilation or linking error.

The last step is to call `build_uniforms` on the program. Doing so does not consume the program,
so you can call `build_uniforms` multiple times on the same program.

```no_run
# let program: glium_core::Program = unsafe { std::mem::uninitialized() };
let mut uniforms = program.build_uniforms();

uniforms.set_value("uMatrix", [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0f32]
]);
```

## Drawing

Draw by calling `display.draw()`. This function call will return a `Target` object which can
be used to draw things.

Buffers are automatically swapped when the `Target` is destroyed.

Once you are done drawing, you can `target.finish()` or let it go out of the scope.

```no_run
# let display: glium_core::Display = unsafe { std::mem::uninitialized() };
# let vertex_buffer: glium_core::VertexBuffer<u8> = unsafe { std::mem::uninitialized() };
# let index_buffer: glium_core::IndexBuffer = unsafe { std::mem::uninitialized() };
# let uniforms: glium_core::ProgramUniforms = unsafe { std::mem::uninitialized() };
let mut target = display.draw();
target.clear_color(0.0, 0.0, 0.0, 0.0);
target.draw(glium_core::BasicDraw(&vertex_buffer, &index_buffer, &uniforms));
target.finish();
```

*/

#[phase(plugin)]
extern crate compile_msg;

#[phase(plugin)]
extern crate gl_generator;

extern crate glutin;
extern crate libc;
extern crate native;
extern crate time;

#[doc(hidden)]
pub use data_types::GLDataTuple;

pub use index_buffer::IndexBuffer;
pub use vertex_buffer::{VertexBuffer, VertexBindings, VertexFormat};
pub use program::{Program, ProgramUniforms};
pub use texture::Texture;

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

mod context;
mod data_types;
mod index_buffer;
mod program;
mod texture;
mod vertex_buffer;

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
mod gl {
    generate_gl_bindings!("gl", "core", "3.3", "struct")
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
pub enum BlendingFunction {
    /// Always replace the destination pixel by the source.
    AlwaysReplace,

    /// Linear interpolation of the source pixel by the source pixel's alpha.
    LerpBySourceAlpha,

    /// Linear interpolation of the source pixel by the destination pixel's alpha.
    LerpByDestinationAlpha
}

/// Culling mode.
/// 
/// Describes how triangles could be filtered before the fragment part.
pub enum BackfaceCullingMode {
    /// All triangles are always drawn.
    CullingDisabled,

    /// Triangles whose vertices are counter-clock-wise won't be drawn.
    CullCounterClockWise,

    /// Triangles whose indices are clock-wise won't be drawn.
    CullClockWise
}

/// Function to use for out-of-bounds samples.
///
/// This is how GL must handle samples that are outside the texture.
pub enum SamplerWrapFunction {
    /// Samples at coord `x + 1` are mapped to coord `x`.
    Repeat,

    /// Samples at coord `x + 1` are mapped to coord `1 - x`.
    Mirror,

    /// Samples at coord `x + 1` are mapped to coord `1`.
    Clamp
}

/// The function that the GPU will use when loading the value of a texel.
pub enum SamplerFilter {
    /// The nearest texel will be loaded.
    Nearest,

    /// All nearby texels will be loaded and their values will be merged.
    Linear
}

/// The function that the GPU will use to determine whether to write over an existing pixel
///  on the target.
pub enum DepthFunction {
    /// Never replace the target pixel.
    /// 
    /// This option doesn't really make sense, but is here for completeness.
    Ignore,

    /// Always replace the target pixel.
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
/// For each field, `None` means "don't care".
///
/// Example:
/// 
/// ```
/// let params = glium_core::DrawParameters {
///     depth_function: Some(glium_core::IfLess),
///     .. std::default::Default::default()
/// };
/// ```
///
pub struct DrawParameters {
    /// The function that the GPU will use to determine whether to write over an existing pixel
    ///  on the target.
    pub depth_function: Option<DepthFunction>,
}

impl std::default::Default for DrawParameters {
    fn default() -> DrawParameters {
        DrawParameters {
            depth_function: None,
        }
    }
}

impl DrawParameters {
    /// Synchronizes the parmaeters with the current state.
    fn sync(&self, gl: &gl::Gl, state: &mut context::GLState) {
        // TODO: use if let
        if self.depth_function.is_some() {
            let depth_function = self.depth_function.unwrap();
            let depth_function = depth_function.to_glenum();

            if state.depth_func != depth_function {
                gl.DepthFunc(depth_function);
                state.depth_func = depth_function;
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
pub struct BasicDraw<'a, 'b, 'c, V>(pub &'a VertexBuffer<V>, pub &'b IndexBuffer, pub &'c ProgramUniforms);

impl<'a, 'b, 'c, V> DrawCommand for BasicDraw<'a, 'b, 'c, V> {
    fn draw(self, target: &mut Target) {
        let BasicDraw(vertex_buffer, index_buffer, program) = self;

        let fbo_id = target.framebuffer.as_ref().map(|f| f.id);
        let (vb_id, vb_elementssize, vb_bindingsclone) = vertex_buffer::get_clone(vertex_buffer);
        let (ib_id, ib_elemcounts, ib_datatype, ib_primitives) = index_buffer::get_clone(index_buffer);
        let program_id = program::get_program_id(program);
        let (uniforms_textures, uniforms_values) = match program::unwrap_uniforms(program) {
            (a, b) => (a.clone(), b.clone())
        };

        let (tx, rx) = channel();

        target.display.context.exec(proc(gl, state) {
            unsafe {
                gl.BindFramebuffer(gl::FRAMEBUFFER, fbo_id.unwrap_or(0));

                gl.Disable(gl::DEPTH_TEST);
                gl.Enable(gl::BLEND);
                gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

                // binding program
                if state.program != program_id {
                    gl.UseProgram(program_id);
                    state.program = program_id;
                }

                // binding program uniforms
                {
                    let mut active_texture: uint = 0;
                    for (&location, ref texture) in uniforms_textures.iter() {
                        gl.ActiveTexture(gl::TEXTURE0 + active_texture as u32);
                        gl.BindTexture(texture.bind_point, texture.id);
                        gl.Uniform1i(location, active_texture as i32);
                        active_texture = active_texture + 1;
                    }

                    for (&location, &(ref datatype, ref data)) in uniforms_values.iter() {
                        match *datatype {
                            gl::FLOAT       => gl.Uniform1fv(location, 1, data.as_ptr() as *const f32),
                            gl::FLOAT_VEC2  => gl.Uniform2fv(location, 1, data.as_ptr() as *const f32),
                            gl::FLOAT_VEC3  => gl.Uniform3fv(location, 1, data.as_ptr() as *const f32),
                            gl::FLOAT_VEC4  => gl.Uniform4fv(location, 1, data.as_ptr() as *const f32),
                            gl::FLOAT_MAT4  => gl.UniformMatrix4fv(location, 1, 0, data.as_ptr() as *const f32),
                            _ => fail!("Loading uniforms for this type not implemented")
                        }
                        //gl.Uniform1i(location, active_texture as i32);
                    }
                }

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
    fn build_glium_core(self) -> Result<Display, ()>;
}

impl DisplayBuild for glutin::WindowBuilder {
    fn build_glium_core(self) -> Result<Display, ()> {
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
pub struct Display {
    context: Arc<DisplayImpl>
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
        // TODO: requires elevating the GL version
        //self.context.context.exec(proc(gl, _) {
        //    if gl.ReleaseShaderCompiler.is_loaded() {
        //        gl.ReleaseShaderCompiler();
        //    }
        //});
    }

    /// See `VertexBuffer::new`
    #[deprecated = "Use VertexBuffer::new"]
    pub fn build_vertex_buffer<T: VertexFormat + 'static + Send>(&self, data: Vec<T>)
        -> VertexBuffer<T>
    {
        VertexBuffer::new(self, data)
    }

    /// See `IndexBuffer::new`
    #[deprecated = "Use IndexBuffer::new"]
    pub fn build_index_buffer<T: data_types::GLDataType>(&self, prim: PrimitiveType, data: &[T]) -> IndexBuffer {
        IndexBuffer::new(self, prim, data)
    }

    /// Builds a new texture.
    pub fn build_texture<T: data_types::GLDataTuple>(&self, data: &[T], width: uint, height: uint, depth: uint, array_size: uint)
        -> Texture
    {
        Texture::new(self, data, width, height, depth, array_size)
    }
}
