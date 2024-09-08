/*!
Contains everything related to external API memory objects.
*/

// TODO: Add Windows support via EXT_external_objects_win32

use crate::GlObject;
use crate::context::CommandContext;
use crate::gl;
use crate::version::Api;
use crate::version::Version;

use crate::backend::Facade;
use crate::context::Context;
use crate::ContextExt;
use std::rc::Rc;

/// Describes an error encountered during memory object creation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryObjectCreationError {
    /// Driver does not support EXT_memory_object
    MemoryObjectNotSupported,
    /// Driver does not support EXT_memory_object_fd
    MemoryObjectFdNotSupported,
    /// OpenGL returned a null pointer when creating memory object
    NullResult,
}

impl std::fmt::Display for MemoryObjectCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use self::MemoryObjectCreationError::*;

        let desc = match *self {
            MemoryObjectNotSupported => "Driver does not support EXT_memory_object",
            MemoryObjectFdNotSupported => "Driver does not support EXT_memory_object_fd",
            NullResult => "OpenGL returned a null pointer when creating memory object",
        };
        f.write_str(desc)
    }
}

impl std::error::Error for MemoryObjectCreationError {}

/// Describes a memory object created by an external API. In OpenGL there is no distinction
/// between a texture or buffer and its underlying memory. However, in other API's like Vulkan
/// the underlying memory and the image are separate. Thus this type is useful when interfacing
/// with such APIs, as such a memory object can then be used to create a texture or buffer
/// which OpenGL can then interact with.
pub struct MemoryObject {
    context: Rc<Context>,
    id: gl::types::GLuint,
}

impl MemoryObject {
    /// Creates a memory object form an opaque file descriptor.
    #[cfg(target_os = "linux")]
    pub unsafe fn new_from_fd<F: Facade + ?Sized>(
        facade: &F,
        dedicated: bool,
        fd: std::fs::File,
        size: u64,
    ) -> Result<Self, MemoryObjectCreationError> {
        use std::os::unix::io::AsRawFd;
        let ctxt = facade.get_context().make_current();
        let mem_obj: Self = Self::new(facade, &ctxt)?;

        if !ctxt.extensions.gl_ext_memory_object_fd {
            Err(MemoryObjectCreationError::MemoryObjectFdNotSupported)
        } else {
            let dedicated: gl::types::GLint = if dedicated {
                gl::TRUE as i32
            } else {
                gl::FALSE as i32
            };

            ctxt.gl.MemoryObjectParameterivEXT(
                mem_obj.id,
                gl::DEDICATED_MEMORY_OBJECT_EXT,
                &dedicated as *const i32,
            );

            ctxt.gl.ImportMemoryFdEXT(
                mem_obj.id,
                size,
                gl::HANDLE_TYPE_OPAQUE_FD_EXT,
                fd.as_raw_fd(),
            );

            std::mem::forget(fd);

            Ok(mem_obj)
        }
    }

    fn new<F: Facade + ?Sized>(
        facade: &F,
        ctxt: &CommandContext<'_>,
    ) -> Result<Self, MemoryObjectCreationError> {
        if (ctxt.version >= &Version(Api::Gl, 4, 5)
            || ctxt.version >= &Version(Api::GlEs, 3, 2)
            || ctxt.extensions.gl_arb_texture_storage)
            && ctxt.extensions.gl_ext_memory_object
        {
            let id = unsafe {
                let mut id: gl::types::GLuint = 0;
                ctxt.gl.CreateMemoryObjectsEXT(1, &mut id as *mut u32);

                if ctxt.gl.IsMemoryObjectEXT(id) == gl::FALSE {
                    return Err(MemoryObjectCreationError::NullResult);
                } else {
                    id
                }
            };

            Ok(Self {
                context: facade.get_context().clone(),
                id,
            })
        } else {
            Err(MemoryObjectCreationError::MemoryObjectNotSupported)
        }
    }
}

impl GlObject for MemoryObject {
    type Id = gl::types::GLuint;

    #[inline]
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for MemoryObject {
    fn drop(&mut self) {
        let ctxt = self.context.get_context().make_current();
        unsafe { ctxt.gl.DeleteMemoryObjectsEXT(1, &mut self.id as *mut u32) };
    }
}
