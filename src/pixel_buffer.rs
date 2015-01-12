/*!
Pixel buffers are buffer that contain two-dimensional texture data.

Contrary to textures, pixel buffers are stored in a client-defined format. They are used
to transfer data to or from the video memory before or after being turned into a texture.
*/
use Display;
use texture::Texture2dData;

use GlObject;
use buffer::{self, Buffer};
use gl;

/// Buffer that stores the content of a texture.
///
/// The generic type represents the type of pixels that the buffer contains.
pub struct PixelBuffer<T> {
    buffer: Buffer,
    width: Option<u32>,
}

impl<T> PixelBuffer<T> where T: Texture2dData {
    /// Builds a new buffer with an uninitialized content.
    pub fn new_empty(display: &Display, capacity: usize) -> PixelBuffer<T> {
        PixelBuffer {
            buffer: Buffer::new_empty::<buffer::PixelUnpackBuffer>(display, 1, capacity,
                                                                   gl::DYNAMIC_READ),
            width: None,
        }
    }

    /// Returns the size in bytes of the buffer.
    pub fn get_size(&self) -> usize {
        self.buffer.get_total_size()
    }

    /// Copies the content of the pixel buffer to the RAM.
    ///
    /// This operation is slow and should be done outside of the rendering loop.
    ///
    /// ## Panic
    ///
    /// Panics if the pixel buffer is empty.
    #[cfg(feature = "gl_extensions")]       // TODO: remove
    pub fn read(&self) -> T {
        let data = self.buffer.read::<buffer::PixelPackBuffer, _>();
        Texture2dData::from_vec(data, self.width.expect("The pixel buffer is empty"))
    }
}

impl<T> GlObject for PixelBuffer<T> {
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
    }
}

// TODO: remove this hack
#[doc(hidden)]
pub fn store_width<T>(b: &mut PixelBuffer<T>, width: u32) {
    b.width = Some(width);
}
