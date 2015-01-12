use std::sync::Arc;

use fbo::{self, FramebufferAttachments};

use DisplayImpl;
use Surface;

use gl;

pub fn clear_color(display: &Arc<DisplayImpl>, framebuffer: Option<&FramebufferAttachments>,
    red: f32, green: f32, blue: f32, alpha: f32)
{
    let fbo_id = display.framebuffer_objects.as_ref().unwrap()
                        .get_framebuffer_for_drawing(framebuffer, &display.context);

    let (red, green, blue, alpha) = (
        red as gl::types::GLclampf,
        green as gl::types::GLclampf,
        blue as gl::types::GLclampf,
        alpha as gl::types::GLclampf
    );

    display.context.exec(move |: mut ctxt| {
        fbo::bind_framebuffer(&mut ctxt, fbo_id, true, false);

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
    
    let fbo_id = display.framebuffer_objects.as_ref().unwrap()
                        .get_framebuffer_for_drawing(framebuffer, &display.context);

    display.context.exec(move |: mut ctxt| {
        fbo::bind_framebuffer(&mut ctxt, fbo_id, true, false);

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

    let fbo_id = display.framebuffer_objects.as_ref().unwrap()
                        .get_framebuffer_for_drawing(framebuffer, &display.context);

    display.context.exec(move |: mut ctxt| {
        fbo::bind_framebuffer(&mut ctxt, fbo_id, true, false);

        unsafe {
            if ctxt.state.clear_stencil != value {
                ctxt.gl.ClearStencil(value);
                ctxt.state.clear_stencil = value;
            }

            ctxt.gl.Clear(gl::STENCIL_BUFFER_BIT);
        }
    });
}

