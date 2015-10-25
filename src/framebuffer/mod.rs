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

# A note on restrictions

Some restrictions apply when you use framebuffers:

 - All textures must have an internal format that is renderable. Not all formats are supported.

 - All attachments must have the same number of samples, or must all have multisampling disabled.
   For example you can't create a texture with 4x multisampling, another texture with 2x
   multisampling, and draw on them simultaneously.

 - On old hardware all the framebuffer attachments must have the same dimensions (on more recent
   hardware the intersection between all the attachments is taken if all attachments don't have
   the same dimensions). You can use the `is_dimensions_mismatch_supported` function to check
   what the hardware supports.

 - You will get undefined results if you try to sample to a texture mipmap attached to the
   framebuffer that you are using. This is not enforced by glium as it depends on your shader's
   source code.

# Empty framebuffers

Modern OpenGL implementations support empty framebuffers. This is handled by glium with the
`EmptyFrameBuffer` struct.

You can check whether they are supported by calling `EmptyFrameBuffer::is_supported(&display)`.

# Layered framebuffers

Not yet supported

*/
use std::rc::Rc;
use smallvec::SmallVec;

use texture::Texture2d;
use texture::TextureAnyImage;
use TextureExt;

use backend::Facade;
use context::Context;
use CapabilitiesSource;
use version::Version;
use version::Api;

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

pub use self::default_fb::{DefaultFramebufferAttachment, DefaultFramebuffer};
pub use self::render_buffer::{RenderBuffer, RenderBufferAny, DepthRenderBuffer};
pub use self::render_buffer::{StencilRenderBuffer, DepthStencilRenderBuffer};
pub use self::render_buffer::CreationError as RenderBufferCreationError;
pub use fbo::is_dimensions_mismatch_supported;
pub use fbo::ValidationError;

mod default_fb;
mod render_buffer;

/// A framebuffer which has only one color attachment.
pub struct SimpleFrameBuffer<'a> {
    context: Rc<Context>,
    attachments: fbo::ValidatedAttachments<'a>,
}

impl<'a> SimpleFrameBuffer<'a> {
    /// Creates a `SimpleFrameBuffer` with a single color attachment and no depth
    /// nor stencil buffer.
    #[inline]
    pub fn new<F, C>(facade: &F, color: C) -> Result<SimpleFrameBuffer<'a>, ValidationError>
                     where C: ToColorAttachment<'a>, F: Facade
    {
        SimpleFrameBuffer::new_impl(facade, color.to_color_attachment(), None, None, None)
    }

    /// Creates a `SimpleFrameBuffer` with a single color attachment and a depth
    /// buffer, but no stencil buffer.
    #[inline]
    pub fn with_depth_buffer<F, C, D>(facade: &F, color: C, depth: D)
                                      -> Result<SimpleFrameBuffer<'a>, ValidationError>
                                      where C: ToColorAttachment<'a>,
                                            D: ToDepthAttachment<'a>, F: Facade
    {
        SimpleFrameBuffer::new_impl(facade, color.to_color_attachment(),
                                    Some(depth.to_depth_attachment()), None, None)
    }

    /// Creates a `SimpleFrameBuffer` with a single color attachment, a depth
    /// buffer, and a stencil buffer.
    #[inline]
    pub fn with_depth_and_stencil_buffer<F, C, D, S>(facade: &F, color: C, depth: D,
                                                     stencil: S)
                                                     -> Result<SimpleFrameBuffer<'a>,
                                                               ValidationError>
                                                     where C: ToColorAttachment<'a>,
                                                           D: ToDepthAttachment<'a>,
                                                           S: ToStencilAttachment<'a>, F: Facade
    {
        SimpleFrameBuffer::new_impl(facade, color.to_color_attachment(),
                                    Some(depth.to_depth_attachment()),
                                    Some(stencil.to_stencil_attachment()), None)
    }

    /// Creates a `SimpleFrameBuffer` with a single color attachment and a stencil
    /// buffer, but no depth buffer.
    #[inline]
    pub fn with_stencil_buffer<F, C, S>(facade: &F, color: C, stencil: S)
                                        -> Result<SimpleFrameBuffer<'a>, ValidationError>
                                        where C: ToColorAttachment<'a>, S: ToStencilAttachment<'a>,
                                              F: Facade
    {
        SimpleFrameBuffer::new_impl(facade, color.to_color_attachment(), None,
                                    Some(stencil.to_stencil_attachment()), None)
    }

    /// Creates a `SimpleFrameBuffer` with a single color attachment and a depth-stencil buffer.
    #[inline]
    pub fn with_depth_stencil_buffer<F, C, D>(facade: &F, color: C, depthstencil: D)
                                              -> Result<SimpleFrameBuffer<'a>, ValidationError>
                                              where C: ToColorAttachment<'a>,
                                                    D: ToDepthStencilAttachment<'a>, F: Facade
    {
        SimpleFrameBuffer::new_impl(facade, color.to_color_attachment(), None, None,
                                    Some(depthstencil.to_depth_stencil_attachment()))
    }


    fn new_impl<F>(facade: &F, color: ColorAttachment<'a>, depth: Option<DepthAttachment<'a>>,
                   stencil: Option<StencilAttachment<'a>>,
                   depthstencil: Option<DepthStencilAttachment<'a>>)
                   -> Result<SimpleFrameBuffer<'a>, ValidationError> where F: Facade
    {
        let color = match color {
            ColorAttachment::Texture(tex) => fbo::RegularAttachment::Texture(tex),
            ColorAttachment::RenderBuffer(buffer) => fbo::RegularAttachment::RenderBuffer(buffer),
        };

        let depth = depth.map(|depth| match depth {
            DepthAttachment::Texture(tex) => fbo::RegularAttachment::Texture(tex),
            DepthAttachment::RenderBuffer(buffer) => fbo::RegularAttachment::RenderBuffer(buffer),
        });

        let stencil = stencil.map(|stencil|  match stencil {
            StencilAttachment::Texture(tex) => fbo::RegularAttachment::Texture(tex),
            StencilAttachment::RenderBuffer(buffer) => fbo::RegularAttachment::RenderBuffer(buffer),
        });

        let depthstencil = depthstencil.map(|depthstencil| match depthstencil {
            DepthStencilAttachment::Texture(tex) => fbo::RegularAttachment::Texture(tex),
            DepthStencilAttachment::RenderBuffer(buffer) => fbo::RegularAttachment::RenderBuffer(buffer),
        });

        let attachments = fbo::FramebufferAttachments::Regular(fbo::FramebufferSpecificAttachments {
            colors: { let mut v = SmallVec::new(); v.push((0, color)); v },
            depth_stencil: if let (Some(depth), Some(stencil)) = (depth, stencil) {
                fbo::DepthStencilAttachments::DepthAndStencilAttachments(depth, stencil)
            } else if let Some(depth) = depth {
                fbo::DepthStencilAttachments::DepthAttachment(depth)
            } else if let Some(stencil) = stencil {
                fbo::DepthStencilAttachments::StencilAttachment(stencil)
            } else if let Some(depthstencil) = depthstencil {
                fbo::DepthStencilAttachments::DepthStencilAttachment(depthstencil)
            } else {
                fbo::DepthStencilAttachments::None
            }
        });

        let attachments = try!(attachments.validate(facade));

        Ok(SimpleFrameBuffer {
            context: facade.get_context().clone(),
            attachments: attachments,
        })
    }
}

impl<'a> Surface for SimpleFrameBuffer<'a> {
    #[inline]
    fn clear(&mut self, rect: Option<&Rect>, color: Option<(f32, f32, f32, f32)>, color_srgb: bool,
             depth: Option<f32>, stencil: Option<i32>)
    {
        ops::clear(&self.context, Some(&self.attachments), rect, color, color_srgb, depth, stencil);
    }

    #[inline]
    fn get_dimensions(&self) -> (u32, u32) {
        self.attachments.get_dimensions()
    }

    #[inline]
    fn get_depth_buffer_bits(&self) -> Option<u16> {
        self.attachments.get_depth_buffer_bits()
    }

    #[inline]
    fn get_stencil_buffer_bits(&self) -> Option<u16> {
        self.attachments.get_stencil_buffer_bits()
    }

    fn draw<'b, 'v, V, I, U>(&mut self, vb: V, ib: I, program: &::Program,
        uniforms: &U, draw_parameters: &::DrawParameters) -> Result<(), DrawError>
        where I: Into<::index::IndicesSource<'b>>, U: ::uniforms::Uniforms,
        V: ::vertex::MultiVerticesSource<'v>
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

        ops::draw(&self.context, Some(&self.attachments), vb,
                  ib.into(), program, uniforms, draw_parameters, self.get_dimensions())
    }

    #[inline]
    fn blit_color<S>(&self, source_rect: &Rect, target: &S, target_rect: &BlitTarget,
                     filter: uniforms::MagnifySamplerFilter) where S: Surface
    {
        target.blit_from_simple_framebuffer(self, source_rect, target_rect, filter)
    }

    #[inline]
    fn blit_from_frame(&self, source_rect: &Rect, target_rect: &BlitTarget,
                       filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, None, self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }

    #[inline]
    fn blit_from_simple_framebuffer(&self, source: &SimpleFrameBuffer,
                                    source_rect: &Rect, target_rect: &BlitTarget,
                                    filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, source.get_attachments(), self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }

    #[inline]
    fn blit_from_multioutput_framebuffer(&self, source: &MultiOutputFrameBuffer,
                                         source_rect: &Rect, target_rect: &BlitTarget,
                                         filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, source.get_attachments(), self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }
}

impl<'a> FboAttachments for SimpleFrameBuffer<'a> {
    #[inline]
    fn get_attachments(&self) -> Option<&fbo::ValidatedAttachments> {
        Some(&self.attachments)
    }
}

/// This struct is useless for the moment.
pub struct MultiOutputFrameBuffer<'a> {
    context: Rc<Context>,
    example_attachments: fbo::ValidatedAttachments<'a>,
    color_attachments: Vec<(String, fbo::RegularAttachment<'a>)>,
    depth_attachment: Option<fbo::RegularAttachment<'a>>,
    stencil_attachment: Option<fbo::RegularAttachment<'a>>,
}

impl<'a> MultiOutputFrameBuffer<'a> {
    /// Creates a new `MultiOutputFrameBuffer`.
    ///
    /// # Panic
    ///
    /// Panics if all attachments don't have the same dimensions.
    #[inline]
    pub fn new<F>(facade: &F, color_attachments: &[(&str, &'a Texture2d)])
                  -> Result<MultiOutputFrameBuffer<'a>, ValidationError> where F: Facade
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
    #[inline]
    pub fn with_depth_buffer<F, D>(facade: &F, color_attachments: &[(&str, &'a Texture2d)],
                                   depth: D) -> Result<MultiOutputFrameBuffer<'a>, ValidationError>
                                   where D: ToDepthAttachment<'a>, F: Facade
    {
        MultiOutputFrameBuffer::new_impl(facade, color_attachments, Some(depth),
                                         None::<&StencilRenderBuffer>)
    }

    fn new_impl<F, D, S>(facade: &F, color: &[(&str, &'a Texture2d)],
                         depth: Option<D>, stencil: Option<S>)
                         -> Result<MultiOutputFrameBuffer<'a>, ValidationError>
                         where D: ToDepthAttachment<'a>, F: Facade
    {
        let color = color.iter().map(|&(name, tex)| {
            let atch = tex.to_color_attachment();
            let atch = if let ColorAttachment::Texture(t) = atch { t } else { panic!() };
            (name.to_owned(), fbo::RegularAttachment::Texture(atch))
        }).collect::<Vec<_>>();

        let example_color = {
            let mut v = SmallVec::new();
            for e in color.iter().enumerate().map(|(index, &(_, tex))| { (index as u32, tex) }) {
                v.push(e);
            }
            v
        };

        let depth = depth.map(|depth| match depth.to_depth_attachment() {
            DepthAttachment::Texture(tex) => fbo::RegularAttachment::Texture(tex),
            DepthAttachment::RenderBuffer(buffer) => fbo::RegularAttachment::RenderBuffer(buffer),
        });

        let stencil = None;/*stencil.map(|stencil|  match color {
            StencilAttachment::Texture(tex) => fbo::RegularAttachment::TextureLayer {
                texture: tex.get_texture(), layer: tex.get_layer(), level: tex.get_level()
            },
            StencilAttachment::RenderBuffer(buffer) => fbo::RegularAttachment::RenderBuffer(buffer),
        });*/       // TODO:

        let depthstencil = None;/*depthstencil.map(|depthstencil| match color {
            DepthStencilAttachment::Texture(tex) => fbo::RegularAttachment::TextureLayer {
                texture: tex.get_texture(), layer: tex.get_layer(), level: tex.get_level()
            },
            DepthStencilAttachment::RenderBuffer(buffer) => fbo::RegularAttachment::RenderBuffer(buffer),
        });*/       // TODO:

        let example_attachments = try!(fbo::FramebufferAttachments::Regular(fbo::FramebufferSpecificAttachments {
            colors: example_color,
            depth_stencil: if let (Some(depth), Some(stencil)) = (depth, stencil) {
                fbo::DepthStencilAttachments::DepthAndStencilAttachments(depth, stencil)
            } else if let Some(depth) = depth {
                fbo::DepthStencilAttachments::DepthAttachment(depth)
            } else if let Some(stencil) = stencil {
                fbo::DepthStencilAttachments::StencilAttachment(stencil)
            } else if let Some(depthstencil) = depthstencil {
                fbo::DepthStencilAttachments::DepthStencilAttachment(depthstencil)
            } else {
                fbo::DepthStencilAttachments::None
            }
        }).validate(facade));

        Ok(MultiOutputFrameBuffer {
            context: facade.get_context().clone(),
            example_attachments: example_attachments,
            color_attachments: color,
            depth_attachment: depth,
            stencil_attachment: stencil,
        })
    }

    fn build_attachments(&self, program: &Program) -> fbo::ValidatedAttachments {
        let mut colors = SmallVec::new();

        for &(ref name, attachment) in self.color_attachments.iter() {
            let location = match program.get_frag_data_location(&name) {
                Some(l) => l,
                None => panic!("The fragment output `{}` was not found in the program", name)
            };

            colors.push((location, attachment));
        }

        fbo::FramebufferAttachments::Regular(fbo::FramebufferSpecificAttachments {
            colors: colors,
            depth_stencil: if let Some(depth) = self.depth_attachment {
                fbo::DepthStencilAttachments::DepthAttachment(depth)
            } else {        // FIXME: other cases
                fbo::DepthStencilAttachments::None
            },
        }).validate(&self.context).unwrap()
    }
}

impl<'a> Surface for MultiOutputFrameBuffer<'a> {
    #[inline]
    fn clear(&mut self, rect: Option<&Rect>, color: Option<(f32, f32, f32, f32)>, color_srgb: bool,
             depth: Option<f32>, stencil: Option<i32>)
    {
        ops::clear(&self.context, Some(&self.example_attachments), rect,
                   color, color_srgb, depth, stencil);
    }

    #[inline]
    fn get_dimensions(&self) -> (u32, u32) {
        self.example_attachments.get_dimensions()
    }

    #[inline]
    fn get_depth_buffer_bits(&self) -> Option<u16> {
        self.example_attachments.get_depth_buffer_bits()
    }

    #[inline]
    fn get_stencil_buffer_bits(&self) -> Option<u16> {
        self.example_attachments.get_stencil_buffer_bits()
    }

    fn draw<'i, 'v, V, I, U>(&mut self, vb: V, ib: I, program: &::Program,
        uniforms: &U, draw_parameters: &::DrawParameters) -> Result<(), DrawError>
        where I: Into<::index::IndicesSource<'i>>,
        U: ::uniforms::Uniforms, V: ::vertex::MultiVerticesSource<'v>
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

        ops::draw(&self.context, Some(&self.build_attachments(program)), vb,
                  ib.into(), program, uniforms, draw_parameters, self.get_dimensions())
    }

    #[inline]
    fn blit_color<S>(&self, source_rect: &Rect, target: &S, target_rect: &BlitTarget,
                     filter: uniforms::MagnifySamplerFilter) where S: Surface
    {
        target.blit_from_multioutput_framebuffer(self, source_rect, target_rect, filter)
    }

    #[inline]
    fn blit_from_frame(&self, source_rect: &Rect, target_rect: &BlitTarget,
                       filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, None, self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }

    #[inline]
    fn blit_from_simple_framebuffer(&self, source: &SimpleFrameBuffer,
                                    source_rect: &Rect, target_rect: &BlitTarget,
                                    filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, source.get_attachments(), self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }

    #[inline]
    fn blit_from_multioutput_framebuffer(&self, source: &MultiOutputFrameBuffer,
                                         source_rect: &Rect, target_rect: &BlitTarget,
                                         filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, source.get_attachments(), self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }
}

impl<'a> FboAttachments for MultiOutputFrameBuffer<'a> {
    #[inline]
    fn get_attachments(&self) -> Option<&fbo::ValidatedAttachments> {
        unimplemented!();
    }
}

/// A framebuffer with no attachment at all.
///
/// Note that this is only supported on recent hardware.
pub struct EmptyFrameBuffer {
    context: Rc<Context>,
    attachments: fbo::ValidatedAttachments<'static>,
}

impl<'a> EmptyFrameBuffer {
    /// Returns true if empty framebuffers are supported by the backend.
    pub fn is_supported<C>(context: &C) -> bool where C: CapabilitiesSource {
        context.get_version() >= &Version(Api::Gl, 4, 3) ||
        context.get_version() >= &Version(Api::GlEs, 3, 1) ||
        context.get_extensions().gl_arb_framebuffer_no_attachments
    }

    /// Returns true if layered empty framebuffers are supported by the backend.
    pub fn is_layered_supported<C>(context: &C) -> bool where C: CapabilitiesSource {
        context.get_version() >= &Version(Api::Gl, 4, 3) ||
        context.get_version() >= &Version(Api::GlEs, 3, 2) ||
        context.get_extensions().gl_arb_framebuffer_no_attachments
    }

    /// Returns the maximum width of empty framebuffers that the backend supports, or `None` if
    /// empty framebuffers are not supported.
    pub fn get_max_supported_width<C>(context: &C) -> Option<u32> where C: CapabilitiesSource {
        context.get_capabilities().max_framebuffer_width.map(|v| v as u32)
    }

    /// Returns the maximum height of empty framebuffers that the backend supports, or `None` if
    /// empty framebuffers are not supported.
    pub fn get_max_supported_height<C>(context: &C) -> Option<u32> where C: CapabilitiesSource {
        context.get_capabilities().max_framebuffer_height.map(|v| v as u32)
    }

    /// Returns the maximum number of samples of empty framebuffers that the backend supports,
    /// or `None` if empty framebuffers are not supported.
    pub fn get_max_supported_samples<C>(context: &C) -> Option<u32> where C: CapabilitiesSource {
        context.get_capabilities().max_framebuffer_samples.map(|v| v as u32)
    }

    /// Returns the maximum number of layers of empty framebuffers that the backend supports,
    /// or `None` if layered empty framebuffers are not supported.
    pub fn get_max_supported_layers<C>(context: &C) -> Option<u32> where C: CapabilitiesSource {
        context.get_capabilities().max_framebuffer_layers.map(|v| v as u32)
    }

    /// Creates a `EmptyFrameBuffer`.
    ///
    /// # Panic
    ///
    /// Panicks if `layers` or `samples` is equal to `Some(0)`.
    ///
    #[inline]
    pub fn new<F>(facade: &F, width: u32, height: u32, layers: Option<u32>,
                  samples: Option<u32>, fixed_samples: bool)
                  -> Result<EmptyFrameBuffer, ValidationError> where F: Facade
    {
        let context = facade.get_context();

        let attachments = fbo::FramebufferAttachments::Empty {
            width: width,
            height: height,
            layers: layers,
            samples: samples,
            fixed_samples: fixed_samples,
        };

        let attachments = try!(attachments.validate(context));

        Ok(EmptyFrameBuffer {
            context: context.clone(),
            attachments: attachments,
        })
    }
}

impl Surface for EmptyFrameBuffer {
    #[inline]
    fn clear(&mut self, rect: Option<&Rect>, color: Option<(f32, f32, f32, f32)>, color_srgb: bool,
             depth: Option<f32>, stencil: Option<i32>)
    {
        ops::clear(&self.context, Some(&self.attachments), rect, color, color_srgb, depth, stencil);
    }

    #[inline]
    fn get_dimensions(&self) -> (u32, u32) {
        self.attachments.get_dimensions()
    }

    #[inline]
    fn get_depth_buffer_bits(&self) -> Option<u16> {
        None
    }

    #[inline]
    fn get_stencil_buffer_bits(&self) -> Option<u16> {
        None
    }

    fn draw<'b, 'v, V, I, U>(&mut self, vb: V, ib: I, program: &::Program,
        uniforms: &U, draw_parameters: &::DrawParameters) -> Result<(), DrawError>
        where I: Into<::index::IndicesSource<'b>>, U: ::uniforms::Uniforms,
        V: ::vertex::MultiVerticesSource<'v>
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

        ops::draw(&self.context, Some(&self.attachments), vb,
                  ib.into(), program, uniforms, draw_parameters, self.get_dimensions())
    }

    #[inline]
    fn blit_color<S>(&self, source_rect: &Rect, target: &S, target_rect: &BlitTarget,
                     filter: uniforms::MagnifySamplerFilter) where S: Surface
    {
        unimplemented!()        // TODO:
    }

    #[inline]
    fn blit_from_frame(&self, source_rect: &Rect, target_rect: &BlitTarget,
                       filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, None, self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }

    #[inline]
    fn blit_from_simple_framebuffer(&self, source: &SimpleFrameBuffer,
                                    source_rect: &Rect, target_rect: &BlitTarget,
                                    filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, source.get_attachments(), self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }

    #[inline]
    fn blit_from_multioutput_framebuffer(&self, source: &MultiOutputFrameBuffer,
                                         source_rect: &Rect, target_rect: &BlitTarget,
                                         filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, source.get_attachments(), self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }
}

impl FboAttachments for EmptyFrameBuffer {
    #[inline]
    fn get_attachments(&self) -> Option<&fbo::ValidatedAttachments> {
        Some(&self.attachments)
    }
}

/// Describes an attachment for a color buffer.
#[derive(Copy, Clone)]
pub enum ColorAttachment<'a> {
    /// A texture.
    Texture(TextureAnyImage<'a>),
    /// A render buffer.
    RenderBuffer(&'a RenderBuffer),
}

/// Trait for objects that can be used as color attachments.
pub trait ToColorAttachment<'a> {
    /// Builds the `ColorAttachment`.
    fn to_color_attachment(self) -> ColorAttachment<'a>;
}

/// Describes an attachment for a depth buffer.
#[derive(Copy, Clone)]
pub enum DepthAttachment<'a> {
    /// A texture.
    Texture(TextureAnyImage<'a>),
    /// A render buffer.
    RenderBuffer(&'a DepthRenderBuffer),
}

/// Trait for objects that can be used as depth attachments.
pub trait ToDepthAttachment<'a> {
    /// Builds the `DepthAttachment`.
    fn to_depth_attachment(self) -> DepthAttachment<'a>;
}

/// Describes an attachment for a stencil buffer.
#[derive(Copy, Clone)]
pub enum StencilAttachment<'a> {
    /// A texture.
    Texture(TextureAnyImage<'a>),
    /// A render buffer.
    RenderBuffer(&'a StencilRenderBuffer),
}

/// Trait for objects that can be used as stencil attachments.
pub trait ToStencilAttachment<'a> {
    /// Builds the `StencilAttachment`.
    fn to_stencil_attachment(self) -> StencilAttachment<'a>;
}

/// Describes an attachment for a depth and stencil buffer.
#[derive(Copy, Clone)]
pub enum DepthStencilAttachment<'a> {
    /// A texture.
    Texture(TextureAnyImage<'a>),
    /// A render buffer.
    RenderBuffer(&'a DepthStencilRenderBuffer),
}

/// Trait for objects that can be used as depth and stencil attachments.
pub trait ToDepthStencilAttachment<'a> {
    /// Builds the `DepthStencilAttachment`.
    fn to_depth_stencil_attachment(self) -> DepthStencilAttachment<'a>;
}
