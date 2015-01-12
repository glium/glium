/*!
Pixel buffers are buffer that contain two-dimensional texture data.

**Note**: pixel buffers are unusable for the moment (they are not yet implemented).

*/
use Display;
use texture::Texture2dData;

use GlObject;
use buffer::{self, Buffer};
use gl;

/// Buffer that stores the content of a texture.
///
/// The generic type represents the type of pixels that the buffer contains.
///
/// **Note**: pixel buffers are unusable for the moment (they are not yet implemented).
pub struct PixelBuffer<T> {
    buffer: Buffer,
}

impl<T> PixelBuffer<T> where T: Texture2dData {
    /// Builds a new buffer with an uninitialized content.
    pub fn new_empty(display: &Display, capacity: usize) -> PixelBuffer<T> {
        PixelBuffer {
            buffer: Buffer::new_empty::<buffer::PixelUnpackBuffer>(display, 1, capacity,
                                                                   gl::DYNAMIC_READ),
        }
    }

    /// Returns the size in bytes of the buffer.
    pub fn get_size(&self) -> usize {
        self.buffer.get_total_size()
    }
}

impl<T> GlObject for PixelBuffer<T> {
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
    }
}
