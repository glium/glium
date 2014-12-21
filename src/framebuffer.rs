use std::kinds::marker::ContravariantLifetime;
use std::mem;
use std::sync::Arc;

use texture::{Texture, Texture2d};
use uniforms::Uniforms;
use {DisplayImpl, VertexBuffer, Program, DrawParameters, Rect, Surface, GlObject};
use IndicesSource;

use {program, vertex_array_object};
use {gl, context};

/// A framebuffer that you can use to draw things.
///
/// ## Low-level informations
///
/// Creating a `FrameBuffer` does **not** immediatly create a FrameBuffer Object. Instead, it is
/// created when you first use it.
///
/// FBOs are stored globally (in the `Display` object), which means that if you create a
/// `FrameBuffer`, destroy it, and then recreate the exact same `FrameBuffer`, the FBO previously
/// used will be re-used.
///
/// Note that these informations are implementation details and may change in the future.
///
pub struct FrameBuffer<'a> {
    display: Arc<DisplayImpl>,
    attachments: FramebufferAttachments,
    marker: ContravariantLifetime<'a>,
    dimensions: Option<(u32, u32)>,
}

impl<'a> FrameBuffer<'a> {
    /// Creates an empty framebuffer.
    pub fn new(display: &::Display) -> FrameBuffer<'a> {
        FrameBuffer {
            display: display.context.clone(),
            attachments: FramebufferAttachments {
                colors: Vec::new(),
                depth: None,
                stencil: None
            },
            marker: ContravariantLifetime,
            dimensions: None,
        }
    }

    /// Attach an additional texture to this framebuffer.
    pub fn with_color_texture(mut self, texture: &'a Texture2d) -> FrameBuffer<'a> {
        // TODO: check existing dimensions
        self.attachments.colors.push(texture.get_implementation().get_id());
        self.dimensions = Some((texture.get_width(), texture.get_height().unwrap_or(1)));
        self
    }
}

impl<'a> Surface for FrameBuffer<'a> {
    fn clear_color(&mut self, red: f32, green: f32, blue: f32, alpha: f32) {
        clear_color(&self.display, Some(&self.attachments), red, green, blue, alpha)
    }

    fn clear_depth(&mut self, value: f32) {
        clear_depth(&self.display, Some(&self.attachments), value)
    }

    fn clear_stencil(&mut self, value: int) {
        clear_stencil(&self.display, Some(&self.attachments), value)
    }

    fn get_dimensions(&self) -> (uint, uint) {
        let dimensions = self.dimensions.expect("no texture was bound to this framebuffer");
        (dimensions.0 as uint, dimensions.1 as uint)
    }

    fn get_depth_buffer_bits(&self) -> Option<u16> {
        None
    }

    fn get_stencil_buffer_bits(&self) -> Option<u16> {
        None
    }

    fn draw<V, I, U>(&mut self, vb: &::VertexBuffer<V>, ib: &I, program: &::Program,
        uniforms: &U, draw_parameters: &::DrawParameters) where I: ::IndicesSource,
        U: ::uniforms::Uniforms
    {
        draw(&self.display, Some(&self.attachments), vb, ib, program, uniforms, draw_parameters)
    }

    fn get_blit_helper(&self) -> ::BlitHelper {
        ::BlitHelper(&self.display, Some(&self.attachments))
    }
}

#[deriving(Hash, Clone, PartialEq, Eq)]
pub struct FramebufferAttachments {
    pub colors: Vec<gl::types::GLuint>,
    pub depth: Option<gl::types::GLuint>,
    pub stencil: Option<gl::types::GLuint>,
}

/// Frame buffer.
pub struct FrameBufferObject {
    display: Arc<DisplayImpl>,
    id: gl::types::GLuint,
    current_read_buffer: gl::types::GLenum,
}

impl FrameBufferObject {
    /// Builds a new FBO.
    fn new(display: Arc<DisplayImpl>) -> FrameBufferObject {
        let (tx, rx) = channel();

        display.context.exec(move |: ctxt| {
            unsafe {
                let id: gl::types::GLuint = mem::uninitialized();
                if ctxt.version >= &context::GlVersion(3, 0) {
                    ctxt.gl.GenFramebuffers(1, mem::transmute(&id));
                } else {
                    ctxt.gl.GenFramebuffersEXT(1, mem::transmute(&id));
                }
                tx.send(id);
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
pub fn draw<V, I, U>(display: &Arc<DisplayImpl>,
    framebuffer: Option<&FramebufferAttachments>, vertex_buffer: &VertexBuffer<V>,
    indices: &I, program: &Program, uniforms: &U, draw_parameters: &DrawParameters)
    where I: IndicesSource, U: Uniforms
{
    let fbo_id = get_framebuffer(display, framebuffer);

    let vao_id = vertex_array_object::get_vertex_array_object(display, vertex_buffer, indices,
                                                              program);

    let ::IndicesSourceHelper { pointer, primitives, data_type,
                                indices_count, .. } = indices.to_indices_source_helper();
    let pointer = pointer.unwrap_or(::std::ptr::null());

    let uniforms = uniforms.to_binder();
    let uniforms_locations = program::get_uniforms_locations(program);
    let draw_parameters = draw_parameters.clone();

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
            draw_parameters.sync(&mut ctxt);

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

        let mut new_fbo = FrameBufferObject::new(display.clone());
        let new_fbo_id = new_fbo.id.clone();
        initialize_fbo(display, &mut new_fbo, framebuffer);
        framebuffers.insert(framebuffer.clone(), new_fbo);
        Some(new_fbo_id)

    } else {
        None
    }
}

fn initialize_fbo(display: &Arc<DisplayImpl>, fbo: &mut FrameBufferObject,
    content: &FramebufferAttachments)
{
    use context::GlVersion;

    let fbo_id = fbo.id;

    if content.depth.is_some() { unimplemented!() }
    if content.stencil.is_some() { unimplemented!() }

    for (slot, texture) in content.colors.iter().enumerate() {
        let tex_id = texture.clone();

        display.context.exec(move |: mut ctxt| {
            unsafe {
                if ctxt.version >= &GlVersion(4, 5) {
                    ctxt.gl.NamedFramebufferTexture(fbo_id, gl::COLOR_ATTACHMENT0 + slot as u32,
                        tex_id, 0);

                } else if ctxt.extensions.gl_ext_direct_state_access &&
                          ctxt.extensions.gl_ext_geometry_shader4
                {
                    ctxt.gl.NamedFramebufferTextureEXT(fbo_id, gl::COLOR_ATTACHMENT0 + slot as u32,
                        tex_id, 0);

                } else if ctxt.version >= &GlVersion(3, 2) {
                    bind_framebuffer(&mut ctxt, Some(fbo_id), true, false);
                    ctxt.gl.FramebufferTexture(gl::DRAW_FRAMEBUFFER, gl::COLOR_ATTACHMENT0 + slot as u32,
                        tex_id, 0);

                } else if ctxt.version >= &GlVersion(3, 0) {
                    bind_framebuffer(&mut ctxt, Some(fbo_id), true, false);
                    ctxt.gl.FramebufferTexture2D(gl::DRAW_FRAMEBUFFER,
                        gl::COLOR_ATTACHMENT0 + slot as u32, gl::TEXTURE_2D, tex_id, 0);

                } else {
                    bind_framebuffer(&mut ctxt, Some(fbo_id), true, true);
                    ctxt.gl.FramebufferTexture2DEXT(gl::FRAMEBUFFER_EXT,
                        gl::COLOR_ATTACHMENT0 + slot as u32, gl::TEXTURE_2D, tex_id, 0);
                }
            }
        });
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
