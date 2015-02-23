use BlitTarget;
use Rect;
use Surface;

use gl;
use context;
use version::Api;

pub fn blit<S1: Surface, S2: Surface>(source: &S1, target: &S2, mask: gl::types::GLbitfield,
    src_rect: &Rect, target_rect: &BlitTarget, filter: gl::types::GLenum)
{
    let ::BlitHelper(display, source) = source.get_blit_helper();
    let ::BlitHelper(_, target) = target.get_blit_helper();

    let src_rect = src_rect.clone();
    let target_rect = target_rect.clone();

    // FIXME: we don't draw on it
    let source = display.framebuffer_objects.as_ref().unwrap()
                        .get_framebuffer_for_drawing(source, &display.context);
    let target = display.framebuffer_objects.as_ref().unwrap()
                        .get_framebuffer_for_drawing(target, &display.context);

    display.context.exec(move |ctxt| {
        unsafe {
            // trying to do a named blit if possible
            if ctxt.version >= &context::GlVersion(Api::Gl, 4, 5) {
                ctxt.gl.BlitNamedFramebuffer(source, target,
                    src_rect.left as gl::types::GLint,
                    src_rect.bottom as gl::types::GLint,
                    (src_rect.left + src_rect.width) as gl::types::GLint,
                    (src_rect.bottom + src_rect.height) as gl::types::GLint,
                    target_rect.left as gl::types::GLint, target_rect.bottom as gl::types::GLint,
                    (target_rect.left as i32 + target_rect.width) as gl::types::GLint,
                    (target_rect.bottom as i32 + target_rect.height) as gl::types::GLint, mask, filter);

                return;
            }

            // binding source framebuffer
            if ctxt.state.read_framebuffer != source {
                if ctxt.version >= &context::GlVersion(Api::Gl, 3, 0) {
                    ctxt.gl.BindFramebuffer(gl::READ_FRAMEBUFFER, source);
                    ctxt.state.read_framebuffer = source;

                } else {
                    ctxt.gl.BindFramebufferEXT(gl::READ_FRAMEBUFFER_EXT, source);
                    ctxt.state.read_framebuffer = source;
                }
            }

            // binding target framebuffer
            if ctxt.state.draw_framebuffer != target {
                if ctxt.version >= &context::GlVersion(Api::Gl, 3, 0) {
                    ctxt.gl.BindFramebuffer(gl::DRAW_FRAMEBUFFER, target);
                    ctxt.state.draw_framebuffer = target;

                } else {
                    ctxt.gl.BindFramebufferEXT(gl::DRAW_FRAMEBUFFER_EXT, target);
                    ctxt.state.draw_framebuffer = target;
                }
            }

            // doing the blit
            if ctxt.version >= &context::GlVersion(Api::Gl, 3, 0) {
                ctxt.gl.BlitFramebuffer(src_rect.left as gl::types::GLint,
                    src_rect.bottom as gl::types::GLint,
                    (src_rect.left + src_rect.width) as gl::types::GLint,
                    (src_rect.bottom + src_rect.height) as gl::types::GLint,
                    target_rect.left as gl::types::GLint, target_rect.bottom as gl::types::GLint,
                    (target_rect.left as i32 + target_rect.width) as gl::types::GLint,
                    (target_rect.bottom as i32 + target_rect.height) as gl::types::GLint, mask, filter);

            } else {
                ctxt.gl.BlitFramebufferEXT(src_rect.left as gl::types::GLint,
                    src_rect.bottom as gl::types::GLint,
                    (src_rect.left + src_rect.width) as gl::types::GLint,
                    (src_rect.bottom + src_rect.height) as gl::types::GLint,
                    target_rect.left as gl::types::GLint, target_rect.bottom as gl::types::GLint,
                    (target_rect.left as i32 + target_rect.width) as gl::types::GLint,
                    (target_rect.bottom as i32 + target_rect.height) as gl::types::GLint, mask, filter);
            }
        }
    });
}
