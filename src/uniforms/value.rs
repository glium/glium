use program;
use texture;
use sync::LinearSyncFence;
use uniforms::UniformBlock;
use uniforms::SamplerBehavior;
use uniforms::buffer::TypelessUniformBuffer;

use std::default::Default;
use std::sync::mpsc::Sender;

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

/// Represents a value that can be used as the value of a uniform.
pub trait IntoUniformValue<'a> {
    /// Builds a `UniformValue`.
    fn into_uniform_value(self) -> UniformValue<'a>;
}

/// Represents a value to bind to a uniform.
#[allow(missing_docs)]
pub enum UniformValue<'a> {
    /// Contains a handle to the buffer, and a function that indicates whether this buffer
    /// can be binded on a block with the given layout.
    /// The last parameter is a sender which must be used to send a `SyncFence` that expires when
    /// the buffer has finished being used.
    Block(&'a TypelessUniformBuffer, Box<Fn(&program::UniformBlock) -> bool + 'static>, Option<Sender<LinearSyncFence>>),
    SignedInt(i32),
    UnsignedInt(u32),
    Float(f32),
    /// 2x2 column-major matrix. The second parameter describes whether to transpose it.
    Mat2([[f32; 2]; 2], bool),
    /// 3x3 column-major matrix. The second parameter describes whether to transpose it.
    Mat3([[f32; 3]; 3], bool),
    /// 4x4 column-major matrix. The second parameter describes whether to transpose it.
    Mat4([[f32; 4]; 4], bool),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Texture1d(&'a texture::Texture1d, Option<SamplerBehavior>),
    CompressedTexture1d(&'a texture::CompressedTexture1d, Option<SamplerBehavior>),
    IntegralTexture1d(&'a texture::IntegralTexture1d, Option<SamplerBehavior>),
    UnsignedTexture1d(&'a texture::UnsignedTexture1d, Option<SamplerBehavior>),
    DepthTexture1d(&'a texture::DepthTexture1d, Option<SamplerBehavior>),
    Texture2d(&'a texture::Texture2d, Option<SamplerBehavior>),
    CompressedTexture2d(&'a texture::CompressedTexture2d, Option<SamplerBehavior>),
    IntegralTexture2d(&'a texture::IntegralTexture2d, Option<SamplerBehavior>),
    UnsignedTexture2d(&'a texture::UnsignedTexture2d, Option<SamplerBehavior>),
    DepthTexture2d(&'a texture::DepthTexture2d, Option<SamplerBehavior>),
    Texture2dMultisample(&'a texture::Texture2dMultisample, Option<SamplerBehavior>),
    IntegralTexture2dMultisample(&'a texture::IntegralTexture2dMultisample, Option<SamplerBehavior>),
    UnsignedTexture2dMultisample(&'a texture::UnsignedTexture2dMultisample, Option<SamplerBehavior>),
    DepthTexture2dMultisample(&'a texture::DepthTexture2dMultisample, Option<SamplerBehavior>),
    Texture3d(&'a texture::Texture3d, Option<SamplerBehavior>),
    CompressedTexture3d(&'a texture::CompressedTexture3d, Option<SamplerBehavior>),
    IntegralTexture3d(&'a texture::IntegralTexture3d, Option<SamplerBehavior>),
    UnsignedTexture3d(&'a texture::UnsignedTexture3d, Option<SamplerBehavior>),
    DepthTexture3d(&'a texture::DepthTexture3d, Option<SamplerBehavior>),
    Texture1dArray(&'a texture::Texture1dArray, Option<SamplerBehavior>),
    CompressedTexture1dArray(&'a texture::CompressedTexture1dArray, Option<SamplerBehavior>),
    IntegralTexture1dArray(&'a texture::IntegralTexture1dArray, Option<SamplerBehavior>),
    UnsignedTexture1dArray(&'a texture::UnsignedTexture1dArray, Option<SamplerBehavior>),
    DepthTexture1dArray(&'a texture::DepthTexture1dArray, Option<SamplerBehavior>),
    Texture2dArray(&'a texture::Texture2dArray, Option<SamplerBehavior>),
    CompressedTexture2dArray(&'a texture::CompressedTexture2dArray, Option<SamplerBehavior>),
    IntegralTexture2dArray(&'a texture::IntegralTexture2dArray, Option<SamplerBehavior>),
    UnsignedTexture2dArray(&'a texture::UnsignedTexture2dArray, Option<SamplerBehavior>),
    DepthTexture2dArray(&'a texture::DepthTexture2dArray, Option<SamplerBehavior>),
    Texture2dArrayMultisample(&'a texture::Texture2dArrayMultisample, Option<SamplerBehavior>),
    IntegralTexture2dArrayMultisample(&'a texture::IntegralTexture2dArrayMultisample, Option<SamplerBehavior>),
    UnsignedTexture2dArrayMultisample(&'a texture::UnsignedTexture2dArrayMultisample, Option<SamplerBehavior>),
    DepthTexture2dArrayMultisample(&'a texture::DepthTexture2dArrayMultisample, Option<SamplerBehavior>),
}

impl<'a> UniformValue<'a> {
    /// Returns true if this value can be used with a uniform of the given type.
    pub fn is_usable_with(&self, ty: &UniformType) -> bool {
        match (self, *ty) {
            (&UniformValue::SignedInt(_), UniformType::Int) => true,
            (&UniformValue::UnsignedInt(_), UniformType::UnsignedInt) => true,
            (&UniformValue::Float(_), UniformType::Float) => true,
            (&UniformValue::Mat2(_,_ ), UniformType::FloatMat2) => true,
            (&UniformValue::Mat3(_, _), UniformType::FloatMat3) => true,
            (&UniformValue::Mat4(_, _), UniformType::FloatMat4) => true,
            (&UniformValue::Vec2(_), UniformType::FloatVec2) => true,
            (&UniformValue::Vec3(_), UniformType::FloatVec3) => true,
            (&UniformValue::Vec4(_), UniformType::FloatVec4) => true,
            (&UniformValue::Texture1d(_, _), UniformType::Sampler1d) => true,
            (&UniformValue::CompressedTexture1d(_, _), UniformType::Sampler1d) => true,
            (&UniformValue::IntegralTexture1d(_, _), UniformType::ISampler1d) => true,
            (&UniformValue::UnsignedTexture1d(_, _), UniformType::USampler1d) => true,
            (&UniformValue::DepthTexture1d(_, _), UniformType::Sampler1d) => true,
            (&UniformValue::Texture2d(_, _), UniformType::Sampler2d) => true,
            (&UniformValue::CompressedTexture2d(_, _), UniformType::Sampler2d) => true,
            (&UniformValue::IntegralTexture2d(_, _), UniformType::ISampler2d) => true,
            (&UniformValue::UnsignedTexture2d(_, _), UniformType::USampler2d) => true,
            (&UniformValue::DepthTexture2d(_, _), UniformType::Sampler2d) => true,
            (&UniformValue::Texture3d(_, _), UniformType::Sampler3d) => true,
            (&UniformValue::CompressedTexture3d(_, _), UniformType::Sampler3d) => true,
            (&UniformValue::IntegralTexture3d(_, _), UniformType::ISampler3d) => true,
            (&UniformValue::UnsignedTexture3d(_, _), UniformType::USampler3d) => true,
            (&UniformValue::DepthTexture3d(_, _), UniformType::Sampler3d) => true,
            (&UniformValue::Texture1dArray(_, _), UniformType::Sampler1dArray) => true,
            (&UniformValue::CompressedTexture1dArray(_, _), UniformType::Sampler1dArray) => true,
            (&UniformValue::IntegralTexture1dArray(_, _), UniformType::ISampler1dArray) => true,
            (&UniformValue::UnsignedTexture1dArray(_, _), UniformType::USampler1dArray) => true,
            (&UniformValue::DepthTexture1dArray(_, _), UniformType::Sampler1dArray) => true,
            (&UniformValue::Texture2dArray(_, _), UniformType::Sampler2dArray) => true,
            (&UniformValue::CompressedTexture2dArray(_, _), UniformType::Sampler2dArray) => true,
            (&UniformValue::IntegralTexture2dArray(_, _), UniformType::ISampler2dArray) => true,
            (&UniformValue::UnsignedTexture2dArray(_, _), UniformType::USampler2dArray) => true,
            (&UniformValue::DepthTexture2dArray(_, _), UniformType::Sampler2dArray) => true,
            _ => false,
        }
    }
}

// TODO: implement for each type individually instead
impl<'a, T> UniformBlock for T where T: IntoUniformValue<'a> + Copy + Send + Default {
    fn matches(block: &program::UniformBlock) -> bool {
        use std::mem;

        if block.members.len() != 1 {
            return false;
        }

        if block.size != mem::size_of::<T>() {
            return false;
        }

        let ref member = block.members[0];

        if member.offset != 0 {
            return false;
        }

        let me: T = Default::default();
        if !me.into_uniform_value().is_usable_with(&member.ty) {
            return false;
        }

        if member.size.is_some() {
            return false;
        }

        true
    }
}

impl IntoUniformValue<'static> for i8 {
    fn into_uniform_value(self) -> UniformValue<'static> {
        UniformValue::SignedInt(self as i32)
    }
}

impl IntoUniformValue<'static> for u8 {
    fn into_uniform_value(self) -> UniformValue<'static> {
        UniformValue::UnsignedInt(self as u32)
    }
}

impl IntoUniformValue<'static> for i16 {
    fn into_uniform_value(self) -> UniformValue<'static> {
        UniformValue::SignedInt(self as i32)
    }
}

impl IntoUniformValue<'static> for u16 {
    fn into_uniform_value(self) -> UniformValue<'static> {
        UniformValue::UnsignedInt(self as u32)
    }
}

impl IntoUniformValue<'static> for i32 {
    fn into_uniform_value(self) -> UniformValue<'static> {
        UniformValue::SignedInt(self as i32)
    }
}

impl IntoUniformValue<'static> for u32 {
    fn into_uniform_value(self) -> UniformValue<'static> {
        UniformValue::UnsignedInt(self as u32)
    }
}

impl IntoUniformValue<'static> for f32 {
    fn into_uniform_value(self) -> UniformValue<'static> {
        UniformValue::Float(self)
    }
}

impl IntoUniformValue<'static> for [[f32; 2]; 2] {
    fn into_uniform_value(self) -> UniformValue<'static> {
        UniformValue::Mat2(self, false)
    }
}

impl IntoUniformValue<'static> for [[f32; 3]; 3] {
    fn into_uniform_value(self) -> UniformValue<'static> {
        UniformValue::Mat3(self, false)
    }
}

impl IntoUniformValue<'static> for [[f32; 4]; 4] {
    fn into_uniform_value(self) -> UniformValue<'static> {
        UniformValue::Mat4(self, false)
    }
}

impl IntoUniformValue<'static> for (f32, f32) {
    fn into_uniform_value(self) -> UniformValue<'static> {
        UniformValue::Vec2([self.0, self.1])
    }
}

impl IntoUniformValue<'static> for (f32, f32, f32) {
    fn into_uniform_value(self) -> UniformValue<'static> {
        UniformValue::Vec3([self.0, self.1, self.2])
    }
}

impl IntoUniformValue<'static> for (f32, f32, f32, f32) {
    fn into_uniform_value(self) -> UniformValue<'static> {
        UniformValue::Vec4([self.0, self.1, self.2, self.3])
    }
}

impl IntoUniformValue<'static> for [f32; 2] {
    fn into_uniform_value(self) -> UniformValue<'static> {
        UniformValue::Vec2(self)
    }
}

impl IntoUniformValue<'static> for [f32; 3] {
    fn into_uniform_value(self) -> UniformValue<'static> {
        UniformValue::Vec3(self)
    }
}

impl IntoUniformValue<'static> for [f32; 4] {
    fn into_uniform_value(self) -> UniformValue<'static> {
        UniformValue::Vec4(self)
    }
}

#[cfg(feature = "nalgebra")]
impl IntoUniformValue<'static> for nalgebra::Mat2<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_array();
        my_value.into_uniform_value()
    }
}

#[cfg(feature = "nalgebra")]
impl IntoUniformValue<'static> for nalgebra::Mat3<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_array();
        my_value.into_uniform_value()
    }
}

#[cfg(feature = "nalgebra")]
impl IntoUniformValue<'static> for nalgebra::Mat4<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_array();
        my_value.into_uniform_value()
    }
}

#[cfg(feature = "nalgebra")]
impl IntoUniformValue<'static> for nalgebra::Ortho3<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.to_mat(); // Bind to a Mat4
        my_value.into_uniform_value()
    }
}

#[cfg(feature = "nalgebra")]
impl IntoUniformValue<'static> for nalgebra::OrthoMat3<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_mat(); // Bind to a Mat4
        my_value.into_uniform_value()
    }
}

#[cfg(feature = "nalgebra")]
impl IntoUniformValue<'static> for nalgebra::Persp3<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.to_mat(); // Bind to a Mat4
        my_value.into_uniform_value()
    }
}

#[cfg(feature = "nalgebra")]
impl IntoUniformValue<'static> for nalgebra::PerspMat3<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_mat(); // Bind to a Mat4
        my_value.into_uniform_value()
    }
}

#[cfg(feature = "nalgebra")]
impl IntoUniformValue<'static> for nalgebra::Pnt2<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_array();
        my_value.into_uniform_value()
    }
}

#[cfg(feature = "nalgebra")]
impl IntoUniformValue<'static> for nalgebra::Pnt3<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_array();
        my_value.into_uniform_value()
    }
}

#[cfg(feature = "nalgebra")]
impl IntoUniformValue<'static> for nalgebra::Pnt4<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_array();
        my_value.into_uniform_value()
    }
}

#[cfg(feature = "nalgebra")]
impl IntoUniformValue<'static> for nalgebra::Quat<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_array();
        my_value.into_uniform_value()
    }
}

#[cfg(feature = "nalgebra")]
impl IntoUniformValue<'static> for nalgebra::Rot2<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.submat(); // Bind to a Mat2
        my_value.into_uniform_value()
    }
}

#[cfg(feature = "nalgebra")]
impl IntoUniformValue<'static> for nalgebra::Rot3<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.submat(); // Bind to a Mat3
        my_value.into_uniform_value()
    }
}

#[cfg(feature = "nalgebra")]
impl IntoUniformValue<'static> for nalgebra::Rot4<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.submat(); // Bind to a Mat4
        my_value.into_uniform_value()
    }
}

#[cfg(feature = "nalgebra")]
impl IntoUniformValue<'static> for nalgebra::UnitQuat<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.quat(); // Bind to a Quat
        my_value.into_uniform_value()
    }
}

#[cfg(feature = "nalgebra")]
impl IntoUniformValue<'static> for nalgebra::Vec2<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_array();
        my_value.into_uniform_value()
    }
}

#[cfg(feature = "nalgebra")]
impl IntoUniformValue<'static> for nalgebra::Vec3<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_array();
        my_value.into_uniform_value()
    }
}

#[cfg(feature = "nalgebra")]
impl IntoUniformValue<'static> for nalgebra::Vec4<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_array();
        my_value.into_uniform_value()
    }
}


#[cfg(feature = "cgmath")]
impl IntoUniformValue<'static> for cgmath::Matrix2<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        my_value.into_uniform_value()
    }
}

#[cfg(feature = "cgmath")]
impl IntoUniformValue<'static> for cgmath::Matrix3<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        my_value.into_uniform_value()
    }
}

#[cfg(feature = "cgmath")]
impl IntoUniformValue<'static> for cgmath::Matrix4<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        my_value.into_uniform_value()
    }
}

#[cfg(feature = "cgmath")]
impl IntoUniformValue<'static> for cgmath::Vector2<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        my_value.into_uniform_value()
    }
}

#[cfg(feature = "cgmath")]
impl IntoUniformValue<'static> for cgmath::Vector3<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        my_value.into_uniform_value()
    }
}

#[cfg(feature = "cgmath")]
impl IntoUniformValue<'static> for cgmath::Vector4<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        my_value.into_uniform_value()
    }
}
