/*!
Pixel buffers are buffer that contain two-dimensional texture data.

**Note**: pixel buffers are unusable for the moment (they are not yet implemented).

*/
use Display;
use texture::PixelValue;

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

impl<T> PixelBuffer<T> where T: PixelValue {
    /// Builds a new buffer with an uninitialized content.
    pub fn new_empty(display: &Display, capacity: usize) -> PixelBuffer<T> {
        PixelBuffer {
            buffer: Buffer::new_empty::<buffer::PixelUnpackBuffer>(display, 1, capacity,
                                                                   gl::DYNAMIC_READ),
        }
    }

    /// Turns a `PixelBuffer<T>` into a `PixelBuffer<U>` without any check.
    pub unsafe fn transmute<U>(self) -> PixelBuffer<U> where U: PixelValue {
        PixelBuffer { buffer: self.buffer }
    }
}
