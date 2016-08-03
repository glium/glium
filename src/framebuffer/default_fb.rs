//! Contains everything related to the default framebuffer.

use std::rc::Rc;
use TextureExt;

use backend::Facade;
use context::Context;

use DrawParameters;
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
use framebuffer;
use index;
use vertex;

/// One of the color attachments on the default framebuffer.
#[derive(Copy, Clone, Debug)]
pub enum DefaultFramebufferAttachment {
    /// The backbuffer for the left eye. Equivalent to the backbuffer if stereoscopy is disabled.
    BackLeft,
    /// The backbuffer for the right eye. May not be present.
    BackRight,
    /// The frontbuffer for the left eye. Equivalent to the frontbuffer if stereoscopy is disabled.
    /// May not be accessible.
    FrontLeft,
    /// The frontbuffer for the right eye. May not be present or accessible.
    FrontRight,
}

/// A framebuffer which has only one color attachment.
pub struct DefaultFramebuffer {
    context: Rc<Context>,
    attachment: DefaultFramebufferAttachment,
}

impl DefaultFramebuffer {
    /// Creates a `DefaultFramebuffer` with the back left buffer.
    #[inline]
    pub fn back_left<F: ?Sized>(facade: &F) -> DefaultFramebuffer where F: Facade {
        DefaultFramebuffer {
            context: facade.get_context().clone(),
            attachment: DefaultFramebufferAttachment::BackLeft,
        }
    }
}

impl Surface for DefaultFramebuffer {
    #[inline]
    fn clear(&mut self, rect: Option<&Rect>, color: Option<(f32, f32, f32, f32)>, color_srgb: bool,
             depth: Option<f32>, stencil: Option<i32>)
    {
        // TODO: wrong attachment
        ops::clear(&self.context, None, None, color, color_srgb, depth, stencil);
    }

    fn get_dimensions(&self) -> (u32, u32) {
        self.context.get_framebuffer_dimensions()
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

        // TODO: wrong attachment
        ops::draw(&self.context, None, vertex_buffer, index_buffer.into(), program,
                  uniforms, draw_parameters, self.get_dimensions())
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

impl FboAttachments for DefaultFramebuffer {
    #[inline]
    fn get_attachments(&self) -> Option<&fbo::ValidatedAttachments> {
        None
    }
}
