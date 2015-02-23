/*!
Contains everything related to the internal handling of framebuffer objects.

This module allows creating framebuffer objects. However it **does not** check whether
the framebuffer object is complete (ie. if everything is valid). This is the module user's job.

Here are the rules taken from the official wiki:

Attachment Completeness

Each attachment point itself must be complete according to these rules. Empty attachments
(attachments with no image attached) are complete by default. If an image is attached, it must
adhere to the following rules:

The source object for the image still exists and has the same type it was attached with.
The image has a non-zero width and height (the height of a 1D image is assumed to be 1). The
  width/height must also be less than GL_MAX_FRAMEBUFFER_WIDTH and GL_MAX_FRAMEBUFFER_HEIGHT
  respectively (if GL 4.3/ARB_framebuffer_no_attachments).
The layer for 3D or array textures attachments is less than the depth of the texture. It must
  also be less than GL_MAX_FRAMEBUFFER_LAYERS (if GL 4.3/ARB_framebuffer_no_attachments).
The number of samples must be less than GL_MAX_FRAMEBUFFER_SAMPLES (if
  GL 4.3/ARB_framebuffer_no_attachments).
The image's format must match the attachment point's requirements, as defined above.
  Color-renderable formats for color attachments, etc.

Completeness Rules

These are the rules for framebuffer completeness. The order of these rules matters.

If the target​ of glCheckFramebufferStatus references the Default Framebuffer (ie: FBO object
  number 0 is bound), and the default framebuffer does not exist, then you will get
  GL_FRAMEBUFFER_UNDEFINEZ. If the default framebuffer exists, then you always get
  GL_FRAMEBUFFER_COMPLETE. The rest of the rules apply when an FBO is bound.
All attachments must be attachment complete. (GL_FRAMEBUFFER_INCOMPLETE_ATTACHMENT when false).
There must be at least one image attached to the FBO, or if OpenGL 4.3 or
  ARB_framebuffer_no_attachment is available, the GL_FRAMEBUFFER_DEFAULT_WIDTH and
  GL_FRAMEBUFFER_DEFAULT_HEIGHT parameters of the framebuffer must both be non-zero.
  (GL_FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT when false).
Each draw buffers must either specify color attachment points that have images attached or
  must be GL_NONE. (GL_FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER when false). Note that this test is
  not performed if OpenGL 4.1 or ARB_ES2_compatibility is available.
If the read buffer is set, then it must specify an attachment point that has an image
  attached. (GL_FRAMEBUFFER_INCOMPLETE_READ_BUFFER when false). Note that this test is not
  performed if OpenGL 4.1 or ARB_ES2_compatibility is available.
All images must have the same number of multisample samples.
  (GL_FRAMEBUFFER_INCOMPLETE_MULTISAMPLE when false).
If a layered image is attached to one attachment, then all attachments must be layered
  attachments. The attached layers do not have to have the same number of layers, nor do the
  layers have to come from the same kind of texture (a cubemap color texture can be paired
  with an array depth texture) (GL_FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS when false).

*/
use std::collections::hash_state::DefaultState;
use std::collections::HashMap;
use std::default::Default;
use std::mem;
use std::sync::Mutex;
use std::sync::mpsc::channel;

use GlObject;

use gl;
use context;
use context::GlVersion;
use version::Api;
use util::FnvHasher;

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct FramebufferAttachments {
    pub colors: Vec<(u32, Attachment)>,
    pub depth_stencil: FramebufferDepthStencilAttachments,
}

#[derive(Hash, Clone, PartialEq, Eq)]
pub enum FramebufferDepthStencilAttachments {
    None,
    DepthAttachment(Attachment),
    StencilAttachment(Attachment),
    DepthAndStencilAttachments(Attachment, Attachment),
    DepthStencilAttachment(Attachment),
}

#[derive(Hash, Copy, Clone, PartialEq, Eq)]
pub enum Attachment {
    Texture {
        bind_point: gl::types::GLenum,      // must be GL_TEXTURE_3D, GL_TEXTURE_2D_ARRAY, etc.
        id: gl::types::GLuint,
        level: u32,
        layer: u32,
    },
    RenderBuffer(gl::types::GLuint),
}

/// Manages all the framebuffer objects.
///
/// `cleanup` **must** be called when destroying the container, otherwise `Drop` will panic.
pub struct FramebuffersContainer {
    framebuffers: Mutex<HashMap<FramebufferAttachments, FrameBufferObject, DefaultState<FnvHasher>>>,
}

/// Frame buffer.
struct FrameBufferObject {
    id: gl::types::GLuint,
    current_read_buffer: gl::types::GLenum,
}

impl FramebuffersContainer {
    pub fn new() -> FramebuffersContainer {
        FramebuffersContainer {
            framebuffers: Mutex::new(HashMap::with_hash_state(Default::default())),
        }
    }

    pub fn purge_all(&self, context: &context::Context) {
        self.purge_if(|_| true, context);
    }

    pub fn purge_texture(&self, texture: gl::types::GLuint, context: &context::Context) {
        self.purge_if(|a| {
            match a {
                &Attachment::Texture { id, .. } if id == texture => true,
                _ => false 
            }
        }, context);
    }

    pub fn purge_renderbuffer(&self, renderbuffer: gl::types::GLuint,
                              context: &context::Context)
    {
        self.purge_if(|a| a == &Attachment::RenderBuffer(renderbuffer), context);
    }

    fn purge_if<F>(&self, condition: F, context: &context::Context)
                   where F: Fn(&Attachment) -> bool
    {
        let mut framebuffers = self.framebuffers.lock().unwrap();

        let mut attachments = Vec::new();
        for (key, _) in framebuffers.iter() {
            match key.depth_stencil {
                FramebufferDepthStencilAttachments::None => (),
                FramebufferDepthStencilAttachments::DepthAttachment(ref depth) => {
                    if condition(depth) {
                        attachments.push(key.clone());
                        continue;
                    }
                },
                FramebufferDepthStencilAttachments::StencilAttachment(ref stencil) => {
                    if condition(stencil) {
                        attachments.push(key.clone());
                        continue;
                    }
                },
                FramebufferDepthStencilAttachments::DepthAndStencilAttachments(ref depth,
                                                                               ref stencil) =>
                {
                    if condition(depth) || condition(stencil) {
                        attachments.push(key.clone());
                        continue;
                    }
                },
                FramebufferDepthStencilAttachments::DepthStencilAttachment(ref depth_stencil) => {
                    if condition(depth_stencil) {
                        attachments.push(key.clone());
                        continue;
                    }
                }
            }

            if key.colors.iter().find(|&&(_, ref id)| condition(id)).is_some() {
                attachments.push(key.clone());
                continue;
            }
        }

        for atch in attachments.into_iter() {
            framebuffers.remove(&atch).unwrap().destroy(context);
        }
    }

    pub fn cleanup(self, context: &context::Context) {
        let mut other = HashMap::with_hash_state(Default::default());
        mem::swap(&mut *self.framebuffers.lock().unwrap(), &mut other);

        for (_, obj) in other.into_iter() {
            obj.destroy(context);
        }
    }

    pub fn get_framebuffer_for_drawing(&self, attachments: Option<&FramebufferAttachments>,
                                       context: &context::Context) -> gl::types::GLuint
    {
        if let Some(attachments) = attachments {
            self.get_framebuffer(attachments, context)
        } else {
            0
        }
    }

    pub fn get_framebuffer_for_reading(&self, attachment: &Attachment, context: &context::Context)
                                       -> (gl::types::GLuint, gl::types::GLenum)
    {
        for (attachments, fbo) in self.framebuffers.lock().unwrap().iter() {
            for &(key, ref atc) in attachments.colors.iter() {
                if atc == attachment {
                    return (fbo.get_id(), gl::COLOR_ATTACHMENT0 + key);
                }
            }
        }

        let attachments = FramebufferAttachments {
            colors: vec![(0, attachment.clone())],
            depth_stencil: FramebufferDepthStencilAttachments::None,
        };

        let framebuffer = self.get_framebuffer_for_drawing(Some(&attachments), context);
        (framebuffer, gl::COLOR_ATTACHMENT0)
    }

    fn get_framebuffer(&self, framebuffer: &FramebufferAttachments,
                       context: &context::Context) -> gl::types::GLuint
    {
        let mut framebuffers = self.framebuffers.lock().unwrap();

        if let Some(value) = framebuffers.get(framebuffer) {
            return value.id;
        }

        let new_fbo = FrameBufferObject::new(context, framebuffer);
        let new_fbo_id = new_fbo.id.clone();
        framebuffers.insert(framebuffer.clone(), new_fbo);
        new_fbo_id
    }
}

impl Drop for FramebuffersContainer {
    fn drop(&mut self) {
        if self.framebuffers.lock().unwrap().len() != 0 {
            panic!()
        }
    }
}

impl FrameBufferObject {
    /// Builds a new FBO.
    fn new(context: &context::Context, attachments: &FramebufferAttachments) -> FrameBufferObject {
        let (tx, rx) = channel();
        let attachments = attachments.clone();

        context.exec(move |mut ctxt| {
            // TODO: move outside of the gl thread
            if attachments.colors.len() > ctxt.capabilities.max_draw_buffers as usize {
                panic!("Trying to attach {} color buffers, but the hardware only supports {}",
                       attachments.colors.len(), ctxt.capabilities.max_draw_buffers);
            }

            unsafe {
                let mut id = mem::uninitialized();

                if ctxt.version >= &context::GlVersion(Api::Gl, 4, 5) ||
                    ctxt.extensions.gl_arb_direct_state_access
                {
                    ctxt.gl.CreateFramebuffers(1, &mut id);
                } else if ctxt.version >= &context::GlVersion(Api::Gl, 3, 0) {
                    ctxt.gl.GenFramebuffers(1, &mut id);
                    bind_framebuffer(&mut ctxt, id, true, false);
                } else {
                    ctxt.gl.GenFramebuffersEXT(1, &mut id);
                    bind_framebuffer(&mut ctxt, id, true, false);
                }

                tx.send(id).unwrap();

                let mut raw_attachments: Vec<gl::types::GLenum> = Vec::new();

                for &(slot, atchmnt) in attachments.colors.iter() {
                    attach(&mut ctxt, gl::COLOR_ATTACHMENT0 + slot as u32, id, atchmnt);
                    raw_attachments.push(gl::COLOR_ATTACHMENT0 + slot as u32);
                }

                match attachments.depth_stencil {
                    FramebufferDepthStencilAttachments::None => (),
                    FramebufferDepthStencilAttachments::DepthAttachment(depth) => {
                        attach(&mut ctxt, gl::DEPTH_ATTACHMENT, id, depth);
                    },
                    FramebufferDepthStencilAttachments::StencilAttachment(stencil) => {
                        attach(&mut ctxt, gl::STENCIL_ATTACHMENT, id, stencil);
                    },
                    FramebufferDepthStencilAttachments::DepthAndStencilAttachments(depth,
                                                                                   stencil) =>
                    {
                        attach(&mut ctxt, gl::DEPTH_ATTACHMENT, id, depth);
                        attach(&mut ctxt, gl::STENCIL_ATTACHMENT, id, stencil);
                    },
                    FramebufferDepthStencilAttachments::DepthStencilAttachment(depth_stencil) => {
                        attach(&mut ctxt, gl::DEPTH_STENCIL_ATTACHMENT, id, depth_stencil);
                    },
                };

                if ctxt.version >= &GlVersion(Api::Gl, 4, 5) || ctxt.extensions.gl_arb_direct_state_access {
                    ctxt.gl.NamedFramebufferDrawBuffers(id, raw_attachments.len()
                                                        as gl::types::GLsizei,
                                                        raw_attachments.as_ptr());

                } else if ctxt.version >= &GlVersion(Api::Gl, 2, 0) {
                    bind_framebuffer(&mut ctxt, id, true, false);
                    ctxt.gl.DrawBuffers(raw_attachments.len() as gl::types::GLsizei,
                                        raw_attachments.as_ptr());

                } else {
                    unimplemented!();       // FIXME: use an extension
                }
            }
        });

        FrameBufferObject {
            id: rx.recv().unwrap(),
            current_read_buffer: gl::BACK,
        }
    }

    fn destroy(self, context: &context::Context) {
        let id = self.id;

        context.exec(move |ctxt| {
            unsafe {
                // unbinding framebuffer
                if ctxt.version >= &context::GlVersion(Api::Gl, 3, 0) {
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

                } else if ctxt.extensions.gl_ext_framebuffer_object {
                    if ctxt.state.draw_framebuffer == id || ctxt.state.read_framebuffer == id {
                        ctxt.gl.BindFramebufferEXT(gl::FRAMEBUFFER_EXT, 0);
                        ctxt.state.draw_framebuffer = 0;
                        ctxt.state.read_framebuffer = 0;
                    }

                } else {
                    unreachable!();
                }

                // deleting
                if ctxt.version >= &context::GlVersion(Api::Gl, 3, 0) {
                    ctxt.gl.DeleteFramebuffers(1, [ id ].as_ptr());
                } else if ctxt.extensions.gl_ext_framebuffer_object {
                    ctxt.gl.DeleteFramebuffersEXT(1, [ id ].as_ptr());
                } else {
                    unreachable!();
                }
            }
        });
    }
}

impl GlObject for FrameBufferObject {
    type Id = gl::types::GLuint;
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

pub fn bind_framebuffer(ctxt: &mut context::CommandContext, fbo_id: gl::types::GLuint,
                        draw: bool, read: bool)
{
    if draw && ctxt.state.draw_framebuffer != fbo_id {
        unsafe {
            if ctxt.version >= &context::GlVersion(Api::Gl, 3, 0) {
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
            if ctxt.version >= &context::GlVersion(Api::Gl, 3, 0) {
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

unsafe fn attach(ctxt: &mut context::CommandContext, slot: gl::types::GLenum,
                 id: gl::types::GLuint, attachment: Attachment)
{
    if ctxt.version >= &GlVersion(Api::Gl, 4, 5) || ctxt.extensions.gl_arb_direct_state_access {
        match attachment {
            Attachment::Texture { id: tex_id, level, layer, .. } => {
                if layer == 0 {
                    ctxt.gl.NamedFramebufferTexture(id, slot, tex_id,
                                                    level as gl::types::GLint);
                } else {
                    ctxt.gl.NamedFramebufferTextureLayer(id, slot, tex_id,
                                                         level as gl::types::GLint,
                                                         layer as gl::types::GLint);
                }
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
            Attachment::Texture { id: tex_id, level, layer, .. } => {
                if layer == 0 {
                    ctxt.gl.NamedFramebufferTextureEXT(id, slot, tex_id,
                                                       level as gl::types::GLint);
                } else {
                    ctxt.gl.NamedFramebufferTextureLayerEXT(id, slot, tex_id,
                                                            level as gl::types::GLint,
                                                            layer as gl::types::GLint);
                }
            },
            Attachment::RenderBuffer(buf_id) => {
                ctxt.gl.NamedFramebufferRenderbufferEXT(id, slot, gl::RENDERBUFFER,
                                                        buf_id);
            },
        }

    } else if ctxt.version >= &GlVersion(Api::Gl, 3, 2) {
        bind_framebuffer(ctxt, id, true, false);

        match attachment {
            Attachment::Texture { id: tex_id, level, layer, .. } => {
                if layer == 0 {
                    ctxt.gl.FramebufferTexture(gl::DRAW_FRAMEBUFFER,
                                               slot, tex_id, level as gl::types::GLint);
                } else {
                    ctxt.gl.FramebufferTextureLayer(gl::DRAW_FRAMEBUFFER,
                                                    slot, tex_id,
                                                    level as gl::types::GLint,
                                                    layer as gl::types::GLint);
                }
            },
            Attachment::RenderBuffer(buf_id) => {
                ctxt.gl.FramebufferRenderbuffer(gl::DRAW_FRAMEBUFFER, slot,
                                                gl::RENDERBUFFER, buf_id);
            },
        }

    } else if ctxt.version >= &GlVersion(Api::Gl, 3, 0) {
        bind_framebuffer(ctxt, id, true, false);

        match attachment {
            Attachment::Texture { bind_point, id: tex_id, level, layer } => {
                match bind_point {
                    gl::TEXTURE_1D | gl::TEXTURE_RECTANGLE => {
                        assert!(layer == 0);
                        ctxt.gl.FramebufferTexture1D(gl::DRAW_FRAMEBUFFER,
                                                     slot, bind_point, tex_id,
                                                     level as gl::types::GLint);
                    },
                    gl::TEXTURE_2D | gl::TEXTURE_2D_MULTISAMPLE | gl::TEXTURE_1D_ARRAY => {
                        assert!(layer == 0);
                        ctxt.gl.FramebufferTexture2D(gl::DRAW_FRAMEBUFFER,
                                                     slot, bind_point, tex_id,
                                                     level as gl::types::GLint);
                    },
                    gl::TEXTURE_3D | gl::TEXTURE_2D_ARRAY | gl::TEXTURE_2D_MULTISAMPLE_ARRAY => {
                        ctxt.gl.FramebufferTextureLayer(gl::DRAW_FRAMEBUFFER,
                                                        slot, tex_id,
                                                        level as gl::types::GLint,
                                                        layer as gl::types::GLint);
                    },
                    _ => unreachable!()
                }
            },
            Attachment::RenderBuffer(buf_id) => {
                ctxt.gl.FramebufferRenderbuffer(gl::DRAW_FRAMEBUFFER, slot,
                                                gl::RENDERBUFFER, buf_id);
            },
        }

    } else if ctxt.extensions.gl_ext_framebuffer_object {
        bind_framebuffer(ctxt, id, true, true);

        match attachment {
            Attachment::Texture { bind_point, id: tex_id, level, layer } => {
                match bind_point {
                    gl::TEXTURE_1D | gl::TEXTURE_RECTANGLE => {
                        assert!(layer == 0);
                        ctxt.gl.FramebufferTexture1DEXT(gl::FRAMEBUFFER_EXT,
                                                        slot, bind_point, tex_id,
                                                        level as gl::types::GLint);
                    },
                    gl::TEXTURE_2D | gl::TEXTURE_2D_MULTISAMPLE | gl::TEXTURE_1D_ARRAY => {
                        assert!(layer == 0);
                        ctxt.gl.FramebufferTexture2DEXT(gl::FRAMEBUFFER_EXT,
                                                        slot, bind_point, tex_id,
                                                        level as gl::types::GLint);
                    },
                    gl::TEXTURE_3D | gl::TEXTURE_2D_ARRAY | gl::TEXTURE_2D_MULTISAMPLE_ARRAY => {
                        ctxt.gl.FramebufferTexture3DEXT(gl::FRAMEBUFFER_EXT,
                                                        slot, bind_point, tex_id,
                                                        level as gl::types::GLint,
                                                        layer as gl::types::GLint);
                    },
                    _ => unreachable!()
                }
            },
            Attachment::RenderBuffer(buf_id) => {
                ctxt.gl.FramebufferRenderbufferEXT(gl::DRAW_FRAMEBUFFER, slot,
                                                   gl::RENDERBUFFER, buf_id);
            },
        }

    } else {
        unreachable!();
    }
}
