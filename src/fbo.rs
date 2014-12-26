use std::mem;
use std::sync::Arc;

use uniforms::Uniforms;
use {DisplayImpl, Program, DrawParameters, Rect, Surface, GlObject, ToGlEnum};
use index_buffer::IndicesSource;
use vertex_buffer::VerticesSource;

use {program, vertex_array_object};
use {gl, context};

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

/// Draws everything.
pub fn draw<I, U>(display: &Arc<DisplayImpl>,
    framebuffer: Option<&FramebufferAttachments>, vertex_buffer: VerticesSource,
    indices: &IndicesSource<I>, program: &Program, uniforms: &U, draw_parameters: &DrawParameters,
    dimensions: (u32, u32)) where U: Uniforms, I: ::index_buffer::Index
{
    let fbo_id = get_framebuffer(display, framebuffer);

    let vao_id = vertex_array_object::get_vertex_array_object(display, vertex_buffer.clone(),
                                                              indices, program);

    let pointer = match indices {
        &IndicesSource::IndexBuffer { .. } => ::std::ptr::null(),
        &IndicesSource::Buffer { ref pointer, .. } => pointer.as_ptr() as *const ::libc::c_void,
    };

    let primitives = indices.get_primitives_type().to_glenum();
    let data_type = indices.get_indices_type().to_glenum();
    assert!(indices.get_offset() == 0); // not yet implemented
    let indices_count = indices.get_length();

    let uniforms = uniforms.to_binder();
    let uniforms_locations = program::get_uniforms_locations(program);
    let draw_parameters = draw_parameters.clone();

    let VerticesSource::VertexBuffer(vertex_buffer) = vertex_buffer;
    let vb_id = vertex_buffer.get_id();
    let program_id = program.get_id();

    display.context.exec(move |: mut ctxt| {
        unsafe {
            bind_framebuffer(&mut ctxt, fbo_id, true, false);

            // binding program
            if ctxt.state.program != program_id {
                ctxt.gl.UseProgram(program_id);
                ctxt.state.program = program_id;
            }

            // binding program uniforms
            let mut active_texture = gl::TEXTURE0;
            uniforms.0.call((&mut ctxt, box |&: name| {
                uniforms_locations
                    .get(name)
                    .map(|val| val.location)
            }, &mut active_texture));

            // binding VAO
            if ctxt.state.vertex_array != vao_id {
                ctxt.gl.BindVertexArray(vao_id);
                ctxt.state.vertex_array = vao_id;
            }

            // binding vertex buffer
            if ctxt.state.array_buffer_binding != vb_id {
                ctxt.gl.BindBuffer(gl::ARRAY_BUFFER, vb_id);
                ctxt.state.array_buffer_binding = vb_id;
            }

            // sync-ing parameters
            draw_parameters.sync(&mut ctxt, dimensions);

            // drawing
            ctxt.gl.DrawElements(primitives, indices_count as i32, data_type, pointer);
        }
    });
}

pub fn clear_color(display: &Arc<DisplayImpl>, framebuffer: Option<&FramebufferAttachments>,
    red: f32, green: f32, blue: f32, alpha: f32)
{
    let fbo_id = get_framebuffer(display, framebuffer);

    let (red, green, blue, alpha) = (
        red as gl::types::GLclampf,
        green as gl::types::GLclampf,
        blue as gl::types::GLclampf,
        alpha as gl::types::GLclampf
    );

    display.context.exec(move |: mut ctxt| {
        bind_framebuffer(&mut ctxt, fbo_id, true, false);

        unsafe {
            if ctxt.state.clear_color != (red, green, blue, alpha) {
                ctxt.gl.ClearColor(red, green, blue, alpha);
                ctxt.state.clear_color = (red, green, blue, alpha);
            }

            ctxt.gl.Clear(gl::COLOR_BUFFER_BIT);
        }
    });
}

pub fn clear_depth(display: &Arc<DisplayImpl>, framebuffer: Option<&FramebufferAttachments>,
    value: f32)
{
    let value = value as gl::types::GLclampf;
    let fbo_id = get_framebuffer(display, framebuffer);

    display.context.exec(move |: mut ctxt| {
        bind_framebuffer(&mut ctxt, fbo_id, true, false);

        unsafe {
            if ctxt.state.clear_depth != value {
                ctxt.gl.ClearDepth(value as f64);        // TODO: find out why this needs "as"
                ctxt.state.clear_depth = value;
            }

            ctxt.gl.Clear(gl::DEPTH_BUFFER_BIT);
        }
    });
}

pub fn clear_stencil(display: &Arc<DisplayImpl>, framebuffer: Option<&FramebufferAttachments>,
    value: int)
{
    let value = value as gl::types::GLint;
    let fbo_id = get_framebuffer(display, framebuffer);

    display.context.exec(move |: mut ctxt| {
        bind_framebuffer(&mut ctxt, fbo_id, true, false);

        unsafe {
            if ctxt.state.clear_stencil != value {
                ctxt.gl.ClearStencil(value);
                ctxt.state.clear_stencil = value;
            }

            ctxt.gl.Clear(gl::STENCIL_BUFFER_BIT);
        }
    });
}

pub fn blit<S1: Surface, S2: Surface>(source: &S1, target: &S2, mask: gl::types::GLbitfield,
    src_rect: &Rect, target_rect: &Rect, filter: gl::types::GLenum)
{
    let ::BlitHelper(display, source) = source.get_blit_helper();
    let ::BlitHelper(_, target) = target.get_blit_helper();

    let src_rect = src_rect.clone();
    let target_rect = target_rect.clone();

    let source = get_framebuffer(display, source);
    let target = get_framebuffer(display, target);

    display.context.exec(move |: ctxt| {
        unsafe {
            // trying to do a named blit if possible
            if ctxt.version >= &context::GlVersion(4, 5) {
                ctxt.gl.BlitNamedFramebuffer(source.unwrap_or(0), target.unwrap_or(0),
                    src_rect.left as gl::types::GLint,
                    src_rect.bottom as gl::types::GLint,
                    (src_rect.left + src_rect.width) as gl::types::GLint,
                    (src_rect.bottom + src_rect.height) as gl::types::GLint,
                    target_rect.left as gl::types::GLint, target_rect.bottom as gl::types::GLint,
                    (target_rect.left + target_rect.width) as gl::types::GLint,
                    (target_rect.bottom + target_rect.height) as gl::types::GLint, mask, filter);

                return;
            }

            // binding source framebuffer
            if ctxt.state.read_framebuffer != source.unwrap_or(0) {
                if ctxt.version >= &context::GlVersion(3, 0) {
                    ctxt.gl.BindFramebuffer(gl::READ_FRAMEBUFFER, source.unwrap_or(0));
                    ctxt.state.read_framebuffer = source.unwrap_or(0);

                } else {
                    ctxt.gl.BindFramebufferEXT(gl::READ_FRAMEBUFFER_EXT, source.unwrap_or(0));
                    ctxt.state.read_framebuffer = source.unwrap_or(0);
                }
            }

            // binding target framebuffer
            if ctxt.state.draw_framebuffer != target.unwrap_or(0) {
                if ctxt.version >= &context::GlVersion(3, 0) {
                    ctxt.gl.BindFramebuffer(gl::DRAW_FRAMEBUFFER, target.unwrap_or(0));
                    ctxt.state.draw_framebuffer = target.unwrap_or(0);

                } else {
                    ctxt.gl.BindFramebufferEXT(gl::DRAW_FRAMEBUFFER_EXT, target.unwrap_or(0));
                    ctxt.state.draw_framebuffer = target.unwrap_or(0);
                }
            }

            // doing the blit
            if ctxt.version >= &context::GlVersion(3, 0) {
                ctxt.gl.BlitFramebuffer(src_rect.left as gl::types::GLint,
                    src_rect.bottom as gl::types::GLint,
                    (src_rect.left + src_rect.width) as gl::types::GLint,
                    (src_rect.bottom + src_rect.height) as gl::types::GLint,
                    target_rect.left as gl::types::GLint, target_rect.bottom as gl::types::GLint,
                    (target_rect.left + target_rect.width) as gl::types::GLint,
                    (target_rect.bottom + target_rect.height) as gl::types::GLint, mask, filter);

            } else {
                ctxt.gl.BlitFramebufferEXT(src_rect.left as gl::types::GLint,
                    src_rect.bottom as gl::types::GLint,
                    (src_rect.left + src_rect.width) as gl::types::GLint,
                    (src_rect.bottom + src_rect.height) as gl::types::GLint,
                    target_rect.left as gl::types::GLint, target_rect.bottom as gl::types::GLint,
                    (target_rect.left + target_rect.width) as gl::types::GLint,
                    (target_rect.bottom + target_rect.height) as gl::types::GLint, mask, filter);
            }
        }
    });
}

fn get_framebuffer(display: &Arc<DisplayImpl>, framebuffer: Option<&FramebufferAttachments>)
    -> Option<gl::types::GLuint>
{
    if let Some(framebuffer) = framebuffer {
        let mut framebuffers = display.framebuffer_objects.lock();

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

fn bind_framebuffer(ctxt: &mut context::CommandContext, fbo_id: Option<gl::types::GLuint>,
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
