use std::{error::Error, fmt};

use crate::{gl, image_format::FormatNotSupportedError, memory_object::MemoryObjectCreationError};

/// Describes a tiling mode used in texture storage by an external API
pub enum ExternalTilingMode {
    /// Corresponds to VK_IMAGE_TILING_OPTIMAL
    Optimal,
    /// Corresponds to VK_IMAGE_TILING_LINEAR
    Linear
}

impl Into<crate::gl::types::GLenum> for ExternalTilingMode {
    fn into(self) -> crate::gl::types::GLenum {
        match self {
            ExternalTilingMode::Optimal => gl::OPTIMAL_TILING_EXT,
            ExternalTilingMode::Linear => gl::LINEAR_TILING_EXT,
        }
    }
}

/// Contains parameters needed to import a texture created in an external
/// API into OpenGL. Must match with parameters used by external memory.
pub struct ImportParameters {
    /// Describes whether this memory was created as "dedicated" by the external API.
    pub dedicated_memory: bool,
    /// Size of the memory object in bytes.
    pub size: u64,
    /// Offset of the memory object in bytes.
    pub offset: u64,
    /// Tiling mode used in the memory object.
    pub tiling: ExternalTilingMode,
}


/// Error that can happen when importing a texture.
#[derive(Debug, Clone, Copy)]
pub enum TextureImportError {
    /// A specific format for the texture was not given.
    FormatNotPresent,
    /// An error ocurred during memory object creation
    MemoryObjectCreation(MemoryObjectCreationError),
    /// Texture format not supported by this OpenGL context
    FormatNotSupported(FormatNotSupportedError),
}

impl fmt::Display for TextureImportError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::TextureImportError::*;
        match *self {
            FormatNotPresent => write!(fmt, "A specific format for the texture was not given."),
            MemoryObjectCreation(e) => e.fmt(fmt),
            FormatNotSupported(e) => e.fmt(fmt),
        }
    }
}

impl From<MemoryObjectCreationError> for TextureImportError {
    fn from(e: MemoryObjectCreationError) -> Self {
        Self::MemoryObjectCreation(e)
    }
}

impl Error for TextureImportError {}
