/*!
Framebuffers allows you to customize the color, depth and stencil buffers you will draw on.

In order to draw on a texture, use a `SimpleFrameBuffer`. This framebuffer is compatible with
shaders that write to `gl_FragColor`.

```no_run
# let display: glium::Display = unsafe { ::std::mem::uninitialized() };
# let texture: glium::texture::Texture2d = unsafe { ::std::mem::uninitialized() };
let framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display, &texture);
// framebuffer.draw(...);    // draws over `texture`
```

Instead if your shader wants to write to multiple color buffers at once, you must use
a `MultiOutputFrameBuffer`.

```no_run
# let display: glium::Display = unsafe { ::std::mem::uninitialized() };
# let texture1: glium::texture::Texture2d = unsafe { ::std::mem::uninitialized() };
# let texture2: glium::texture::Texture2d = unsafe { ::std::mem::uninitialized() };
let output = &[ ("output1", &texture1), ("output2", &texture2) ];
let framebuffer = glium::framebuffer::MultiOutputFrameBuffer::new(&display, output);
// framebuffer.draw(...);

// example shader:
// 
//     out vec4 output1;
//     out vec4 output2;
//
//     void main() {
//         output1 = vec4(0.0, 0.0, 0.5, 1.0);
//         output2 = vec4(1.0, 0.7, 1.0, 1.0);
//     }
```

**Note**: depth-stencil attachments are not yet implemented.

*/
#![experimental]

use std::kinds::marker::ContravariantLifetime;

use texture::{Texture, Texture2d, DepthTexture2d, StencilTexture2d, DepthStencilTexture2d};
use fbo::FramebufferAttachments;

use {Display, Program, Surface, GlObject};

use {fbo, gl, ops};

/// A framebuffer which has only one color attachment.
pub struct SimpleFrameBuffer<'a> {
    display: Display,
    attachments: FramebufferAttachments,
    marker: ContravariantLifetime<'a>,
    dimensions: (u32, u32),
    depth_buffer_bits: Option<u16>,
    stencil_buffer_bits: Option<u16>,
}

impl<'a> SimpleFrameBuffer<'a> {
    /// Creates a `SimpleFrameBuffer` with a single color attachment and no depth
    /// nor stencil buffer.
    pub fn new<C>(display: &Display, color: &'a C) -> SimpleFrameBuffer<'a>
                  where C: ToColorAttachment
    {
        use render_buffer;

        SimpleFrameBuffer::new_impl(display, color, None::<&render_buffer::DepthRenderBuffer>,
                                    None::<&render_buffer::StencilRenderBuffer>)
    }

    /// Creates a `SimpleFrameBuffer` with a single color attachment and a depth
    /// buffer, but no stencil buffer.
    pub fn with_depth_buffer<C, D>(display: &Display, color: &'a C, depth: &'a D)
                                   -> SimpleFrameBuffer<'a>
                                   where C: ToColorAttachment, D: ToDepthAttachment
    {
        use render_buffer;

        SimpleFrameBuffer::new_impl(display, color, Some(depth),
                                    None::<&render_buffer::StencilRenderBuffer>)
    }

    /// Creates a `SimpleFrameBuffer` with a single color attachment, a depth
    /// buffer, and a stencil buffer.
    pub fn with_depth_and_stencil_buffer<C, D, S>(display: &Display, color: &'a C, depth: &'a D,
                                                  stencil: &'a S) -> SimpleFrameBuffer<'a>
                                                  where C: ToColorAttachment, D: ToDepthAttachment,
                                                  S: ToStencilAttachment
    {
        SimpleFrameBuffer::new_impl(display, color, Some(depth), Some(stencil))
    }

    /// Creates a `SimpleFrameBuffer` with a single color attachment and a stencil
    /// buffer, but no buffer buffer.
    pub fn with_stencil_buffer<C, S>(display: &Display, color: &'a C, stencil: &'a S)
                                     -> SimpleFrameBuffer<'a>
                                     where C: ToColorAttachment, S: ToStencilAttachment
    {
        use render_buffer;

        SimpleFrameBuffer::new_impl(display, color, None::<&render_buffer::DepthRenderBuffer>,
                                    Some(stencil))
    }


    fn new_impl<C, D, S>(display: &Display, color: &'a C, depth: Option<&'a D>,
                         stencil: Option<&'a S>) -> SimpleFrameBuffer<'a>
                         where C: ToColorAttachment, D: ToDepthAttachment, S: ToStencilAttachment
    {
        let (dimensions, color_attachment) = match color.to_color_attachment() {
            ColorAttachment::Texture2d(tex) => {
                let dimensions = (tex.get_width(), tex.get_height().unwrap());
                let id = fbo::Attachment::Texture(tex.get_id());
                (dimensions, id)
            },

            ColorAttachment::RenderBuffer(buffer) => {
                let dimensions = buffer.get_dimensions();
                let id = fbo::Attachment::RenderBuffer(buffer.get_id());
                (dimensions, id)
            },
        };

        let (depth, depth_bits) = if let Some(depth) = depth {
            match depth.to_depth_attachment() {
                DepthAttachment::Texture2d(tex) => {
                    if (tex.get_width(), tex.get_height().unwrap()) != dimensions {
                        panic!("The depth attachment must have the same dimensions \
                                as the color attachment");
                    }

                    (Some(fbo::Attachment::Texture(tex.get_id())), Some(32))      // FIXME: wrong number
                },

                DepthAttachment::RenderBuffer(buffer) => {
                    // TODO: dimensions

                    (Some(fbo::Attachment::RenderBuffer(buffer.get_id())), Some(32))      // FIXME: wrong number
                },
            }

        } else {
            (None, None)
        };

        let (stencil, stencil_bits) = if let Some(stencil) = stencil {
            match stencil.to_stencil_attachment() {
                StencilAttachment::Texture2d(tex) => {
                    if (tex.get_width(), tex.get_height().unwrap()) != dimensions {
                        panic!("The stencil attachment must have the same dimensions \
                                as the color attachment");
                    }

                    (Some(fbo::Attachment::Texture(tex.get_id())), Some(8))       // FIXME: wrong number
                },

                StencilAttachment::RenderBuffer(buffer) => {
                    // TODO: dimensions

                    (Some(fbo::Attachment::RenderBuffer(buffer.get_id())), Some(8))
                },
            }

        } else {
            (None, None)
        };

        SimpleFrameBuffer {
            display: display.clone(),
            attachments: FramebufferAttachments {
                colors: vec![(0, color_attachment)],
                depth: depth,
                stencil: stencil,
            },
            marker: ContravariantLifetime,
            dimensions: dimensions,
            depth_buffer_bits: depth_bits,
            stencil_buffer_bits: stencil_bits,
        }
    }
}

impl<'a> Surface for SimpleFrameBuffer<'a> {
    fn clear_color(&mut self, red: f32, green: f32, blue: f32, alpha: f32) {
        ops::clear_color(&self.display.context, Some(&self.attachments), red, green, blue, alpha)
    }

    fn clear_depth(&mut self, value: f32) {
        ops::clear_depth(&self.display.context, Some(&self.attachments), value)
    }

    fn clear_stencil(&mut self, value: int) {
        ops::clear_stencil(&self.display.context, Some(&self.attachments), value)
    }

    fn get_dimensions(&self) -> (uint, uint) {
        (self.dimensions.0 as uint, self.dimensions.1 as uint)
    }

    fn get_depth_buffer_bits(&self) -> Option<u16> {
        self.depth_buffer_bits
    }

    fn get_stencil_buffer_bits(&self) -> Option<u16> {
        self.stencil_buffer_bits
    }

    fn draw<'b, 'v, V, I, ID, U>(&mut self, vb: V, ib: &I, program: &::Program,
        uniforms: U, draw_parameters: &::DrawParameters) where I: ::index_buffer::ToIndicesSource<ID>,
        U: ::uniforms::Uniforms, ID: ::index_buffer::Index, V: ::vertex_buffer::IntoVerticesSource<'v>
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

        ops::draw(&self.display, Some(&self.attachments), vb.into_vertices_source(),
                  &ib.to_indices_source(), program, uniforms, draw_parameters, self.dimensions)
    }

    fn get_blit_helper(&self) -> ::BlitHelper {
        ::BlitHelper(&self.display.context, Some(&self.attachments))
    }
}

/// This struct is useless for the moment.
pub struct MultiOutputFrameBuffer<'a> {
    display: Display,
    marker: ContravariantLifetime<'a>,
    dimensions: (u32, u32),
    color_attachments: Vec<(String, gl::types::GLuint)>,
}

impl<'a> MultiOutputFrameBuffer<'a> {
    /// Creates a new `MultiOutputFramebuffer`.
    ///
    /// # Panic
    ///
    /// Panics if all attachments don't have the same dimensions.
    ///
    pub fn new(display: &Display, color_attachments: &[(&str, &'a Texture2d)])
               -> MultiOutputFrameBuffer<'a>
    {
        let mut attachments = Vec::new();
        let mut dimensions = None;

        for &(name, texture) in color_attachments.iter() {
            let tex_dims = (texture.get_width(), texture.get_height().unwrap());

            if let Some(ref dimensions) = dimensions {
                if dimensions != &tex_dims {
                    panic!("All textures of a MultiOutputFrameBuffer must have \
                            the same dimensions");
                }
            }

            dimensions = Some(tex_dims);
            attachments.push((name.to_string(), texture.get_id()));
        }

        if dimensions.is_none() {
            panic!("Cannot pass an empty color_attachments when \
                    creating a MultiOutputFrameBuffer");
        }

        MultiOutputFrameBuffer {
            display: display.clone(),
            marker: ContravariantLifetime,
            dimensions: dimensions.unwrap(),
            color_attachments: attachments,
        }
    }

    fn build_attachments(&self, program: &Program) -> FramebufferAttachments {
        let mut colors = Vec::new();

        for &(ref name, texture) in self.color_attachments.iter() {
            let location = match program.get_frag_data_location(name.as_slice()) {
                Some(l) => l,
                None => panic!("The fragment output `{}` was not found in the program", name)
            };

            colors.push((location, fbo::Attachment::Texture(texture)));
        }

        FramebufferAttachments {
            colors: colors,
            depth: None,
            stencil: None,
        }
    }
}

/// Describes an attachment for a color buffer.
#[deriving(Copy, Clone)]
pub enum ColorAttachment<'a> {
    /// A texture.
    Texture2d(&'a Texture2d),
    /// A render buffer.
    RenderBuffer(&'a ::render_buffer::RenderBuffer),
}

/// Trait for objects that can be used as color attachments.
pub trait ToColorAttachment {
    /// Builds the `ColorAttachment`.
    fn to_color_attachment(&self) -> ColorAttachment;
}

/// Describes an attachment for a depth buffer.
#[deriving(Copy, Clone)]
pub enum DepthAttachment<'a> {
    /// A texture.
    Texture2d(&'a DepthTexture2d),
    /// A render buffer.
    RenderBuffer(&'a ::render_buffer::DepthRenderBuffer),
}

/// Trait for objects that can be used as depth attachments.
pub trait ToDepthAttachment {
    /// Builds the `DepthAttachment`.
    fn to_depth_attachment(&self) -> DepthAttachment;
}

/// Describes an attachment for a stencil buffer.
#[deriving(Copy, Clone)]
pub enum StencilAttachment<'a> {
    /// A texture.
    Texture2d(&'a StencilTexture2d),
    /// A render buffer.
    RenderBuffer(&'a ::render_buffer::StencilRenderBuffer),
}

/// Trait for objects that can be used as stencil attachments.
pub trait ToStencilAttachment {
    /// Builds the `StencilAttachment`.
    fn to_stencil_attachment(&self) -> StencilAttachment;
}

/// Describes an attachment for a depth and stencil buffer.
#[deriving(Copy, Clone)]
pub enum DepthStencilAttachment<'a> {
    /// A texture.
    Texture2d(&'a DepthStencilTexture2d),
    /// A render buffer.
    RenderBuffer(&'a ::render_buffer::DepthStencilRenderBuffer),
}

/// Trait for objects that can be used as depth and stencil attachments.
pub trait ToDepthStencilAttachment {
    /// Builds the `DepthStencilAttachment`.
    fn to_depth_stencil_attachment(&self) -> DepthStencilAttachment;
}
