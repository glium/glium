#![cfg(feature = "vk_interop")]

use std::{fs::File, mem, rc::Rc};

use crate::{Context, ContextExt};

use crate::{backend::Facade, context::CommandContext, gl};

/// Describes an error encountered during semaphore creation
#[derive(Debug, Clone, Copy)]
pub enum SemaphoreCreationError {
    /// Driver does not support EXT_semaphore
    SemaphoreObjectNotSupported,
    /// Driver does not support EXT_semaphore_fd
    SemaphoreObjectFdNotSupported,
    /// OpenGL returned a null pointer when creating semaphore
    NullResult,
}

/// Describes a semaphore which can be used for OpenGL-Vulkan command queue synchronization
pub struct Semaphore {
    context: Rc<Context>,
    id: gl::types::GLuint,
    backing_fd: Option<File>,
}

impl Semaphore {
    /// Creates a semaphore imported from an opaque file descriptor.
    #[cfg(target_os = "linux")]
    pub unsafe fn new_from_fd<F: Facade + ?Sized>(
        facade: &F,
        fd: File,
    ) -> Result<Self, SemaphoreCreationError> {
        use std::os::unix::io::AsRawFd;

        let ctxt = facade.get_context().make_current();
        let sem = Self::new(facade, &ctxt)?;

        if ctxt.extensions.gl_ext_semaphore_fd {
            ctxt.gl
                .ImportSemaphoreFdEXT(sem.id, gl::HANDLE_TYPE_OPAQUE_FD_EXT, fd.as_raw_fd());

            if ctxt.gl.IsSemaphoreEXT(sem.id) == gl::FALSE {
                Err(SemaphoreCreationError::NullResult)
            } else {
                Ok(sem)
            }
        } else {
            Err(SemaphoreCreationError::SemaphoreObjectFdNotSupported)
        }
    }

    fn new<F: Facade + ?Sized>(
        facade: &F,
        ctxt: &CommandContext<'_>,
    ) -> Result<Self, SemaphoreCreationError> {
        if ctxt.extensions.gl_ext_semaphore {
            let id = unsafe {
                let mut id: gl::types::GLuint = 0;
                ctxt.gl.GenSemaphoresEXT(1, mem::transmute(&mut id));
                id
            };

            Ok(Self {
                context: facade.get_context().clone(),
                id,
                backing_fd: None,
            })
        } else {
            Err(SemaphoreCreationError::SemaphoreObjectNotSupported)
        }
    }

    /// The semaphore blocks the GPU's command queue until the semaphore is signalled. This does not block the CPU.
    pub fn wait(&self) {
        let ctxt = self.context.get_context().make_current();
        unsafe {
            ctxt.gl.WaitSemaphoreEXT(
                self.id,
                0,
                std::ptr::null(),
                0,
                std::ptr::null(),
                std::ptr::null(),
            )
        }
    }

    /// Sends signal through semaphore.
    pub fn signal(&self) {
        let ctxt = self.context.get_context().make_current();
        unsafe {
            ctxt.gl.SignalSemaphoreEXT(
                self.id,
                0,
                std::ptr::null(),
                0,
                std::ptr::null(),
                std::ptr::null(),
            )
        }
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        let ctxt = self.context.get_context().make_current();
        unsafe { ctxt.gl.DeleteSemaphoresEXT(1, mem::transmute(&self.id)) };
    }
}
