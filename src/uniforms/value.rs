use program;
use program::BlockLayout;
use program::ShaderStage;
use texture;

use uniforms::AsUniformValue;
use uniforms::LayoutMismatchError;
use uniforms::UniformBlock;
use uniforms::SamplerBehavior;

use buffer::BufferAnySlice;

/// Type of a uniform in a program.
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UniformType {
    Float,
    FloatVec2,
    FloatVec3,
    FloatVec4,
    Double,
    DoubleVec2,
    DoubleVec3,
    DoubleVec4,
    Int,
    IntVec2,
    IntVec3,
    IntVec4,
    UnsignedInt,
    UnsignedIntVec2,
    UnsignedIntVec3,
    UnsignedIntVec4,
    Int64,
    Int64Vec2,
    Int64Vec3,
    Int64Vec4,
    UnsignedInt64,
    UnsignedInt64Vec2,
    UnsignedInt64Vec3,
    UnsignedInt64Vec4,
    Bool,
    BoolVec2,
    BoolVec3,
    BoolVec4,
    FloatMat2,
    FloatMat3,
    FloatMat4,
    FloatMat2x3,
    FloatMat2x4,
    FloatMat3x2,
    FloatMat3x4,
    FloatMat4x2,
    FloatMat4x3,
    DoubleMat2,
    DoubleMat3,
    DoubleMat4,
    DoubleMat2x3,
    DoubleMat2x4,
    DoubleMat3x2,
    DoubleMat3x4,
    DoubleMat4x2,
    DoubleMat4x3,
    Sampler1d,
    ISampler1d,
    USampler1d,
    Sampler2d,
    ISampler2d,
    USampler2d,
    Sampler3d,
    ISampler3d,
    USampler3d,
    Sampler1dArray,
    ISampler1dArray,
    USampler1dArray,
    Sampler2dArray,
    ISampler2dArray,
    USampler2dArray,
    SamplerCube,
    ISamplerCube,
    USamplerCube,
    Sampler2dRect,
    ISampler2dRect,
    USampler2dRect,
    Sampler2dRectShadow,
    SamplerCubeArray,
    ISamplerCubeArray,
    USamplerCubeArray,
    SamplerBuffer,
    ISamplerBuffer,
    USamplerBuffer,
    Sampler2dMultisample,
    ISampler2dMultisample,
    USampler2dMultisample,
    Sampler2dMultisampleArray,
    ISampler2dMultisampleArray,
    USampler2dMultisampleArray,
    Sampler1dShadow,
    Sampler2dShadow,
    SamplerCubeShadow,
    Sampler1dArrayShadow,
    Sampler2dArrayShadow,
    SamplerCubeArrayShadow,
    Image1d,
    IImage1d,
    UImage1d,
    Image2d,
    IImage2d,
    UImage2d,
    Image3d,
    IImage3d,
    UImage3d,
    Image2dRect,
    IImage2dRect,
    UImage2dRect,
    ImageCube,
    IImageCube,
    UImageCube,
    ImageBuffer,
    IImageBuffer,
    UImageBuffer,
    Image1dArray,
    IImage1dArray,
    UImage1dArray,
    Image2dArray,
    IImage2dArray,
    UImage2dArray,
    Image2dMultisample,
    IImage2dMultisample,
    UImage2dMultisample,
    Image2dMultisampleArray,
    IImage2dMultisampleArray,
    UImage2dMultisampleArray,
    AtomicCounterUint,
}

/// Represents a value to bind to a uniform.
#[allow(missing_docs)]
#[derive(Copy)]
pub enum UniformValue<'a> {
    /// Contains a handle to the buffer, and a function that indicates whether this buffer
    /// can be binded on a block with the given layout.
    /// The last parameter is a sender which must be used to send a `SyncFence` that expires when
    /// the buffer has finished being used.
    Block(BufferAnySlice<'a>, fn(&program::UniformBlock) -> Result<(), LayoutMismatchError>),
    Subroutine(ShaderStage, &'a str),
    SignedInt(i32),
    UnsignedInt(u32),
    Float(f32),
    /// 2x2 column-major matrix.
    Mat2([[f32; 2]; 2]),
    /// 3x3 column-major matrix.
    Mat3([[f32; 3]; 3]),
    /// 4x4 column-major matrix.
    Mat4([[f32; 4]; 4]),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    IntVec2([i32; 2]),
    IntVec3([i32; 3]),
    IntVec4([i32; 4]),
    UnsignedIntVec2([u32; 2]),
    UnsignedIntVec3([u32; 3]),
    UnsignedIntVec4([u32; 4]),
    Bool(bool),
    BoolVec2([bool; 2]),
    BoolVec3([bool; 3]),
    BoolVec4([bool; 4]),
    Double(f64),
    DoubleVec2([f64; 2]),
    DoubleVec3([f64; 3]),
    DoubleVec4([f64; 4]),
    DoubleMat2([[f64;2]; 2]),
    DoubleMat3([[f64;3]; 3]),
    DoubleMat4([[f64;4]; 4]),
    Int64(i64),
    Int64Vec2([i64; 2]),
    Int64Vec3([i64; 3]),
    Int64Vec4([i64; 4]),
    UnsignedInt64(u64),
    UnsignedInt64Vec2([u64; 2]),
    UnsignedInt64Vec3([u64; 3]),
    UnsignedInt64Vec4([u64; 4]),
    Texture1d(&'a texture::Texture1d, Option<SamplerBehavior>),
    CompressedTexture1d(&'a texture::CompressedTexture1d, Option<SamplerBehavior>),
    SrgbTexture1d(&'a texture::SrgbTexture1d, Option<SamplerBehavior>),
    CompressedSrgbTexture1d(&'a texture::CompressedSrgbTexture1d, Option<SamplerBehavior>),
    IntegralTexture1d(&'a texture::IntegralTexture1d, Option<SamplerBehavior>),
    UnsignedTexture1d(&'a texture::UnsignedTexture1d, Option<SamplerBehavior>),
    DepthTexture1d(&'a texture::DepthTexture1d, Option<SamplerBehavior>),
    Texture2d(&'a texture::Texture2d, Option<SamplerBehavior>),
    CompressedTexture2d(&'a texture::CompressedTexture2d, Option<SamplerBehavior>),
    SrgbTexture2d(&'a texture::SrgbTexture2d, Option<SamplerBehavior>),
    CompressedSrgbTexture2d(&'a texture::CompressedSrgbTexture2d, Option<SamplerBehavior>),
    IntegralTexture2d(&'a texture::IntegralTexture2d, Option<SamplerBehavior>),
    UnsignedTexture2d(&'a texture::UnsignedTexture2d, Option<SamplerBehavior>),
    DepthTexture2d(&'a texture::DepthTexture2d, Option<SamplerBehavior>),
    Texture2dMultisample(&'a texture::Texture2dMultisample, Option<SamplerBehavior>),
    SrgbTexture2dMultisample(&'a texture::SrgbTexture2dMultisample, Option<SamplerBehavior>),
    IntegralTexture2dMultisample(&'a texture::IntegralTexture2dMultisample, Option<SamplerBehavior>),
    UnsignedTexture2dMultisample(&'a texture::UnsignedTexture2dMultisample, Option<SamplerBehavior>),
    DepthTexture2dMultisample(&'a texture::DepthTexture2dMultisample, Option<SamplerBehavior>),
    Texture3d(&'a texture::Texture3d, Option<SamplerBehavior>),
    CompressedTexture3d(&'a texture::CompressedTexture3d, Option<SamplerBehavior>),
    SrgbTexture3d(&'a texture::SrgbTexture3d, Option<SamplerBehavior>),
    CompressedSrgbTexture3d(&'a texture::CompressedSrgbTexture3d, Option<SamplerBehavior>),
    IntegralTexture3d(&'a texture::IntegralTexture3d, Option<SamplerBehavior>),
    UnsignedTexture3d(&'a texture::UnsignedTexture3d, Option<SamplerBehavior>),
    DepthTexture3d(&'a texture::DepthTexture3d, Option<SamplerBehavior>),
    Texture1dArray(&'a texture::Texture1dArray, Option<SamplerBehavior>),
    CompressedTexture1dArray(&'a texture::CompressedTexture1dArray, Option<SamplerBehavior>),
    SrgbTexture1dArray(&'a texture::SrgbTexture1dArray, Option<SamplerBehavior>),
    CompressedSrgbTexture1dArray(&'a texture::CompressedSrgbTexture1dArray, Option<SamplerBehavior>),
    IntegralTexture1dArray(&'a texture::IntegralTexture1dArray, Option<SamplerBehavior>),
    UnsignedTexture1dArray(&'a texture::UnsignedTexture1dArray, Option<SamplerBehavior>),
    DepthTexture1dArray(&'a texture::DepthTexture1dArray, Option<SamplerBehavior>),
    Texture2dArray(&'a texture::Texture2dArray, Option<SamplerBehavior>),
    CompressedTexture2dArray(&'a texture::CompressedTexture2dArray, Option<SamplerBehavior>),
    SrgbTexture2dArray(&'a texture::SrgbTexture2dArray, Option<SamplerBehavior>),
    CompressedSrgbTexture2dArray(&'a texture::CompressedSrgbTexture2dArray, Option<SamplerBehavior>),
    IntegralTexture2dArray(&'a texture::IntegralTexture2dArray, Option<SamplerBehavior>),
    UnsignedTexture2dArray(&'a texture::UnsignedTexture2dArray, Option<SamplerBehavior>),
    DepthTexture2dArray(&'a texture::DepthTexture2dArray, Option<SamplerBehavior>),
    Texture2dMultisampleArray(&'a texture::Texture2dMultisampleArray, Option<SamplerBehavior>),
    SrgbTexture2dMultisampleArray(&'a texture::SrgbTexture2dMultisampleArray, Option<SamplerBehavior>),
    IntegralTexture2dMultisampleArray(&'a texture::IntegralTexture2dMultisampleArray, Option<SamplerBehavior>),
    UnsignedTexture2dMultisampleArray(&'a texture::UnsignedTexture2dMultisampleArray, Option<SamplerBehavior>),
    DepthTexture2dMultisampleArray(&'a texture::DepthTexture2dMultisampleArray, Option<SamplerBehavior>),
    Cubemap(&'a texture::Cubemap, Option<SamplerBehavior>),
    CompressedCubemap(&'a texture::CompressedCubemap, Option<SamplerBehavior>),
    SrgbCubemap(&'a texture::SrgbCubemap, Option<SamplerBehavior>),
    CompressedSrgbCubemap(&'a texture::CompressedSrgbCubemap, Option<SamplerBehavior>),
    IntegralCubemap(&'a texture::IntegralCubemap, Option<SamplerBehavior>),
    UnsignedCubemap(&'a texture::UnsignedCubemap, Option<SamplerBehavior>),
    DepthCubemap(&'a texture::DepthCubemap, Option<SamplerBehavior>),
    CubemapArray(&'a texture::CubemapArray, Option<SamplerBehavior>),
    CompressedCubemapArray(&'a texture::CompressedCubemapArray, Option<SamplerBehavior>),
    SrgbCubemapArray(&'a texture::SrgbCubemapArray, Option<SamplerBehavior>),
    CompressedSrgbCubemapArray(&'a texture::CompressedSrgbCubemapArray, Option<SamplerBehavior>),
    IntegralCubemapArray(&'a texture::IntegralCubemapArray, Option<SamplerBehavior>),
    UnsignedCubemapArray(&'a texture::UnsignedCubemapArray, Option<SamplerBehavior>),
    DepthCubemapArray(&'a texture::DepthCubemapArray, Option<SamplerBehavior>),
    BufferTexture(texture::buffer_texture::BufferTextureRef<'a>),
}

impl<'a> Clone for UniformValue<'a> {
    #[inline]
    fn clone(&self) -> UniformValue<'a> {
        *self
    }
}

impl<'a> UniformValue<'a> {
    /// Returns true if this value can be used with a uniform of the given type.
    pub fn is_usable_with(&self, ty: &UniformType) -> bool {
        match (self, *ty) {
            (&UniformValue::Bool(_), UniformType::Bool) => true,
            (&UniformValue::SignedInt(_), UniformType::Int) => true,
            (&UniformValue::UnsignedInt(_), UniformType::UnsignedInt) => true,
            (&UniformValue::Float(_), UniformType::Float) => true,
            (&UniformValue::Mat2(_), UniformType::FloatMat2) => true,
            (&UniformValue::Mat3(_), UniformType::FloatMat3) => true,
            (&UniformValue::Mat4(_), UniformType::FloatMat4) => true,
            (&UniformValue::Vec2(_), UniformType::FloatVec2) => true,
            (&UniformValue::Vec3(_), UniformType::FloatVec3) => true,
            (&UniformValue::Vec4(_), UniformType::FloatVec4) => true,
            (&UniformValue::IntVec2(_), UniformType::IntVec2) => true,
            (&UniformValue::IntVec3(_), UniformType::IntVec3) => true,
            (&UniformValue::IntVec4(_), UniformType::IntVec4) => true,
            (&UniformValue::UnsignedIntVec2(_), UniformType::UnsignedIntVec2) => true,
            (&UniformValue::UnsignedIntVec3(_), UniformType::UnsignedIntVec3) => true,
            (&UniformValue::UnsignedIntVec4(_), UniformType::UnsignedIntVec4) => true,
            (&UniformValue::BoolVec2(_), UniformType::BoolVec2) => true,
            (&UniformValue::BoolVec3(_), UniformType::BoolVec3) => true,
            (&UniformValue::BoolVec4(_), UniformType::BoolVec4) => true,
            (&UniformValue::Double(_), UniformType::Double) => true,
            (&UniformValue::DoubleMat2(_), UniformType::DoubleMat2) => true,
            (&UniformValue::DoubleMat3(_), UniformType::DoubleMat3) => true,
            (&UniformValue::DoubleMat4(_), UniformType::DoubleMat4) => true,
            (&UniformValue::DoubleVec2(_), UniformType::DoubleVec2) => true,
            (&UniformValue::DoubleVec3(_), UniformType::DoubleVec3) => true,
            (&UniformValue::DoubleVec4(_), UniformType::DoubleVec4) => true,
            (&UniformValue::Texture1d(_, _), UniformType::Sampler1d) => true,
            (&UniformValue::CompressedTexture1d(_, _), UniformType::Sampler1d) => true,
            (&UniformValue::SrgbTexture1d(_, _), UniformType::Sampler1d) => true,
            (&UniformValue::CompressedSrgbTexture1d(_, _), UniformType::Sampler1d) => true,
            (&UniformValue::IntegralTexture1d(_, _), UniformType::ISampler1d) => true,
            (&UniformValue::UnsignedTexture1d(_, _), UniformType::USampler1d) => true,
            (&UniformValue::DepthTexture1d(_, _), UniformType::Sampler1d) => true,
            (&UniformValue::Texture2d(_, _), UniformType::Sampler2d) => true,
            (&UniformValue::CompressedTexture2d(_, _), UniformType::Sampler2d) => true,
            (&UniformValue::SrgbTexture2d(_, _), UniformType::Sampler2d) => true,
            (&UniformValue::CompressedSrgbTexture2d(_, _), UniformType::Sampler2d) => true,
            (&UniformValue::IntegralTexture2d(_, _), UniformType::ISampler2d) => true,
            (&UniformValue::UnsignedTexture2d(_, _), UniformType::USampler2d) => true,
            (&UniformValue::DepthTexture2d(_, _), UniformType::Sampler2d) => true,
            (&UniformValue::Texture3d(_, _), UniformType::Sampler3d) => true,
            (&UniformValue::CompressedTexture3d(_, _), UniformType::Sampler3d) => true,
            (&UniformValue::SrgbTexture3d(_, _), UniformType::Sampler3d) => true,
            (&UniformValue::CompressedSrgbTexture3d(_, _), UniformType::Sampler3d) => true,
            (&UniformValue::IntegralTexture3d(_, _), UniformType::ISampler3d) => true,
            (&UniformValue::UnsignedTexture3d(_, _), UniformType::USampler3d) => true,
            (&UniformValue::DepthTexture3d(_, _), UniformType::Sampler3d) => true,
            (&UniformValue::Texture1dArray(_, _), UniformType::Sampler1dArray) => true,
            (&UniformValue::CompressedTexture1dArray(_, _), UniformType::Sampler1dArray) => true,
            (&UniformValue::SrgbTexture1dArray(_, _), UniformType::Sampler1dArray) => true,
            (&UniformValue::CompressedSrgbTexture1dArray(_, _), UniformType::Sampler1dArray) => true,
            (&UniformValue::IntegralTexture1dArray(_, _), UniformType::ISampler1dArray) => true,
            (&UniformValue::UnsignedTexture1dArray(_, _), UniformType::USampler1dArray) => true,
            (&UniformValue::DepthTexture1dArray(_, _), UniformType::Sampler1dArray) => true,
            (&UniformValue::Texture2dArray(_, _), UniformType::Sampler2dArray) => true,
            (&UniformValue::CompressedTexture2dArray(_, _), UniformType::Sampler2dArray) => true,
            (&UniformValue::SrgbTexture2dArray(_, _), UniformType::Sampler2dArray) => true,
            (&UniformValue::CompressedSrgbTexture2dArray(_, _), UniformType::Sampler2dArray) => true,
            (&UniformValue::IntegralTexture2dArray(_, _), UniformType::ISampler2dArray) => true,
            (&UniformValue::UnsignedTexture2dArray(_, _), UniformType::USampler2dArray) => true,
            (&UniformValue::DepthTexture2dArray(_, _), UniformType::Sampler2dArray) => true,
            (&UniformValue::Cubemap(_, _), UniformType::SamplerCube) => true,
            (&UniformValue::CompressedCubemap(_, _), UniformType::SamplerCube) => true,
            (&UniformValue::SrgbCubemap(_, _), UniformType::SamplerCube) => true,
            (&UniformValue::CompressedSrgbCubemap(_, _), UniformType::SamplerCube) => true,
            (&UniformValue::IntegralCubemap(_, _), UniformType::ISamplerCube) => true,
            (&UniformValue::UnsignedCubemap(_, _), UniformType::USamplerCube) => true,
            (&UniformValue::DepthCubemap(_, _), UniformType::SamplerCube) => true,
            (&UniformValue::CubemapArray(_, _), UniformType::SamplerCubeArray) => true,
            (&UniformValue::CompressedCubemapArray(_, _), UniformType::SamplerCubeArray) => true,
            (&UniformValue::SrgbCubemapArray(_, _), UniformType::SamplerCubeArray) => true,
            (&UniformValue::CompressedSrgbCubemapArray(_, _), UniformType::SamplerCubeArray) => true,
            (&UniformValue::IntegralCubemapArray(_, _), UniformType::ISamplerCubeArray) => true,
            (&UniformValue::UnsignedCubemapArray(_, _), UniformType::USamplerCubeArray) => true,
            (&UniformValue::DepthCubemapArray(_, _), UniformType::SamplerCubeArray) => true,
            (&UniformValue::BufferTexture(tex), UniformType::SamplerBuffer) => {
                tex.get_texture_type() == texture::buffer_texture::BufferTextureType::Float
            },
            (&UniformValue::BufferTexture(tex), UniformType::ISamplerBuffer) => {
                tex.get_texture_type() == texture::buffer_texture::BufferTextureType::Integral
            },
            (&UniformValue::BufferTexture(tex), UniformType::USamplerBuffer) => {
                tex.get_texture_type() == texture::buffer_texture::BufferTextureType::Unsigned
            },
            (&UniformValue::Texture2dMultisample(..), UniformType::Sampler2dMultisample) => true,
            (&UniformValue::SrgbTexture2dMultisample(..), UniformType::Sampler2dMultisample) => true,
            (&UniformValue::IntegralTexture2dMultisample(..), UniformType::ISampler2dMultisample) => true,
            (&UniformValue::UnsignedTexture2dMultisample(..), UniformType::USampler2dMultisample) => true,
            (&UniformValue::DepthTexture2dMultisample(..), UniformType::Sampler2dMultisample) => true,
            _ => false,
        }
    }
}

macro_rules! impl_uniform_block_basic {
    ($ty:ty, $uniform_ty:expr) => (
        impl UniformBlock for $ty {
            fn matches(layout: &program::BlockLayout, base_offset: usize)
                       -> Result<(), LayoutMismatchError>
            {
                if let &BlockLayout::BasicType { ty, offset_in_buffer } = layout {
                    if ty != $uniform_ty {
                        return Err(LayoutMismatchError::TypeMismatch {
                            expected: ty,
                            obtained: $uniform_ty,
                        });
                    }

                    if offset_in_buffer != base_offset {
                        return Err(LayoutMismatchError::OffsetMismatch {
                            expected: offset_in_buffer,
                            obtained: base_offset,
                        });
                    }

                    Ok(())

                } else {
                    Err(LayoutMismatchError::LayoutMismatch {
                        expected: layout.clone(),
                        obtained: BlockLayout::BasicType {
                            ty: $uniform_ty,
                            offset_in_buffer: base_offset,
                        }
                    })
                }
            }

            #[inline]
            fn build_layout(base_offset: usize) -> program::BlockLayout {
                BlockLayout::BasicType {
                    ty: $uniform_ty,
                    offset_in_buffer: base_offset,
                }
            }
        }
    );

    ([$base_ty:expr; $num:expr], $simd_ty:expr) => (
        impl UniformBlock for [$base_ty:expr; $num:expr] {
            fn matches(layout: &program::BlockLayout, base_offset: usize)
                       -> Result<(), LayoutMismatchError>
            {
                if let &BlockLayout::BasicType { ty, offset_in_buffer } = layout {
                    if ty != $simd_ty {
                        return Err(LayoutMismatchError::TypeMismatch {
                            expected: ty,
                            obtained: $simd_ty,
                        });
                    }

                    if offset_in_buffer != base_offset {
                        return Err(LayoutMismatchError::OffsetMismatch {
                            expected: offset_in_buffer,
                            obtained: base_offset,
                        });
                    }

                    Ok(())

                } else if let &BlockLayout::Array { ref content, length } = layout {
                    if length != $num || <$base_ty as UniformBlock>::matches(content).is_err() {
                        return Err(LayoutMismatchError::LayoutMismatch {
                            expected: layout.clone(),
                            obtained: BlockLayout::Array {
                                content: Box::new(<$base_ty as UniformBlock>::build_layout(base_offset)),
                                length: $num,
                            },
                        });
                    }

                    Ok(())

                } else {
                    Err(LayoutMismatchError::LayoutMismatch {
                        expected: layout.clone(),
                        obtained: self.build_layout(base_offset),
                    })
                }
            }

            #[inline]
            fn build_layout(base_offset: usize) -> program::BlockLayout {
                BlockLayout::BasicType {
                    ty: $simd_ty,
                    offset_in_buffer: base_offset,
                }
            }
        }
    )
}

impl AsUniformValue for i8 {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::SignedInt(*self as i32)
    }
}

impl AsUniformValue for u8 {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::UnsignedInt(*self as u32)
    }
}

impl AsUniformValue for i16 {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::SignedInt(*self as i32)
    }
}

impl AsUniformValue for u16 {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::UnsignedInt(*self as u32)
    }
}

impl AsUniformValue for i32 {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::SignedInt(*self as i32)
    }
}

impl_uniform_block_basic!(i32, UniformType::Int);

impl AsUniformValue for [i32; 2] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::IntVec2(*self)
    }
}

impl_uniform_block_basic!([i32; 2], UniformType::IntVec2);

impl AsUniformValue for (i32, i32) {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::IntVec2([self.0, self.1])
    }
}

impl_uniform_block_basic!((i32, i32), UniformType::IntVec2);

impl AsUniformValue for [i32; 3] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::IntVec3(*self)
    }
}

impl_uniform_block_basic!([i32; 3], UniformType::IntVec3);

impl AsUniformValue for (i32, i32, i32) {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::IntVec3([self.0, self.1, self.2])
    }
}

impl_uniform_block_basic!((i32, i32, i32), UniformType::IntVec3);

impl AsUniformValue for [i32; 4] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::IntVec4(*self)
    }
}

impl_uniform_block_basic!([i32; 4], UniformType::IntVec4);

impl AsUniformValue for (i32, i32, i32, i32) {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::IntVec4([self.0, self.1, self.2, self.3])
    }
}

impl_uniform_block_basic!((i32, i32, i32, i32), UniformType::IntVec4);

impl AsUniformValue for u32 {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::UnsignedInt(*self as u32)
    }
}

impl_uniform_block_basic!(u32, UniformType::UnsignedInt);

impl AsUniformValue for [u32; 2] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::UnsignedIntVec2(*self)
    }
}

impl_uniform_block_basic!([u32; 2], UniformType::UnsignedIntVec2);

impl AsUniformValue for (u32, u32) {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::UnsignedIntVec2([self.0, self.1])
    }
}

impl_uniform_block_basic!((u32, u32), UniformType::UnsignedIntVec2);

impl AsUniformValue for [u32; 3] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::UnsignedIntVec3(*self)
    }
}

impl_uniform_block_basic!([u32; 3], UniformType::UnsignedIntVec3);

impl AsUniformValue for (u32, u32, u32) {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::UnsignedIntVec3([self.0, self.1, self.2])
    }
}

impl_uniform_block_basic!((u32, u32, u32), UniformType::UnsignedIntVec3);

impl AsUniformValue for [u32; 4] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::UnsignedIntVec4(*self)
    }
}

impl_uniform_block_basic!([u32; 4], UniformType::UnsignedIntVec4);

impl AsUniformValue for (u32, u32, u32, u32) {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::UnsignedIntVec4([self.0, self.1, self.2, self.3])
    }
}

impl_uniform_block_basic!((u32, u32, u32, u32), UniformType::UnsignedIntVec4);

impl AsUniformValue for bool {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Bool(*self)
    }
}

impl_uniform_block_basic!(bool, UniformType::Bool);

impl AsUniformValue for [bool; 2] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::BoolVec2(*self)
    }
}

impl_uniform_block_basic!([bool; 2], UniformType::BoolVec2);

impl AsUniformValue for (bool, bool) {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::BoolVec2([self.0, self.1])
    }
}

impl_uniform_block_basic!((bool, bool), UniformType::BoolVec2);

impl AsUniformValue for [bool; 3] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::BoolVec3(*self)
    }
}

impl_uniform_block_basic!([bool; 3], UniformType::BoolVec3);

impl AsUniformValue for (bool, bool, bool) {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::BoolVec3([self.0, self.1, self.2])
    }
}

impl_uniform_block_basic!((bool, bool, bool), UniformType::BoolVec3);

impl AsUniformValue for [bool; 4] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::BoolVec4(*self)
    }
}

impl_uniform_block_basic!([bool; 4], UniformType::BoolVec4);

impl AsUniformValue for (bool, bool, bool, bool) {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::BoolVec4([self.0, self.1, self.2, self.3])
    }
}

impl_uniform_block_basic!((bool, bool, bool, bool), UniformType::BoolVec4);

impl AsUniformValue for f32 {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Float(*self)
    }
}

impl_uniform_block_basic!(f32, UniformType::Float);

impl AsUniformValue for [[f32; 2]; 2] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Mat2(*self)
    }
}

impl_uniform_block_basic!([[f32; 2]; 2], UniformType::FloatMat2);

impl AsUniformValue for [[f32; 3]; 3] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Mat3(*self)
    }
}

impl_uniform_block_basic!([[f32; 3]; 3], UniformType::FloatMat3);

impl AsUniformValue for [[f32; 4]; 4] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Mat4(*self)
    }
}

impl_uniform_block_basic!([[f32; 4]; 4], UniformType::FloatMat4);

impl AsUniformValue for (f32, f32) {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vec2([self.0, self.1])
    }
}

impl_uniform_block_basic!((f32, f32), UniformType::FloatVec2);

impl AsUniformValue for (f32, f32, f32) {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vec3([self.0, self.1, self.2])
    }
}

impl_uniform_block_basic!((f32, f32, f32), UniformType::FloatVec3);

impl AsUniformValue for (f32, f32, f32, f32) {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vec4([self.0, self.1, self.2, self.3])
    }
}

impl_uniform_block_basic!((f32, f32, f32, f32), UniformType::FloatVec4);

impl AsUniformValue for [f32; 2] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vec2(*self)
    }
}

impl_uniform_block_basic!([f32; 2], UniformType::FloatVec2);

impl AsUniformValue for [f32; 3] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vec3(*self)
    }
}

impl_uniform_block_basic!([f32; 3], UniformType::FloatVec3);

impl AsUniformValue for [f32; 4] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vec4(*self)
    }
}

impl_uniform_block_basic!([f32; 4], UniformType::FloatVec4);

//TODO bool, i32, u32 and f64 should also be implemented as cgmath and nalgebra variants (i.e. nalgebra::Vec3<f64>).
// Start of double type variants
impl AsUniformValue for f64 {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Double(*self)
    }
}

impl_uniform_block_basic!(f64, UniformType::Double);

impl AsUniformValue for [f64; 2] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::DoubleVec2(*self)
    }
}

impl_uniform_block_basic!([f64; 2], UniformType::DoubleVec2);

impl AsUniformValue for (f64, f64) {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::DoubleVec2([self.0, self.1])
    }
}

impl_uniform_block_basic!((f64, f64), UniformType::DoubleVec2);

impl AsUniformValue for [f64; 3] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::DoubleVec3(*self)
    }
}

impl_uniform_block_basic!([f64; 3], UniformType::DoubleVec3);

impl AsUniformValue for (f64, f64, f64) {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::DoubleVec3([self.0, self.1, self.2])
    }
}

impl_uniform_block_basic!((f64, f64, f64), UniformType::DoubleVec3);

impl AsUniformValue for [f64; 4] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::DoubleVec4(*self)
    }
}

impl_uniform_block_basic!([f64; 4], UniformType::DoubleVec4);

impl AsUniformValue for (f64, f64, f64, f64) {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::DoubleVec4([self.0, self.1, self.2, self.3])
    }
}

impl_uniform_block_basic!((f64, f64, f64, f64), UniformType::DoubleVec4);

impl AsUniformValue for [[f64; 2]; 2] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::DoubleMat2(*self)
    }
}

impl_uniform_block_basic!([[f64; 2]; 2], UniformType::DoubleMat2);

impl AsUniformValue for [[f64; 3]; 3] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::DoubleMat3(*self)
    }
}

impl_uniform_block_basic!([[f64; 3]; 3], UniformType::DoubleMat3);

impl AsUniformValue for [[f64; 4]; 4] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::DoubleMat4(*self)
    }
}

impl_uniform_block_basic!([[f64; 4]; 4], UniformType::DoubleMat4);

impl AsUniformValue for i64 {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Int64(*self as i64)
    }
}

impl_uniform_block_basic!(i64, UniformType::Int);

impl AsUniformValue for [i64; 2] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Int64Vec2(*self)
    }
}

impl_uniform_block_basic!([i64; 2], UniformType::Int64Vec2);

impl AsUniformValue for (i64, i64) {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Int64Vec2([self.0, self.1])
    }
}

impl_uniform_block_basic!((i64, i64), UniformType::Int64Vec2);

impl AsUniformValue for [i64; 3] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Int64Vec3(*self)
    }
}

impl_uniform_block_basic!([i64; 3], UniformType::Int64Vec3);

impl AsUniformValue for (i64, i64, i64) {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Int64Vec3([self.0, self.1, self.2])
    }
}

impl_uniform_block_basic!((i64, i64, i64), UniformType::Int64Vec3);

impl AsUniformValue for [i64; 4] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Int64Vec4(*self)
    }
}

impl_uniform_block_basic!([i64; 4], UniformType::Int64Vec4);

impl AsUniformValue for (i64, i64, i64, i64) {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Int64Vec4([self.0, self.1, self.2, self.3])
    }
}

impl_uniform_block_basic!((i64, i64, i64, i64), UniformType::Int64Vec4);

impl AsUniformValue for u64 {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::UnsignedInt64(*self as u64)
    }
}

impl_uniform_block_basic!(u64, UniformType::UnsignedInt64);

impl AsUniformValue for [u64; 2] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::UnsignedInt64Vec2(*self)
    }
}

impl_uniform_block_basic!([u64; 2], UniformType::UnsignedInt64Vec2);

impl AsUniformValue for (u64, u64) {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::UnsignedInt64Vec2([self.0, self.1])
    }
}

impl_uniform_block_basic!((u64, u64), UniformType::UnsignedInt64Vec2);

impl AsUniformValue for [u64; 3] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::UnsignedInt64Vec3(*self)
    }
}

impl_uniform_block_basic!([u64; 3], UniformType::UnsignedInt64Vec3);

impl AsUniformValue for (u64, u64, u64) {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::UnsignedInt64Vec3([self.0, self.1, self.2])
    }
}

impl_uniform_block_basic!((u64, u64, u64), UniformType::UnsignedInt64Vec3);

impl AsUniformValue for [u64; 4] {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::UnsignedInt64Vec4(*self)
    }
}

impl_uniform_block_basic!([u64; 4], UniformType::UnsignedInt64Vec4);

impl AsUniformValue for (u64, u64, u64, u64) {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::UnsignedInt64Vec4([self.0, self.1, self.2, self.3])
    }
}

impl_uniform_block_basic!((u64, u64, u64, u64), UniformType::UnsignedInt64Vec4);

// Subroutines
impl<'a> AsUniformValue for (&'a str, ShaderStage) {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Subroutine(self.1, self.0)
    }
}
