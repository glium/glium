use Display;
use buffer::{self, Buffer};

use std::ops::{Deref, DerefMut};

use context;
use gl;

/// Buffer that contains a uniform block.
pub struct UniformBuffer<T> {
    buffer: Buffer,
}

impl<T> UniformBuffer<T> where T: Copy + Send {
    /// Uploads data in the uniforms buffer.
    ///
    /// ## Features
    ///
    /// Only available if the `gl_uniform_blocks` feature is enabled.
    #[cfg(feature = "gl_uniform_blocks")]
    pub fn new(display: &Display, data: T) -> UniformBuffer<T> {
        let buffer = Buffer::new::<buffer::UniformBuffer, _>(display, vec![data],
                                                             gl::STATIC_DRAW);

        UniformBuffer {
            buffer: buffer,
        }
    }

    /// Uploads data in the uniforms buffer.
    pub fn new_if_supported(display: &Display, data: T) -> Option<UniformBuffer<T>> {
        if display.context.context.get_version() < &context::GlVersion(3, 1) &&
           !display.context.context.get_extensions().gl_arb_uniform_buffer_object
        {
            None

        } else {
            let buffer = Buffer::new::<buffer::UniformBuffer, _>(display, vec![data],
                                                                 gl::STATIC_DRAW);

            Some(UniformBuffer {
                buffer: buffer,
            })
        }
    }

    /// Modifies the content of the buffer.
    pub fn upload(&mut self, data: T) {
        self.buffer.upload::<buffer::UniformBuffer, _>(0, vec![data])
    }

    /// Maps the buffer to allow write access to it.
    ///
    /// **Warning**: using this function can slow things down a lot, because it
    /// waits for all the previous commands to be executed before returning.
    pub fn map<'a>(&'a mut self) -> Mapping<'a, T> {
        Mapping(self.buffer.map::<buffer::UniformBuffer, T>(0, 1))
    }

    /// Reads the content of the buffer.
    ///
    /// This function is usually better if are just doing one punctual read, while `map`
    /// is better if you want to have multiple small reads.
    ///
    /// # Features
    ///
    /// Only available if the `gl_read_buffer` feature is enabled.
    #[cfg(feature = "gl_read_buffer")]
    pub fn read(&self) -> T {
        self.buffer.read::<buffer::UniformBuffer, T>().into_iter().next().unwrap()
    }

    /// Reads the content of the buffer.
    pub fn read_if_supported(&self) -> Option<T> {
        let res = self.buffer.read_if_supported::<buffer::UniformBuffer, T>();
        res.map(|res| res.into_iter().next().unwrap())
    }
}

/// A mapping of a uniform buffer.
pub struct Mapping<'a, T>(buffer::Mapping<'a, buffer::UniformBuffer, T>);

impl<'a, T> Deref for Mapping<'a, T> {
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T {
        self.0.deref().get(0).unwrap()
    }
}

impl<'a, T> DerefMut for Mapping<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
        self.0.deref_mut().get_mut(0).unwrap()
    }
}
