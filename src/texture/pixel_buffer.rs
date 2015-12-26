/*!
Pixel buffers are buffers that contain two-dimensional texture data.

Contrary to textures, pixel buffers are stored in a client-defined format. They are used
to transfer data to or from the video memory, before or after being turned into a texture.
*/
use std::borrow::Cow;
use std::borrow::ToOwned;
use std::cell::Cell;
use std::ops::{Deref, DerefMut};

use backend::Facade;

use buffer::BufferCreationError;
use buffer::Create as BufferCreate;
use buffer::Storage as BufferStorage;
use buffer::Read as BufferRead;
use buffer::BufferAnySlice;
use buffer::{ReadError, DynamicBuffer, BufferType, BufferMode};

use GlObject;
use BufferExt;
use gl;

use texture::PixelValue;
use texture::Texture2dDataSink;

pub type PixelBuffer<T> = PixelStorage<DynamicBuffer<[T]>>;

/// Buffer that stores the content of a texture.
///
/// The generic type represents the type of pixels that the buffer contains.
pub struct PixelStorage<T> where T: BufferStorage {
    buffer: T,
    dimensions: Cell<Option<(u32, u32)>>,
}

impl<T, P> PixelStorage<T> where T: BufferCreate<Content = [P]>, P: PixelValue {
    /// Builds a new pixel buffer from data.
    #[inline]
    pub fn new<F>(facade: &F, data: &[P])
                  -> Result<PixelStorage<T>, BufferCreationError>
                  where F: Facade
    {
        let buffer = try!(BufferCreate::new(facade, data, BufferType::PixelPackBuffer));

        Ok(PixelStorage {
            buffer: buffer,
            dimensions: Cell::new(None),
        })
    }

    /// Builds a new empty pixel buffer.
    #[inline]
    pub fn empty<F>(facade: &F, capacity: usize) -> Result<PixelStorage<T>, BufferCreationError>
        where F: Facade
    {
        let buffer = try!(BufferCreate::empty_array(facade, capacity, BufferType::PixelPackBuffer));

        Ok(PixelStorage {
            buffer: buffer,
            dimensions: Cell::new(None),
        })
    }

    /// DEPRECATED. Builds a new empty pixel buffer.
    #[inline]
    pub fn new_empty<F>(facade: &F, capacity: usize) -> PixelStorage<T>
        where F: Facade
    {
        PixelStorage::empty(facade, capacity).unwrap()
    }
}

impl<T, P> PixelStorage<T> where T: BufferRead<Content = [P]>, P: PixelValue {
    /// Reads the content of the pixel buffer.
    #[inline]
    pub fn read_as_texture_2d<S>(&self) -> Result<S, ReadError>
        where S: Texture2dDataSink<P>,
              [P]: ToOwned<Owned = Vec<P>>,
    {
        let dimensions = self.dimensions.get().expect("The pixel buffer is empty");
        let data = try!(self.read());
        Ok(S::from_raw(Cow::Owned(data), dimensions.0, dimensions.1))
    }
}

impl_buffer_wrapper!(PixelStorage, buffer, [dimensions]);

// TODO: remove this hack
#[doc(hidden)]
#[inline]
pub fn store_infos<T>(b: &PixelStorage<T>, dimensions: (u32, u32)) where T: BufferStorage {
    b.dimensions.set(Some(dimensions));
}
