/*!

Without bindless textures, using a texture in a shader requires binding the texture to a specific
bind point before drawing. This not only slows down rendering, but may also prevent you from
grouping multiple draw calls into one because of the limitation to the number of available
texture units.

Instead, bindless textures allow you to manually manipulate pointers to textures in video memory.
You can use thousands of textures if you want.

# Initialization

Before using a bindless texture, you must turn it into a `ResidentTexture`. This is done by
calling `resident` on the texture you want.

Bindless textures are a very recent feature that is supported only by recent hardware and
drivers. `resident` will return an `Err` if this feature is not supported.

```no_run
# use glutin::surface::{ResizeableSurface, SurfaceTypeTrait};
# fn example<T>(display: glium::Display<T>, texture: glium::texture::Texture2d)
#     where T: SurfaceTypeTrait + ResizeableSurface {
let texture = texture.resident().unwrap();
# }
```

In a real application, you will likely manage a `Vec<ResidentTexture>`.

# Usage

You can then use a `TextureHandle` as if it was a pointer to a texture. A `TextureHandle` can be
built from a `&ResidentTexture` and can't outlive it.

```no_run
#[macro_use]
extern crate glium;

# fn main() {
#[derive(Copy, Clone)]
struct UniformBuffer<'a> {
    texture: glium::texture::TextureHandle<'a>,
    some_value: f32,
}

implement_uniform_block!(UniformBuffer<'a>, texture, some_value);

# use glutin::surface::{ResizeableSurface, SurfaceTypeTrait};
# fn example<T>(display: glium::Display<T>, texture: glium::texture::bindless::ResidentTexture)
#     where T: SurfaceTypeTrait + ResizeableSurface {
let uniform_buffer = glium::uniforms::UniformBuffer::new(&display, UniformBuffer {
    texture: glium::texture::TextureHandle::new(&texture, &Default::default()),
    some_value: 5.0,
});
# }
# }
```

Inside your shader, you can refer to the texture with a traditional `sampler*` variable. Glium
currently doesn't check whether the type of your texture matches the expected type (but it may
do in the future). Binding the wrong type of texture may lead to undefined values when sampling
the texture.

*/
use crate::texture::any::TextureAny;
use crate::TextureExt;
use crate::GlObject;

use crate::ContextExt;
use crate::gl;

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use crate::program::BlockLayout;
use crate::uniforms::AsUniformValue;
use crate::uniforms::LayoutMismatchError;
use crate::uniforms::UniformBlock;
use crate::uniforms::UniformValue;
use crate::uniforms::UniformType;
use crate::uniforms::SamplerBehavior;

/// A texture that is resident in video memory. This allows you to use bindless textures in your
/// shaders.
pub struct ResidentTexture {
    texture: Option<TextureAny>,
    handle: gl::types::GLuint64,
}

impl ResidentTexture {
    /// Takes ownership of the given texture and makes it resident.
    // TODO: sampler
    pub fn new(texture: TextureAny) -> Result<ResidentTexture, BindlessTexturesNotSupportedError> {
        let handle = {
            let mut ctxt = texture.get_context().make_current();

            if !ctxt.extensions.gl_arb_bindless_texture {
                return Err(BindlessTexturesNotSupportedError);
            }

            let handle = unsafe { ctxt.gl.GetTextureHandleARB(texture.get_id()) };
            unsafe { ctxt.gl.MakeTextureHandleResidentARB(handle) };
            ctxt.resident_texture_handles.push(handle);
            handle
        };

        // store the handle in the context
        Ok(ResidentTexture {
            texture: Some(texture),
            handle,
        })
    }

    /// Unwraps the texture and restores it.
    #[inline]
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

    #[inline]
    fn deref(&self) -> &TextureAny {
        self.texture.as_ref().unwrap()
    }
}

impl DerefMut for ResidentTexture {
    #[inline]
    fn deref_mut(&mut self) -> &mut TextureAny {
        self.texture.as_mut().unwrap()
    }
}

impl Drop for ResidentTexture {
    #[inline]
    fn drop(&mut self) {
        self.into_inner_impl();
    }
}

/// Represents a handle to a texture. Contains a raw pointer to a texture that is hidden from you.
#[derive(Copy, Clone)]
pub struct TextureHandle<'a> {
    value: gl::types::GLuint64,
    marker: PhantomData<&'a ResidentTexture>,
}

impl<'a> TextureHandle<'a> {
    /// Builds a new handle.
    #[inline]
    pub fn new(texture: &'a ResidentTexture, _: &SamplerBehavior) -> TextureHandle<'a> {
        // FIXME: take sampler into account
        TextureHandle {
            value: texture.handle,
            marker: PhantomData,
        }
    }

    /// Sets the value to the given texture.
    #[inline]
    pub fn set(&mut self, texture: &'a ResidentTexture, _: &SamplerBehavior) {
        // FIXME: take sampler into account
        self.value = texture.handle;
    }
}

impl<'a> AsUniformValue for TextureHandle<'a> {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue<'_> {
        // TODO: u64
        unimplemented!();
    }
}

impl<'a> UniformBlock for TextureHandle<'a> {
    fn matches(layout: &BlockLayout, base_offset: usize)
               -> Result<(), LayoutMismatchError>
    {
        if let BlockLayout::BasicType { ty, offset_in_buffer } = *layout {
            // TODO: unfortunately we have no idea what the exact type of this handle is
            //       strong typing should be considered
            //
            //       however there is no safety problem here ; the worse that can happen in case of
            //       wrong type is zeroes or undefined data being returned when sampling
            match ty {
                UniformType::Sampler1d => (),
                UniformType::ISampler1d => (),
                UniformType::USampler1d => (),
                UniformType::Sampler2d => (),
                UniformType::ISampler2d => (),
                UniformType::USampler2d => (),
                UniformType::Sampler3d => (),
                UniformType::ISampler3d => (),
                UniformType::USampler3d => (),
                UniformType::Sampler1dArray => (),
                UniformType::ISampler1dArray => (),
                UniformType::USampler1dArray => (),
                UniformType::Sampler2dArray => (),
                UniformType::ISampler2dArray => (),
                UniformType::USampler2dArray => (),
                UniformType::SamplerCube => (),
                UniformType::ISamplerCube => (),
                UniformType::USamplerCube => (),
                UniformType::Sampler2dRect => (),
                UniformType::ISampler2dRect => (),
                UniformType::USampler2dRect => (),
                UniformType::Sampler2dRectShadow => (),
                UniformType::SamplerCubeArray => (),
                UniformType::ISamplerCubeArray => (),
                UniformType::USamplerCubeArray => (),
                UniformType::SamplerBuffer => (),
                UniformType::ISamplerBuffer => (),
                UniformType::USamplerBuffer => (),
                UniformType::Sampler2dMultisample => (),
                UniformType::ISampler2dMultisample => (),
                UniformType::USampler2dMultisample => (),
                UniformType::Sampler2dMultisampleArray => (),
                UniformType::ISampler2dMultisampleArray => (),
                UniformType::USampler2dMultisampleArray => (),
                UniformType::Sampler1dShadow => (),
                UniformType::Sampler2dShadow => (),
                UniformType::SamplerCubeShadow => (),
                UniformType::Sampler1dArrayShadow => (),
                UniformType::Sampler2dArrayShadow => (),
                UniformType::SamplerCubeArrayShadow => (),

                _ => return Err(LayoutMismatchError::TypeMismatch {
                    expected: ty,
                    obtained: UniformType::Sampler2d,       // TODO: wrong
                })
            }

            if offset_in_buffer != base_offset {
                return Err(LayoutMismatchError::OffsetMismatch {
                    expected: offset_in_buffer,
                    obtained: base_offset,
                });
            }

            Ok(())

        } else if let BlockLayout::Struct { members } = layout {
            if members.len() == 1 {
                <TextureHandle<'_> as UniformBlock>::matches(&members[0].1, base_offset)

            } else {
                Err(LayoutMismatchError::LayoutMismatch {
                    expected: layout.clone(),
                    obtained: BlockLayout::BasicType {
                        ty: UniformType::Sampler2d,       // TODO: wrong
                        offset_in_buffer: base_offset,
                    }
                })
            }

        } else {
            Err(LayoutMismatchError::LayoutMismatch {
                expected: layout.clone(),
                obtained: BlockLayout::BasicType {
                    ty: UniformType::Sampler2d,       // TODO: wrong
                    offset_in_buffer: base_offset,
                }
            })
        }
    }

    #[inline]
    fn build_layout(base_offset: usize) -> BlockLayout {
        BlockLayout::BasicType {
            ty: UniformType::Sampler2d,       // TODO: wrong
            offset_in_buffer: base_offset,
        }
    }
}

// TODO: implement `vertex::Attribute` on `TextureHandle`

/// Bindless textures are not supported.
#[derive(Debug, Copy, Clone)]
pub struct BindlessTexturesNotSupportedError;

#[cfg(test)]
mod test {
    use std::mem;
    use super::TextureHandle;

    #[test]
    fn texture_handle_size() {
        assert_eq!(mem::size_of::<TextureHandle<'_>>(), 8);
    }
}
