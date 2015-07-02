/*!

Handles bindless textures.




*/
use texture::any::TextureAny;
use TextureExt;
use GlObject;

use ContextExt;
use gl;

use std::marker::PhantomData;

/// A texture that is resident in video memory. This allows you to use bindless textures in your
/// shaders.
pub struct ResidentTexture {
    texture: Option<TextureAny>,
    handle: gl::types::GLuint64,
}

impl ResidentTexture {
    /// Takes ownership of the given texture and makes it resident.
    ///
    /// # Features
    ///
    /// Only available if the 'gl_bindless_textures' feature is enabled.
    ///
    #[cfg(gl_bindless_textures)]
    pub fn new(texture: TextureAny) -> ResidentTexture {
        ResidentTexture::new_if_supported(texture).unwrap()
    }

    /// Takes ownership of the given texture and makes it resident.
    // TODO: sampler
    pub fn new_if_supported(texture: TextureAny) -> Option<ResidentTexture> {
        let handle = {
            let mut ctxt = texture.get_context().make_current();

            if !ctxt.extensions.gl_arb_bindless_texture {
                return None;
            }

            let handle = unsafe { ctxt.gl.GetTextureHandleARB(texture.get_id()) };
            unsafe { ctxt.gl.MakeTextureHandleResidentARB(handle) };
            ctxt.resident_texture_handles.push(handle);
            handle
        };

        // store the handle in the context
        Some(ResidentTexture {
            texture: Some(texture),
            handle: handle,
        })
    }

    /// Unwraps the texture and restores it.
    pub fn into_inner(mut self) -> TextureAny {
        self.into_inner_impl()
    }

    /// Implementation of `into_inner`. Also called by the destructor.
    fn into_inner_impl(&mut self) -> TextureAny {
        let texture = self.texture.take().unwrap();

        {
            let mut ctxt = texture.get_context().make_current();
            unsafe { ctxt.gl.MakeTextureHandleNonResidentARB(self.handle) };
            ctxt.resident_texture_handles.retain(|&t| t != self.handle);
        }

        texture
    }
}

impl Drop for ResidentTexture {
    fn drop(&mut self) {
        self.into_inner_impl();
    }
}

/// Handle to a texture.
#[derive(Copy, Clone)]
pub struct TextureHandle<'a> {
    value: gl::types::GLuint64,
    marker: PhantomData<&'a ResidentTexture>,
}

impl<'a> TextureHandle<'a> {
    /// Builds a new handle.
    pub fn new(texture: &'a ResidentTexture) -> TextureHandle<'a> {
        TextureHandle {
            value: texture.handle,
            marker: PhantomData,
        }
    }

    /// Sets the value to the given texture.
    pub fn set(&mut self, texture: &'a ResidentTexture) {
        self.value = texture.handle;
    }
}

impl<'a> ::uniforms::AsUniformValue for TextureHandle<'a> {
    fn as_uniform_value(&self) -> ::uniforms::UniformValue {
        // TODO: u64
        unimplemented!();
    }

    fn matches(_: &::uniforms::UniformType) -> bool {
        // FIXME: hack to make bindless textures work
        true
    }
}

// TODO: implement `vertex::Attribute` on `TextureHandle`

#[cfg(test)]
mod test {
    use std::mem;
    use super::TextureHandle;

    #[test]
    fn texture_handle_size() {
        assert_eq!(mem::size_of::<TextureHandle>(), 8);
    }
}
