/*!
Pixel buffers are buffers that contain two-dimensional texture data.

Contrary to textures, pixel buffers are stored in a client-defined format. They are used
to transfer data to or from the video memory, before or after being turned into a texture.
*/
use std::borrow::Cow;
use std::cell::Cell;
use std::ops::{Deref, DerefMut};

use backend::Facade;
use context::CommandContext;
use ContextExt;

use GlObject;
use BufferViewExt;
use buffer::{BufferView, BufferType};
use gl;

use texture::PixelValue;
use texture::Texture2dDataSink;

/// Buffer that stores the content of a texture.
///
/// The generic type represents the type of pixels that the buffer contains.
pub struct PixelBuffer<T> where T: PixelValue {
    buffer: BufferView<T>,
    dimensions: Cell<Option<(u32, u32)>>,
}

impl<T> PixelBuffer<T> where T: PixelValue {
    /// Builds a new buffer with an uninitialized content.
    pub fn new_empty<F>(facade: &F, capacity: usize) -> PixelBuffer<T> where F: Facade {
        PixelBuffer {
            buffer: BufferView::empty(facade, BufferType::PixelPackBuffer, capacity,
                                      false).unwrap(),
            dimensions: Cell::new(None),
        }
    }

    /// Reads the content of the pixel buffer.
    ///
    /// # Features
    ///
    /// Only available if the `gl_read_buffer` feature is enabled.
    #[cfg(feature = "gl_read_buffer")]
    pub fn read_as_texture_2d<S>(&self) -> S where S: Texture2dDataSink<T> {
        let dimensions = self.dimensions.get().expect("The pixel buffer is empty");
        S::from_raw(Cow::Owned(self.read()), dimensions.0, dimensions.1)
    }

    /// Reads the content of the pixel buffer. Returns `None` if this operation is not supported.
    pub fn read_as_texture_2d_if_supported<S>(&self) -> Option<S> where S: Texture2dDataSink<T> {
        let dimensions = self.dimensions.get().expect("The pixel buffer is empty");
        self.read_if_supported().map(|data| {
            S::from_raw(Cow::Owned(data), dimensions.0, dimensions.1)
        })
    }
}

impl<T> Deref for PixelBuffer<T> where T: PixelValue {
    type Target = BufferView<T>;

    fn deref(&self) -> &BufferView<T> {
        &self.buffer
    }
}

impl<T> DerefMut for PixelBuffer<T> where T: PixelValue {
    fn deref_mut(&mut self) -> &mut BufferView<T> {
        &mut self.buffer
    }
}

impl<T> BufferViewExt for PixelBuffer<T> where T: PixelValue {
    fn get_offset_bytes(&self) -> usize {
        self.buffer.get_offset_bytes()
    }

    fn get_buffer_id(&self, ctxt: &mut CommandContext) -> gl::types::GLuint {
        self.buffer.get_buffer_id(ctxt)
    }

    fn bind_to(&self, ctxt: &mut CommandContext, ty: BufferType) {
        self.buffer.bind_to(ctxt, ty)
    }

    fn indexed_bind_to(&self, ctxt: &mut CommandContext, ty: BufferType, index: gl::types::GLuint) {
        self.buffer.indexed_bind_to(ctxt, ty, index)
    }
}

// TODO: rework this
impl<T> GlObject for PixelBuffer<T> where T: PixelValue {
    type Id = gl::types::GLuint;

    fn get_id(&self) -> gl::types::GLuint {
        let ctxt = self.buffer.get_context();
        let mut ctxt = ctxt.make_current();
        self.buffer.get_buffer_id(&mut ctxt)
    }
}

// TODO: remove this hack
#[doc(hidden)]
pub fn store_infos<T>(b: &PixelBuffer<T>, dimensions: (u32, u32)) where T: PixelValue {
    b.dimensions.set(Some(dimensions));
}
