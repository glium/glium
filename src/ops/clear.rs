use crate::fbo::{self, ValidatedAttachments};

use crate::context::Context;
use crate::ContextExt;
use crate::Rect;

use crate::QueryExt;
use crate::draw_parameters::TimeElapsedQuery;

use crate::Api;
use crate::version::Version;
use crate::gl;


pub fn clear(context: &Context, framebuffer: Option<&ValidatedAttachments<'_>>,
             rect: Option<&Rect>, color: Option<(f32, f32, f32, f32)>, color_srgb: bool,
             depth: Option<f32>, stencil: Option<i32>)
{
    unsafe {
        let mut ctxt = context.make_current();

        let fbo_id = fbo::FramebuffersContainer::get_framebuffer_for_drawing(&mut ctxt, framebuffer);
        fbo::bind_framebuffer(&mut ctxt, fbo_id, true, false);

        if ctxt.state.enabled_rasterizer_discard {
            ctxt.gl.Disable(gl::RASTERIZER_DISCARD);
            ctxt.state.enabled_rasterizer_discard = false;
        }

        if ctxt.state.color_mask != (1, 1, 1, 1) {
            ctxt.state.color_mask = (1, 1, 1, 1);
            ctxt.gl.ColorMask(1, 1, 1, 1);
        }

        if ctxt.version >= &Version(Api::Gl, 3, 0) || ctxt.extensions.gl_arb_framebuffer_srgb ||
           ctxt.extensions.gl_ext_framebuffer_srgb || ctxt.extensions.gl_ext_srgb_write_control
        {
            if !color_srgb && !ctxt.state.enabled_framebuffer_srgb {
                ctxt.gl.Enable(gl::FRAMEBUFFER_SRGB);
                ctxt.state.enabled_framebuffer_srgb = true;

            } else if color_srgb && ctxt.state.enabled_framebuffer_srgb {
                ctxt.gl.Disable(gl::FRAMEBUFFER_SRGB);
                ctxt.state.enabled_framebuffer_srgb = false;
            }
        }

        TimeElapsedQuery::end_conditional_render(&mut ctxt);

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

        } else if ctxt.state.enabled_scissor_test {
            ctxt.gl.Disable(gl::SCISSOR_TEST);
            ctxt.state.enabled_scissor_test = false;
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
