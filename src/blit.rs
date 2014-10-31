/*!
Contains traits related to blitting surfaces.

*/

use std::sync::Arc;

use {context, gl};
use uniforms::SamplerFilter;
use BlitSurfaceImpl;

/// 
#[deriving(Show, Clone, Default)]
pub struct Rect {
    /// 
    pub left: u32,
    /// 
    pub top: u32,
    /// 
    pub width: u32,
    /// 
    pub height: u32,
}

/// Surface that can be blitted (ie. copied) to another one.
///
/// ## Panic
///
/// The blit functions will panic if the source and destination don't belong to the same display.
///
pub trait BlitSurface {
    /// Copies the surface to a target surface.
    ///
    /// `filter` only matters if the source and destination don't have the same size.
    fn blit_to<S: BlitSurface>(&self, src_rect: &Rect, target: &S, target_rect: &Rect,
        filter: SamplerFilter)
    {
        blit(self, target, gl::COLOR_BUFFER_BIT, src_rect, target_rect, filter.to_glenum())
    }

    /// Returns the dimensions of the surface.
    fn get_dimensions(&self) -> (u32, u32);

    /// Copies the entire surface to the entire target.
    fn fill<S: BlitSurface>(&self, target: &S, filter: SamplerFilter) {
        let src_dim = self.get_dimensions();
        let src_rect = Rect { left: 0, top: 0, width: src_dim.0, height: src_dim.1 };
        let target_dim = target.get_dimensions();
        let target_rect = Rect { left: 0, top: 0, width: target_dim.0, height: target_dim.1 };
        self.blit_to(&src_rect, target, &target_rect, filter)
    }

    /// Don't implement this, or redirect the call to another implementation.
    #[doc(hidden)]
    unsafe fn get_implementation(&self) -> BlitSurfaceImpl;
}

fn blit<S1: BlitSurface, S2: BlitSurface>(source: &S1, target: &S2, mask: gl::types::GLbitfield,
    src_rect: &Rect, target_rect: &Rect, filter: gl::types::GLenum)
{
    let source = unsafe { source.get_implementation() };
    let target = unsafe { target.get_implementation() };

    let BlitSurfaceImpl{display, fbo: source, ..} = source;
    let target = target.fbo;

    let src_rect = src_rect.clone();
    let target_rect = target_rect.clone();

    display.context.exec(proc(gl, state, version, _) {
        unsafe {
            // trying to do a named blit if possible
            if version >= &context::GlVersion(4, 5) {
                gl.BlitNamedFramebuffer(source.unwrap_or(0), target.unwrap_or(0),
                    src_rect.left as gl::types::GLint,
                    src_rect.top as gl::types::GLint,
                    (src_rect.left + src_rect.width) as gl::types::GLint,
                    (src_rect.top + src_rect.height) as gl::types::GLint,
                    target_rect.left as gl::types::GLint, target_rect.top as gl::types::GLint,
                    (target_rect.left + target_rect.width) as gl::types::GLint,
                    (target_rect.top + target_rect.height) as gl::types::GLint, mask, filter);

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
                    src_rect.top as gl::types::GLint,
                    (src_rect.left + src_rect.width) as gl::types::GLint,
                    (src_rect.top + src_rect.height) as gl::types::GLint,
                    target_rect.left as gl::types::GLint, target_rect.top as gl::types::GLint,
                    (target_rect.left + target_rect.width) as gl::types::GLint,
                    (target_rect.top + target_rect.height) as gl::types::GLint, mask, filter);

            } else {
                gl.BlitFramebufferEXT(src_rect.left as gl::types::GLint,
                    src_rect.top as gl::types::GLint,
                    (src_rect.left + src_rect.width) as gl::types::GLint,
                    (src_rect.top + src_rect.height) as gl::types::GLint,
                    target_rect.left as gl::types::GLint, target_rect.top as gl::types::GLint,
                    (target_rect.left + target_rect.width) as gl::types::GLint,
                    (target_rect.top + target_rect.height) as gl::types::GLint, mask, filter);
            }
        }
    });
}
