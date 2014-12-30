use texture;
use uniforms::SamplerBehavior;

use cgmath;
use nalgebra;

/// Type of a uniform in a program.
#[allow(missing_docs)]
#[deriving(Copy, Clone, Show, PartialEq, Eq)]
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
    SamplerCubeArray,
    ISamplerCubeArray,
    USamplerCubeArray,
    SamplerBuffer,
    ISamplerBuffer,
    USamplerBuffer,
    Sampler2dMultisample,
    ISampler2dMultisample,
    USampler2dMultisample,
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
#[deriving(Clone, Copy)]
#[allow(missing_docs)]
pub enum UniformValue<'a> {
    SignedInt(i32),
    UnsignedInt(u32),
    Float(f32),
    /// 2x2 column-major matrix.
    Mat2([[f32, ..2], ..2]),
    /// 3x3 column-major matrix.
    Mat3([[f32, ..3], ..3]),
    /// 4x4 column-major matrix.
    Mat4([[f32, ..4], ..4]),
    Vec2([f32, ..2]),
    Vec3([f32, ..3]),
    Vec4([f32, ..4]),
    Texture1d(&'a texture::Texture1d, Option<SamplerBehavior>),
    CompressedTexture1d(&'a texture::CompressedTexture1d, Option<SamplerBehavior>),
    IntegralTexture1d(&'a texture::IntegralTexture1d, Option<SamplerBehavior>),
    UnsignedTexture1d(&'a texture::UnsignedTexture1d, Option<SamplerBehavior>),
    Texture2d(&'a texture::Texture2d, Option<SamplerBehavior>),
    CompressedTexture2d(&'a texture::CompressedTexture2d, Option<SamplerBehavior>),
    IntegralTexture2d(&'a texture::IntegralTexture2d, Option<SamplerBehavior>),
    UnsignedTexture2d(&'a texture::UnsignedTexture2d, Option<SamplerBehavior>),
    Texture3d(&'a texture::Texture3d, Option<SamplerBehavior>),
    CompressedTexture3d(&'a texture::CompressedTexture3d, Option<SamplerBehavior>),
    IntegralTexture3d(&'a texture::IntegralTexture3d, Option<SamplerBehavior>),
    UnsignedTexture3d(&'a texture::UnsignedTexture3d, Option<SamplerBehavior>),
    Texture1dArray(&'a texture::Texture1dArray, Option<SamplerBehavior>),
    CompressedTexture1dArray(&'a texture::CompressedTexture1dArray, Option<SamplerBehavior>),
    IntegralTexture1dArray(&'a texture::IntegralTexture1dArray, Option<SamplerBehavior>),
    UnsignedTexture1dArray(&'a texture::UnsignedTexture1dArray, Option<SamplerBehavior>),
    Texture2dArray(&'a texture::Texture2dArray, Option<SamplerBehavior>),
    CompressedTexture2dArray(&'a texture::CompressedTexture2dArray, Option<SamplerBehavior>),
    IntegralTexture2dArray(&'a texture::IntegralTexture2dArray, Option<SamplerBehavior>),
    UnsignedTexture2dArray(&'a texture::UnsignedTexture2dArray, Option<SamplerBehavior>),
}

impl<'a> UniformValue<'a> {
    /// Returns the corresponding `UniformType`.
    pub fn get_type(&self) -> UniformType {
        match *self {
            UniformValue::Texture1d(_, _) => UniformType::Sampler1d,
            UniformValue::CompressedTexture1d(_, _) => UniformType::Sampler1d,
            UniformValue::IntegralTexture1d(_, _) => UniformType::ISampler1d,
            UniformValue::UnsignedTexture1d(_, _) => UniformType::USampler1d,
            UniformValue::Texture2d(_, _) => UniformType::Sampler2d,
            UniformValue::CompressedTexture2d(_, _) => UniformType::Sampler2d,
            UniformValue::IntegralTexture2d(_, _) => UniformType::ISampler2d,
            UniformValue::UnsignedTexture2d(_, _) => UniformType::USampler2d,
            UniformValue::Texture3d(_, _) => UniformType::Sampler3d,
            UniformValue::CompressedTexture3d(_, _) => UniformType::Sampler3d,
            UniformValue::IntegralTexture3d(_, _) => UniformType::ISampler3d,
            UniformValue::UnsignedTexture3d(_, _) => UniformType::USampler3d,
            UniformValue::Texture1dArray(_, _) => UniformType::Sampler1dArray,
            UniformValue::CompressedTexture1dArray(_, _) => UniformType::Sampler1dArray,
            UniformValue::IntegralTexture1dArray(_, _) => UniformType::ISampler1dArray,
            UniformValue::UnsignedTexture1dArray(_, _) => UniformType::USampler1dArray,
            UniformValue::Texture2dArray(_, _) => UniformType::Sampler2dArray,
            UniformValue::CompressedTexture2dArray(_, _) => UniformType::Sampler2dArray,
            UniformValue::IntegralTexture2dArray(_, _) => UniformType::ISampler2dArray,
            UniformValue::UnsignedTexture2dArray(_, _) => UniformType::USampler2dArray,
            _ => unimplemented!()
        }
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

impl IntoUniformValue<'static> for [[f32, ..2], ..2] {
    fn into_uniform_value(self) -> UniformValue<'static> {
        UniformValue::Mat2(self)
    }
}

impl IntoUniformValue<'static> for [[f32, ..3], ..3] {
    fn into_uniform_value(self) -> UniformValue<'static> {
        UniformValue::Mat3(self)
    }
}

impl IntoUniformValue<'static> for [[f32, ..4], ..4] {
    fn into_uniform_value(self) -> UniformValue<'static> {
        UniformValue::Mat4(self)
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

impl IntoUniformValue<'static> for [f32, ..2] {
    fn into_uniform_value(self) -> UniformValue<'static> {
        UniformValue::Vec2(self)
    }
}

impl IntoUniformValue<'static> for [f32, ..3] {
    fn into_uniform_value(self) -> UniformValue<'static> {
        UniformValue::Vec3(self)
    }
}

impl IntoUniformValue<'static> for [f32, ..4] {
    fn into_uniform_value(self) -> UniformValue<'static> {
        UniformValue::Vec4(self)
    }
}

impl IntoUniformValue<'static> for nalgebra::Mat2<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_array();
        my_value.into_uniform_value()
    }
}

impl IntoUniformValue<'static> for nalgebra::Mat3<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_array();
        my_value.into_uniform_value()
    }
}

impl IntoUniformValue<'static> for nalgebra::Mat4<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_array();
        my_value.into_uniform_value()
    }
}

impl IntoUniformValue<'static> for nalgebra::Ortho3<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.to_mat(); // Bind to a Mat4
        my_value.into_uniform_value()
    }
}

impl IntoUniformValue<'static> for nalgebra::OrthoMat3<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_mat(); // Bind to a Mat4
        my_value.into_uniform_value()
    }
}

impl IntoUniformValue<'static> for nalgebra::Persp3<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.to_mat(); // Bind to a Mat4
        my_value.into_uniform_value()
    }
}

impl IntoUniformValue<'static> for nalgebra::PerspMat3<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_mat(); // Bind to a Mat4
        my_value.into_uniform_value()
    }
}

impl IntoUniformValue<'static> for nalgebra::Pnt2<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_array();
        my_value.into_uniform_value()
    }
}

impl IntoUniformValue<'static> for nalgebra::Pnt3<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_array();
        my_value.into_uniform_value()
    }
}

impl IntoUniformValue<'static> for nalgebra::Pnt4<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_array();
        my_value.into_uniform_value()
    }
}

impl IntoUniformValue<'static> for nalgebra::Quat<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_array();
        my_value.into_uniform_value()
    }
}

impl IntoUniformValue<'static> for nalgebra::Rot2<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.submat(); // Bind to a Mat2
        my_value.into_uniform_value()
    }
}

impl IntoUniformValue<'static> for nalgebra::Rot3<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.submat(); // Bind to a Mat3
        my_value.into_uniform_value()
    }
}

impl IntoUniformValue<'static> for nalgebra::Rot4<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.submat(); // Bind to a Mat4
        my_value.into_uniform_value()
    }
}

impl IntoUniformValue<'static> for nalgebra::UnitQuat<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.quat(); // Bind to a Quat
        my_value.into_uniform_value()
    }
}

impl IntoUniformValue<'static> for nalgebra::Vec2<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_array();
        my_value.into_uniform_value()
    }
}

impl IntoUniformValue<'static> for nalgebra::Vec3<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_array();
        my_value.into_uniform_value()
    }
}

impl IntoUniformValue<'static> for nalgebra::Vec4<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        let my_value = self.as_array();
        my_value.into_uniform_value()
    }
}


impl IntoUniformValue<'static> for cgmath::Matrix2<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        my_value.into_uniform_value()
    }
}

impl IntoUniformValue<'static> for cgmath::Matrix3<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        my_value.into_uniform_value()
    }
}

impl IntoUniformValue<'static> for cgmath::Matrix4<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        my_value.into_uniform_value()
    }
}

impl IntoUniformValue<'static> for cgmath::Vector2<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        my_value.into_uniform_value()
    }
}

impl IntoUniformValue<'static> for cgmath::Vector3<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        my_value.into_uniform_value()
    }
}

impl IntoUniformValue<'static> for cgmath::Vector4<f32> {
    fn into_uniform_value(self) -> UniformValue<'static> {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        my_value.into_uniform_value()
    }
}
