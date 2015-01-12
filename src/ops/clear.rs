use std::sync::Arc;

use fbo::{self, FramebufferAttachments};

use DisplayImpl;
use Surface;

use gl;

pub fn clear_color(display: &Arc<DisplayImpl>, framebuffer: Option<&FramebufferAttachments>,
                   red: f32, green: f32, blue: f32, alpha: f32)
{
    clear_impl(display, framebuffer, Some((red, green, blue, alpha)), None, None)
}

pub fn clear_depth(display: &Arc<DisplayImpl>, framebuffer: Option<&FramebufferAttachments>,
                   value: f32)
{
    clear_impl(display, framebuffer, None, Some(value), None)
}

pub fn clear_stencil(display: &Arc<DisplayImpl>, framebuffer: Option<&FramebufferAttachments>,
                     value: i32)
{
    clear_impl(display, framebuffer, None, None, Some(value))
}

fn clear_impl(display: &Arc<DisplayImpl>, framebuffer: Option<&FramebufferAttachments>,
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
