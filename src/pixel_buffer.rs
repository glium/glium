/*!
Pixel buffers are buffers that contain two-dimensional texture data.

Contrary to textures, pixel buffers are stored in a client-defined format. They are used
to transfer data to or from the video memory, before or after being turned into a texture.
 */
use std::marker::PhantomData;

use backend::Facade;

use GlObject;
use BufferViewExt;
use buffer::{BufferView, BufferViewAny, BufferType};
use gl;

/// Buffer that stores the content of a texture.
///
/// The generic type represents the type of pixels that the buffer contains.
pub struct PixelBuffer<T> {
    buffer: BufferViewAny,
    dimensions: Option<(u32, u32)>,
    marker: PhantomData<T>,
}

impl<T> PixelBuffer<T> {
    /// Builds a new buffer with an uninitialized content.
    pub fn new_empty<F>(facade: &F, capacity: usize) -> PixelBuffer<T> where F: Facade {
        PixelBuffer {
            buffer: BufferView::<u8>::empty(facade, BufferType::PixelPackBuffer, capacity,
                                           false).unwrap().into(),
            dimensions: None,
            marker: PhantomData,
        }
    }

    /// Returns the length of the buffer, in number of pixels.
    pub fn len(&self) -> usize {
        self.buffer.get_elements_count()
    }
}

// TODO: rework this
impl<T> GlObject for PixelBuffer<T> {
    type Id = gl::types::GLuint;
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_buffer_id()
    }
}

// TODO: remove this hack
#[doc(hidden)]
pub fn store_infos<T>(b: &mut PixelBuffer<T>, dimensions: (u32, u32)) {
    b.dimensions = Some(dimensions);
}
