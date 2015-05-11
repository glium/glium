use fbo::{self, FramebufferAttachments};

use context::Context;
use ContextExt;
use Rect;

use Surface;

use Api;
use version::Version;
use gl;


pub fn clear(context: &Context, framebuffer: Option<&FramebufferAttachments>,
             rect: Option<&Rect>, color: Option<(f32, f32, f32, f32)>, depth: Option<f32>,
             stencil: Option<i32>)
{
    unsafe {
        let mut ctxt = context.make_current();

        let fbo_id = context.framebuffer_objects.as_ref().unwrap()
                            .get_framebuffer_for_drawing(framebuffer, &mut ctxt);

        fbo::bind_framebuffer(&mut ctxt, fbo_id, true, false);

        if ctxt.state.enabled_rasterizer_discard {
            ctxt.gl.Disable(gl::RASTERIZER_DISCARD);
            ctxt.state.enabled_rasterizer_discard = false;
        }

        if let Some(_) = ctxt.state.conditional_render {
            if ctxt.version >= &Version(Api::Gl, 3, 0) {
                ctxt.gl.EndConditionalRender();
            } else if ctxt.extensions.gl_nv_conditional_render {
                ctxt.gl.EndConditionalRenderNV();
            } else {
                unreachable!();
            }

            ctxt.state.conditional_render = None;
        }

        if let Some(rect) = rect {
            let rect = (rect.left as gl::types::GLint, rect.bottom as gl::types::GLint,
                        rect.width as gl::types::GLsizei, rect.height as gl::types::GLsizei);

            if ctxt.state.scissor != Some(rect) {
                ctxt.gl.Scissor(rect.0, rect.1, rect.2, rect.3);
                ctxt.state.scissor = Some(rect);
            }

            if !ctxt.state.enabled_scissor_test {
                ctxt.gl.Enable(gl::SCISSOR_TEST);
                ctxt.state.enabled_scissor_test = true;
            }

        } else {
            if ctxt.state.enabled_scissor_test {
                ctxt.gl.Disable(gl::SCISSOR_TEST);
                ctxt.state.enabled_scissor_test = false;
            }
        }

        let mut flags = 0;

        if let Some(color) = color {
            let color = (color.0 as gl::types::GLclampf, color.1 as gl::types::GLclampf,
                         color.2 as gl::types::GLclampf, color.3 as gl::types::GLclampf);

            flags |= gl::COLOR_BUFFER_BIT;

            if ctxt.state.clear_color != color {
                ctxt.gl.ClearColor(color.0, color.1, color.2, color.3);
                ctxt.state.clear_color = color;
            }
        }

        if let Some(depth) = depth {
            let depth = depth as gl::types::GLclampf;

            flags |= gl::DEPTH_BUFFER_BIT;

            if ctxt.state.clear_depth != depth {
                if ctxt.version >= &Version(Api::Gl, 1, 0) {
                    ctxt.gl.ClearDepth(depth as gl::types::GLclampd);
                } else if ctxt.version >= &Version(Api::GlEs, 2, 0) {
                    ctxt.gl.ClearDepthf(depth);
                } else {
                    unreachable!();
                }

                ctxt.state.clear_depth = depth;
            }

            if !ctxt.state.depth_mask {
                ctxt.gl.DepthMask(gl::TRUE);
                ctxt.state.depth_mask = true;
            }
        }

        if let Some(stencil) = stencil {
            let stencil = stencil as gl::types::GLint;

            flags |= gl::STENCIL_BUFFER_BIT;

            if ctxt.state.clear_stencil != stencil {
                ctxt.gl.ClearStencil(stencil);
                ctxt.state.clear_stencil = stencil;
            }
        }

        ctxt.gl.Clear(flags);
    }
}
