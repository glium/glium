/*!
Pixel buffers are buffers that contain two-dimensional texture data.

Contrary to textures, pixel buffers are stored in a client-defined format. They are used
to transfer data to or from the video memory, before or after being turned into a texture.
*/
use std::borrow::Cow;
use std::cell::Cell;
use std::ops::{Deref, DerefMut};

use backend::Facade;

use GlObject;
use buffer::{ReadError, Buffer, BufferType, BufferMode};
use gl;

use texture::PixelValue;
use texture::Texture2dDataSink;

/// Buffer that stores the content of a texture.
///
/// The generic type represents the type of pixels that the buffer contains.
pub struct PixelBuffer<T> where T: PixelValue {
    buffer: Buffer<[T]>,
    dimensions: Cell<Option<(u32, u32)>>,
}

impl<T> PixelBuffer<T> where T: PixelValue {
    /// Builds a new buffer with an uninitialized content.
    #[inline]
    pub fn new_empty<F: ?Sized>(facade: &F, capacity: usize) -> PixelBuffer<T> where F: Facade {
        PixelBuffer {
            buffer: Buffer::empty_array(facade, BufferType::PixelPackBuffer, capacity,
                                            BufferMode::Default).unwrap(),
            dimensions: Cell::new(None),
        }
    }

    /// Reads the content of the pixel buffer.
    #[inline]
    pub fn read_as_texture_2d<S>(&self) -> Result<S, ReadError> where S: Texture2dDataSink<T> {
        let dimensions = self.dimensions.get().expect("The pixel buffer is empty");
        let data = self.read()?;
        Ok(S::from_raw(Cow::Owned(data), dimensions.0, dimensions.1))
    }
}

impl<T> Deref for PixelBuffer<T> where T: PixelValue {
    type Target = Buffer<[T]>;

    #[inline]
    fn deref(&self) -> &Buffer<[T]> {
        &self.buffer
    }
}

impl<T> DerefMut for PixelBuffer<T> where T: PixelValue {
    #[inline]
    fn deref_mut(&mut self) -> &mut Buffer<[T]> {
        &mut self.buffer
    }
}

// TODO: rework this
impl<T> GlObject for PixelBuffer<T> where T: PixelValue {
    type Id = gl::types::GLuint;

    #[inline]
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
    }
}

// TODO: remove this hack
#[doc(hidden)]
#[inline]
pub fn store_infos<T>(b: &PixelBuffer<T>, dimensions: (u32, u32)) where T: PixelValue {
    b.dimensions.set(Some(dimensions));
}
