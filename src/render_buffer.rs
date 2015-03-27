/*!

A render buffer is similar to a texture, but is optimized for usage as a draw target.

Contrary to a texture, you can't sample nor modify the content of a render buffer.
You should prefer render buffers over textures when you know that you don't need to read or modify
the data of the render buffer.

*/
use std::rc::Rc;
use std::mem;

use framebuffer::{ColorAttachment, ToColorAttachment};
use framebuffer::{DepthAttachment, ToDepthAttachment};
use framebuffer::{StencilAttachment, ToStencilAttachment};
use framebuffer::{DepthStencilAttachment, ToDepthStencilAttachment};
use texture::{UncompressedFloatFormat, DepthFormat, StencilFormat, DepthStencilFormat};

use gl;
use {GlObject, ToGlEnum};
use backend::Facade;
use context::Context;
use ContextExt;
use version::Version;
use version::Api;

/// A render buffer is similar to a texture, but is optimized for usage as a draw target.
///
/// Contrary to a texture, you can't sample or modify the content of the `RenderBuffer`.
pub struct RenderBuffer {
    buffer: RenderBufferImpl,
}

impl RenderBuffer {
    /// Builds a new render buffer.
    pub fn new<F>(facade: &F, format: UncompressedFloatFormat, width: u32, height: u32)
                  -> RenderBuffer where F: Facade
    {
        RenderBuffer {
            buffer: RenderBufferImpl::new(facade, format.to_glenum(), width, height)
        }
    }

    /// Returns the dimensions of the render buffer.
    pub fn get_dimensions(&self) -> (u32, u32) {
        (self.buffer.width, self.buffer.height)
    }
}

impl ToColorAttachment for RenderBuffer {
    fn to_color_attachment(&self) -> ColorAttachment {
        ColorAttachment::RenderBuffer(self)
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
    buffer: RenderBufferImpl,
}

impl DepthRenderBuffer {
    /// Builds a new render buffer.
    pub fn new<F>(facade: &F, format: DepthFormat, width: u32, height: u32)
                  -> DepthRenderBuffer where F: Facade
    {
        DepthRenderBuffer {
            buffer: RenderBufferImpl::new(facade, format.to_glenum(), width, height)
        }
    }

    /// Returns the dimensions of the render buffer.
    pub fn get_dimensions(&self) -> (u32, u32) {
        (self.buffer.width, self.buffer.height)
    }
}

impl ToDepthAttachment for DepthRenderBuffer {
    fn to_depth_attachment(&self) -> DepthAttachment {
        DepthAttachment::RenderBuffer(self)
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
    buffer: RenderBufferImpl,
}

impl StencilRenderBuffer {
    /// Builds a new render buffer.
    pub fn new<F>(facade: &F, format: StencilFormat, width: u32, height: u32)
                  -> StencilRenderBuffer where F: Facade
    {
        StencilRenderBuffer {
            buffer: RenderBufferImpl::new(facade, format.to_glenum(), width, height)
        }
    }

    /// Returns the dimensions of the render buffer.
    pub fn get_dimensions(&self) -> (u32, u32) {
        (self.buffer.width, self.buffer.height)
    }
}

impl ToStencilAttachment for StencilRenderBuffer {
    fn to_stencil_attachment(&self) -> StencilAttachment {
        StencilAttachment::RenderBuffer(self)
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
    buffer: RenderBufferImpl,
}

impl DepthStencilRenderBuffer {
    /// Builds a new render buffer.
    pub fn new<F>(facade: &F, format: DepthStencilFormat, width: u32, height: u32)
                  -> DepthStencilRenderBuffer where F: Facade
    {
        DepthStencilRenderBuffer {
            buffer: RenderBufferImpl::new(facade, format.to_glenum(), width, height)
        }
    }

    /// Returns the dimensions of the render buffer.
    pub fn get_dimensions(&self) -> (u32, u32) {
        (self.buffer.width, self.buffer.height)
    }
}

impl ToDepthStencilAttachment for DepthStencilRenderBuffer {
    fn to_depth_stencil_attachment(&self) -> DepthStencilAttachment {
        DepthStencilAttachment::RenderBuffer(self)
    }
}

impl GlObject for DepthStencilRenderBuffer {
    type Id = gl::types::GLuint;
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
    }
}

/// The implementation
struct RenderBufferImpl {
    context: Rc<Context>,
    id: gl::types::GLuint,
    width: u32,
    height: u32,
}

impl RenderBufferImpl {
    /// Builds a new render buffer.
    fn new<F>(facade: &F, format: gl::types::GLenum, width: u32, height: u32)
              -> RenderBufferImpl where F: Facade
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

        RenderBufferImpl {
            context: facade.get_context().clone(),
            id: id,
            width: width,
            height: height,
        }
    }
}

impl Drop for RenderBufferImpl {
    fn drop(&mut self) {
        unsafe {
            let mut ctxt = self.context.make_current();

            // removing FBOs which contain this buffer
            self.context.framebuffer_objects.as_ref().unwrap()
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

impl GlObject for RenderBufferImpl {
    type Id = gl::types::GLuint;
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}
