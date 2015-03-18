use std::rc::Rc;

use fbo::{self, FramebufferAttachments};

use Display;
use Surface;

use Api;
use context::GlVersion;
use gl;


pub fn clear(display: &Display, framebuffer: Option<&FramebufferAttachments>,
             color: Option<(f32, f32, f32, f32)>, depth: Option<f32>, stencil: Option<i32>)
{
    unsafe {
        let fbo_id = display.context.framebuffer_objects.as_ref().unwrap()
                            .get_framebuffer_for_drawing(framebuffer, &display.context.context);

        let mut ctxt = display.context.context.make_current();

        fbo::bind_framebuffer(&mut ctxt, fbo_id, true, false);

        if ctxt.state.enabled_rasterizer_discard {
            ctxt.gl.Disable(gl::RASTERIZER_DISCARD);
            ctxt.state.enabled_rasterizer_discard = false;
        }

        if ctxt.state.enabled_scissor_test {
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
                if ctxt.version >= &GlVersion(Api::Gl, 1, 0) {
                    ctxt.gl.ClearDepth(depth as gl::types::GLclampd);
                } else if ctxt.version >= &GlVersion(Api::GlEs, 2, 0) {
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
