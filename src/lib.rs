/*!
Easy-to-use, high-level, OpenGL3+ wrapper.

Glium is based on glutin - a cross-platform crate for building an OpenGL window and handling
application events.

Glium provides a **Display** which extends the **glutin::WindowedContext** with a high-level, safe API.

# Initialization

The initialisation of a glium display occurs in several steps.

```no_run
extern crate glium;

fn main() {
    // 1. The **winit::EventsLoop** for handling events.
    let mut events_loop = glium::glutin::EventsLoop::new();
    // 2. Parameters for building the Window.
    let wb = glium::glutin::WindowBuilder::new()
        .with_dimensions(1024, 768)
        .with_title("Hello world");
    // 3. Parameters for building the OpenGL context.
    let cb = glium::glutin::ContextBuilder::new();
    // 4. Build the Display with the given window and OpenGL context parameters and register the
    //    window with the events_loop.
    let display = glium::Display::new(wb, cb, &events_loop).unwrap();
}
```

The `display` object is the most important object of this library and is used when you build
buffers, textures, etc. and when you draw.

You can clone it and pass it around. However it doesn't implement the `Send` and `Sync` traits,
meaning that you can't pass it to another thread.

The display has ownership of both the window and context, and also provides some methods related to
domains such as events handling.

# Overview

OpenGL is similar to a drawing software: you draw something, then draw over it, then over it
again, etc. until you are satisfied of the result.

Once you have a `display`, you can call `let mut frame = display.draw();` to start drawing. This
`frame` object implements [the `Surface` trait](trait.Surface.html) and provides some functions
such as `clear_color`, but also allows you to draw with the rendering pipeline.

In order to draw something, you will need to pass:

 - A source of vertices (see the [`vertex`](vertex/index.html) module)
 - A source of indices (see the [`index`](index/index.html) module)
 - A program that contains the shader that the GPU will execute (see the
   [`program`](program/index.html) module)
 - A list of uniforms for the program (see the [`uniforms`](uniforms/index.html) module)
 - Draw parameters to customize the drawing process (see the
   [`draw_parameters`](draw_parameters/index.html) module)

Once you have finished drawing, you can call `frame.finish()` to swap buffers and present the
result to the user.

# OpenGL equivalents in glium

 - **Bind points**: Glium automatically binds and unbinds buffers, textures, etc. in an optimized
   way.
 - **Buffers**: Buffers are strongly typed and can be used through `vertex::VertexBuffer`,
   `index::IndexBuffer` or `uniforms::UniformBuffer`.
 - **Debug output**: If you compile in debug mode, glium registers a debug output callback and
   panics if an OpenGL error happens.
 - **Framebuffer Objects**: FBOs are automatically managed by glium and are stored in the `Context`
   object. You can specify the attachments that you wish with the `framebuffer` module.
 - **Instancing**: Instancing is done either by passing a `vertex::EmptyInstanceAttributes` marker
   or one or several references to vertex buffers wrapped inside a `PerInstance` struct. See the
   `vertex` module for more infos.
 - **Memory barriers**: Calling `glMemoryBarrier` is automatically handled by glium, however you
   still need to call `memoryBarrier()` in your GLSL code in some situations.
 - **Programs**: See the `program` module.
 - **Query objects**: The corresponding structs are in the `draw_parameters` module. They are
   passed as draw parameters.
 - **Renderbuffer**: See the `framebuffer` module.
 - **Render to texture**: If you just want to draw on a texture, you can call
   `texture.as_surface()`. For more advanced options, see the `framebuffer` module.
 - **Samplers**: Samplers are automatically managed by glium and are stored in the `Context`
   object. You can specify how a texture should be sampled by using a `Sampler` dummy object
   in the `uniforms` module.
 - **Shaders**: You can't manually create individual shaders. Instead you must create whole
   programs at once.
 - **Textures**: Textures are strongly typed and are found in the `texture` module.
 - **Uniform blocks**: If your program uses uniform blocks, you must pass a reference to a
   uniform buffer for the name of the block when drawing.
 - **Vertex array objects**: VAOs are automatically managed by glium if the backend supports them.

*/
#![warn(missing_docs)]

// TODO: remove these when everything is implemented
#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use]
extern crate lazy_static;

extern crate memoffset;
extern crate backtrace;
extern crate smallvec;
extern crate fnv;
extern crate takeable_option;

#[cfg(feature = "glutin")]
pub use backend::glutin::glutin;
pub use context::Profile;
pub use draw_parameters::{Blend, BlendingFunction, LinearBlendingFactor, BackfaceCullingMode};
pub use draw_parameters::{Depth, DepthTest, PolygonMode, DrawParameters, StencilTest, StencilOperation};
pub use draw_parameters::{Smooth};
pub use index::IndexBuffer;
pub use vertex::{VertexBuffer, Vertex, VertexFormat};
pub use program::{Program, ProgramCreationError};
pub use program::ProgramCreationError::{CompilationError, LinkingError, ShaderTypeNotSupported};
pub use sync::{LinearSyncFence, SyncFence};
pub use texture::Texture2d;
pub use version::{Api, Version, get_supported_glsl_version};
pub use ops::ReadError;

use std::rc::Rc;
use std::thread;
use std::error::Error;
use std::fmt;
use std::hash::BuildHasherDefault;
use std::collections::HashMap;

use fnv::FnvHasher;

use context::Context;
use context::CommandContext;

#[macro_use]
mod macros;

pub mod backend;
pub mod buffer;
pub mod debug;
pub mod draw_parameters;
pub mod framebuffer;
pub mod index;
pub mod pixel_buffer;
pub mod program;
pub mod uniforms;
pub mod vertex;
pub mod texture;

mod context;
mod fbo;
mod image_format;
mod ops;
mod sampler_object;
mod sync;
mod utils;
mod version;
mod vertex_array_object;

mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

#[doc(hidden)]
pub use memoffset::offset_of as __glium_offset_of;

/// The main object of this library. Controls the whole display.
///
/// This object contains a smart pointer to the real implementation.
/// Cloning the display allows you to easily share the `Display` object throughout
/// your program.
#[cfg(feature = "glutin")]
pub use backend::glutin::Display;
#[cfg(feature = "glutin")]
pub use backend::glutin::headless::Headless as HeadlessRenderer;

/// Trait for objects that describe the capabilities of an OpenGL backend.
pub trait CapabilitiesSource {
    /// Returns the version of the backend.
    fn get_version(&self) -> &version::Version;

    /// Returns the list of extensions that are supported.
    fn get_extensions(&self) -> &context::ExtensionsList;

    /// Returns the capabilities of the backend.
    fn get_capabilities(&self) -> &context::Capabilities;
}

/// Trait for objects that are OpenGL objects.
pub trait GlObject {
    /// The type of identifier for this object.
    type Id;

    /// Returns the id of the object.
    fn get_id(&self) -> Self::Id;
}

/// Handle to a shader or a program.
// TODO: Handle(null()) is equal to Id(0)
#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub enum Handle {
    /// A numeric identifier.
    Id(gl::types::GLuint),
    /// A `GLhandleARB`.
    Handle(gl::types::GLhandleARB),
}

unsafe impl Send for Handle {}

/// Internal trait for enums that can be turned into GLenum.
trait ToGlEnum {
    /// Returns the value.
    fn to_glenum(&self) -> gl::types::GLenum;
}

/// Internal trait for subbuffers.
trait BufferExt {
    /// Returns the number of bytes from the start of the buffer to this subbuffer.
    fn get_offset_bytes(&self) -> usize;

    /// Calls `glMemoryBarrier(GL_VERTEX_ATTRIB_ARRAY_BARRIER_BIT)` if necessary.
    fn prepare_for_vertex_attrib_array(&self, &mut CommandContext);

    /// Calls `glMemoryBarrier(ELEMENT_ARRAY_BARRIER_BIT)` if necessary.
    fn prepare_for_element_array(&self, &mut CommandContext);

    /// Binds the buffer to `GL_ELEMENT_ARRAY_BUFFER` regardless of the current vertex array object.
    fn bind_to_element_array(&self, &mut CommandContext);

    /// Makes sure that the buffer is bound to the `GL_PIXEL_PACK_BUFFER` and calls
    /// `glMemoryBarrier(GL_PIXEL_BUFFER_BARRIER_BIT)` if necessary.
    fn prepare_and_bind_for_pixel_pack(&self, &mut CommandContext);

    /// Makes sure that nothing is bound to `GL_PIXEL_PACK_BUFFER`.
    fn unbind_pixel_pack(&mut CommandContext);

    /// Makes sure that the buffer is bound to the `GL_PIXEL_UNPACK_BUFFER` and calls
    /// `glMemoryBarrier(GL_PIXEL_BUFFER_BARRIER_BIT)` if necessary.
    fn prepare_and_bind_for_pixel_unpack(&self, &mut CommandContext);

    /// Makes sure that nothing is bound to `GL_PIXEL_UNPACK_BUFFER`.
    fn unbind_pixel_unpack(&mut CommandContext);

    /// Makes sure that the buffer is bound to the `GL_QUERY_BUFFER` and calls
    /// `glMemoryBarrier(GL_PIXEL_BUFFER_BARRIER_BIT)` if necessary.
    fn prepare_and_bind_for_query(&self, &mut CommandContext);

    /// Makes sure that nothing is bound to `GL_QUERY_BUFFER`.
    fn unbind_query(&mut CommandContext);

    /// Makes sure that the buffer is bound to the `GL_DRAW_INDIRECT_BUFFER` and calls
    /// `glMemoryBarrier(GL_COMMAND_BARRIER_BIT)` if necessary.
    fn prepare_and_bind_for_draw_indirect(&self, &mut CommandContext);

    /// Makes sure that the buffer is bound to the `GL_DISPATCH_INDIRECT_BUFFER` and calls
    /// `glMemoryBarrier(GL_COMMAND_BARRIER_BIT)` if necessary.
    fn prepare_and_bind_for_dispatch_indirect(&self, &mut CommandContext);

    /// Makes sure that the buffer is bound to the indexed `GL_UNIFORM_BUFFER` point and calls
    /// `glMemoryBarrier(GL_UNIFORM_BARRIER_BIT)` if necessary.
    fn prepare_and_bind_for_uniform(&self, &mut CommandContext, index: gl::types::GLuint);

    /// Makes sure that the buffer is bound to the indexed `GL_SHARED_STORAGE_BUFFER` point and calls
    /// `glMemoryBarrier(GL_SHADER_STORAGE_BARRIER_BIT)` if necessary.
    fn prepare_and_bind_for_shared_storage(&self, &mut CommandContext, index: gl::types::GLuint);

    /// Binds the buffer to `GL_TRANSFORM_FEEDBACk_BUFFER` regardless of the current transform
    /// feedback object.
    fn bind_to_transform_feedback(&self, &mut CommandContext, index: gl::types::GLuint);
}

/// Internal trait for subbuffer slices.
trait BufferSliceExt<'a> {
    /// Tries to get an object where to write a fence.
    ///
    /// If this function returns `None`, no fence will be created nor written.
    fn add_fence(&self) -> Option<buffer::Inserter<'a>>;
}

/// Internal trait for contexts.
trait ContextExt {
    /// Sets whether the context's debug output callback should take errors into account.
    fn set_report_debug_output_errors(&self, value: bool);

    /// Start executing OpenGL commands by checking the current context.
    fn make_current(&self) -> context::CommandContext;

    /// Returns the capabilities of the backend.
    fn capabilities(&self) -> &context::Capabilities;
}

/// Internal trait for programs.
trait ProgramExt {
    /// Calls `glUseProgram` and enables/disables `GL_PROGRAM_POINT_SIZE` and
    /// `GL_FRAMEBUFFER_SRGB`.
    fn use_program(&self, ctxt: &mut context::CommandContext);

    /// Changes the value of a uniform of the program.
    fn set_uniform(&self, ctxt: &mut context::CommandContext, uniform_location: gl::types::GLint,
                   value: &RawUniformValue);

    /// Changes the uniform block binding of the program.
    fn set_uniform_block_binding(&self, ctxt: &mut context::CommandContext,
                                 block_location: gl::types::GLuint, value: gl::types::GLuint);

    /// Changes the shader storage block binding of the program.
    fn set_shader_storage_block_binding(&self, ctxt: &mut context::CommandContext,
                                        block_location: gl::types::GLuint,
                                        value: gl::types::GLuint);

    /// Changes the subroutine uniform bindings of a program.
    fn set_subroutine_uniforms_for_stage(&self, ctxt: &mut context::CommandContext,
                                         stage: program::ShaderStage,
                                         indices: &[gl::types::GLuint]);

    fn get_uniform(&self, name: &str) -> Option<&program::Uniform>;

    fn get_uniform_blocks(&self) -> &HashMap<String, program::UniformBlock, BuildHasherDefault<FnvHasher>>;

    fn get_shader_storage_blocks(&self) -> &HashMap<String, program::UniformBlock, BuildHasherDefault<FnvHasher>>;

    fn get_subroutine_data(&self) -> &program::SubroutineData;
}

/// Internal trait for queries.
trait QueryExt {
    fn begin_query(&self, ctxt: &mut CommandContext) -> Result<(), DrawError>;

    fn end_samples_passed_query(ctxt: &mut CommandContext);

    fn end_time_elapsed_query(ctxt: &mut CommandContext);

    fn end_primitives_generated_query(ctxt: &mut CommandContext);

    fn end_transform_feedback_primitives_written_query(ctxt: &mut CommandContext);

    fn begin_conditional_render(&self, ctxt: &mut CommandContext, wait: bool, per_region: bool);

    fn end_conditional_render(ctxt: &mut CommandContext);

    /// Returns true if the query has never been used.
    fn is_unused(&self) -> bool;
}

/// Internal trait for textures.
trait TextureExt {
    /// Returns the ID of the texture.
    fn get_texture_id(&self) -> gl::types::GLuint;

    /// Returns the context associated to this texture.
    fn get_context(&self) -> &Rc<Context>;

    /// Returns the bind point of the texture.
    fn get_bind_point(&self) -> gl::types::GLenum;

    /// Makes sure that the texture is bound to the current texture unit and returns the
    /// bind point to use to access the texture (eg. `GL_TEXTURE_2D`, `GL_TEXTURE_3D`, etc.).
    fn bind_to_current(&self, &mut CommandContext) -> gl::types::GLenum;
}

/// Internal trait for textures.
trait TextureMipmapExt {
    /// Changes some parts of the texture.
    fn upload_texture<'a, P>(&self, x_offset: u32, y_offset: u32, z_offset: u32,
                             (image_format::ClientFormatAny, std::borrow::Cow<'a, [P]>), width: u32,
                             height: Option<u32>, depth: Option<u32>,
                             regen_mipmaps: bool)
                             -> Result<(), ()>   // TODO return a better Result!?
                             where P: Send + Copy + Clone + 'a;

    fn download_compressed_data(&self) -> Option<(image_format::ClientFormatAny, Vec<u8>)>;
}

/// Internal trait for transform feedback sessions.
trait TransformFeedbackSessionExt {
    /// Updates the state of OpenGL to make the transform feedback session current.
    ///
    /// The second parameter must be the primitive type of the input vertex data.
    fn bind(&self, &mut CommandContext, index::PrimitiveType);

    /// Ensures that transform feedback is disabled.
    fn unbind(&mut CommandContext);

    /// Ensures that a buffer isn't used by transform feedback.
    fn ensure_buffer_out_of_transform_feedback(&mut CommandContext, gl::types::GLuint);
}

/// Internal trait for uniforms handling.
trait UniformsExt {
    /// Binds the uniforms to a given program.
    ///
    /// Will replace texture and buffer bind points.
    fn bind_uniforms<'a, P>(&'a self, &mut CommandContext, &P, &mut Vec<buffer::Inserter<'a>>)
                            -> Result<(), DrawError> where P: ProgramExt;
}


/// A raw value of a uniform. "Raw" means that it's passed directly with `glUniform`. Textures
/// for example are just passed as integers.
///
/// Blocks and subroutines are not included.
#[derive(Copy, Clone, Debug)]
#[allow(missing_docs)]
pub enum RawUniformValue {
    SignedInt(gl::types::GLint),
    UnsignedInt(gl::types::GLuint),
    Float(gl::types::GLfloat),
    /// 2x2 column-major matrix.
    Mat2([[gl::types::GLfloat; 2]; 2]),
    /// 3x3 column-major matrix.
    Mat3([[gl::types::GLfloat; 3]; 3]),
    /// 4x4 column-major matrix.
    Mat4([[gl::types::GLfloat; 4]; 4]),
    Vec2([gl::types::GLfloat; 2]),
    Vec3([gl::types::GLfloat; 3]),
    Vec4([gl::types::GLfloat; 4]),
    IntVec2([gl::types::GLint; 2]),
    IntVec3([gl::types::GLint; 3]),
    IntVec4([gl::types::GLint; 4]),
    UnsignedIntVec2([gl::types::GLuint; 2]),
    UnsignedIntVec3([gl::types::GLuint; 3]),
    UnsignedIntVec4([gl::types::GLuint; 4]),

    // Double precision primitives
    Double(gl::types::GLdouble),
    DoubleMat2([[gl::types::GLdouble; 2]; 2]),
    DoubleMat3([[gl::types::GLdouble; 3]; 3]),
    DoubleMat4([[gl::types::GLdouble; 4]; 4]),
    DoubleVec2([gl::types::GLdouble; 2]),
    DoubleVec3([gl::types::GLdouble; 3]),
    DoubleVec4([gl::types::GLdouble; 4]),
    Int64(gl::types::GLint64),
    Int64Vec2([gl::types::GLint64; 2]),
    Int64Vec3([gl::types::GLint64; 3]),
    Int64Vec4([gl::types::GLint64; 4]),
    UnsignedInt64(gl::types::GLuint64),
    UnsignedInt64Vec2([gl::types::GLuint64; 2]),
    UnsignedInt64Vec3([gl::types::GLuint64; 3]),
    UnsignedInt64Vec4([gl::types::GLuint64; 4]),
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
/// this is not necessarily *exactly* what happens. Backends are free to do whatever they want
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
///
/// ## Step 3: Geometry shader (optional)
///
/// If you specify a geometry shader, then the GPU will invoke it once for each primitive.
///
/// The geometry shader can output multiple primitives.
///
/// ## Step 4: Transform feedback (optional)
///
/// TODO:
/// TODO: talk about `transform_feedback_primitives_written_query` as well
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
/// If a query has been specified through `primitives_generated_query`, then its value is updated.
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
/// in the draw parameters. If you specify the `smooth` parameter, then the borders of the
/// primitives will see their alpha value adjusted.
///
/// <img alt="" src="data:image/png;base64,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" />
///
/// The attributes of each vertex that are marked as `smooth` (which is the default value) are
/// being interpolated, and the GPU assigns a value for each attribute for each pixel.
///
/// ## Step 9: Fragment shader
///
/// The GPU now executes the fragment shader once for each pixel of each primitive.
/// The vertex attributes that were interpolated at the previous step are passed to the fragment
/// shader.
///
/// The fragment shader must return the color to write by setting the value of `gl_FragColor`.
///
/// If the target framebuffer has multisampling enabled, then each pixel of the target image is
/// in turn split into four subpixels. The output of the fragment shader is copied and written into
/// each subpixel. If `multisampling` in the draw parameters is `true` (its default value), only
/// subpixels that belong to the triangle are written.
///
/// If a query has been specified through `samples_passed_query`, then its value is updated.
///
/// ## Step 10: Pixel ownership
///
/// This step is mostly an implementation detail. If the window you are drawing on is not on the
/// foreground, or if it is partially obstructed, then the pixels that are not on the
/// foreground will be discarded.
///
/// This is only relevant if you draw to the default framebuffer.
///
/// This step has to be taken into account in some situations. For example if you query the number
/// of samples that have been written, the ones that don't pass the pixel ownership test won't
/// count.
///
/// ## Step 11: Scissor test
///
/// If `scissor` has been specified, then all the pixels that are outside of this rect
/// are discarded.
///
/// ## Step 12: Depth test
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
/// ## Step 13: Stencil test
///
/// Similar to the depth buffer, surfaces can also have a stencil buffer.
///
/// The `stencil_test_clockwise` and `stencil_test_counter_clockwise` draw parameters specify
/// the operation to use to check whether or not each pixel passes the stencil test. Pixels that
/// fail the stencil test won't be drawn on the screen.
///
/// The `*_clockwise` members are relevant for polygons that are displayed clockwise, and the
/// `*_counter_clockwise` members are relevant for polygons that are displayed counterclockwise.
/// See also the `face culling` step for more infos. Lines and points always use the `*_clockwise`
/// members.
///
/// There are three possibilities for each pixel: either it fails the stencil test, or it passes
/// the stencil test but failed the depth test, or it passes both the stencil test and depth test.
/// You can specify for each of these three situations what to do with the value in the stencil
/// buffer with the draw parameters.
///
/// ## Step 14: Blending
///
/// For each pixel to write, the GPU takes the RGBA color that the fragment shader has returned
/// and the existing RGBA color already written on the surface, and merges the two.
///
/// The way they are merged depends on the value of `blending_function`. This allows you to choose
/// how alpha colors are merged together.
///
/// See the documentation of `BlendingFunction` fore more informations.
///
/// ## Step 15: Dithering (optional)
///
/// If `dithering` is `true` in the draw parameters, then a dithering algorithm is applied.
///
/// When you draw a gradient of colors, the boundary between each individual color value is
/// visible. Thanks to an optical illusion, a dithering algorithm will change the color values
/// of some pixels and hide the boundaries.
///
/// ## Step 16: Conversion to sRGB
///
/// If the target has sRGB enabled, then the output of the fragment will get some gamma correction.
///
/// Monitors don't show colors linearly. For example a value of `0.5` isn't shown half as bright
/// as a value of `1.0`. Instead the monitor will show colors as darker than they should be.
/// In order to fix this problem, each pixel is modified to be made slightly brighter with the same
/// factor as the monitor makes them darker.
///
/// ## Step 17: End
///
/// This is finally the step where colors are being written. The `color_mask` parameter allow you
/// to specify whether each color component (red, green, blue and alpha) is written to the color
/// buffer.
///
pub trait Surface {
    /// Clears some attachments of the target.
    fn clear(&mut self, rect: Option<&Rect>, color: Option<(f32, f32, f32, f32)>, color_srgb: bool,
             depth: Option<f32>, stencil: Option<i32>);

    /// Clears the color attachment of the target.
    fn clear_color(&mut self, red: f32, green: f32, blue: f32, alpha: f32) {
        self.clear(None, Some((red, green, blue, alpha)), false, None, None);
    }

    /// Clears the color attachment of the target. The color is in sRGB format.
    fn clear_color_srgb(&mut self, red: f32, green: f32, blue: f32, alpha: f32) {
        self.clear(None, Some((red, green, blue, alpha)), true, None, None);
    }

    /// Clears the depth attachment of the target.
    fn clear_depth(&mut self, value: f32) {
        self.clear(None, None, false, Some(value), None);
    }

    /// Clears the stencil attachment of the target.
    fn clear_stencil(&mut self, value: i32) {
        self.clear(None, None, false, None, Some(value));
    }

    /// Clears the color and depth attachments of the target.
    fn clear_color_and_depth(&mut self, color: (f32, f32, f32, f32), depth: f32) {
        self.clear(None, Some(color), false, Some(depth), None);
    }

    /// Clears the color and depth attachments of the target. The color is in sRGB format.
    fn clear_color_srgb_and_depth(&mut self, color: (f32, f32, f32, f32), depth: f32) {
        self.clear(None, Some(color), true, Some(depth), None);
    }

    /// Clears the color and stencil attachments of the target.
    fn clear_color_and_stencil(&mut self, color: (f32, f32, f32, f32), stencil: i32) {
        self.clear(None, Some(color), false, None, Some(stencil));
    }

    /// Clears the color and stencil attachments of the target. The color is in sRGB format.
    fn clear_color_srgb_and_stencil(&mut self, color: (f32, f32, f32, f32), stencil: i32) {
        self.clear(None, Some(color), true, None, Some(stencil));
    }

    /// Clears the depth and stencil attachments of the target.
    fn clear_depth_and_stencil(&mut self, depth: f32, stencil: i32) {
        self.clear(None, None, false, Some(depth), Some(stencil));
    }

    /// Clears the color, depth and stencil attachments of the target.
    fn clear_all(&mut self, color: (f32, f32, f32, f32), depth: f32, stencil: i32) {
        self.clear(None, Some(color), false, Some(depth), Some(stencil));
    }

    /// Clears the color, depth and stencil attachments of the target. The color is in sRGB format.
    fn clear_all_srgb(&mut self, color: (f32, f32, f32, f32), depth: f32, stencil: i32) {
        self.clear(None, Some(color), true, Some(depth), Some(stencil));
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
    /// This is probably the most complex function of glium. Check out the rest of the
    /// documentation for example how to use it.
    ///
    /// See above for what happens exactly on the GPU when you draw.
    fn draw<'a, 'b, V, I, U>(&mut self, V, I, program: &Program, uniforms: &U,
        draw_parameters: &DrawParameters) -> Result<(), DrawError> where
        V: vertex::MultiVerticesSource<'b>, I: Into<index::IndicesSource<'a>>,
        U: uniforms::Uniforms;

    /// Blits from the default framebuffer.
    fn blit_from_frame(&self, source_rect: &Rect, target_rect: &BlitTarget,
                       filter: uniforms::MagnifySamplerFilter);

    /// Blits from a simple framebuffer.
    fn blit_from_simple_framebuffer(&self, source: &framebuffer::SimpleFrameBuffer,
                                    source_rect: &Rect, target_rect: &BlitTarget,
                                    filter: uniforms::MagnifySamplerFilter);

    /// Blits from a multi-output framebuffer.
    fn blit_from_multioutput_framebuffer(&self, source: &framebuffer::MultiOutputFrameBuffer,
                                         source_rect: &Rect, target_rect: &BlitTarget,
                                         filter: uniforms::MagnifySamplerFilter);

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
    fn blit_color<S>(&self, source_rect: &Rect, target: &S, target_rect: &BlitTarget,
                     filter: uniforms::MagnifySamplerFilter) where S: Surface;

    /// Copies the entire surface to a target surface. See `blit_color`.
    #[inline]
    fn blit_whole_color_to<S>(&self, target: &S, target_rect: &BlitTarget,
        filter: uniforms::MagnifySamplerFilter) where S: Surface
    {
        let src_dim = self.get_dimensions();
        let src_rect = Rect { left: 0, bottom: 0, width: src_dim.0 as u32, height: src_dim.1 as u32 };
        self.blit_color(&src_rect, target, target_rect, filter)
    }

    /// Copies the entire surface to the entire target. See `blit_color`.
    #[inline]
    fn fill<S>(&self, target: &S, filter: uniforms::MagnifySamplerFilter) where S: Surface {
        let src_dim = self.get_dimensions();
        let src_rect = Rect { left: 0, bottom: 0, width: src_dim.0 as u32, height: src_dim.1 as u32 };
        let target_dim = target.get_dimensions();
        let target_rect = BlitTarget { left: 0, bottom: 0, width: target_dim.0 as i32, height: target_dim.1 as i32 };
        self.blit_color(&src_rect, target, &target_rect, filter)
    }
}

/// Private trait for framebuffer-like objects that provide attachments.
trait FboAttachments {
    /// Returns the list of attachments of this FBO, or `None` if it is the default framebuffer.
    fn get_attachments(&self) -> Option<&fbo::ValidatedAttachments>;
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
        /// The error giving more details about the mismatch.
        err: uniforms::LayoutMismatchError,
    },

    /// Tried to bind a subroutine uniform like a regular uniform value.
    SubroutineUniformToValue {
        /// Name of the uniform you are trying to bind.
        name: String,
    },

    /// Not all subroutine uniforms of a shader stage were set.
    SubroutineUniformMissing {
        /// Shader stage with missing bindings.
        stage: program::ShaderStage,
        /// The expected number of bindings.
        expected_count: usize,
        /// The number of bindings defined by the user.
        real_count: usize,

    },

    /// A non-existent subroutine was referenced.
    SubroutineNotFound {
        /// The stage the subroutine was searched for.
        stage: program::ShaderStage,
        /// The invalid name of the subroutine.
        name: String
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

    /// See the documentation of the `draw_parameters` module for infos.
    WrongQueryOperation,

    /// You requested smoothing, but this is not supported by the backend.
    SmoothingNotSupported,

    /// The requested provoking vertex is not supported by the backend.
    ProvokingVertexNotSupported,

    /// Discarding rasterizer output isn't supported by the backend.
    RasterizerDiscardNotSupported,

    /// Depth clamping isn't supported by the backend.
    DepthClampNotSupported,

    /// One of the blending parameters is not supported by the backend.
    BlendingParameterNotSupported,

    /// Restarting indices (multiple objects per draw call) is not supported by the backend.
    FixedIndexRestartingNotSupported,

    /// Tried to enable a clip plane that does not exist.
    ClipPlaneIndexOutOfBounds,
}

impl Error for DrawError {
    fn description(&self) -> &str {
        use self::DrawError::*;
        match *self {
            NoDepthBuffer =>
                "A depth function has been requested but no depth buffer is available",
            AttributeTypeMismatch =>
                "The type of a vertex attribute in the vertices source doesn't match what the program requires",
            AttributeMissing =>
                "One of the attributes required by the program is missing from the vertex format",
            ViewportTooLarge =>
                "The viewport's dimensions are not supported by the backend",
            InvalidDepthRange =>
                "The depth range is outside of the `(0, 1)` range",
            UniformTypeMismatch { .. } =>
                "The type of a uniform doesn't match what the program requires",
            UniformBufferToValue { .. } =>
                "Tried to bind a uniform buffer to a single uniform value",
            UniformValueToBlock { .. } =>
                "Tried to bind a single uniform value to a uniform block",
            UniformBlockLayoutMismatch { .. } =>
                "The layout of the content of the uniform buffer does not match the layout of the block",
            SubroutineUniformToValue { .. } =>
                "Tried to bind a subroutine uniform like a regular uniform value",
            SubroutineUniformMissing { .. } =>
                "Not all subroutine uniforms of a shader stage were set",
            SubroutineNotFound { .. } =>
                "A non-existent subroutine was referenced",
            UnsupportedVerticesPerPatch =>
                "The number of vertices per patch that has been requested is not supported",
            TessellationNotSupported =>
                "Trying to use tessellation, but this is not supported by the underlying hardware",
            TessellationWithoutPatches =>
                "Using a program which contains tessellation shaders, but without submitting patches",
            SamplersNotSupported => "
                Trying to use a sampler, but they are not supported by the backend",
            InstancesCountMismatch =>
                "When you use instancing, all vertices sources must have the same size",
            VerticesSourcesLengthMismatch =>
                "If you don't use indices, then all vertices sources must have the same size",
            TransformFeedbackNotSupported =>
                "Requested not to draw primitives, but this is not supported by the backend",
            WrongQueryOperation =>
                "Wrong query operation",
            SmoothingNotSupported =>
                "Trying to use smoothing, but this is not supported by the backend",
            ProvokingVertexNotSupported =>
                "Trying to set the provoking vertex, but this is not supported by the backend",
            RasterizerDiscardNotSupported =>
                "Discarding rasterizer output is not supported by the backend",
            DepthClampNotSupported =>
                "The depth clamp mode is not supported by the backend",
            BlendingParameterNotSupported =>
                "One the blending parameters is not supported by the backend",
            FixedIndexRestartingNotSupported =>
                "Restarting indices (multiple objects per draw call) is not supported by the backend",
            ClipPlaneIndexOutOfBounds =>
                "Tried to enable a clip plane that does not exist."
        }
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::DrawError::*;
        match *self {
            UniformBlockLayoutMismatch { ref err, .. } => Some(err),
            _ => None,
        }
    }
}


impl fmt::Display for DrawError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::DrawError::*;
        match *self {
            UniformTypeMismatch { ref name, ref expected } =>
                write!(
                    fmt,
                    "{}, got: {:?}, expected: {:?}",
                    self.description(),
                    name,
                    expected,
                ),
            UniformBufferToValue { ref name } =>
                write!(
                    fmt,
                    "{}: {}",
                    self.description(),
                    name,
                ),
            UniformValueToBlock { ref name } =>
                write!(
                    fmt,
                    "{}: {}",
                    self.description(),
                    name,
                ),
            UniformBlockLayoutMismatch { ref name, ref err } =>
                write!(
                    fmt,
                    "{}: {}, caused by {}",
                    self.description(),
                    name,
                    err,
                ),
            _ =>
                write!(fmt, "{}", self.description()),
        }
    }
}

/// Error that can happen when swapping buffers.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SwapBuffersError {
    /// The OpenGL context has been lost and needs to be recreated. The `Display` and all the
    /// objects associated to it (textures, buffers, programs, etc.) need to be recreated from
    /// scratch.
    ///
    /// Operations will have no effect. Functions that read textures, buffers, etc. from OpenGL
    /// will return uninitialized data instead.
    ///
    /// A context loss usually happens on mobile devices when the user puts the application on
    /// sleep and wakes it up later. However any OpenGL implementation can theoretically lose the
    /// context at any time. Can only happen if calling `is_context_loss_possible()` returns true.
    ContextLost,
    /// The buffers have already been swapped.
    ///
    /// This error can be returned when `set_finish()` is called multiple times, or `finish()` is
    /// called after `set_finish()`.
    AlreadySwapped,
}

impl Error for SwapBuffersError {
    fn description(&self) -> &str {
        use self::SwapBuffersError::*;
        match *self {
            ContextLost =>
                "the OpenGL context has been lost and needs to be recreated",
            AlreadySwapped =>
                "the buffers have already been swapped",
        }
    }
}

impl fmt::Display for SwapBuffersError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", self.description())
    }
}

/// Implementation of `Surface`, targeting the default framebuffer.
///
/// The back- and front-buffers are swapped when you call `finish`.
///
/// You **must** call either `finish` or `set_finish` or else the destructor will panic.
pub struct Frame {
    context: Rc<Context>,
    dimensions: (u32, u32),
    destroyed: bool,        // TODO: use a linear type instead.
}

impl Frame {
    /// Builds a new `Frame`. Use the `draw` function on `Display` instead of this function.
    #[inline]
    pub fn new(context: Rc<Context>, dimensions: (u32, u32)) -> Frame {
        Frame {
            context: context,
            dimensions: dimensions,
            destroyed: false,
        }
    }

    /// Stop drawing, swap the buffers, and consume the Frame.
    ///
    /// See the documentation of `SwapBuffersError` about what is being returned.
    #[inline]
    pub fn finish(mut self) -> Result<(), SwapBuffersError> {
        self.set_finish()
    }

    /// Stop drawing, swap the buffers.
    ///
    /// The Frame can now be dropped regularly.  Calling `finish()` or `set_finish()` again will
    /// cause `Err(SwapBuffersError::AlreadySwapped)` to be returned.
    #[inline]
    pub fn set_finish(&mut self) -> Result<(), SwapBuffersError> {
        if self.destroyed {
            return Err(SwapBuffersError::AlreadySwapped);
        }

        self.destroyed = true;
        self.context.swap_buffers()
    }
}

impl Surface for Frame {
    #[inline]
    fn clear(&mut self, rect: Option<&Rect>, color: Option<(f32, f32, f32, f32)>, color_srgb: bool,
             depth: Option<f32>, stencil: Option<i32>)
    {
        ops::clear(&self.context, None, rect, color, color_srgb, depth, stencil);
    }

    fn get_dimensions(&self) -> (u32, u32) {
        self.dimensions
    }

    fn get_depth_buffer_bits(&self) -> Option<u16> {
        self.context.capabilities().depth_bits
    }

    fn get_stencil_buffer_bits(&self) -> Option<u16> {
        self.context.capabilities().stencil_bits
    }

    fn draw<'a, 'b, V, I, U>(&mut self, vertex_buffer: V,
                         index_buffer: I, program: &Program, uniforms: &U,
                         draw_parameters: &DrawParameters) -> Result<(), DrawError>
                         where I: Into<index::IndicesSource<'a>>, U: uniforms::Uniforms,
                         V: vertex::MultiVerticesSource<'b>
    {
        if !self.has_depth_buffer() && (draw_parameters.depth.test.requires_depth_buffer() ||
                draw_parameters.depth.write)
        {
            return Err(DrawError::NoDepthBuffer);
        }

        if let Some(viewport) = draw_parameters.viewport {
            if viewport.width > self.context.capabilities().max_viewport_dims.0
                    as u32
            {
                return Err(DrawError::ViewportTooLarge);
            }
            if viewport.height > self.context.capabilities().max_viewport_dims.1
                    as u32
            {
                return Err(DrawError::ViewportTooLarge);
            }
        }

        ops::draw(&self.context, None, vertex_buffer, index_buffer.into(), program,
                  uniforms, draw_parameters, (self.dimensions.0 as u32, self.dimensions.1 as u32))
    }

    #[inline]
    fn blit_color<S>(&self, source_rect: &Rect, target: &S, target_rect: &BlitTarget,
                     filter: uniforms::MagnifySamplerFilter) where S: Surface
    {
        target.blit_from_frame(source_rect, target_rect, filter)
    }

    #[inline]
    fn blit_from_frame(&self, source_rect: &Rect, target_rect: &BlitTarget,
                       filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, None, self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }

    #[inline]
    fn blit_from_simple_framebuffer(&self, source: &framebuffer::SimpleFrameBuffer,
                                    source_rect: &Rect, target_rect: &BlitTarget,
                                    filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, source.get_attachments(), self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }

    #[inline]
    fn blit_from_multioutput_framebuffer(&self, source: &framebuffer::MultiOutputFrameBuffer,
                                         source_rect: &Rect, target_rect: &BlitTarget,
                                         filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, source.get_attachments(), self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }
}

impl FboAttachments for Frame {
    #[inline]
    fn get_attachments(&self) -> Option<&fbo::ValidatedAttachments> {
        None
    }
}

impl Drop for Frame {
    #[inline]
    fn drop(&mut self) {
        if !thread::panicking() {
            assert!(self.destroyed, "The `Frame` object must be explicitly destroyed \
                                     by calling `.finish()`");
        }
    }
}

/// Returned during Context creation if the OpenGL implementation is too old.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncompatibleOpenGl(pub String);

impl fmt::Display for IncompatibleOpenGl {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

impl Error for IncompatibleOpenGl {
    #[inline]
    fn description(&self) -> &str {
        "The OpenGL implementation is too old to work with glium"
    }
}

#[allow(dead_code)]
#[inline]
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
        gl::CONTEXT_LOST => Some("GL_CONTEXT_LOST"),
        _ => Some("Unknown glGetError return value")
    }
}
