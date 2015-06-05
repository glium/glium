/*!

A render buffer is similar to a texture, but is optimized for usage as a draw target.

Contrary to a texture, you can't sample nor modify the content of a render buffer.
You should prefer render buffers over textures when you know that you don't need to read or modify
the data of the render buffer.

*/
use std::rc::Rc;
use std::ops::{Deref, DerefMut};
use std::mem;

use framebuffer::{ColorAttachment, ToColorAttachment};
use framebuffer::{DepthAttachment, ToDepthAttachment};
use framebuffer::{StencilAttachment, ToStencilAttachment};
use framebuffer::{DepthStencilAttachment, ToDepthStencilAttachment};
use texture::{UncompressedFloatFormat, DepthFormat, StencilFormat, DepthStencilFormat};

use image_format;

use gl;
use GlObject;
use backend::Facade;
use context::Context;
use ContextExt;
use version::Version;
use version::Api;

/// A render buffer is similar to a texture, but is optimized for usage as a draw target.
///
/// Contrary to a texture, you can't sample or modify the content of the `RenderBuffer`.
pub struct RenderBuffer {
    buffer: RenderBufferAny,
}

impl RenderBuffer {
    /// Builds a new render buffer.
    pub fn new<F>(facade: &F, format: UncompressedFloatFormat, width: u32, height: u32)
                  -> RenderBuffer where F: Facade
    {
        let format = image_format::TextureFormatRequest::Specific(image_format::TextureFormat::UncompressedFloat(format));
        let (_, format) = image_format::format_request_to_glenum(&facade.get_context(), None, format).unwrap();
        let format = format.expect("Format not supported");

        RenderBuffer {
            buffer: RenderBufferAny::new(facade, format, width, height)
        }
    }
}

impl ToColorAttachment for RenderBuffer {
    fn to_color_attachment(&self) -> ColorAttachment {
        ColorAttachment::RenderBuffer(self)
    }
}

impl Deref for RenderBuffer {
    type Target = RenderBufferAny;

    fn deref(&self) -> &RenderBufferAny {
        &self.buffer
    }
}

impl DerefMut for RenderBuffer {
    fn deref_mut(&mut self) -> &mut RenderBufferAny {
        &mut self.buffer
    }
}

impl GlObject for RenderBuffer {
    type Id = gl::types::GLuint;
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
    pub fn new<F>(facade: &F, format: DepthFormat, width: u32, height: u32)
                  -> DepthRenderBuffer where F: Facade
    {
        let format = image_format::TextureFormatRequest::Specific(image_format::TextureFormat::DepthFormat(format));
        let (_, format) = image_format::format_request_to_glenum(&facade.get_context(), None, format).unwrap();
        let format = format.expect("Format not supported");

        DepthRenderBuffer {
            buffer: RenderBufferAny::new(facade, format, width, height)
        }
    }
}

impl ToDepthAttachment for DepthRenderBuffer {
    fn to_depth_attachment(&self) -> DepthAttachment {
        DepthAttachment::RenderBuffer(self)
    }
}

impl Deref for DepthRenderBuffer {
    type Target = RenderBufferAny;

    fn deref(&self) -> &RenderBufferAny {
        &self.buffer
    }
}

impl DerefMut for DepthRenderBuffer {
    fn deref_mut(&mut self) -> &mut RenderBufferAny {
        &mut self.buffer
    }
}

impl GlObject for DepthRenderBuffer {
    type Id = gl::types::GLuint;
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
    pub fn new<F>(facade: &F, format: StencilFormat, width: u32, height: u32)
                  -> StencilRenderBuffer where F: Facade
    {
        let format = image_format::TextureFormatRequest::Specific(image_format::TextureFormat::StencilFormat(format));
        let (_, format) = image_format::format_request_to_glenum(&facade.get_context(), None, format).unwrap();
        let format = format.expect("Format not supported");

        StencilRenderBuffer {
            buffer: RenderBufferAny::new(facade, format, width, height)
        }
    }
}

impl ToStencilAttachment for StencilRenderBuffer {
    fn to_stencil_attachment(&self) -> StencilAttachment {
        StencilAttachment::RenderBuffer(self)
    }
}

impl Deref for StencilRenderBuffer {
    type Target = RenderBufferAny;

    fn deref(&self) -> &RenderBufferAny {
        &self.buffer
    }
}

impl DerefMut for StencilRenderBuffer {
    fn deref_mut(&mut self) -> &mut RenderBufferAny {
        &mut self.buffer
    }
}

impl GlObject for StencilRenderBuffer {
    type Id = gl::types::GLuint;
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
    pub fn new<F>(facade: &F, format: DepthStencilFormat, width: u32, height: u32)
                  -> DepthStencilRenderBuffer where F: Facade
    {
        let format = image_format::TextureFormatRequest::Specific(image_format::TextureFormat::DepthStencilFormat(format));
        let (_, format) = image_format::format_request_to_glenum(&facade.get_context(), None, format).unwrap();
        let format = format.expect("Format not supported");

        DepthStencilRenderBuffer {
            buffer: RenderBufferAny::new(facade, format, width, height)
        }
    }
}

impl ToDepthStencilAttachment for DepthStencilRenderBuffer {
    fn to_depth_stencil_attachment(&self) -> DepthStencilAttachment {
        DepthStencilAttachment::RenderBuffer(self)
    }
}

impl Deref for DepthStencilRenderBuffer {
    type Target = RenderBufferAny;

    fn deref(&self) -> &RenderBufferAny {
        &self.buffer
    }
}

impl DerefMut for DepthStencilRenderBuffer {
    fn deref_mut(&mut self) -> &mut RenderBufferAny {
        &mut self.buffer
    }
}

impl GlObject for DepthStencilRenderBuffer {
    type Id = gl::types::GLuint;
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
}

impl RenderBufferAny {
    /// Builds a new render buffer.
    fn new<F>(facade: &F, format: gl::types::GLenum, width: u32, height: u32)
              -> RenderBufferAny where F: Facade
    {
        // TODO: check that dimensions don't exceed GL_MAX_RENDERBUFFER_SIZE
        let mut ctxt = facade.get_context().make_current();

        let id = unsafe {
            let mut id = mem::uninitialized();

            if ctxt.version >= &Version(Api::Gl, 4, 5) ||
                ctxt.extensions.gl_arb_direct_state_access
            {
                ctxt.gl.CreateRenderbuffers(1, &mut id);
                ctxt.gl.NamedRenderbufferStorage(id, format, width as gl::types::GLsizei,
                                                 height as gl::types::GLsizei);

            } else if ctxt.version >= &Version(Api::Gl, 3, 0) ||
                      ctxt.version >= &Version(Api::GlEs, 2, 0)
            {
                ctxt.gl.GenRenderbuffers(1, &mut id);
                ctxt.gl.BindRenderbuffer(gl::RENDERBUFFER, id);
                ctxt.state.renderbuffer = id;
                // FIXME: gles2 only supports very few formats
                ctxt.gl.RenderbufferStorage(gl::RENDERBUFFER, format,
                                            width as gl::types::GLsizei,
                                            height as gl::types::GLsizei);

            } else if ctxt.extensions.gl_ext_framebuffer_object {
                ctxt.gl.GenRenderbuffersEXT(1, &mut id);
                ctxt.gl.BindRenderbufferEXT(gl::RENDERBUFFER_EXT, id);
                ctxt.state.renderbuffer = id;
                ctxt.gl.RenderbufferStorageEXT(gl::RENDERBUFFER_EXT, format,
                                               width as gl::types::GLsizei,
                                               height as gl::types::GLsizei);

            } else {
                unreachable!();
            }

            id
        };

        RenderBufferAny {
            context: facade.get_context().clone(),
            id: id,
            width: width,
            height: height,
        }
    }

    /// Returns the dimensions of the render buffer.
    pub fn get_dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl Drop for RenderBufferAny {
    fn drop(&mut self) {
        unsafe {
            let mut ctxt = self.context.make_current();

            // removing FBOs which contain this buffer
            self.context.get_framebuffer_objects()
                        .purge_renderbuffer(self.id, &mut ctxt);

            if ctxt.version >= &Version(Api::Gl, 3, 0) ||
               ctxt.version >= &Version(Api::GlEs, 2, 0)
            {
                if ctxt.state.renderbuffer == self.id {
                    ctxt.gl.BindRenderbuffer(gl::RENDERBUFFER, 0);
                    ctxt.state.renderbuffer = 0;
                }
                ctxt.gl.DeleteRenderbuffers(1, [ self.id ].as_ptr());

            } else if ctxt.extensions.gl_ext_framebuffer_object {
                if ctxt.state.renderbuffer == self.id {
                    ctxt.gl.BindRenderbufferEXT(gl::RENDERBUFFER_EXT, 0);
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
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}
