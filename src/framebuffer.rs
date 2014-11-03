use std::kinds::marker::ContravariantLifetime;
use std::{mem, ptr};
use std::sync::Arc;

use texture::{mod, Texture};
use uniforms::Uniforms;
use {DisplayImpl, VertexBuffer, IndexBuffer, Program, DrawParameters, Rect, Surface};

use {vertex_buffer, index_buffer, program};
use {gl, context, libc};

/// A framebuffer that you can use to draw things.
pub struct FrameBuffer<'a> {
    display: Arc<DisplayImpl>,
    attachments: FramebufferAttachments,
    marker: ContravariantLifetime<'a>,
    dimensions: Option<(u32, u32)>,
}

impl<'a> FrameBuffer<'a> {
    /// Creates an empty framebuffer.
    pub fn new<'a>(display: &::Display) -> FrameBuffer<'a> {
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
    pub fn with_texture<T: 'a>(mut self, texture: &'a T) -> FrameBuffer<'a> where T: Texture {
        // TODO: check existing dimensions
        self.attachments.colors.push(texture::get_id(texture.get_implementation()));
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

    fn draw<V, U>(&mut self, vb: &::VertexBuffer<V>, ib: &::IndexBuffer, program: &::Program,
        uniforms: &U, draw_parameters: &::DrawParameters) where U: ::uniforms::Uniforms
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

        display.context.exec(proc(gl, _, version, _) {
            unsafe {
                let id: gl::types::GLuint = mem::uninitialized();
                if version >= &context::GlVersion(3, 0) {
                    gl.GenFramebuffers(1, mem::transmute(&id));
                } else {
                    gl.GenFramebuffersEXT(1, mem::transmute(&id));
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
        self.display.context.exec(proc(gl, state, version, _) {
            unsafe {
                // unbinding framebuffer
                if version >= &context::GlVersion(3, 0) {
                    if state.draw_framebuffer == Some(id) && state.read_framebuffer == Some(id) {
                        gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
                        state.draw_framebuffer = None;
                        state.read_framebuffer = None;

                    } else if state.draw_framebuffer == Some(id) {
                        gl.BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
                        state.draw_framebuffer = None;

                    } else if state.read_framebuffer == Some(id) {
                        gl.BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
                        state.read_framebuffer = None;
                    }

                } else {
                    if state.draw_framebuffer == Some(id) || state.read_framebuffer == Some(id) {
                        gl.BindFramebufferEXT(gl::FRAMEBUFFER_EXT, 0);
                        state.draw_framebuffer = None;
                        state.read_framebuffer = None;
                    }
                }

                // deleting
                if version >= &context::GlVersion(3, 0) {
                    gl.DeleteFramebuffers(1, [ id ].as_ptr());
                } else {
                    gl.DeleteFramebuffersEXT(1, [ id ].as_ptr());
                }
            }
        });
    }
}

/// Render buffer.
#[allow(dead_code)]     // TODO: remove
pub struct RenderBuffer {
    display: Arc<DisplayImpl>,
    id: gl::types::GLuint,
}

#[allow(dead_code)]     // TODO: remove
impl RenderBuffer {
    /// Builds a new render buffer.
    fn new(display: Arc<DisplayImpl>) -> RenderBuffer {
        let (tx, rx) = channel();

        display.context.exec(proc(gl, _, version, _) {
            unsafe {
                let id: gl::types::GLuint = mem::uninitialized();
                if version >= &context::GlVersion(3, 0) {
                    gl.GenRenderbuffers(1, mem::transmute(&id));
                } else {
                    gl.GenRenderbuffersEXT(1, mem::transmute(&id));
                }

                tx.send(id);
            }
        });

        RenderBuffer {
            display: display,
            id: rx.recv(),
        }
    }
}

impl Drop for RenderBuffer {
    fn drop(&mut self) {
        let id = self.id.clone();
        self.display.context.exec(proc(gl, _, version, _) {
            unsafe {
                if version >= &context::GlVersion(3, 0) {
                    gl.DeleteRenderbuffers(1, [ id ].as_ptr());
                } else {
                    gl.DeleteRenderbuffersEXT(1, [ id ].as_ptr());
                }
            }
        });
    }
}

/// Draws everything.
pub fn draw<V, U: Uniforms>(display: &Arc<DisplayImpl>,
    framebuffer: Option<&FramebufferAttachments>, vertex_buffer: &VertexBuffer<V>,
    index_buffer: &IndexBuffer, program: &Program, uniforms: &U, draw_parameters: &DrawParameters)
{
    let fbo_id = get_framebuffer(display, framebuffer);

    let (vb_id, vb_elementssize, vb_bindingsclone) = vertex_buffer::get_clone(vertex_buffer);
    let (ib_id, ib_elemcounts, ib_datatype, ib_primitives) =
        index_buffer::get_clone(index_buffer);
    let program_id = program::get_program_id(program);
    let uniforms = uniforms.to_binder();
    let uniforms_locations = program::get_uniforms_locations(program);
    let draw_parameters = draw_parameters.clone();

    let (tx, rx) = channel();

    display.context.exec(proc(gl, state, version, extensions) {
        unsafe {
            bind_framebuffer(gl, state, version, extensions, fbo_id, true, false);

            // binding program
            if state.program != program_id {
                gl.UseProgram(program_id);
                state.program = program_id;
            }

            // binding program uniforms
            uniforms.0(gl, |name| {
                uniforms_locations
                    .find_equiv(name)
                    .map(|val| val.0)
            });

            // binding vertex buffer
            if state.array_buffer_binding != Some(vb_id) {
                gl.BindBuffer(gl::ARRAY_BUFFER, vb_id);
                state.array_buffer_binding = Some(vb_id);
            }

            // binding index buffer
            if state.element_array_buffer_binding != Some(ib_id) {
                gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ib_id);
                state.element_array_buffer_binding = Some(ib_id);
            }

            // binding vertex buffer
            let mut locations = Vec::new();
            for &(ref name, vertex_buffer::VertexAttrib { offset, data_type, elements_count })
                in vb_bindingsclone.iter()
            {
                let loc = gl.GetAttribLocation(program_id, name.to_c_str().unwrap());
                locations.push(loc);

                if loc != -1 {
                    match data_type {
                        gl::BYTE | gl::UNSIGNED_BYTE | gl::SHORT | gl::UNSIGNED_SHORT |
                        gl::INT | gl::UNSIGNED_INT =>
                            gl.VertexAttribIPointer(loc as u32,
                                elements_count as gl::types::GLint, data_type,
                                vb_elementssize as i32, offset as *const libc::c_void),

                        _ => gl.VertexAttribPointer(loc as u32,
                                elements_count as gl::types::GLint, data_type, 0,
                                vb_elementssize as i32, offset as *const libc::c_void)
                    }
                    
                    gl.EnableVertexAttribArray(loc as u32);
                }
            }

            // sync-ing parameters
            draw_parameters.sync(gl, state);
            
            // drawing
            gl.DrawElements(ib_primitives, ib_elemcounts as i32, ib_datatype, ptr::null());

            // disable vertex attrib array
            for l in locations.iter() {
                gl.DisableVertexAttribArray(l.clone() as u32);
            }
        }

        tx.send(());
    });

    // synchronizing with the end of the draw
    // TODO: remove that after making sure that everything is ok
    rx.recv();
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

    display.context.exec(proc(gl, state, version, extensions) {
        bind_framebuffer(gl, state, version, extensions, fbo_id, true, false);

        if state.clear_color != (red, green, blue, alpha) {
            gl.ClearColor(red, green, blue, alpha);
            state.clear_color = (red, green, blue, alpha);
        }

        gl.Clear(gl::COLOR_BUFFER_BIT);
    });
}

pub fn clear_depth(display: &Arc<DisplayImpl>, framebuffer: Option<&FramebufferAttachments>,
    value: f32)
{
    let value = value as gl::types::GLclampf;
    let fbo_id = get_framebuffer(display, framebuffer);

    display.context.exec(proc(gl, state, version, extensions) {
        bind_framebuffer(gl, state, version, extensions, fbo_id, true, false);

        if state.clear_depth != value {
            gl.ClearDepth(value as f64);        // TODO: find out why this needs "as"
            state.clear_depth = value;
        }

        gl.Clear(gl::DEPTH_BUFFER_BIT);
    });
}

pub fn clear_stencil(display: &Arc<DisplayImpl>, framebuffer: Option<&FramebufferAttachments>,
    value: int)
{
    let value = value as gl::types::GLint;
    let fbo_id = get_framebuffer(display, framebuffer);

    display.context.exec(proc(gl, state, version, extensions) {
        bind_framebuffer(gl, state, version, extensions, fbo_id, true, false);

        if state.clear_stencil != value {
            gl.ClearStencil(value);
            state.clear_stencil = value;
        }

        gl.Clear(gl::STENCIL_BUFFER_BIT);
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

    display.context.exec(proc(gl, state, version, _) {
        // trying to do a named blit if possible
        if version >= &context::GlVersion(4, 5) {
            gl.BlitNamedFramebuffer(source.unwrap_or(0), target.unwrap_or(0),
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
        if state.read_framebuffer != source {
            if version >= &context::GlVersion(3, 0) {
                gl.BindFramebuffer(gl::READ_FRAMEBUFFER, source.unwrap_or(0));
                state.read_framebuffer = source;

            } else {
                gl.BindFramebufferEXT(gl::READ_FRAMEBUFFER_EXT, source.unwrap_or(0));
                state.read_framebuffer = source;
            }
        }

        // binding target framebuffer
        if state.draw_framebuffer != target {
            if version >= &context::GlVersion(3, 0) {
                gl.BindFramebuffer(gl::DRAW_FRAMEBUFFER, target.unwrap_or(0));
                state.draw_framebuffer = target;

            } else {
                gl.BindFramebufferEXT(gl::DRAW_FRAMEBUFFER_EXT, target.unwrap_or(0));
                state.draw_framebuffer = target;
            }
        }

        // doing the blit
        if version >= &context::GlVersion(3, 0) {
            gl.BlitFramebuffer(src_rect.left as gl::types::GLint,
                src_rect.bottom as gl::types::GLint,
                (src_rect.left + src_rect.width) as gl::types::GLint,
                (src_rect.bottom + src_rect.height) as gl::types::GLint,
                target_rect.left as gl::types::GLint, target_rect.bottom as gl::types::GLint,
                (target_rect.left + target_rect.width) as gl::types::GLint,
                (target_rect.bottom + target_rect.height) as gl::types::GLint, mask, filter);

        } else {
            gl.BlitFramebufferEXT(src_rect.left as gl::types::GLint,
                src_rect.bottom as gl::types::GLint,
                (src_rect.left + src_rect.width) as gl::types::GLint,
                (src_rect.bottom + src_rect.height) as gl::types::GLint,
                target_rect.left as gl::types::GLint, target_rect.bottom as gl::types::GLint,
                (target_rect.left + target_rect.width) as gl::types::GLint,
                (target_rect.bottom + target_rect.height) as gl::types::GLint, mask, filter);
        }
    });
}

fn get_framebuffer(display: &Arc<DisplayImpl>, framebuffer: Option<&FramebufferAttachments>)
    -> Option<gl::types::GLuint>
{
    if let Some(framebuffer) = framebuffer {
        let mut framebuffers = display.framebuffer_objects.lock();

        if let Some(value) = framebuffers.find(framebuffer) {
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

        display.context.exec(proc(gl, state, version, extensions) {
            if version >= &GlVersion(4, 5) {
                gl.NamedFramebufferTexture(fbo_id, gl::COLOR_ATTACHMENT0 + slot as u32, tex_id, 0);

            } else if extensions.gl_ext_direct_state_access &&
                      extensions.gl_ext_geometry_shader4
            {
                gl.NamedFramebufferTextureEXT(fbo_id, gl::COLOR_ATTACHMENT0 + slot as u32, tex_id,
                    0);

            } else if version >= &GlVersion(3, 2) {
                bind_framebuffer(gl, state, version, extensions, Some(fbo_id), true, false);
                gl.FramebufferTexture(gl::DRAW_FRAMEBUFFER, gl::COLOR_ATTACHMENT0 + slot as u32,
                    tex_id, 0);

            } else if version >= &GlVersion(3, 0) {
                bind_framebuffer(gl, state, version, extensions, Some(fbo_id), true, false);
                gl.FramebufferTexture2D(gl::DRAW_FRAMEBUFFER, gl::COLOR_ATTACHMENT0 + slot as u32,
                    gl::TEXTURE_2D, tex_id, 0);

            } else {
                bind_framebuffer(gl, state, version, extensions, Some(fbo_id), true, true);
                gl.FramebufferTexture2DEXT(gl::FRAMEBUFFER_EXT, gl::COLOR_ATTACHMENT0 + slot as u32,
                    gl::TEXTURE_2D, tex_id, 0);
            }
        });
    }
}

fn bind_framebuffer(gl: &gl::Gl, state: &mut context::GLState, version: &context::GlVersion,
    _: &context::ExtensionsList, fbo_id: Option<gl::types::GLuint>, draw: bool, read: bool)
{
    if draw && state.draw_framebuffer != fbo_id {
        if version >= &context::GlVersion(3, 0) {
            gl.BindFramebuffer(gl::DRAW_FRAMEBUFFER, fbo_id.unwrap_or(0));
            state.draw_framebuffer = fbo_id.clone();
        } else {
            gl.BindFramebufferEXT(gl::FRAMEBUFFER_EXT, fbo_id.unwrap_or(0));
            state.draw_framebuffer = fbo_id.clone();
            state.read_framebuffer = fbo_id.clone();
        }
    }

    if read && state.read_framebuffer != fbo_id {
        if version >= &context::GlVersion(3, 0) {
            gl.BindFramebuffer(gl::READ_FRAMEBUFFER, fbo_id.unwrap_or(0));
            state.read_framebuffer = fbo_id.clone();
        } else {
            gl.BindFramebufferEXT(gl::FRAMEBUFFER_EXT, fbo_id.unwrap_or(0));
            state.draw_framebuffer = fbo_id.clone();
            state.read_framebuffer = fbo_id.clone();
        }
    }
}
