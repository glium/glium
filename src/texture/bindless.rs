/*!

Handles bindless textures.




*/
use texture::any::TextureAny;
use TextureExt;
use GlObject;

use ContextExt;
use gl;

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use uniforms::AsUniformValue;
use uniforms::UniformValue;
use uniforms::UniformType;
use uniforms::SamplerBehavior;

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

impl Deref for ResidentTexture {
    type Target = TextureAny;

    fn deref(&self) -> &TextureAny {
        self.texture.as_ref().unwrap()
    }
}

impl DerefMut for ResidentTexture {
    fn deref_mut(&mut self) -> &mut TextureAny {
        self.texture.as_mut().unwrap()
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
    pub fn new(texture: &'a ResidentTexture, _: &SamplerBehavior) -> TextureHandle<'a> {
        // FIXME: take sampler into account
        TextureHandle {
            value: texture.handle,
            marker: PhantomData,
        }
    }

    /// Sets the value to the given texture.
    pub fn set(&mut self, texture: &'a ResidentTexture, _: &SamplerBehavior) {
        // FIXME: take sampler into account
        self.value = texture.handle;
    }
}

impl<'a> AsUniformValue for TextureHandle<'a> {
    fn as_uniform_value(&self) -> UniformValue {
        // TODO: u64
        unimplemented!();
    }

    fn matches(ty: &UniformType) -> bool {
        // TODO: unfortunately we have no idea what the exact type of this handle is
        //       strong typing should be considered
        //
        //       however there is no safety problem here ; the worse that can happen in case of
        //       wrong type is zeroes or undefined data being returned when sampling
        match *ty {
            UniformType::Sampler1d => true,
            UniformType::ISampler1d => true,
            UniformType::USampler1d => true,
            UniformType::Sampler2d => true,
            UniformType::ISampler2d => true,
            UniformType::USampler2d => true,
            UniformType::Sampler3d => true,
            UniformType::ISampler3d => true,
            UniformType::USampler3d => true,
            UniformType::Sampler1dArray => true,
            UniformType::ISampler1dArray => true,
            UniformType::USampler1dArray => true,
            UniformType::Sampler2dArray => true,
            UniformType::ISampler2dArray => true,
            UniformType::USampler2dArray => true,
            UniformType::SamplerCube => true,
            UniformType::ISamplerCube => true,
            UniformType::USamplerCube => true,
            UniformType::Sampler2dRect => true,
            UniformType::ISampler2dRect => true,
            UniformType::USampler2dRect => true,
            UniformType::Sampler2dRectShadow => true,
            UniformType::SamplerCubeArray => true,
            UniformType::ISamplerCubeArray => true,
            UniformType::USamplerCubeArray => true,
            UniformType::SamplerBuffer => true,
            UniformType::ISamplerBuffer => true,
            UniformType::USamplerBuffer => true,
            UniformType::Sampler2dMultisample => true,
            UniformType::ISampler2dMultisample => true,
            UniformType::USampler2dMultisample => true,
            UniformType::Sampler2dMultisampleArray => true,
            UniformType::ISampler2dMultisampleArray => true,
            UniformType::USampler2dMultisampleArray => true,
            UniformType::Sampler1dShadow => true,
            UniformType::Sampler2dShadow => true,
            UniformType::SamplerCubeShadow => true,
            UniformType::Sampler1dArrayShadow => true,
            UniformType::Sampler2dArrayShadow => true,
            UniformType::SamplerCubeArrayShadow => true,
            _ => false
        }
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
