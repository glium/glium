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

        display.context.context.exec(move |: ctxt| {
            unsafe {
                let id: gl::types::GLuint = mem::uninitialized();
                if ctxt.version >= &context::GlVersion(3, 0) {
                    ctxt.gl.GenRenderbuffers(1, mem::transmute(&id));
                } else {
                    ctxt.gl.GenRenderbuffersEXT(1, mem::transmute(&id));
                }

                tx.send(id);

                // TODO: check that dimensions don't exceed GL_MAX_RENDERBUFFER_SIZE
                if ctxt.version >= &context::GlVersion(4, 5) {
                    ctxt.gl.NamedRenderbufferStorage(id, format, width as gl::types::GLsizei,
                                                height as gl::types::GLsizei);

                } else if ctxt.extensions.gl_ext_direct_state_access {
                    ctxt.gl.NamedRenderbufferStorageEXT(id, format, width as gl::types::GLsizei,
                                                   height as gl::types::GLsizei);

                } else {
                    ctxt.gl.BindRenderbuffer(gl::RENDERBUFFER, id);
                    ctxt.state.renderbuffer = id;
                    ctxt.gl.RenderbufferStorage(gl::RENDERBUFFER, format, width as gl::types::GLsizei,
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
        self.display.context.exec(move |: ctxt| {
            unsafe {
                if ctxt.state.renderbuffer == id {
                    ctxt.gl.BindRenderbuffer(gl::RENDERBUFFER, 0);
                    ctxt.state.renderbuffer = 0;
                }

                if ctxt.version >= &context::GlVersion(3, 0) {
                    ctxt.gl.DeleteRenderbuffers(1, [ id ].as_ptr());
                } else {
                    ctxt.gl.DeleteRenderbuffersEXT(1, [ id ].as_ptr());
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
