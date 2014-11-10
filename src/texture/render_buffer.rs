use std::sync::Arc;
use std::mem;

use super::UncompressedFloatFormat;

use {gl, context};
use {GlObject, DisplayImpl};

/// A render buffer is similar to a texture, but is optimized for usage as a draw target.
///
/// Contrary to a texture, you can't read or modify the content of the `RenderBuffer` directly.
pub struct RenderBuffer {
    display: Arc<DisplayImpl>,
    id: gl::types::GLuint,
}

impl RenderBuffer {
    /// Builds a new render buffer.
    fn new(display: &::Display, format: UncompressedFloatFormat, width: u32, height: u32)
        -> RenderBuffer
    {
        let format = format.to_gl_enum();
        let (tx, rx) = channel();

        display.context.context.exec(proc(gl, state, version, extensions) {
            unsafe {
                let id: gl::types::GLuint = mem::uninitialized();
                if version >= &context::GlVersion(3, 0) {
                    gl.GenRenderbuffers(1, mem::transmute(&id));
                } else {
                    gl.GenRenderbuffersEXT(1, mem::transmute(&id));
                }

                tx.send(id);

                // TODO: check that dimensions don't exceed GL_MAX_RENDERBUFFER_SIZE
                if version >= &context::GlVersion(4, 5) {
                    gl.NamedRenderbufferStorage(id, format, width as gl::types::GLsizei,
                                                height as gl::types::GLsizei);

                } else if extensions.gl_ext_direct_state_access {
                    gl.NamedRenderbufferStorageEXT(id, format, width as gl::types::GLsizei,
                                                   height as gl::types::GLsizei);

                } else {
                    gl.BindRenderbuffer(gl::RENDERBUFFER, id);
                    state.renderbuffer = Some(id);
                    gl.RenderbufferStorage(gl::RENDERBUFFER, format, width as gl::types::GLsizei,
                                           height as gl::types::GLsizei);
                }
            }
        });

        RenderBuffer {
            display: display.context.clone(),
            id: rx.recv(),
        }
    }
}

impl Drop for RenderBuffer {
    fn drop(&mut self) {
        let id = self.id.clone();
        self.display.context.exec(proc(gl, state, version, _) {
            unsafe {
                if state.renderbuffer == Some(id) {
                    gl.BindRenderbuffer(gl::RENDERBUFFER, 0);
                    state.renderbuffer = None;
                }

                if version >= &context::GlVersion(3, 0) {
                    gl.DeleteRenderbuffers(1, [ id ].as_ptr());
                } else {
                    gl.DeleteRenderbuffersEXT(1, [ id ].as_ptr());
                }
            }
        });
    }
}

impl GlObject for RenderBuffer {
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}
