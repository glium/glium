/*!

A render buffer is similar to a texture, but is optimized for usage as a draw target.

Contrary to a texture, you can't sample nor modify the content of a render buffer.
You should prefer render buffers over textures when you know that you don't need to read or modify
the data of the render buffer.

*/
use std::rc::Rc;
use std::ops::{Deref, DerefMut};
use std::fmt;
use std::error::Error;

use framebuffer::{ColorAttachment, ToColorAttachment};
use framebuffer::{DepthAttachment, ToDepthAttachment};
use framebuffer::{StencilAttachment, ToStencilAttachment};
use framebuffer::{DepthStencilAttachment, ToDepthStencilAttachment};
use texture::{UncompressedFloatFormat, DepthFormat, StencilFormat, DepthStencilFormat, TextureKind};

use image_format;

use gl;
use GlObject;
use fbo::FramebuffersContainer;
use backend::Facade;
use context::Context;
use ContextExt;
use version::Version;
use version::Api;

/// Error while creating a render buffer.
#[derive(Copy, Clone, Debug)]
pub enum CreationError {
    /// The requested format is not supported.
    FormatNotSupported,
}

impl fmt::Display for CreationError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

impl Error for CreationError {
    fn description(&self) -> &str {
        use self::CreationError::*;
        match *self {
            FormatNotSupported => "The requested format is not supported",
        }
    }
}

impl From<image_format::FormatNotSupportedError> for CreationError {
    fn from(_: image_format::FormatNotSupportedError) -> CreationError {
        CreationError::FormatNotSupported
    }
}

/// A render buffer is similar to a texture, but is optimized for usage as a draw target.
///
/// Contrary to a texture, you can't sample or modify the content of the `RenderBuffer`.
pub struct RenderBuffer {
    buffer: RenderBufferAny,
}

impl RenderBuffer {
    /// Builds a new render buffer.
    pub fn new<F: ?Sized>(facade: &F, format: UncompressedFloatFormat, width: u32, height: u32)
                  -> Result<RenderBuffer, CreationError> where F: Facade
    {
        let format = image_format::TextureFormatRequest::Specific(image_format::TextureFormat::UncompressedFloat(format));
        let format = image_format::format_request_to_glenum(&facade.get_context(), format, image_format::RequestType::Renderbuffer)?;

        Ok(RenderBuffer {
            buffer: RenderBufferAny::new(facade, format, TextureKind::Float, width, height, None)
        })
    }
}

impl<'a> ToColorAttachment<'a> for &'a RenderBuffer {
    #[inline]
    fn to_color_attachment(self) -> ColorAttachment<'a> {
        ColorAttachment::RenderBuffer(self)
    }
}

impl Deref for RenderBuffer {
    type Target = RenderBufferAny;

    #[inline]
    fn deref(&self) -> &RenderBufferAny {
        &self.buffer
    }
}

impl DerefMut for RenderBuffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut RenderBufferAny {
        &mut self.buffer
    }
}

impl GlObject for RenderBuffer {
    type Id = gl::types::GLuint;

    #[inline]
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
    }
}

/// A render buffer is similar to a texture, but is optimized for usage as a draw target.
///
/// Contrary to a texture, you can't sample or modify the content of the `DepthRenderBuffer` directly.
pub struct DepthRenderBuffer {
    buffer: RenderBufferAny,
}

impl DepthRenderBuffer {
    /// Builds a new render buffer.
    pub fn new<F: ?Sized>(facade: &F, format: DepthFormat, width: u32, height: u32)
                  -> Result<DepthRenderBuffer, CreationError> where F: Facade
    {
        let format = image_format::TextureFormatRequest::Specific(image_format::TextureFormat::DepthFormat(format));
        let format = image_format::format_request_to_glenum(&facade.get_context(), format, image_format::RequestType::Renderbuffer)?;

        Ok(DepthRenderBuffer {
            buffer: RenderBufferAny::new(facade, format, TextureKind::Depth, width, height, None)
        })
    }
}

impl<'a> ToDepthAttachment<'a> for &'a DepthRenderBuffer {
    fn to_depth_attachment(self) -> DepthAttachment<'a> {
        DepthAttachment::RenderBuffer(self)
    }
}

impl Deref for DepthRenderBuffer {
    type Target = RenderBufferAny;

    #[inline]
    fn deref(&self) -> &RenderBufferAny {
        &self.buffer
    }
}

impl DerefMut for DepthRenderBuffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut RenderBufferAny {
        &mut self.buffer
    }
}

impl GlObject for DepthRenderBuffer {
    type Id = gl::types::GLuint;

    #[inline]
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
    }
}

/// A render buffer is similar to a texture, but is optimized for usage as a draw target.
///
/// Contrary to a texture, you can't sample or modify the content of the `StencilRenderBuffer` directly.
pub struct StencilRenderBuffer {
    buffer: RenderBufferAny,
}

impl StencilRenderBuffer {
    /// Builds a new render buffer.
    pub fn new<F: ?Sized>(facade: &F, format: StencilFormat, width: u32, height: u32)
                  -> Result<StencilRenderBuffer, CreationError> where F: Facade
    {
        let format = image_format::TextureFormatRequest::Specific(image_format::TextureFormat::StencilFormat(format));
        let format = image_format::format_request_to_glenum(&facade.get_context(), format, image_format::RequestType::Renderbuffer)?;

        Ok(StencilRenderBuffer {
            buffer: RenderBufferAny::new(facade, format, TextureKind::Stencil, width, height, None)
        })
    }
}

impl<'a> ToStencilAttachment<'a> for &'a StencilRenderBuffer {
    #[inline]
    fn to_stencil_attachment(self) -> StencilAttachment<'a> {
        StencilAttachment::RenderBuffer(self)
    }
}

impl Deref for StencilRenderBuffer {
    type Target = RenderBufferAny;

    #[inline]
    fn deref(&self) -> &RenderBufferAny {
        &self.buffer
    }
}

impl DerefMut for StencilRenderBuffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut RenderBufferAny {
        &mut self.buffer
    }
}

impl GlObject for StencilRenderBuffer {
    type Id = gl::types::GLuint;

    #[inline]
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
    }
}

/// A render buffer is similar to a texture, but is optimized for usage as a draw target.
///
/// Contrary to a texture, you can't sample or modify the content of the `DepthStencilRenderBuffer` directly.
pub struct DepthStencilRenderBuffer {
    buffer: RenderBufferAny,
}

impl DepthStencilRenderBuffer {
    /// Builds a new render buffer.
    pub fn new<F: ?Sized>(facade: &F, format: DepthStencilFormat, width: u32, height: u32)
                  -> Result<DepthStencilRenderBuffer, CreationError> where F: Facade
    {
        let format = image_format::TextureFormatRequest::Specific(image_format::TextureFormat::DepthStencilFormat(format));
        let format = image_format::format_request_to_glenum(&facade.get_context(), format, image_format::RequestType::Renderbuffer)?;

        Ok(DepthStencilRenderBuffer {
            buffer: RenderBufferAny::new(facade, format, TextureKind::DepthStencil, width, height, None)
        })
    }
}

impl<'a> ToDepthStencilAttachment<'a> for &'a DepthStencilRenderBuffer {
    #[inline]
    fn to_depth_stencil_attachment(self) -> DepthStencilAttachment<'a> {
        DepthStencilAttachment::RenderBuffer(self)
    }
}

impl Deref for DepthStencilRenderBuffer {
    type Target = RenderBufferAny;

    #[inline]
    fn deref(&self) -> &RenderBufferAny {
        &self.buffer
    }
}

impl DerefMut for DepthStencilRenderBuffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut RenderBufferAny {
        &mut self.buffer
    }
}

impl GlObject for DepthStencilRenderBuffer {
    type Id = gl::types::GLuint;

    #[inline]
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
    }
}

/// A RenderBuffer of indeterminate type.
pub struct RenderBufferAny {
    context: Rc<Context>,
    id: gl::types::GLuint,
    width: u32,
    height: u32,
    samples: Option<u32>,
    kind: TextureKind,
}

impl RenderBufferAny {
    /// Builds a new render buffer.
    fn new<F: ?Sized>(facade: &F, format: gl::types::GLenum, kind: TextureKind, width: u32, height: u32,
              samples: Option<u32>) -> RenderBufferAny
        where F: Facade
    {
        unsafe {
            // TODO: check that dimensions don't exceed GL_MAX_RENDERBUFFER_SIZE
            // FIXME: gles2 only supports very few formats
            let mut ctxt = facade.get_context().make_current();
            let mut id = 0;

            if ctxt.version >= &Version(Api::Gl, 4, 5) ||
               ctxt.extensions.gl_arb_direct_state_access
            {
                ctxt.gl.CreateRenderbuffers(1, &mut id);
                if let Some(samples) = samples {
                    ctxt.gl.NamedRenderbufferStorageMultisample(id, samples as gl::types::GLsizei,
                                                                format, width as gl::types::GLsizei,
                                                                height as gl::types::GLsizei);
                } else {
                    ctxt.gl.NamedRenderbufferStorage(id, format, width as gl::types::GLsizei,
                                                     height as gl::types::GLsizei);
                }

            } else if samples.is_some() && (ctxt.version >= &Version(Api::Gl, 3, 0) ||
                                            ctxt.version >= &Version(Api::GlEs, 3, 0) ||
                                            ctxt.extensions.gl_apple_framebuffer_multisample ||
                                            ctxt.extensions.gl_angle_framebuffer_multisample ||
                                            ctxt.extensions.gl_ext_multisampled_render_to_texture ||
                                            ctxt.extensions.gl_nv_framebuffer_multisample)
            {
                ctxt.gl.GenRenderbuffers(1, &mut id);
                ctxt.gl.BindRenderbuffer(gl::RENDERBUFFER, id);
                ctxt.state.renderbuffer = id;

                let samples = samples.unwrap();

                if ctxt.version >= &Version(Api::Gl, 3, 0) ||
                   ctxt.version >= &Version(Api::GlEs, 3, 0)
                {
                    ctxt.gl.RenderbufferStorageMultisample(gl::RENDERBUFFER,
                                                           samples as gl::types::GLsizei,
                                                           format,
                                                           width as gl::types::GLsizei,
                                                           height as gl::types::GLsizei);

                } else if ctxt.extensions.gl_apple_framebuffer_multisample {
                    ctxt.gl.RenderbufferStorageMultisampleAPPLE(gl::RENDERBUFFER,
                                                                samples as gl::types::GLsizei,
                                                                format,
                                                                width as gl::types::GLsizei,
                                                                height as gl::types::GLsizei);

                } else if ctxt.extensions.gl_angle_framebuffer_multisample {
                    ctxt.gl.RenderbufferStorageMultisampleANGLE(gl::RENDERBUFFER,
                                                                samples as gl::types::GLsizei,
                                                                format,
                                                                width as gl::types::GLsizei,
                                                                height as gl::types::GLsizei);

                } else if ctxt.extensions.gl_ext_multisampled_render_to_texture {
                    ctxt.gl.RenderbufferStorageMultisampleEXT(gl::RENDERBUFFER,
                                                              samples as gl::types::GLsizei,
                                                              format,
                                                              width as gl::types::GLsizei,
                                                              height as gl::types::GLsizei);

                } else if ctxt.extensions.gl_nv_framebuffer_multisample {
                    ctxt.gl.RenderbufferStorageMultisampleNV(gl::RENDERBUFFER,
                                                             samples as gl::types::GLsizei,
                                                             format,
                                                             width as gl::types::GLsizei,
                                                             height as gl::types::GLsizei);

                } else {
                    unreachable!();
                }

            } else if samples.is_none() && (ctxt.version >= &Version(Api::Gl, 3, 0) ||
                                            ctxt.version >= &Version(Api::GlEs, 2, 0))
            {
                ctxt.gl.GenRenderbuffers(1, &mut id);
                ctxt.gl.BindRenderbuffer(gl::RENDERBUFFER, id);
                ctxt.state.renderbuffer = id;
                ctxt.gl.RenderbufferStorage(gl::RENDERBUFFER, format,
                                            width as gl::types::GLsizei,
                                            height as gl::types::GLsizei);

            } else if samples.is_some() && ctxt.extensions.gl_ext_framebuffer_object &&
                      ctxt.extensions.gl_ext_framebuffer_multisample
            {
                ctxt.gl.GenRenderbuffersEXT(1, &mut id);
                ctxt.gl.BindRenderbufferEXT(gl::RENDERBUFFER_EXT, id);
                ctxt.state.renderbuffer = id;

                let samples = samples.unwrap();
                ctxt.gl.RenderbufferStorageMultisampleEXT(gl::RENDERBUFFER_EXT,
                                                          samples as gl::types::GLsizei,
                                                          format,
                                                          width as gl::types::GLsizei,
                                                          height as gl::types::GLsizei);

            } else if samples.is_none() && ctxt.extensions.gl_ext_framebuffer_object {
                ctxt.gl.GenRenderbuffersEXT(1, &mut id);
                ctxt.gl.BindRenderbufferEXT(gl::RENDERBUFFER_EXT, id);
                ctxt.state.renderbuffer = id;
                ctxt.gl.RenderbufferStorageEXT(gl::RENDERBUFFER_EXT, format,
                                               width as gl::types::GLsizei,
                                               height as gl::types::GLsizei);

            } else {
                unreachable!();
            }

            RenderBufferAny {
                context: facade.get_context().clone(),
                id: id,
                width: width,
                height: height,
                samples: samples,
                kind: kind,
            }
        }
    }

    /// Returns the dimensions of the render buffer.
    #[inline]
    pub fn get_dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Returns the number of samples of the render buffer, or `None` if multisampling isn't
    /// enabled.
    #[inline]
    pub fn get_samples(&self) -> Option<u32> {
        self.samples
    }

    /// Returns the context used to create this renderbuffer.
    #[inline]
    pub fn get_context(&self) -> &Rc<Context> {
        &self.context
    }

    /// Returns the kind of renderbuffer.
    #[inline]
    pub fn kind(&self) -> TextureKind {
        self.kind
    }

    /// Determines the number of depth and stencil bits in the format of this render buffer.
    pub fn get_depth_stencil_bits(&self) -> (u16, u16) {
        unsafe {
            let ctxt = self.context.make_current();
            let mut depth_bits: gl::types::GLint = 0;
            let mut stencil_bits: gl::types::GLint = 0;
            ctxt.gl.BindRenderbuffer(gl::RENDERBUFFER, self.id);
            // FIXME: GL version considerations
            ctxt.gl.GetRenderbufferParameteriv(gl::RENDERBUFFER, gl::RENDERBUFFER_DEPTH_SIZE, &mut depth_bits);
            ctxt.gl.GetRenderbufferParameteriv(gl::RENDERBUFFER, gl::RENDERBUFFER_STENCIL_SIZE, &mut stencil_bits);
            ctxt.gl.BindRenderbuffer(gl::RENDERBUFFER, 0);
            (depth_bits as u16, stencil_bits as u16)
        }
    }
}

impl Drop for RenderBufferAny {
    fn drop(&mut self) {
        unsafe {
            let mut ctxt = self.context.make_current();

            // removing FBOs which contain this buffer
            FramebuffersContainer::purge_renderbuffer(&mut ctxt, self.id);

            if ctxt.version >= &Version(Api::Gl, 3, 0) ||
               ctxt.version >= &Version(Api::GlEs, 2, 0)
            {
                if ctxt.state.renderbuffer == self.id {
                    ctxt.state.renderbuffer = 0;
                }

                ctxt.gl.DeleteRenderbuffers(1, [ self.id ].as_ptr());

            } else if ctxt.extensions.gl_ext_framebuffer_object {
                if ctxt.state.renderbuffer == self.id {
                    ctxt.state.renderbuffer = 0;
                }

                ctxt.gl.DeleteRenderbuffersEXT(1, [ self.id ].as_ptr());

            } else {
                unreachable!();
            }
        }
    }
}

impl GlObject for RenderBufferAny {
    type Id = gl::types::GLuint;

    #[inline]
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}
