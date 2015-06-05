/*!
Contains everything related to the internal handling of framebuffer objects.

*/
/*
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
use std::collections::HashMap;
use std::cmp;
use std::mem;
use std::cell::RefCell;
use std::marker::PhantomData;

use GlObject;
use TextureExt;

use texture::TextureAny;
use framebuffer::RenderBufferAny;

use gl;
use context::CommandContext;
use version::Version;
use version::Api;

/// Represents the attachments to use for an OpenGL framebuffer.
#[derive(Clone)]
pub struct FramebufferAttachments<'a> {
    /// List of color attachments. The first parameter of the tuple is the index, and the
    /// second element is the attachment.
    pub colors: Vec<(u32, Attachment<'a>)>,

    /// The depth and/or stencil attachment to use.
    pub depth_stencil: FramebufferDepthStencilAttachments<'a>,
}

/// Depth and/or stencil attachment to use.
#[derive(Copy, Clone)]
pub enum FramebufferDepthStencilAttachments<'a> {
    /// No depth or stencil buffer.
    None,

    /// A depth attachment.
    DepthAttachment(Attachment<'a>),

    /// A stencil attachment.
    StencilAttachment(Attachment<'a>),

    /// A depth attachment and a stencil attachment.
    DepthAndStencilAttachments(Attachment<'a>, Attachment<'a>),

    /// A single attachment that serves as both depth and stencil buffer.
    DepthStencilAttachment(Attachment<'a>),
}

/// Describes a single framebuffer attachment.
#[derive(Copy, Clone)]
pub enum Attachment<'a> {
    /// An entire texture.
    Texture {
        /// The texture.
        texture: &'a TextureAny,
        /// Mipmap level to use. The main texture is level 0.
        level: u32,
    },
    /// A layer of a texture. Layers are in texture arrays or in 3D textures.
    TextureLayer {
        /// The texture.
        texture: &'a TextureAny,
        /// The layer.
        layer: u32,
        /// Mipmap level to use. The main texture is level 0.
        level: u32,
    },
    /// A renderbuffer.
    RenderBuffer(&'a RenderBufferAny),
}

impl<'a> FramebufferAttachments<'a> {
    /// After building a `FramebufferAttachments` struct, you must use this function
    /// to "compile" the attachments and make sure that they are valid together.
    pub fn validate(self)
                    -> Result<ValidatedAttachments<'a>, ValidationError>
    {
        // turning the attachments into raw attachments
        let (raw_attachments, dimensions, depth_bits, stencil_bits) = {
            fn handle_attachment(a: &Attachment, dim: &mut Option<(u32, u32)>,
                                 num_bits: Option<&mut Option<u16>>)
                                 -> RawAttachment
            {
                match a {
                    &Attachment::Texture { ref texture, level } => {
                        if let Some(num_bits) = num_bits {
                            *num_bits = Some(texture.get_internal_format_if_supported()
                                               .map(|f| f.get_total_bits()).unwrap_or(24) as u16);     // TODO: how to handle this?
                        }

                        match dim {
                            d @ &mut None => *d = Some((texture.get_width(), texture.get_height().unwrap_or(1))),
                            &mut Some((ref mut x, ref mut y)) => {
                                *x = cmp::min(*x, texture.get_width());
                                *y = cmp::min(*y, texture.get_height().unwrap_or(1));
                            }
                        }

                        RawAttachment::Texture {
                            texture: texture.get_id(),
                            bind_point: texture.get_bind_point(),
                            layer: 0,
                            level: level,       // TODO: check validity
                        }
                    },
                    &Attachment::TextureLayer { ref texture, level, layer } => {
                        if let Some(num_bits) = num_bits {
                            *num_bits = Some(texture.get_internal_format_if_supported()
                                               .map(|f| f.get_total_bits()).unwrap_or(24) as u16);     // TODO: how to handle this?
                        }

                        match dim {
                            d @ &mut None => *d = Some((texture.get_width(), texture.get_height().unwrap_or(1))),
                            &mut Some((ref mut x, ref mut y)) => {
                                *x = cmp::min(*x, texture.get_width());
                                *y = cmp::min(*y, texture.get_height().unwrap_or(1));
                            }
                        }

                        RawAttachment::Texture {
                            texture: texture.get_id(),
                            bind_point: texture.get_bind_point(),
                            layer: layer,       // TODO: check validity
                            level: level,       // TODO: check validity
                        }
                    },
                    &Attachment::RenderBuffer(ref buffer) => {
                        if let Some(num_bits) = num_bits {
                            *num_bits = Some(24);    // FIXME: totally random
                        }

                        match dim {
                            d @ &mut None => *d = Some(buffer.get_dimensions()),
                            &mut Some((ref mut x, ref mut y)) => {
                                let curr = buffer.get_dimensions();
                                *x = cmp::min(*x, curr.0);
                                *y = cmp::min(*y, curr.1);
                            }
                        }

                        RawAttachment::RenderBuffer(buffer.get_id())
                    },
                }
            }

            // TODO: check number of samples
            // TODO: check layering

            // the dimensions of the framebuffer object
            let mut dimensions = None;
            // number of depth bits
            let mut depth_bits = None;
            // number of stencil bits
            let mut stencil_bits = None;

            let mut raw_attachments = RawAttachments {
                color: Vec::with_capacity(self.colors.len()),
                depth: None,
                stencil: None,
                depth_stencil: None,
            };

            for &(index, ref a) in &self.colors {
                raw_attachments.color.push((index, handle_attachment(a, &mut dimensions, None)));
            }

            match self.depth_stencil {
                FramebufferDepthStencilAttachments::None => (),
                FramebufferDepthStencilAttachments::DepthAttachment(ref a) => {
                    raw_attachments.depth = Some(handle_attachment(a, &mut dimensions, Some(&mut depth_bits)));
                },
                FramebufferDepthStencilAttachments::StencilAttachment(ref a) => {
                    raw_attachments.stencil = Some(handle_attachment(a, &mut dimensions, Some(&mut stencil_bits)));
                },
                FramebufferDepthStencilAttachments::DepthAndStencilAttachments(ref d, ref s) => {
                    raw_attachments.depth = Some(handle_attachment(d, &mut dimensions, Some(&mut depth_bits)));
                    raw_attachments.stencil = Some(handle_attachment(s, &mut dimensions, Some(&mut stencil_bits)));
                },
                FramebufferDepthStencilAttachments::DepthStencilAttachment(ref a) => {
                    raw_attachments.depth_stencil = Some(handle_attachment(a, &mut dimensions, None));      // FIXME: bit counts
                },
            }

            let dimensions = match dimensions {
                Some(d) => d,
                None => return Err(ValidationError::EmptyFramebufferObjectsNotSupported)
            };

            (raw_attachments, dimensions, depth_bits, stencil_bits)
        };

        Ok(ValidatedAttachments {
            raw: raw_attachments,
            marker: PhantomData,
            dimensions: dimensions,
            depth_buffer_bits: depth_bits,
            stencil_buffer_bits: stencil_bits,
        })
    }
}

/// Represents attachments that have been validated and are usable.
#[derive(Clone)]
pub struct ValidatedAttachments<'a> {
    marker: PhantomData<&'a ()>,
    raw: RawAttachments,
    dimensions: (u32, u32),
    depth_buffer_bits: Option<u16>,
    stencil_buffer_bits: Option<u16>,
}

impl<'a> ValidatedAttachments<'a> {
    /// Returns the dimensions that the framebuffer will have if you use these attachments.
    pub fn get_dimensions(&self) -> (u32, u32) {
        self.dimensions
    }

    /// Returns the number of bits of precision of the depth buffer, or `None` if there is no
    /// depth buffer. Also works for depth-stencil buffers.
    pub fn get_depth_buffer_bits(&self) -> Option<u16> {
        self.depth_buffer_bits
    }

    /// Returns the number of bits of precision of the stencil buffer, or `None` if there is no
    /// stencil buffer. Also works for depth-stencil buffers.
    pub fn get_stencil_buffer_bits(&self) -> Option<u16> {
        self.stencil_buffer_bits
    }
}

/// An error that can happen while validating attachments.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ValidationError {
    EmptyFramebufferObjectsNotSupported,
}

/// Data structure stored in the hashmap.
#[derive(Hash, Clone, Eq, PartialEq)]
struct RawAttachments {
    color: Vec<(u32, RawAttachment)>,
    depth: Option<RawAttachment>,
    stencil: Option<RawAttachment>,
    depth_stencil: Option<RawAttachment>,
}

/// Single attachment.
#[derive(Hash, Copy, Clone, Eq, PartialEq)]
enum RawAttachment {
    Texture {
        bind_point: gl::types::GLenum,
        texture: gl::types::GLuint,
        layer: u32,
        level: u32,
    },
    RenderBuffer(gl::types::GLuint),
}

/// Manages all the framebuffer objects.
///
/// `cleanup` **must** be called when destroying the container, otherwise `Drop` will panic.
pub struct FramebuffersContainer {
    framebuffers: RefCell<HashMap<RawAttachments, FrameBufferObject>>,
}

/// Frame buffer.
struct FrameBufferObject {
    id: gl::types::GLuint,
    current_read_buffer: gl::types::GLenum,
}

impl FramebuffersContainer {
    /// Initializes the container.
    pub fn new() -> FramebuffersContainer {
        FramebuffersContainer {
            framebuffers: RefCell::new(HashMap::new()),
        }
    }

    /// Destroys all framebuffer objects. This is used when using a new context for example.
    pub fn purge_all(&self, ctxt: &mut CommandContext) {
        let mut other = HashMap::new();
        mem::swap(&mut *self.framebuffers.borrow_mut(), &mut other);

        for (_, obj) in other.into_iter() {
            obj.destroy(ctxt);
        }
    }

    /// Destroys all framebuffer objects that contain a precise texture.
    pub fn purge_texture(&self, texture: gl::types::GLuint, ctxt: &mut CommandContext) {
        self.purge_if(|a| {
            match a {
                &RawAttachment::Texture { texture: id, .. } if id == texture => true,
                _ => false 
            }
        }, ctxt);
    }

    /// Destroys all framebuffer objects that contain a precise renderbuffer.
    pub fn purge_renderbuffer(&self, renderbuffer: gl::types::GLuint,
                              ctxt: &mut CommandContext)
    {
        self.purge_if(|a| a == &RawAttachment::RenderBuffer(renderbuffer), ctxt);
    }

    /// Destroys all framebuffer objects that match a certain condition.
    fn purge_if<F>(&self, condition: F, mut ctxt: &mut CommandContext)
                   where F: Fn(&RawAttachment) -> bool
    {
        let mut framebuffers = self.framebuffers.borrow_mut();

        let mut attachments = Vec::with_capacity(0);
        for (key, _) in framebuffers.iter() {
            if key.color.iter().find(|&&(_, ref id)| condition(id)).is_some() {
                attachments.push(key.clone());
                continue;
            }

            if let Some(ref atch) = key.depth {
                if condition(atch) {
                    attachments.push(key.clone());
                    continue;
                }
            }

            if let Some(ref atch) = key.stencil {
                if condition(atch) {
                    attachments.push(key.clone());
                    continue;
                }
            }

            if let Some(ref atch) = key.depth_stencil {
                if condition(atch) {
                    attachments.push(key.clone());
                    continue;
                }
            }
        }

        for atch in attachments.into_iter() {
            framebuffers.remove(&atch).unwrap().destroy(ctxt);
        }
    }

    /// Destroys all framebuffer objects.
    ///
    /// This is very similar to `purge_all`, but optimized for when the container will soon
    /// be destroyed.
    pub fn cleanup(self, ctxt: &mut CommandContext) {
        let mut other = HashMap::with_capacity(0);
        mem::swap(&mut *self.framebuffers.borrow_mut(), &mut other);

        for (_, obj) in other.into_iter() {
            obj.destroy(ctxt);
        }
    }

    ///
    /// # Unsafety
    ///
    /// After calling this function, you **must** make sure to call `purge_texture`
    /// and/or `purge_renderbuffer` when one of the attachment is destroyed.
    pub fn get_framebuffer_for_drawing(&self, attachments: Option<&ValidatedAttachments>,
                                       ctxt: &mut CommandContext) -> gl::types::GLuint
    {
        if let Some(attachments) = attachments {
            self.get_framebuffer(attachments, ctxt)
        } else {
            0
        }
    }

    ///
    /// # Unsafety
    ///
    /// After calling this function, you **must** make sure to call `purge_texture`
    /// and/or `purge_renderbuffer` when one of the attachment is destroyed.
    pub fn get_framebuffer_for_reading(&self, attachment: &Attachment, ctxt: &mut CommandContext)
                                       -> (gl::types::GLuint, gl::types::GLenum)
    {
        // TODO: restore this optimisation
        /*for (attachments, fbo) in self.framebuffers.borrow_mut().iter() {
            for &(key, ref atc) in attachments.color.iter() {
                if atc == attachment {
                    return (fbo.get_id(), gl::COLOR_ATTACHMENT0 + key);
                }
            }
        }*/

        let attachments = FramebufferAttachments {
            colors: vec![(0, attachment.clone())],
            depth_stencil: FramebufferDepthStencilAttachments::None,
        }.validate().unwrap();

        let framebuffer = self.get_framebuffer_for_drawing(Some(&attachments), ctxt);
        (framebuffer, gl::COLOR_ATTACHMENT0)
    }

    ///
    /// # Unsafety
    ///
    /// After calling this function, you **must** make sure to call `purge_texture`
    /// and/or `purge_renderbuffer` when one of the attachment is destroyed.
    fn get_framebuffer(&self, attachments: &ValidatedAttachments,
                       ctxt: &mut CommandContext) -> gl::types::GLuint
    {

        let mut framebuffers = self.framebuffers.borrow_mut();
        if let Some(value) = framebuffers.get(&attachments.raw) {
            return value.id;
        }

        let new_fbo = FrameBufferObject::new(ctxt, &attachments.raw);
        let new_fbo_id = new_fbo.id.clone();
        framebuffers.insert(attachments.raw.clone(), new_fbo);
        new_fbo_id
    }
}

impl Drop for FramebuffersContainer {
    fn drop(&mut self) {
        if self.framebuffers.borrow().len() != 0 {
            panic!()
        }
    }
}

impl FrameBufferObject {
    /// Builds a new FBO.
    fn new(mut ctxt: &mut CommandContext, attachments: &RawAttachments) -> FrameBufferObject {
        if attachments.color.len() > ctxt.capabilities.max_draw_buffers as usize {
            panic!("Trying to attach {} color buffers, but the hardware only supports {}",
                   attachments.color.len(), ctxt.capabilities.max_draw_buffers);
        }

        let id = unsafe {
            let mut id = mem::uninitialized();

            if ctxt.version >= &Version(Api::Gl, 4, 5) ||
                ctxt.extensions.gl_arb_direct_state_access
            {
                ctxt.gl.CreateFramebuffers(1, &mut id);
            } else if ctxt.version >= &Version(Api::Gl, 3, 0) ||
                ctxt.version >= &Version(Api::GlEs, 2, 0)
            {
                ctxt.gl.GenFramebuffers(1, &mut id);
                bind_framebuffer(&mut ctxt, id, true, false);
            } else {
                ctxt.gl.GenFramebuffersEXT(1, &mut id);
                bind_framebuffer(&mut ctxt, id, true, false);
            }

            let mut raw_attachments: Vec<gl::types::GLenum> = Vec::new();

            for &(slot, atchmnt) in attachments.color.iter() {
                attach(&mut ctxt, gl::COLOR_ATTACHMENT0 + slot as u32, id, atchmnt);
                raw_attachments.push(gl::COLOR_ATTACHMENT0 + slot as u32);
            }

            if let Some(depth) = attachments.depth {
                attach(&mut ctxt, gl::DEPTH_ATTACHMENT, id, depth);
            }
            if let Some(stencil) = attachments.stencil {
                attach(&mut ctxt, gl::STENCIL_ATTACHMENT, id, stencil);
            }
            if let Some(depth_stencil) = attachments.depth_stencil {
                attach(&mut ctxt, gl::DEPTH_STENCIL_ATTACHMENT, id, depth_stencil);
            }

            if ctxt.version >= &Version(Api::Gl, 4, 5) ||
               ctxt.extensions.gl_arb_direct_state_access
            {
                ctxt.gl.NamedFramebufferDrawBuffers(id, raw_attachments.len()
                                                    as gl::types::GLsizei,
                                                    raw_attachments.as_ptr());

            } else if ctxt.version >= &Version(Api::Gl, 2, 0) ||
                      ctxt.version >= &Version(Api::GlEs, 3, 0)
            {
                bind_framebuffer(&mut ctxt, id, true, false);
                ctxt.gl.DrawBuffers(raw_attachments.len() as gl::types::GLsizei,
                                    raw_attachments.as_ptr());

            } else if ctxt.version >= &Version(Api::GlEs, 2, 0) {
                assert_eq!(raw_attachments, &[gl::COLOR_ATTACHMENT0]);

            } else {
                unimplemented!();       // FIXME: use an extension
            }

            id
        };

        FrameBufferObject {
            id: id,
            current_read_buffer: gl::BACK,
        }
    }

    fn destroy(self, mut ctxt: &mut CommandContext) {
        unsafe {
            // unbinding framebuffer
            if ctxt.version >= &Version(Api::Gl, 3, 0) {
                if ctxt.state.draw_framebuffer == self.id && ctxt.state.read_framebuffer == self.id {
                    ctxt.gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
                    ctxt.state.draw_framebuffer = 0;
                    ctxt.state.read_framebuffer = 0;

                } else if ctxt.state.draw_framebuffer == self.id {
                    ctxt.gl.BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
                    ctxt.state.draw_framebuffer = 0;

                } else if ctxt.state.read_framebuffer == self.id {
                    ctxt.gl.BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
                    ctxt.state.read_framebuffer = 0;
                }

            } else if ctxt.version >= &Version(Api::GlEs, 2, 0) {
                if ctxt.state.draw_framebuffer == self.id || ctxt.state.read_framebuffer == self.id {
                    ctxt.gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
                    ctxt.state.draw_framebuffer = 0;
                    ctxt.state.read_framebuffer = 0;
                }

            } else if ctxt.extensions.gl_ext_framebuffer_object {
                if ctxt.state.draw_framebuffer == self.id || ctxt.state.read_framebuffer == self.id {
                    ctxt.gl.BindFramebufferEXT(gl::FRAMEBUFFER_EXT, 0);
                    ctxt.state.draw_framebuffer = 0;
                    ctxt.state.read_framebuffer = 0;
                }

            } else {
                unreachable!();
            }

            // deleting
            if ctxt.version >= &Version(Api::Gl, 3, 0) ||
                ctxt.version >= &Version(Api::GlEs, 2, 0)
            {
                ctxt.gl.DeleteFramebuffers(1, [ self.id ].as_ptr());
            } else if ctxt.extensions.gl_ext_framebuffer_object {
                ctxt.gl.DeleteFramebuffersEXT(1, [ self.id ].as_ptr());
            } else {
                unreachable!();
            }
        }
    }
}

impl GlObject for FrameBufferObject {
    type Id = gl::types::GLuint;
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

pub fn bind_framebuffer(ctxt: &mut CommandContext, fbo_id: gl::types::GLuint,
                        draw: bool, read: bool)
{
    if draw && ctxt.state.draw_framebuffer != fbo_id {
        unsafe {
            if ctxt.version >= &Version(Api::Gl, 3, 0) {
                ctxt.gl.BindFramebuffer(gl::DRAW_FRAMEBUFFER, fbo_id);
                ctxt.state.draw_framebuffer = fbo_id;
            } else if ctxt.version >= &Version(Api::GlEs, 2, 0) {
                ctxt.gl.BindFramebuffer(gl::FRAMEBUFFER, fbo_id);
                ctxt.state.draw_framebuffer = fbo_id;
                ctxt.state.read_framebuffer = fbo_id;
            } else {
                ctxt.gl.BindFramebufferEXT(gl::FRAMEBUFFER_EXT, fbo_id);
                ctxt.state.draw_framebuffer = fbo_id;
                ctxt.state.read_framebuffer = fbo_id;
            }
        }
    }

    if read && ctxt.state.read_framebuffer != fbo_id {
        unsafe {
            if ctxt.version >= &Version(Api::Gl, 3, 0) {
                ctxt.gl.BindFramebuffer(gl::READ_FRAMEBUFFER, fbo_id);
                ctxt.state.read_framebuffer = fbo_id;
            } else if ctxt.version >= &Version(Api::GlEs, 2, 0) {
                ctxt.gl.BindFramebuffer(gl::FRAMEBUFFER, fbo_id);
                ctxt.state.draw_framebuffer = fbo_id;
                ctxt.state.read_framebuffer = fbo_id;
            } else {
                ctxt.gl.BindFramebufferEXT(gl::FRAMEBUFFER_EXT, fbo_id);
                ctxt.state.draw_framebuffer = fbo_id;
                ctxt.state.read_framebuffer = fbo_id;
            }
        }
    }
}

unsafe fn attach(ctxt: &mut CommandContext, slot: gl::types::GLenum,
                 id: gl::types::GLuint, attachment: RawAttachment)
{
    if ctxt.version >= &Version(Api::Gl, 4, 5) || ctxt.extensions.gl_arb_direct_state_access {
        match attachment {
            RawAttachment::Texture { texture: tex_id, level, layer, .. } => {
                if layer == 0 {
                    ctxt.gl.NamedFramebufferTexture(id, slot, tex_id,
                                                    level as gl::types::GLint);
                } else {
                    ctxt.gl.NamedFramebufferTextureLayer(id, slot, tex_id,
                                                         level as gl::types::GLint,
                                                         layer as gl::types::GLint);
                }
            },
            RawAttachment::RenderBuffer(buf_id) => {
                ctxt.gl.NamedFramebufferRenderbuffer(id, slot, gl::RENDERBUFFER,
                                                     buf_id);
            },
        }

    } else if ctxt.extensions.gl_ext_direct_state_access &&
              ctxt.extensions.gl_ext_geometry_shader4
    {
        match attachment {
            RawAttachment::Texture { texture: tex_id, level, layer, .. } => {
                if layer == 0 {
                    ctxt.gl.NamedFramebufferTextureEXT(id, slot, tex_id,
                                                       level as gl::types::GLint);
                } else {
                    ctxt.gl.NamedFramebufferTextureLayerEXT(id, slot, tex_id,
                                                            level as gl::types::GLint,
                                                            layer as gl::types::GLint);
                }
            },
            RawAttachment::RenderBuffer(buf_id) => {
                ctxt.gl.NamedFramebufferRenderbufferEXT(id, slot, gl::RENDERBUFFER,
                                                        buf_id);
            },
        }

    } else if ctxt.version >= &Version(Api::Gl, 3, 2) {
        bind_framebuffer(ctxt, id, true, false);

        match attachment {
            RawAttachment::Texture { texture: tex_id, level, layer, .. } => {
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
            RawAttachment::RenderBuffer(buf_id) => {
                ctxt.gl.FramebufferRenderbuffer(gl::DRAW_FRAMEBUFFER, slot,
                                                gl::RENDERBUFFER, buf_id);
            },
        }

    } else if ctxt.version >= &Version(Api::Gl, 3, 0) {
        bind_framebuffer(ctxt, id, true, false);

        match attachment {
            RawAttachment::Texture { bind_point, texture: tex_id, level, layer } => {
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
            RawAttachment::RenderBuffer(buf_id) => {
                ctxt.gl.FramebufferRenderbuffer(gl::DRAW_FRAMEBUFFER, slot,
                                                gl::RENDERBUFFER, buf_id);
            },
        }

    } else if ctxt.version >= &Version(Api::GlEs, 2, 0) {
        bind_framebuffer(ctxt, id, true, true);

        match attachment {
            RawAttachment::Texture { bind_point, texture: tex_id, level, layer } => {
                match bind_point {
                    gl::TEXTURE_2D => {
                        assert!(layer == 0);
                        ctxt.gl.FramebufferTexture2D(gl::FRAMEBUFFER,
                                                     slot, bind_point, tex_id,
                                                     level as gl::types::GLint);
                    },
                    _ => unreachable!()
                }
            },
            RawAttachment::RenderBuffer(buf_id) => {
                ctxt.gl.FramebufferRenderbuffer(gl::DRAW_FRAMEBUFFER, slot,
                                                gl::RENDERBUFFER, buf_id);
            },
        }

    } else if ctxt.extensions.gl_ext_framebuffer_object {
        bind_framebuffer(ctxt, id, true, true);

        match attachment {
            RawAttachment::Texture { bind_point, texture: tex_id, level, layer } => {
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
            RawAttachment::RenderBuffer(buf_id) => {
                ctxt.gl.FramebufferRenderbufferEXT(gl::DRAW_FRAMEBUFFER, slot,
                                                   gl::RENDERBUFFER, buf_id);
            },
        }

    } else {
        unreachable!();
    }
}
