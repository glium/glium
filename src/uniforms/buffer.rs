use Display;
use buffer::{self, Buffer};
use program;
use uniforms::{IntoUniformValue, UniformValue, UniformBlock};

use std::ops::{Deref, DerefMut};

use GlObject;
use context;
use gl;

/// Buffer that contains a uniform block.
#[derive(Show)]
pub struct UniformBuffer<T> {
    buffer: TypelessUniformBuffer,
}

/// Same as `UniformBuffer` but doesn't contain any information about the type.
#[derive(Show)]
pub struct TypelessUniformBuffer {
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
            buffer: TypelessUniformBuffer {
                buffer: buffer,
            }
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
                buffer: TypelessUniformBuffer {
                    buffer: buffer,
                }
            })
        }
    }

    /// Modifies the content of the buffer.
    pub fn upload(&mut self, data: T) {
        self.buffer.buffer.upload::<buffer::UniformBuffer, _>(0, vec![data])
    }

    /// Maps the buffer to allow write access to it.
    ///
    /// **Warning**: using this function can slow things down a lot, because it
    /// waits for all the previous commands to be executed before returning.
    pub fn map<'a>(&'a mut self) -> Mapping<'a, T> {
        Mapping(self.buffer.buffer.map::<buffer::UniformBuffer, T>(0, 1))
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
        self.buffer.buffer.read::<buffer::UniformBuffer, T>().into_iter().next().unwrap()
    }

    /// Reads the content of the buffer.
    pub fn read_if_supported(&self) -> Option<T> {
        let res = self.buffer.buffer.read_if_supported::<buffer::UniformBuffer, T>();
        res.map(|res| res.into_iter().next().unwrap())
    }
}

impl<T> GlObject for UniformBuffer<T> {
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
    }
}

impl GlObject for TypelessUniformBuffer {
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
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

impl<'a, T> IntoUniformValue<'a> for &'a UniformBuffer<T> where T: UniformBlock {
    fn into_uniform_value(self) -> UniformValue<'a> {
        UniformValue::Block(&self.buffer, Box::new(move |&: block: &program::UniformBlock| -> bool {
            UniformBlock::matches(None::<T>, block)
        }))
    }
}
