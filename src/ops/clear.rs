use std::sync::Arc;

use fbo::{self, FramebufferAttachments};

use DisplayImpl;
use Surface;

use gl;


pub fn clear(display: &Arc<DisplayImpl>, framebuffer: Option<&FramebufferAttachments>,
             color: Option<(f32, f32, f32, f32)>, depth: Option<f32>, stencil: Option<i32>)
{
    let color = color.map(|(red, green, blue, alpha)| (
        red as gl::types::GLclampf,
        green as gl::types::GLclampf,
        blue as gl::types::GLclampf,
        alpha as gl::types::GLclampf
    ));

    let depth = depth.map(|value| value as gl::types::GLclampf);
    let stencil = stencil.map(|value| value as gl::types::GLint);

    let flags = if color.is_some() { gl::COLOR_BUFFER_BIT } else { 0 } |
                if depth.is_some() { gl::DEPTH_BUFFER_BIT } else { 0 } |
                if stencil.is_some() { gl::STENCIL_BUFFER_BIT } else { 0 };
    
    let fbo_id = display.framebuffer_objects.as_ref().unwrap()
                        .get_framebuffer_for_drawing(framebuffer, &display.context);

    display.context.exec(move |: mut ctxt| {
        fbo::bind_framebuffer(&mut ctxt, fbo_id, true, false);

        unsafe {
            if let Some(color) = color {
                if ctxt.state.clear_color != color {
                    ctxt.gl.ClearColor(color.0, color.1, color.2, color.3);
                    ctxt.state.clear_color = color;
                }
            }

            if let Some(depth) = depth {
                if ctxt.state.clear_depth != depth {
                    ctxt.gl.ClearDepth(depth as f64);        // TODO: find out why this needs "as"
                    ctxt.state.clear_depth = depth;
                }
            }

            if let Some(stencil) = stencil {
                if ctxt.state.clear_stencil != stencil {
                    ctxt.gl.ClearStencil(stencil);
                    ctxt.state.clear_stencil = stencil;
                }
            }

            ctxt.gl.Clear(flags);
        }
    });
}
