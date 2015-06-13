use program;
use program::BlockLayout;
use texture;
use uniforms::UniformBlock;
use uniforms::SamplerBehavior;

use buffer::BufferViewAnySlice;

use std::mem;

#[cfg(feature = "cgmath")]
use cgmath;
#[cfg(feature = "nalgebra")]
use nalgebra;

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

/// Value that can be used as the value of a uniform.
pub trait AsUniformValue {
    /// Builds a `UniformValue`.
    fn as_uniform_value(&self) -> UniformValue;

    /// If this value is used in a buffer, returns true if it matches the specified type.
    fn matches(&UniformType) -> bool;
}

/// Represents a value to bind to a uniform.
#[allow(missing_docs)]
#[derive(Copy)]
pub enum UniformValue<'a> {
    /// Contains a handle to the buffer, and a function that indicates whether this buffer
    /// can be binded on a block with the given layout.
    /// The last parameter is a sender which must be used to send a `SyncFence` that expires when
    /// the buffer has finished being used.
    Block(BufferViewAnySlice<'a>, fn(&program::UniformBlock) -> bool),
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
}

impl<'a> Clone for UniformValue<'a> {
    fn clone(&self) -> UniformValue<'a> {
        *self
    }
}

impl<'a> UniformValue<'a> {
    /// Returns true if this value can be used with a uniform of the given type.
    pub fn is_usable_with(&self, ty: &UniformType) -> bool {
        match (self, *ty) {
            (&UniformValue::SignedInt(_), UniformType::Int) => true,
            (&UniformValue::UnsignedInt(_), UniformType::UnsignedInt) => true,
            (&UniformValue::Float(_), UniformType::Float) => true,
            (&UniformValue::Mat2(_), UniformType::FloatMat2) => true,
            (&UniformValue::Mat3(_), UniformType::FloatMat3) => true,
            (&UniformValue::Mat4(_), UniformType::FloatMat4) => true,
            (&UniformValue::Vec2(_), UniformType::FloatVec2) => true,
            (&UniformValue::Vec3(_), UniformType::FloatVec3) => true,
            (&UniformValue::Vec4(_), UniformType::FloatVec4) => true,
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
            _ => false,
        }
    }
}

impl<T> UniformBlock for T where T: AsUniformValue + Copy + Send + 'static {
    fn matches(block: &program::UniformBlock) -> bool {
        fn inner_match<T>(layout: &BlockLayout) -> bool where T: AsUniformValue + Copy +
                                                                 Send + 'static
        {
            if let &BlockLayout::BasicType { ty, offset_in_buffer } = layout {
                offset_in_buffer == 0 && <T as AsUniformValue>::matches(&ty)

            } else if let &BlockLayout::Struct { ref members } = layout {
                if members.len() == 1 {
                    inner_match::<T>(&members[0].1)
                } else {
                    false
                }

            } else {
                false
            }
        }

        block.size == mem::size_of::<T>() && inner_match::<T>(&block.layout)
    }
}

impl AsUniformValue for i8 {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::SignedInt(*self as i32)
    }

    fn matches(_: &UniformType) -> bool {
        false
    }
}

impl AsUniformValue for u8 {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::UnsignedInt(*self as u32)
    }

    fn matches(_: &UniformType) -> bool {
        false
    }
}

impl AsUniformValue for i16 {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::SignedInt(*self as i32)
    }

    fn matches(_: &UniformType) -> bool {
        false
    }
}

impl AsUniformValue for u16 {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::UnsignedInt(*self as u32)
    }

    fn matches(_: &UniformType) -> bool {
        false
    }
}

impl AsUniformValue for i32 {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::SignedInt(*self as i32)
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::Int
    }
}

impl AsUniformValue for u32 {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::UnsignedInt(*self as u32)
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::UnsignedInt
    }
}

impl AsUniformValue for f32 {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Float(*self)
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::Float
    }
}

impl AsUniformValue for [[f32; 2]; 2] {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Mat2(*self)
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatMat2
    }
}

impl AsUniformValue for [[f32; 3]; 3] {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Mat3(*self)
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatMat3
    }
}

impl AsUniformValue for [[f32; 4]; 4] {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Mat4(*self)
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatMat4
    }
}

impl AsUniformValue for (f32, f32) {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vec2([self.0, self.1])
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatVec2
    }
}

impl AsUniformValue for (f32, f32, f32) {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vec3([self.0, self.1, self.2])
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatVec3
    }
}

impl AsUniformValue for (f32, f32, f32, f32) {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vec4([self.0, self.1, self.2, self.3])
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatVec4
    }
}

impl AsUniformValue for [f32; 2] {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vec2(*self)
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatVec2
    }
}

impl AsUniformValue for [f32; 3] {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vec3(*self)
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatVec3
    }
}

impl AsUniformValue for [f32; 4] {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vec4(*self)
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatVec4
    }
}

#[cfg(feature = "nalgebra")]
impl AsUniformValue for nalgebra::Mat2<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        let my_value = self.as_array();
        my_value.as_uniform_value()
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatMat2
    }
}

#[cfg(feature = "nalgebra")]
impl AsUniformValue for nalgebra::Mat3<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        let my_value = self.as_array();
        my_value.as_uniform_value()
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatMat3
    }
}

#[cfg(feature = "nalgebra")]
impl AsUniformValue for nalgebra::Mat4<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        let my_value = self.as_array();
        my_value.as_uniform_value()
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatMat4
    }
}

#[cfg(feature = "nalgebra")]
impl AsUniformValue for nalgebra::Ortho3<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        let my_value = self.to_mat(); // Bind to a Mat4
        UniformValue::Mat4(*my_value.as_array())
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatMat4
    }
}

#[cfg(feature = "nalgebra")]
impl AsUniformValue for nalgebra::OrthoMat3<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        let my_value = self.as_mat(); // Bind to a Mat4
        my_value.as_uniform_value()
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatMat4
    }
}

#[cfg(feature = "nalgebra")]
impl AsUniformValue for nalgebra::Persp3<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        let my_value = self.to_mat(); // Bind to a Mat4
        UniformValue::Mat4(*my_value.as_array())
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatMat4
    }
}

#[cfg(feature = "nalgebra")]
impl AsUniformValue for nalgebra::PerspMat3<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        let my_value = self.as_mat(); // Bind to a Mat4
        my_value.as_uniform_value()
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatMat4
    }
}

#[cfg(feature = "nalgebra")]
impl AsUniformValue for nalgebra::Pnt2<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        let my_value = self.as_array();
        my_value.as_uniform_value()
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatVec2
    }
}

#[cfg(feature = "nalgebra")]
impl AsUniformValue for nalgebra::Pnt3<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        let my_value = self.as_array();
        my_value.as_uniform_value()
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatVec3
    }
}

#[cfg(feature = "nalgebra")]
impl AsUniformValue for nalgebra::Pnt4<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        let my_value = self.as_array();
        my_value.as_uniform_value()
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatVec4
    }
}

#[cfg(feature = "nalgebra")]
impl AsUniformValue for nalgebra::Quat<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        let my_value = self.as_array();
        my_value.as_uniform_value()
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatVec4
    }
}

#[cfg(feature = "nalgebra")]
impl AsUniformValue for nalgebra::Rot2<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        let my_value = self.submat(); // Bind to a Mat2
        UniformValue::Mat2(*my_value.as_array())
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatMat2
    }
}

#[cfg(feature = "nalgebra")]
impl AsUniformValue for nalgebra::Rot3<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        let my_value = self.submat(); // Bind to a Mat3
        UniformValue::Mat3(*my_value.as_array())
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatMat3
    }
}

#[cfg(feature = "nalgebra")]
impl AsUniformValue for nalgebra::Rot4<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        let my_value = self.submat(); // Bind to a Mat4
        UniformValue::Mat4(*my_value.as_array())
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatMat4
    }
}

#[cfg(feature = "nalgebra")]
impl AsUniformValue for nalgebra::UnitQuat<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        let my_value = self.quat(); // Bind to a Quat
        UniformValue::Vec4(*my_value.as_array())
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatVec4
    }
}

#[cfg(feature = "nalgebra")]
impl AsUniformValue for nalgebra::Vec2<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        let my_value = self.as_array();
        my_value.as_uniform_value()
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatVec2
    }
}

#[cfg(feature = "nalgebra")]
impl AsUniformValue for nalgebra::Vec3<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        let my_value = self.as_array();
        my_value.as_uniform_value()
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatVec3
    }
}

#[cfg(feature = "nalgebra")]
impl AsUniformValue for nalgebra::Vec4<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        let my_value = self.as_array();
        my_value.as_uniform_value()
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatVec4
    }
}

#[cfg(feature = "cgmath")]
impl AsUniformValue for cgmath::Matrix2<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        UniformValue::Mat2(my_value)
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatMat2
    }
}

#[cfg(feature = "cgmath")]
impl AsUniformValue for cgmath::Matrix3<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        UniformValue::Mat3(my_value)
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatMat3
    }
}

#[cfg(feature = "cgmath")]
impl AsUniformValue for cgmath::Matrix4<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        UniformValue::Mat4(my_value)
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatMat4
    }
}

#[cfg(feature = "cgmath")]
impl AsUniformValue for cgmath::Vector2<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        UniformValue::Vec2(my_value)
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatVec2
    }
}

#[cfg(feature = "cgmath")]
impl AsUniformValue for cgmath::Vector3<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        UniformValue::Vec3(my_value)
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatVec3
    }
}

#[cfg(feature = "cgmath")]
impl AsUniformValue for cgmath::Vector4<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        UniformValue::Vec4(my_value)
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatVec4
    }
}

#[cfg(feature = "cgmath")]
impl AsUniformValue for cgmath::Point2<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        UniformValue::Vec2(my_value)
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatVec2
    }
}

#[cfg(feature = "cgmath")]
impl AsUniformValue for cgmath::Point3<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        UniformValue::Vec3(my_value)
    }

    fn matches(ty: &UniformType) -> bool {
        ty == &UniformType::FloatVec3
    }
}
