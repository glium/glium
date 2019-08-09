/*!

A `BufferTexture` is a special kind of one-dimensional texture that gets its data from a buffer.
Buffer textures have very limited capabilities compared to other texture types.

A buffer texture is composed of two things:
 - A buffer.
 - A texture.

The `BufferTexture` object derefs to a `Buffer`, which allows you to modify the content of the
buffer just like any other buffer type.

The texture aspect of the buffer texture is very limited. The only thing you can do is use the
texture for sampling or image load/store in your shaders. You can't upload or read the texture,
it doesn't have any mipmap, etc.

# Formats

In order to build a `BufferTexture`, the elements of the buffer must implement the
`TextureBufferContent` trait. Even though a buffer can hold any type of data, a buffer texture
only supports some precise formats of data.

Support for various formats has been added in OpenGL over time. The following formats have the
most chances of being supported:

 - `F16F16F16F16`
 - `F32F32F32`
 - `F32F32F32F32`
 - `U32U32U32`
 - `I32I32I32`
 - `U8U8U8U8`
 - `I8I8I8I8`
 - `U16U16U16U16`
 - `I16I16I16I16`
 - `U32U32U32U32` (unsigned only)
 - `I32I32I32I32` (signed only)

# Buffer texture type

The template parameter that you use for `BufferTexture` defines the content of the buffer. For
example a `BufferTexture<(u8, u8, u8, u8)>` contains a list of four-component texels where each
texel is a `u8`. However this data can be interpreted in two different ways: either as a normalized
floating-point (where `0` is interpreted as `0.0` and `255` interpreted as `1.0`) or as an integral
value.

For this reason, you need to pass a `BufferTextureType` when creating the buffer texture.

This type also corresponds to the type of sampler that you must use in your GLSL code. In order
to sample from a buffer texture of type `Float` you need to use a `samplerBuffer`, in order to
sample from a buffer texture of type `Integral` you need to use a `isamplerBuffer`, and in order
to sample from a buffer texture of type `Unsigned` you need to use a `usamplerBuffer`. Using the
wrong type will result in an error.

*/
use std::{ mem, fmt };
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::error::Error;

use gl;
use version::Version;
use version::Api;
use backend::Facade;
use context::Context;
use context::CommandContext;
use ContextExt;
use GlObject;

use TextureExt;

use BufferExt;
use buffer::BufferMode;
use buffer::BufferType;
use buffer::Buffer;
use buffer::BufferCreationError;
use buffer::Content as BufferContent;

use uniforms::AsUniformValue;
use uniforms::UniformValue;

/// Error that can happen while building the texture part of a buffer texture.
#[derive(Copy, Clone, Debug)]
pub enum TextureCreationError {
    /// Buffer textures are not supported at all.
    NotSupported,

    /// The requested format is not supported in combination with the given texture buffer type.
    FormatNotSupported,

    /// The size of the buffer that you are trying to bind exceeds `GL_MAX_TEXTURE_BUFFER_SIZE`.
    TooLarge,
}

impl fmt::Display for TextureCreationError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

impl Error for TextureCreationError {
    fn description(&self) -> &str {
        use self::TextureCreationError::*;
        match *self {
            NotSupported =>
                "Buffer textures are not supported at all",
            FormatNotSupported =>
                "The requested format is not supported in combination with the given texture buffer type",
            TooLarge =>
                "The size of the buffer that you are trying to bind exceeds `GL_MAX_TEXTURE_BUFFER_SIZE`",
        }
    }
}

/// Error that can happen while building a buffer texture.
#[derive(Copy, Clone, Debug)]
pub enum CreationError {
    /// Failed to create the buffer.
    BufferCreationError(BufferCreationError),

    /// Failed to create the texture.
    TextureCreationError(TextureCreationError),
}

impl fmt::Display for CreationError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

impl Error for CreationError {
    fn description(&self) -> &str {
        use self::CreationError::*;
        match *self {
            BufferCreationError(_) =>
                "Failed to create the buffer",
            TextureCreationError(_) =>
                "Failed to create the texture",
        }
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::CreationError::*;
        match *self {
            BufferCreationError(ref err) => Some(err),
            TextureCreationError(ref err) => Some(err),
        }
    }
}

impl From<BufferCreationError> for CreationError {
    #[inline]
    fn from(err: BufferCreationError) -> CreationError {
        CreationError::BufferCreationError(err)
    }
}

impl From<TextureCreationError> for CreationError {
    #[inline]
    fn from(err: TextureCreationError) -> CreationError {
        CreationError::TextureCreationError(err)
    }
}

/// Type of a buffer texture.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BufferTextureType {
    /// The texture will behave as if it contained floating-point data. It can be sampled with
    /// a `samplerBuffer` in your GLSL code.
    ///
    /// If the buffer actually contains integer values, they will be normalized so that `0`
    /// is interpreted as `0.0` and the maximum possible value (for example `255` for `u8`s)
    /// is interpreted as `1.0`.
    Float,

    /// The texture will behave as if it contained signed integral data. It can be sampled with
    /// a `isamplerBuffer` in your GLSL code.
    Integral,

    /// The texture will behave as if it contained unsigned integral data. It can be sampled with
    /// a `usamplerBuffer` in your GLSL code.
    Unsigned,
}

/// A one-dimensional texture that gets its data from a buffer.
pub struct BufferTexture<T> where [T]: BufferContent {
    buffer: Buffer<[T]>,
    texture: gl::types::GLuint,
    ty: BufferTextureType,
}

impl<T> BufferTexture<T> where [T]: BufferContent, T: TextureBufferContent + Copy {
    /// Builds a new texture buffer from data.
    #[inline]
    pub fn new<F: ?Sized>(facade: &F, data: &[T], ty: BufferTextureType)
                  -> Result<BufferTexture<T>, CreationError>
                  where F: Facade
    {
        BufferTexture::new_impl(facade, data, BufferMode::Default, ty)
    }

    /// Builds a new texture buffer from data.
    #[inline]
    pub fn dynamic<F: ?Sized>(facade: &F, data: &[T], ty: BufferTextureType)
                  -> Result<BufferTexture<T>, CreationError>
                      where F: Facade
    {
        BufferTexture::new_impl(facade, data, BufferMode::Dynamic, ty)
    }

    /// Builds a new texture buffer from data.
    #[inline]
    pub fn persistent<F: ?Sized>(facade: &F, data: &[T], ty: BufferTextureType)
                  -> Result<BufferTexture<T>, CreationError>
                         where F: Facade
    {
        BufferTexture::new_impl(facade, data, BufferMode::Persistent, ty)
    }

    /// Builds a new texture buffer from data.
    #[inline]
    pub fn immutable<F: ?Sized>(facade: &F, data: &[T], ty: BufferTextureType)
                        -> Result<BufferTexture<T>, CreationError>
                        where F: Facade
    {
        BufferTexture::new_impl(facade, data, BufferMode::Immutable, ty)
    }

    #[inline]
    fn new_impl<F: ?Sized>(facade: &F, data: &[T], mode: BufferMode, ty: BufferTextureType)
                   -> Result<BufferTexture<T>, CreationError>
                   where F: Facade
    {
        let buffer = Buffer::new(facade, data, BufferType::TextureBuffer, mode)?;
        BufferTexture::from_buffer(facade, buffer, ty).map_err(|(e, _)| e.into())
    }

    /// Builds a new empty buffer buffer.
    #[inline]
    pub fn empty<F: ?Sized>(facade: &F, len: usize, ty: BufferTextureType)
                    -> Result<BufferTexture<T>, CreationError>
                    where F: Facade
    {
        BufferTexture::empty_impl(facade, len, ty, BufferMode::Default)
    }

    /// Builds a new empty buffer buffer.
    #[inline]
    pub fn empty_dynamic<F: ?Sized>(facade: &F, len: usize, ty: BufferTextureType)
                            -> Result<BufferTexture<T>, CreationError>
                            where F: Facade
    {
        BufferTexture::empty_impl(facade, len, ty, BufferMode::Dynamic)
    }

    /// Builds a new empty buffer buffer.
    #[inline]
    pub fn empty_persistent<F: ?Sized>(facade: &F, len: usize, ty: BufferTextureType)
                               -> Result<BufferTexture<T>, CreationError>
                               where F: Facade
    {
        BufferTexture::empty_impl(facade, len, ty, BufferMode::Persistent)
    }

    /// Builds a new empty buffer buffer.
    #[inline]
    pub fn empty_immutable<F: ?Sized>(facade: &F, len: usize, ty: BufferTextureType)
                              -> Result<BufferTexture<T>, CreationError>
                              where F: Facade
    {
        BufferTexture::empty_impl(facade, len, ty, BufferMode::Immutable)
    }

    #[inline]
    fn empty_impl<F: ?Sized>(facade: &F, len: usize, ty: BufferTextureType, mode: BufferMode)
                     -> Result<BufferTexture<T>, CreationError>
                     where F: Facade
    {
        let buffer = Buffer::empty_array(facade, BufferType::TextureBuffer, len, mode)?;
        BufferTexture::from_buffer(facade, buffer, ty).map_err(|(e, _)| e.into())
    }

    /// Builds a new buffer texture by taking ownership of a buffer.
    pub fn from_buffer<F: ?Sized>(context: &F, buffer: Buffer<[T]>, ty: BufferTextureType)
                          -> Result<BufferTexture<T>, (TextureCreationError, Buffer<[T]>)>
                          where F: Facade
    {
        let context = context.get_context();
        let mut ctxt = context.make_current();

        // checking capabilities
        if buffer.get_size() / mem::size_of::<T>() > ctxt.capabilities
                                                         .max_texture_buffer_size.unwrap() as usize
        {
            return Err((TextureCreationError::TooLarge, buffer));
        }

        // before starting, we determine the internal format and check that buffer textures are
        // supported
        let internal_format = if ctxt.version >= &Version(Api::Gl, 3, 0) ||
                                 ctxt.extensions.gl_oes_texture_buffer ||
                                 ctxt.extensions.gl_ext_texture_buffer
        {
            match (T::get_type(), ty) {
                (TextureBufferContentType::U8, BufferTextureType::Float) => gl::R8,
                (TextureBufferContentType::U8, BufferTextureType::Unsigned) => gl::R8UI,
                (TextureBufferContentType::I8, BufferTextureType::Integral) => gl::R8I,
                (TextureBufferContentType::U16, BufferTextureType::Float) => gl::R16,
                (TextureBufferContentType::U16, BufferTextureType::Unsigned) => gl::R16UI,
                (TextureBufferContentType::I16, BufferTextureType::Integral) => gl::R16I,
                (TextureBufferContentType::U32, BufferTextureType::Unsigned) => gl::R32UI,
                (TextureBufferContentType::I32, BufferTextureType::Integral) => gl::R32I,
                (TextureBufferContentType::U8U8, BufferTextureType::Float) => gl::RG8,
                (TextureBufferContentType::U8U8, BufferTextureType::Unsigned) => gl::RG8UI,
                (TextureBufferContentType::I8I8, BufferTextureType::Integral) => gl::RG8I,
                (TextureBufferContentType::U16U16, BufferTextureType::Float) => gl::RG16,
                (TextureBufferContentType::U16U16, BufferTextureType::Unsigned) => gl::RG16UI,
                (TextureBufferContentType::I16I16, BufferTextureType::Integral) => gl::RG16I,
                (TextureBufferContentType::U32U32, BufferTextureType::Unsigned) => gl::RG32UI,
                (TextureBufferContentType::I32I32, BufferTextureType::Integral) => gl::RG32I,
                (TextureBufferContentType::U8U8U8U8, BufferTextureType::Float) => gl::RGBA8,
                (TextureBufferContentType::U8U8U8U8, BufferTextureType::Unsigned) => gl::RGBA8UI,
                (TextureBufferContentType::I8I8I8I8, BufferTextureType::Integral) => gl::RGBA8I,
                (TextureBufferContentType::U16U16U16U16, BufferTextureType::Float) => gl::RGBA16,
                (TextureBufferContentType::U16U16U16U16, BufferTextureType::Unsigned) =>
                                                                                      gl::RGBA16UI,
                (TextureBufferContentType::I16I16I16I16, BufferTextureType::Integral) =>
                                                                                       gl::RGBA16I,
                (TextureBufferContentType::U32U32U32U32, BufferTextureType::Unsigned) =>
                                                                                      gl::RGBA32UI,
                (TextureBufferContentType::I32I32I32I32, BufferTextureType::Integral) =>
                                                                                       gl::RGBA32I,
                (TextureBufferContentType::F16, BufferTextureType::Float) => gl::R16F,
                (TextureBufferContentType::F32, BufferTextureType::Float) => gl::R32F,
                (TextureBufferContentType::F16F16, BufferTextureType::Float) => gl::RG16F,
                (TextureBufferContentType::F32F32, BufferTextureType::Float) => gl::RG32F,
                (TextureBufferContentType::F16F16F16F16, BufferTextureType::Float) => gl::RGBA16F,
                (TextureBufferContentType::F32F32F32F32, BufferTextureType::Float) => gl::RGBA32F,

                (TextureBufferContentType::U32U32U32, BufferTextureType::Unsigned)
                                            if ctxt.version >= &Version(Api::Gl, 4, 0) ||
                                               ctxt.extensions.gl_arb_texture_buffer_object_rgb32
                                                                                    => gl::RGB32UI,
                (TextureBufferContentType::I32I32I32, BufferTextureType::Integral)
                                            if ctxt.version >= &Version(Api::Gl, 4, 0) ||
                                               ctxt.extensions.gl_arb_texture_buffer_object_rgb32
                                                                                    => gl::RGB32I,
                (TextureBufferContentType::F32F32F32, BufferTextureType::Float)
                                            if ctxt.version >= &Version(Api::Gl, 4, 0) ||
                                               ctxt.extensions.gl_arb_texture_buffer_object_rgb32
                                                                                    => gl::RGB32F,

                _ => return Err((TextureCreationError::FormatNotSupported, buffer))
            }

        } else if ctxt.extensions.gl_arb_texture_buffer_object ||
                  ctxt.extensions.gl_ext_texture_buffer_object
        {
            match (T::get_type(), ty) {
                (TextureBufferContentType::U8U8U8U8, BufferTextureType::Float) => gl::RGBA8,
                (TextureBufferContentType::U16U16U16U16, BufferTextureType::Float) => gl::RGBA16,
                (TextureBufferContentType::F16F16F16F16, BufferTextureType::Float) => gl::RGBA16F,
                (TextureBufferContentType::F32F32F32F32, BufferTextureType::Float) => gl::RGBA32F,
                (TextureBufferContentType::I8I8I8I8, BufferTextureType::Integral) => gl::RGBA8I,
                (TextureBufferContentType::I16I16I16I16, BufferTextureType::Integral) =>
                                                                                      gl::RGBA16I,
                (TextureBufferContentType::I32I32I32I32, BufferTextureType::Integral) =>
                                                                                      gl::RGBA32I,
                (TextureBufferContentType::U8U8U8U8, BufferTextureType::Unsigned) => gl::RGBA8UI,
                (TextureBufferContentType::U16U16U16U16, BufferTextureType::Unsigned) =>
                                                                                      gl::RGBA16UI,
                (TextureBufferContentType::U32U32U32U32, BufferTextureType::Unsigned) =>
                                                                                      gl::RGBA32UI,

                (TextureBufferContentType::U32U32U32, BufferTextureType::Unsigned)
                                            if ctxt.extensions.gl_arb_texture_buffer_object_rgb32
                                                                                    => gl::RGB32UI,
                (TextureBufferContentType::I32I32I32, BufferTextureType::Integral)
                                            if ctxt.extensions.gl_arb_texture_buffer_object_rgb32
                                                                                    => gl::RGB32I,
                (TextureBufferContentType::F32F32F32, BufferTextureType::Float)
                                            if ctxt.extensions.gl_arb_texture_buffer_object_rgb32
                                                                                    => gl::RGB32F,

                // TODO: intensity?

                _ => return Err((TextureCreationError::FormatNotSupported, buffer))
            }

        } else {
            return Err((TextureCreationError::NotSupported, buffer));
        };

        // now the texture creation
        debug_assert_eq!(buffer.get_offset_bytes(), 0);
        let id = if ctxt.version >= &Version(Api::Gl, 4, 5) ||
                    ctxt.extensions.gl_arb_direct_state_access
        {
            unsafe {
                let mut id = 0;
                ctxt.gl.CreateTextures(gl::TEXTURE_BUFFER, 1, &mut id);
                ctxt.gl.TextureBuffer(id, internal_format, buffer.get_id());
                id
            }

        } else {
            // reserving the ID
            let id = unsafe {
                let mut id = 0;
                ctxt.gl.GenTextures(1, &mut id);
                id
            };

            // binding the texture
            unsafe {
                ctxt.gl.BindTexture(gl::TEXTURE_BUFFER, id);
                let act = ctxt.state.active_texture as usize;
                ctxt.state.texture_units[act].texture = id;
            }

            // binding the buffer
            if ctxt.version >= &Version(Api::Gl, 3, 0) ||
               ctxt.version >= &Version(Api::GlEs, 3, 2)
            {
                unsafe {
                    ctxt.gl.TexBuffer(gl::TEXTURE_BUFFER, internal_format, buffer.get_id());
                }
            } else if ctxt.extensions.gl_arb_texture_buffer_object {
                unsafe {
                    ctxt.gl.TexBufferARB(gl::TEXTURE_BUFFER, internal_format,
                                         buffer.get_id());
                }
            } else if ctxt.extensions.gl_ext_texture_buffer_object ||
                      ctxt.extensions.gl_ext_texture_buffer
            {
                unsafe {
                    ctxt.gl.TexBufferEXT(gl::TEXTURE_BUFFER, internal_format,
                                         buffer.get_id());
                }
            } else if ctxt.extensions.gl_oes_texture_buffer {
                unsafe {
                    ctxt.gl.TexBufferOES(gl::TEXTURE_BUFFER, internal_format,
                                         buffer.get_id());
                }

            } else {
                // handled during the choice for the internal format
                // note that this panic will leak the texture
                unreachable!();
            }

            id
        };

        Ok(BufferTexture {
            buffer: buffer,
            ty: ty,
            texture: id,
        })
    }
}

impl<T> Deref for BufferTexture<T> where [T]: BufferContent {
    type Target = Buffer<[T]>;

    #[inline]
    fn deref(&self) -> &Buffer<[T]> {
        &self.buffer
    }
}

impl<T> DerefMut for BufferTexture<T> where [T]: BufferContent {
    #[inline]
    fn deref_mut(&mut self) -> &mut Buffer<[T]> {
        &mut self.buffer
    }
}

impl<T> Drop for BufferTexture<T> where [T]: BufferContent {
    fn drop(&mut self) {
        let mut ctxt = self.buffer.get_context().make_current();

        // resetting the bindings
        for tex_unit in ctxt.state.texture_units.iter_mut() {
            if tex_unit.texture == self.texture {
                tex_unit.texture = 0;
            }
        }

        unsafe { ctxt.gl.DeleteTextures(1, [ self.texture ].as_ptr()); }
    }
}

impl<T> BufferTexture<T> where [T]: BufferContent {
    /// Builds a `BufferTextureRef`.
    #[inline]
    pub fn as_buffer_texture_ref(&self) -> BufferTextureRef {
        BufferTextureRef {
            texture: self.texture,
            ty: self.ty,
            marker: PhantomData,
        }
    }
}

impl<T> AsUniformValue for BufferTexture<T> where [T]: BufferContent {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        // FIXME: handle `glMemoryBarrier` for the buffer
        UniformValue::BufferTexture(self.as_buffer_texture_ref())
    }
}

impl<'a, T: 'a> AsUniformValue for &'a BufferTexture<T> where [T]: BufferContent {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        // FIXME: handle `glMemoryBarrier` for the buffer
        UniformValue::BufferTexture(self.as_buffer_texture_ref())
    }
}

/// Holds a reference to a `BufferTexture`.
#[derive(Copy, Clone)]
pub struct BufferTextureRef<'a> {
    texture: gl::types::GLuint,
    ty: BufferTextureType,
    marker: PhantomData<&'a ()>,
}

impl<'a> BufferTextureRef<'a> {
    /// Return the type of the texture.
    #[inline]
    pub fn get_texture_type(&self) -> BufferTextureType {
        self.ty
    }
}

impl<'a> TextureExt for BufferTextureRef<'a> {
    #[inline]
    fn get_texture_id(&self) -> gl::types::GLuint {
        self.texture
    }

    #[inline]
    fn get_context(&self) -> &Rc<Context> {
        unimplemented!();       // TODO:
    }

    #[inline]
    fn get_bind_point(&self) -> gl::types::GLenum {
        gl::TEXTURE_BUFFER
    }

    #[inline]
    fn bind_to_current(&self, ctxt: &mut CommandContext) -> gl::types::GLenum {
        unsafe { ctxt.gl.BindTexture(gl::TEXTURE_BUFFER, self.texture); }
        gl::TEXTURE_BUFFER
    }
}

///
///
/// Note that some three-component types are missing. This is not a mistake. OpenGL doesn't
/// support them.
#[allow(missing_docs)]
pub enum TextureBufferContentType {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U8U8,
    I8I8,
    U16U16,
    I16I16,
    U32U32,
    I32I32,
    U32U32U32,
    I32I32I32,
    U8U8U8U8,
    I8I8I8I8,
    U16U16U16U16,
    I16I16I16I16,
    U32U32U32U32,
    I32I32I32I32,
    F16,
    F32,
    F16F16,
    F32F32,
    F32F32F32,
    F16F16F16F16,
    F32F32F32F32,
}

/// Trait for data types that can be interpreted by a buffer texture.
pub unsafe trait TextureBufferContent: BufferContent {
    /// Returns the enumeration corresponding to elements of this data type.
    fn get_type() -> TextureBufferContentType;
}

unsafe impl TextureBufferContent for u8 {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::U8
    }
}

unsafe impl TextureBufferContent for i8 {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::I8
    }
}

unsafe impl TextureBufferContent for u16 {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::U16
    }
}

unsafe impl TextureBufferContent for i16 {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::I16
    }
}

unsafe impl TextureBufferContent for u32 {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::U32
    }
}

unsafe impl TextureBufferContent for i32 {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::I32
    }
}

unsafe impl TextureBufferContent for (u8, u8) {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::U8U8
    }
}

unsafe impl TextureBufferContent for [u8; 2] {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::U8U8
    }
}

unsafe impl TextureBufferContent for (i8, i8) {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::I8I8
    }
}

unsafe impl TextureBufferContent for [i8; 2] {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::I8I8
    }
}

unsafe impl TextureBufferContent for (u16, u16) {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::U16U16
    }
}

unsafe impl TextureBufferContent for [u16; 2] {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::U16U16
    }
}

unsafe impl TextureBufferContent for (i16, i16) {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::I16I16
    }
}

unsafe impl TextureBufferContent for [i16; 2] {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::I16I16
    }
}

unsafe impl TextureBufferContent for (u32, u32) {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::U32U32
    }
}

unsafe impl TextureBufferContent for [u32; 2] {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::U32U32
    }
}

unsafe impl TextureBufferContent for (i32, i32) {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::I32I32
    }
}

unsafe impl TextureBufferContent for [i32; 2] {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::I32I32
    }
}

unsafe impl TextureBufferContent for (u32, u32, u32) {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::U32U32U32
    }
}

unsafe impl TextureBufferContent for [u32; 3] {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::U32U32U32
    }
}

unsafe impl TextureBufferContent for (i32, i32, i32) {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::I32I32I32
    }
}

unsafe impl TextureBufferContent for [i32; 3] {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::I32I32I32
    }
}

unsafe impl TextureBufferContent for (u8, u8, u8, u8) {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::U8U8U8U8
    }
}

unsafe impl TextureBufferContent for [u8; 4] {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::U8U8U8U8
    }
}

unsafe impl TextureBufferContent for (i8, i8, i8, i8) {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::I8I8I8I8
    }
}

unsafe impl TextureBufferContent for [i8; 4] {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::I8I8I8I8
    }
}

unsafe impl TextureBufferContent for (u16, u16, u16, u16) {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::U16U16U16U16
    }
}

unsafe impl TextureBufferContent for [u16; 4] {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::U16U16U16U16
    }
}

unsafe impl TextureBufferContent for (i16, i16, i16, i16) {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::I16I16I16I16
    }
}

unsafe impl TextureBufferContent for [i16; 4] {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::I16I16I16I16
    }
}

unsafe impl TextureBufferContent for (u32, u32, u32, u32) {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::U32U32U32U32
    }
}

unsafe impl TextureBufferContent for [u32; 4] {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::U32U32U32U32
    }
}

unsafe impl TextureBufferContent for (i32, i32, i32, i32) {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::I32I32I32I32
    }
}

unsafe impl TextureBufferContent for [i32; 4] {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::I32I32I32I32
    }
}

unsafe impl TextureBufferContent for f32 {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::F32
    }
}

unsafe impl TextureBufferContent for (f32, f32) {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::F32F32
    }
}

unsafe impl TextureBufferContent for [f32; 2] {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::F32F32
    }
}

unsafe impl TextureBufferContent for (f32, f32, f32) {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::F32F32F32
    }
}

unsafe impl TextureBufferContent for [f32; 3] {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::F32F32F32
    }
}

unsafe impl TextureBufferContent for (f32, f32, f32, f32) {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::F32F32F32F32
    }
}

unsafe impl TextureBufferContent for [f32; 4] {
    #[inline]
    fn get_type() -> TextureBufferContentType {
        TextureBufferContentType::F32F32F32F32
    }
}
