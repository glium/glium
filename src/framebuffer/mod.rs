/*!
Framebuffers allow you to customize the color, depth and stencil buffers you will draw on.

In order to draw on a texture, use a `SimpleFrameBuffer`. This framebuffer is compatible with
shaders that write to `gl_FragColor`.

```no_run
# let display: glium::Display = unsafe { ::std::mem::uninitialized() };
# let texture: glium::texture::Texture2d = unsafe { ::std::mem::uninitialized() };
let framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display, &texture);
// framebuffer.draw(...);    // draws over `texture`
```

If, however, your shader wants to write to multiple color buffers at once, you must use
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
use std::rc::Rc;

use texture::Texture2d;
use texture::TextureAnyMipmap;

use backend::Facade;
use context::Context;

use FboAttachments;
use Rect;
use BlitTarget;
use ContextExt;
use ToGlEnum;
use ops;
use uniforms;

use {Program, Surface};
use DrawError;

use {fbo, gl};

pub use self::render_buffer::{RenderBuffer, RenderBufferAny, DepthRenderBuffer};
pub use self::render_buffer::{StencilRenderBuffer, DepthStencilRenderBuffer};

mod render_buffer;

/// A framebuffer which has only one color attachment.
pub struct SimpleFrameBuffer<'a> {
    context: Rc<Context>,
    attachments: fbo::ValidatedAttachments<'a>,
}

impl<'a> SimpleFrameBuffer<'a> {
    /// Creates a `SimpleFrameBuffer` with a single color attachment and no depth
    /// nor stencil buffer.
    pub fn new<F, C>(facade: &F, color: &'a C) -> SimpleFrameBuffer<'a>
                  where C: ToColorAttachment, F: Facade
    {
        SimpleFrameBuffer::new_impl(facade, color.to_color_attachment(), None, None, None)
    }

    /// Creates a `SimpleFrameBuffer` with a single color attachment and a depth
    /// buffer, but no stencil buffer.
    pub fn with_depth_buffer<F, C, D>(facade: &F, color: &'a C, depth: &'a D)
                                      -> SimpleFrameBuffer<'a>
                                      where C: ToColorAttachment, D: ToDepthAttachment, F: Facade
    {
        SimpleFrameBuffer::new_impl(facade, color.to_color_attachment(),
                                    Some(depth.to_depth_attachment()), None, None)
    }

    /// Creates a `SimpleFrameBuffer` with a single color attachment, a depth
    /// buffer, and a stencil buffer.
    pub fn with_depth_and_stencil_buffer<F, C, D, S>(facade: &F, color: &'a C, depth: &'a D,
                                                     stencil: &'a S) -> SimpleFrameBuffer<'a>
                                                     where C: ToColorAttachment,
                                                           D: ToDepthAttachment,
                                                           S: ToStencilAttachment, F: Facade
    {
        SimpleFrameBuffer::new_impl(facade, color.to_color_attachment(),
                                    Some(depth.to_depth_attachment()),
                                    Some(stencil.to_stencil_attachment()), None)
    }

    /// Creates a `SimpleFrameBuffer` with a single color attachment and a stencil
    /// buffer, but no depth buffer.
    pub fn with_stencil_buffer<F, C, S>(facade: &F, color: &'a C, stencil: &'a S)
                                        -> SimpleFrameBuffer<'a>
                                        where C: ToColorAttachment, S: ToStencilAttachment,
                                              F: Facade
    {
        SimpleFrameBuffer::new_impl(facade, color.to_color_attachment(), None,
                                    Some(stencil.to_stencil_attachment()), None)
    }

    /// Creates a `SimpleFrameBuffer` with a single color attachment and a depth-stencil buffer.
    pub fn with_depth_stencil_buffer<F, C, D>(facade: &F, color: &'a C, depthstencil: &'a D)
                                              -> SimpleFrameBuffer<'a>
                                              where C: ToColorAttachment,
                                                    D: ToDepthStencilAttachment, F: Facade
    {
        SimpleFrameBuffer::new_impl(facade, color.to_color_attachment(), None, None,
                                    Some(depthstencil.to_depth_stencil_attachment()))
    }


    fn new_impl<F>(facade: &F, color: ColorAttachment<'a>, depth: Option<DepthAttachment<'a>>,
                   stencil: Option<StencilAttachment<'a>>,
                   depthstencil: Option<DepthStencilAttachment<'a>>)
                   -> SimpleFrameBuffer<'a> where F: Facade
    {
        let color = match color {
            ColorAttachment::Texture(tex) => fbo::Attachment::TextureLayer {
                texture: tex.get_texture(), layer: tex.get_layer(), level: tex.get_level()
            },
            ColorAttachment::RenderBuffer(buffer) => fbo::Attachment::RenderBuffer(buffer),
        };

        let depth = depth.map(|depth| match depth {
            DepthAttachment::Texture(tex) => fbo::Attachment::TextureLayer {
                texture: tex.get_texture(), layer: tex.get_layer(), level: tex.get_level()
            },
            DepthAttachment::RenderBuffer(buffer) => fbo::Attachment::RenderBuffer(buffer),
        });

        let stencil = stencil.map(|stencil|  match stencil {
            StencilAttachment::Texture(tex) => fbo::Attachment::TextureLayer {
                texture: tex.get_texture(), layer: tex.get_layer(), level: tex.get_level()
            },
            StencilAttachment::RenderBuffer(buffer) => fbo::Attachment::RenderBuffer(buffer),
        });

        let depthstencil = depthstencil.map(|depthstencil| match depthstencil {
            DepthStencilAttachment::Texture(tex) => fbo::Attachment::TextureLayer {
                texture: tex.get_texture(), layer: tex.get_layer(), level: tex.get_level()
            },
            DepthStencilAttachment::RenderBuffer(buffer) => fbo::Attachment::RenderBuffer(buffer),
        });

        let attachments = fbo::FramebufferAttachments {
            colors: vec![(0, color)],
            depth_stencil: if let (Some(depth), Some(stencil)) = (depth, stencil) {
                fbo::FramebufferDepthStencilAttachments::DepthAndStencilAttachments(depth, stencil)
            } else if let Some(depth) = depth {
                fbo::FramebufferDepthStencilAttachments::DepthAttachment(depth)
            } else if let Some(stencil) = stencil {
                fbo::FramebufferDepthStencilAttachments::StencilAttachment(stencil)
            } else if let Some(depthstencil) = depthstencil {
                fbo::FramebufferDepthStencilAttachments::DepthStencilAttachment(depthstencil)
            } else {
                fbo::FramebufferDepthStencilAttachments::None
            }
        };

        let attachments = attachments.validate().unwrap();

        SimpleFrameBuffer {
            context: facade.get_context().clone(),
            attachments: attachments,
        }
    }
}

impl<'a> Surface for SimpleFrameBuffer<'a> {
    fn clear(&mut self, rect: Option<&Rect>, color: Option<(f32, f32, f32, f32)>,
             depth: Option<f32>, stencil: Option<i32>)
    {
        ops::clear(&self.context, Some(&self.attachments), rect, color, depth, stencil);
    }

    fn get_dimensions(&self) -> (u32, u32) {
        self.attachments.get_dimensions()
    }

    fn get_depth_buffer_bits(&self) -> Option<u16> {
        self.attachments.get_depth_buffer_bits()
    }

    fn get_stencil_buffer_bits(&self) -> Option<u16> {
        self.attachments.get_stencil_buffer_bits()
    }

    fn draw<'b, 'v, V, I, U>(&mut self, vb: V, ib: I, program: &::Program,
        uniforms: &U, draw_parameters: &::DrawParameters) -> Result<(), DrawError>
        where I: Into<::index::IndicesSource<'b>>, U: ::uniforms::Uniforms,
        V: ::vertex::MultiVerticesSource<'v>
    {
        if !self.has_depth_buffer() && (draw_parameters.depth_test.requires_depth_buffer() ||
                        draw_parameters.depth_write)
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

        ops::draw(&self.context, Some(&self.attachments), vb,
                  ib.into(), program, uniforms, draw_parameters, self.get_dimensions())
    }

    fn blit_color<S>(&self, source_rect: &Rect, target: &S, target_rect: &BlitTarget,
                     filter: uniforms::MagnifySamplerFilter) where S: Surface
    {
        target.blit_from_simple_framebuffer(self, source_rect, target_rect, filter)
    }

    fn blit_from_frame(&self, source_rect: &Rect, target_rect: &BlitTarget,
                       filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, None, self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }

    fn blit_from_simple_framebuffer(&self, source: &SimpleFrameBuffer,
                                    source_rect: &Rect, target_rect: &BlitTarget,
                                    filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, source.get_attachments(), self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }

    fn blit_from_multioutput_framebuffer(&self, source: &MultiOutputFrameBuffer,
                                         source_rect: &Rect, target_rect: &BlitTarget,
                                         filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, source.get_attachments(), self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }
}

impl<'a> FboAttachments for SimpleFrameBuffer<'a> {
    fn get_attachments(&self) -> Option<&fbo::ValidatedAttachments> {
        Some(&self.attachments)
    }
}

/// This struct is useless for the moment.
pub struct MultiOutputFrameBuffer<'a> {
    context: Rc<Context>,
    example_attachments: fbo::ValidatedAttachments<'a>,
    color_attachments: Vec<(String, fbo::Attachment<'a>)>,
    depth_attachment: Option<fbo::Attachment<'a>>,
    stencil_attachment: Option<fbo::Attachment<'a>>,
}

impl<'a> MultiOutputFrameBuffer<'a> {
    /// Creates a new `MultiOutputFrameBuffer`.
    ///
    /// # Panic
    ///
    /// Panics if all attachments don't have the same dimensions.
    pub fn new<F>(facade: &F, color_attachments: &[(&str, &'a Texture2d)])
                  -> MultiOutputFrameBuffer<'a> where F: Facade
    {
        MultiOutputFrameBuffer::new_impl(facade, color_attachments,
                                         None::<&DepthRenderBuffer>,
                                         None::<&StencilRenderBuffer>)
    }

    /// Creates a `MultiOutputFrameBuffer` with a depth buffer.
    ///
    /// # Panic
    ///
    /// Panics if all attachments don't have the same dimensions.
    pub fn with_depth_buffer<F, D>(facade: &F, color_attachments: &[(&str, &'a Texture2d)],
                                   depth: &'a D) -> MultiOutputFrameBuffer<'a>
                                   where D: ToDepthAttachment, F: Facade
    {
        MultiOutputFrameBuffer::new_impl(facade, color_attachments, Some(depth),
                                         None::<&StencilRenderBuffer>)
    }

    fn new_impl<F, D, S>(facade: &F, color: &[(&str, &'a Texture2d)],
                         depth: Option<&'a D>, stencil: Option<&'a S>)
                         -> MultiOutputFrameBuffer<'a> where D: ToDepthAttachment, F: Facade
    {
        let color = color.iter().map(|&(name, tex)| {
            (name.to_string(), fbo::Attachment::TextureLayer {
                texture: tex, layer: 0, level: 0
            })
        }).collect::<Vec<_>>();

        let example_color = color.iter().enumerate().map(|(index, &(_, tex))| {
            (index as u32, tex)
        }).collect::<Vec<_>>();

        let depth = depth.map(|depth| match depth.to_depth_attachment() {
            DepthAttachment::Texture(tex) => fbo::Attachment::TextureLayer {
                texture: tex.get_texture(), layer: tex.get_layer(), level: tex.get_level()
            },
            DepthAttachment::RenderBuffer(buffer) => fbo::Attachment::RenderBuffer(buffer),
        });

        let stencil = None;/*stencil.map(|stencil|  match color {
            StencilAttachment::Texture(tex) => fbo::Attachment::TextureLayer {
                texture: tex.get_texture(), layer: tex.get_layer(), level: tex.get_level()
            },
            StencilAttachment::RenderBuffer(buffer) => fbo::Attachment::RenderBuffer(buffer),
        });*/       // TODO: 

        let depthstencil = None;/*depthstencil.map(|depthstencil| match color {
            DepthStencilAttachment::Texture(tex) => fbo::Attachment::TextureLayer {
                texture: tex.get_texture(), layer: tex.get_layer(), level: tex.get_level()
            },
            DepthStencilAttachment::RenderBuffer(buffer) => fbo::Attachment::RenderBuffer(buffer),
        });*/       // TODO: 

        let example_attachments = fbo::FramebufferAttachments {
            colors: example_color,
            depth_stencil: if let (Some(depth), Some(stencil)) = (depth, stencil) {
                fbo::FramebufferDepthStencilAttachments::DepthAndStencilAttachments(depth, stencil)
            } else if let Some(depth) = depth {
                fbo::FramebufferDepthStencilAttachments::DepthAttachment(depth)
            } else if let Some(stencil) = stencil {
                fbo::FramebufferDepthStencilAttachments::StencilAttachment(stencil)
            } else if let Some(depthstencil) = depthstencil {
                fbo::FramebufferDepthStencilAttachments::DepthStencilAttachment(depthstencil)
            } else {
                fbo::FramebufferDepthStencilAttachments::None
            }
        }.validate().unwrap();

        MultiOutputFrameBuffer {
            context: facade.get_context().clone(),
            example_attachments: example_attachments,
            color_attachments: color,
            depth_attachment: depth,
            stencil_attachment: stencil,
        }
    }

    fn build_attachments(&self, program: &Program) -> fbo::ValidatedAttachments {
        let mut colors = Vec::new();

        for &(ref name, attachment) in self.color_attachments.iter() {
            let location = match program.get_frag_data_location(&name) {
                Some(l) => l,
                None => panic!("The fragment output `{}` was not found in the program", name)
            };

            colors.push((location, attachment));
        }

        fbo::FramebufferAttachments {
            colors: colors,
            depth_stencil: if let Some(depth) = self.depth_attachment {
                fbo::FramebufferDepthStencilAttachments::DepthAttachment(depth)
            } else {        // FIXME: other cases
                fbo::FramebufferDepthStencilAttachments::None
            },
        }.validate().unwrap()
    }
}

impl<'a> Surface for MultiOutputFrameBuffer<'a> {
    fn clear(&mut self, rect: Option<&Rect>, color: Option<(f32, f32, f32, f32)>,
             depth: Option<f32>, stencil: Option<i32>)
    {
        ops::clear(&self.context, Some(&self.example_attachments), rect,
                   color, depth, stencil);
    }

    fn get_dimensions(&self) -> (u32, u32) {
        self.example_attachments.get_dimensions()
    }

    fn get_depth_buffer_bits(&self) -> Option<u16> {
        self.example_attachments.get_depth_buffer_bits()
    }

    fn get_stencil_buffer_bits(&self) -> Option<u16> {
        self.example_attachments.get_stencil_buffer_bits()
    }

    fn draw<'i, 'v, V, I, U>(&mut self, vb: V, ib: I, program: &::Program,
        uniforms: &U, draw_parameters: &::DrawParameters) -> Result<(), DrawError>
        where I: Into<::index::IndicesSource<'i>>,
        U: ::uniforms::Uniforms, V: ::vertex::MultiVerticesSource<'v>
    {
        if !self.has_depth_buffer() && (draw_parameters.depth_test.requires_depth_buffer() ||
                draw_parameters.depth_write)
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

        ops::draw(&self.context, Some(&self.build_attachments(program)), vb,
                  ib.into(), program, uniforms, draw_parameters, self.get_dimensions())
    }

    fn blit_color<S>(&self, source_rect: &Rect, target: &S, target_rect: &BlitTarget,
                     filter: uniforms::MagnifySamplerFilter) where S: Surface
    {
        target.blit_from_multioutput_framebuffer(self, source_rect, target_rect, filter)
    }

    fn blit_from_frame(&self, source_rect: &Rect, target_rect: &BlitTarget,
                       filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, None, self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }

    fn blit_from_simple_framebuffer(&self, source: &SimpleFrameBuffer,
                                    source_rect: &Rect, target_rect: &BlitTarget,
                                    filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, source.get_attachments(), self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }

    fn blit_from_multioutput_framebuffer(&self, source: &MultiOutputFrameBuffer,
                                         source_rect: &Rect, target_rect: &BlitTarget,
                                         filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, source.get_attachments(), self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }
}

impl<'a> FboAttachments for MultiOutputFrameBuffer<'a> {
    fn get_attachments(&self) -> Option<&fbo::ValidatedAttachments> {
        unimplemented!();
    }
}

/// Describes an attachment for a color buffer.
#[derive(Copy, Clone)]
pub enum ColorAttachment<'a> {
    /// A texture.
    Texture(TextureAnyMipmap<'a>),
    /// A render buffer.
    RenderBuffer(&'a RenderBuffer),
}

/// Trait for objects that can be used as color attachments.
pub trait ToColorAttachment {
    /// Builds the `ColorAttachment`.
    fn to_color_attachment(&self) -> ColorAttachment;
}

/// Describes an attachment for a depth buffer.
#[derive(Copy, Clone)]
pub enum DepthAttachment<'a> {
    /// A texture.
    Texture(TextureAnyMipmap<'a>),
    /// A render buffer.
    RenderBuffer(&'a DepthRenderBuffer),
}

/// Trait for objects that can be used as depth attachments.
pub trait ToDepthAttachment {
    /// Builds the `DepthAttachment`.
    fn to_depth_attachment(&self) -> DepthAttachment;
}

/// Describes an attachment for a stencil buffer.
#[derive(Copy, Clone)]
pub enum StencilAttachment<'a> {
    /// A texture.
    Texture(TextureAnyMipmap<'a>),
    /// A render buffer.
    RenderBuffer(&'a StencilRenderBuffer),
}

/// Trait for objects that can be used as stencil attachments.
pub trait ToStencilAttachment {
    /// Builds the `StencilAttachment`.
    fn to_stencil_attachment(&self) -> StencilAttachment;
}

/// Describes an attachment for a depth and stencil buffer.
#[derive(Copy, Clone)]
pub enum DepthStencilAttachment<'a> {
    /// A texture.
    Texture(TextureAnyMipmap<'a>),
    /// A render buffer.
    RenderBuffer(&'a DepthStencilRenderBuffer),
}

/// Trait for objects that can be used as depth and stencil attachments.
pub trait ToDepthStencilAttachment {
    /// Builds the `DepthStencilAttachment`.
    fn to_depth_stencil_attachment(&self) -> DepthStencilAttachment;
}
