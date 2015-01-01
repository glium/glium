use std::mem;
use std::sync::Arc;

use DisplayImpl;
use GlObject;

use gl;
use context;

#[deriving(Hash, Clone, PartialEq, Eq)]
pub struct FramebufferAttachments {
    pub colors: Vec<(u32, Attachment)>,
    pub depth: Option<Attachment>,
    pub stencil: Option<Attachment>,
}

#[deriving(Hash, Copy, Clone, PartialEq, Eq)]
pub enum Attachment {
    Texture(gl::types::GLuint),
    RenderBuffer(gl::types::GLuint),
}

/// Frame buffer.
pub struct FrameBufferObject {
    display: Arc<DisplayImpl>,
    id: gl::types::GLuint,
    current_read_buffer: gl::types::GLenum,
}

impl FrameBufferObject {
    /// Builds a new FBO.
    fn new(display: Arc<DisplayImpl>, attachments: &FramebufferAttachments) -> FrameBufferObject {
        let (tx, rx) = channel();
        let attachments = attachments.clone();

        display.context.exec(move |: mut ctxt| {
            use context::GlVersion;

            unsafe fn attach(ctxt: &mut context::CommandContext, slot: gl::types::GLenum,
                             id: gl::types::GLuint, attachment: Attachment)
            {
                if ctxt.version >= &GlVersion(4, 5) {
                    match attachment {
                        Attachment::Texture(tex_id) => {
                            ctxt.gl.NamedFramebufferTexture(id, slot, tex_id, 0);
                        },
                        Attachment::RenderBuffer(buf_id) => {
                            ctxt.gl.NamedFramebufferRenderbuffer(id, slot, gl::RENDERBUFFER,
                                                                 buf_id);
                        },
                    }

                } else if ctxt.extensions.gl_ext_direct_state_access &&
                          ctxt.extensions.gl_ext_geometry_shader4
                {
                    match attachment {
                        Attachment::Texture(tex_id) => {
                            ctxt.gl.NamedFramebufferTextureEXT(id, slot, tex_id, 0);
                        },
                        Attachment::RenderBuffer(buf_id) => {
                            ctxt.gl.NamedFramebufferRenderbufferEXT(id, slot, gl::RENDERBUFFER,
                                                                    buf_id);
                        },
                    }

                } else if ctxt.version >= &GlVersion(3, 2) {
                    bind_framebuffer(ctxt, Some(id), true, false);

                    match attachment {
                        Attachment::Texture(tex_id) => {
                            ctxt.gl.FramebufferTexture(gl::DRAW_FRAMEBUFFER,
                                                       slot, tex_id, 0);
                        },
                        Attachment::RenderBuffer(buf_id) => {
                            ctxt.gl.FramebufferRenderbuffer(gl::DRAW_FRAMEBUFFER, slot,
                                                            gl::RENDERBUFFER, buf_id);
                        },
                    }

                } else if ctxt.version >= &GlVersion(3, 0) {
                    bind_framebuffer(ctxt, Some(id), true, false);

                    match attachment {
                        Attachment::Texture(tex_id) => {
                            ctxt.gl.FramebufferTexture2D(gl::DRAW_FRAMEBUFFER,
                                                         slot, gl::TEXTURE_2D, tex_id, 0);
                        },
                        Attachment::RenderBuffer(buf_id) => {
                            ctxt.gl.FramebufferRenderbuffer(gl::DRAW_FRAMEBUFFER, slot,
                                                            gl::RENDERBUFFER, buf_id);
                        },
                    }

                } else {
                    bind_framebuffer(ctxt, Some(id), true, true);

                    match attachment {
                        Attachment::Texture(tex_id) => {
                            ctxt.gl.FramebufferTexture2DEXT(gl::FRAMEBUFFER_EXT,
                                                            slot, gl::TEXTURE_2D, tex_id, 0);
                        },
                        Attachment::RenderBuffer(buf_id) => {
                            ctxt.gl.FramebufferRenderbufferEXT(gl::DRAW_FRAMEBUFFER, slot,
                                                               gl::RENDERBUFFER, buf_id);
                        },
                    }
                }
            }

            unsafe {
                let id: gl::types::GLuint = mem::uninitialized();
                if ctxt.version >= &context::GlVersion(3, 0) {
                    ctxt.gl.GenFramebuffers(1, mem::transmute(&id));
                } else {
                    ctxt.gl.GenFramebuffersEXT(1, mem::transmute(&id));
                }

                tx.send(id);

                for &(slot, atchmnt) in attachments.colors.iter() {
                    attach(&mut ctxt, gl::COLOR_ATTACHMENT0 + slot as u32, id, atchmnt);
                }

                if let Some(atchmnt) = attachments.depth {
                    attach(&mut ctxt, gl::DEPTH_ATTACHMENT, id, atchmnt);
                }

                if let Some(atchmnt) = attachments.stencil {
                    attach(&mut ctxt, gl::STENCIL_ATTACHMENT, id, atchmnt);
                }
            }
        });

        FrameBufferObject {
            display: display,
            id: rx.recv(),
            current_read_buffer: gl::BACK,
        }
    }
}

impl Drop for FrameBufferObject {
    fn drop(&mut self) {
        let id = self.id.clone();
        self.display.context.exec(move |: ctxt| {
            unsafe {
                // unbinding framebuffer
                if ctxt.version >= &context::GlVersion(3, 0) {
                    if ctxt.state.draw_framebuffer == id && ctxt.state.read_framebuffer == id {
                        ctxt.gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
                        ctxt.state.draw_framebuffer = 0;
                        ctxt.state.read_framebuffer = 0;

                    } else if ctxt.state.draw_framebuffer == id {
                        ctxt.gl.BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
                        ctxt.state.draw_framebuffer = 0;

                    } else if ctxt.state.read_framebuffer == id {
                        ctxt.gl.BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
                        ctxt.state.read_framebuffer = 0;
                    }

                } else {
                    if ctxt.state.draw_framebuffer == id || ctxt.state.read_framebuffer == id {
                        ctxt.gl.BindFramebufferEXT(gl::FRAMEBUFFER_EXT, 0);
                        ctxt.state.draw_framebuffer = 0;
                        ctxt.state.read_framebuffer = 0;
                    }
                }

                // deleting
                if ctxt.version >= &context::GlVersion(3, 0) {
                    ctxt.gl.DeleteFramebuffers(1, [ id ].as_ptr());
                } else {
                    ctxt.gl.DeleteFramebuffersEXT(1, [ id ].as_ptr());
                }
            }
        });
    }
}

impl GlObject for FrameBufferObject {
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

pub fn get_framebuffer(display: &Arc<DisplayImpl>, framebuffer: Option<&FramebufferAttachments>)
    -> Option<gl::types::GLuint>
{
    if let Some(framebuffer) = framebuffer {
        let mut framebuffers = display.framebuffer_objects.lock().unwrap();

        if let Some(value) = framebuffers.get(framebuffer) {
            return Some(value.id);
        }

        let new_fbo = FrameBufferObject::new(display.clone(), framebuffer);
        let new_fbo_id = new_fbo.id.clone();
        framebuffers.insert(framebuffer.clone(), new_fbo);
        Some(new_fbo_id)

    } else {
        None
    }
}

pub fn bind_framebuffer(ctxt: &mut context::CommandContext, fbo_id: Option<gl::types::GLuint>,
                        draw: bool, read: bool)
{
    let fbo_id = fbo_id.unwrap_or(0);

    if draw && ctxt.state.draw_framebuffer != fbo_id {
        unsafe {
            if ctxt.version >= &context::GlVersion(3, 0) {
                ctxt.gl.BindFramebuffer(gl::DRAW_FRAMEBUFFER, fbo_id);
                ctxt.state.draw_framebuffer = fbo_id;
            } else {
                ctxt.gl.BindFramebufferEXT(gl::FRAMEBUFFER_EXT, fbo_id);
                ctxt.state.draw_framebuffer = fbo_id;
                ctxt.state.read_framebuffer = fbo_id;
            }
        }
    }

    if read && ctxt.state.read_framebuffer != fbo_id {
        unsafe {
            if ctxt.version >= &context::GlVersion(3, 0) {
                ctxt.gl.BindFramebuffer(gl::READ_FRAMEBUFFER, fbo_id);
                ctxt.state.read_framebuffer = fbo_id;
            } else {
                ctxt.gl.BindFramebufferEXT(gl::FRAMEBUFFER_EXT, fbo_id);
                ctxt.state.draw_framebuffer = fbo_id;
                ctxt.state.read_framebuffer = fbo_id;
            }
        }
    }
}
