#![cfg(feature = "vk_interop")]

use std::{fs::File, rc::Rc};

use crate::{
    buffer::{Buffer, Content},
    texture::TextureAny,
    Context, ContextExt, GlObject,
};

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

/// Describes a Vulkan image layout that a texture can be in. See https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkImageLayout.html
#[derive(Debug, Clone, Copy)]
pub enum TextureLayout {
    /// Corresponds to VK_IMAGE_LAYOUT_UNDEFINED
    None,
    /// Corresponds to VK_IMAGE_LAYOUT_GENERAL
    General,
    /// Corresponds to VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL
    ColorAttachment,
    /// Corresponds to VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT
    DepthStencilAttachment,
    /// Corresponds to VK_IMAGE_LAYOUT_DEPTH_STENCIL_READ_ONLY_OPTIMAL
    DepthStencilReadOnly,
    /// Corresponds to VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL
    ShaderReadOnly,
    /// Corresponds to VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL
    TransferSrc,
    /// Corresponds to VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL
    TransferDst,
    /// Corresponds to VK_IMAGE_LAYOUT_DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL_KHR
    DepthReadOnlyStencilAttachment,
    /// Corresponds to VK_IMAGE_LAYOUT_DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL_KHR
    DepthAttachmentStencilReadOnly,
}

impl Into<crate::gl::types::GLenum> for TextureLayout {
    fn into(self) -> crate::gl::types::GLenum {
        match self {
            TextureLayout::None => gl::NONE,
            TextureLayout::General => gl::LAYOUT_GENERAL_EXT,
            TextureLayout::ColorAttachment => gl::LAYOUT_COLOR_ATTACHMENT_EXT,
            TextureLayout::DepthStencilAttachment => gl::LAYOUT_DEPTH_STENCIL_ATTACHMENT_EXT,
            TextureLayout::DepthStencilReadOnly => gl::LAYOUT_DEPTH_STENCIL_READ_ONLY_EXT,
            TextureLayout::ShaderReadOnly => gl::LAYOUT_SHADER_READ_ONLY_EXT,
            TextureLayout::TransferSrc => gl::LAYOUT_TRANSFER_SRC_EXT,
            TextureLayout::TransferDst => gl::LAYOUT_TRANSFER_DST_EXT,
            TextureLayout::DepthReadOnlyStencilAttachment => gl::LAYOUT_DEPTH_STENCIL_READ_ONLY_EXT,
            TextureLayout::DepthAttachmentStencilReadOnly => {
                gl::LAYOUT_DEPTH_ATTACHMENT_STENCIL_READ_ONLY_EXT
            }
        }
    }
}

/// Similar to a GL sync object, this describes a semaphore which can be used for OpenGL-Vulkan command queue synchronization.
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
                ctxt.gl.GenSemaphoresEXT(1, &mut id as *mut u32);
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
    /// Same as `wait` but without `buffers` parameter
    pub fn wait_textures(&self, textures: Option<&[(TextureAny, TextureLayout)]>) {
        // Don't care about type parameter
        self.wait::<u32>(textures, None)
    }

    /// The semaphore blocks the GPU's command queue until the semaphore is signalled. This does not block the CPU.
    /// After this completes, the semaphore is returned to the unsignalled state.
    /// Once the operation is complete, the memory corresponding to the passed `textures` and `buffers` is made available to OpenGL.
    /// The layouts given with each texture must match the image layout used in Vulkan directly before the semaphore is signalled by it.
    pub fn wait<T: ?Sized>(
        &self,
        textures: Option<&[(TextureAny, TextureLayout)]>,
        buffers: Option<&[Buffer<T>]>,
    ) where
        T: Content,
    {
        let ctxt = self.context.get_context().make_current();

        let (buffer_ids, buffer_num, _) = match buffers {
            Some(buffs) => {
                let ids = buffs.iter().map(|b| b.get_id()).collect::<Vec<_>>();
                (ids.as_ptr(), buffs.len(), Some(ids))
            }
            None => (std::ptr::null(), 0, None),
        };

        let (texture_ids, texture_layouts, textures_num, _, _) = match textures {
            Some(textures) => {
                let ids = textures.iter().map(|t| t.0.get_id()).collect::<Vec<_>>();
                let layouts = textures
                    .iter()
                    .map(|t| t.1.into())
                    .collect::<Vec<gl::types::GLenum>>();
                (
                    ids.as_ptr(),
                    layouts.as_ptr(),
                    textures.len(),
                    Some(ids),
                    Some(layouts),
                )
            }
            None => (std::ptr::null(), std::ptr::null(), 0, None, None),
        };

        unsafe {
            ctxt.gl.WaitSemaphoreEXT(
                self.id,
                buffer_num as u32,
                buffer_ids,
                textures_num as u32,
                texture_ids,
                texture_layouts,
            )
        }
    }

    /// Same as `signal`, but without buffers parameter
    pub fn signal_textures(&self, textures: Option<&[(TextureAny, TextureLayout)]>) {
        // Don't care about type parameter.
        self.signal::<u32>(textures, None)
    }

    /// Sends signal through semaphore.
    /// The memory of the `textures` and `buffers` is made available to the external API.
    /// Before signalling the semaphore, the `textures` are transitioned into the layout specified.
    pub fn signal<T: ?Sized>(
        &self,
        textures: Option<&[(TextureAny, TextureLayout)]>,
        buffers: Option<&[Buffer<T>]>,
    ) where
        T: Content,
    {
        let (buffer_ids, buffer_num, _) = match buffers {
            Some(buffs) => {
                let ids = buffs.iter().map(|b| b.get_id()).collect::<Vec<_>>();
                (ids.as_ptr(), buffs.len(), Some(ids))
            }
            None => (std::ptr::null(), 0, None),
        };

        let (texture_ids, texture_layouts, textures_num, _, _) = match textures {
            Some(textures) => {
                let ids = textures.iter().map(|t| t.0.get_id()).collect::<Vec<_>>();
                let layouts = textures
                    .iter()
                    .map(|t| t.1.into())
                    .collect::<Vec<gl::types::GLenum>>();
                (
                    ids.as_ptr(),
                    layouts.as_ptr(),
                    textures.len(),
                    Some(ids),
                    Some(layouts),
                )
            }
            None => (std::ptr::null(), std::ptr::null(), 0, None, None),
        };

        let ctxt = self.context.get_context().make_current();
        unsafe {
            ctxt.gl.SignalSemaphoreEXT(
                self.id,
                buffer_num as u32,
                buffer_ids,
                textures_num as u32,
                texture_ids,
                texture_layouts,
            )
        }
    }
}

impl GlObject for Semaphore {
    type Id = gl::types::GLuint;

    #[inline]
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        let ctxt = self.context.get_context().make_current();
        unsafe { ctxt.gl.DeleteSemaphoresEXT(1, &mut self.id as *mut u32) };
    }
}