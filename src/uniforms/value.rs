use std::fmt::{self, Debug};
use std::cmp;

use program;
use program::BlockLayout;
use program::ShaderStage;
use texture;

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
    /// can be bound on a block with the given layout.
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
    BufferTexture(texture::buffer_texture::BufferTextureRef),
}

impl<'a> Clone for UniformValue<'a> {
    #[inline]
    fn clone(&self) -> UniformValue<'a> {
        *self
    }
}

impl<'a> Debug for UniformValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UniformValue::Block(ref i, _) => write!(f, "UniformValue::Block({:?}, fn(&program::UniformBlock) -> Result<(), LayoutMismatchError>)", i),
            UniformValue::Subroutine(ref i, ref s) => write!(f, "UniformValue::Subroutine({:?}, {:?})", i, s),
            UniformValue::Float(ref i) => write!(f, "UniformValue::Float({:?})", i),
            UniformValue::Double(ref i) => write!(f, "UniformValue::Double({:?})", i),
            UniformValue::Int64(ref i) => write!(f, "UniformValue::Int64({:?})", i),
            UniformValue::UnsignedInt64(ref i) => write!(f, "UniformValue::UnsignedInt64({:?})", i),
            UniformValue::SignedInt(ref i) => write!(f, "UniformValue::SignedInt({:?})", i),
            UniformValue::UnsignedInt(ref i) => write!(f, "UniformValue::UnsignedInt({:?})", i),
            UniformValue::IntVec2(ref i) => write!(f, "UniformValue::IntVec2({:?})", i),
            UniformValue::IntVec3(ref i) => write!(f, "UniformValue::IntVec3({:?})", i),
            UniformValue::IntVec4(ref i) => write!(f, "UniformValue::IntVec4({:?})", i),
            UniformValue::UnsignedIntVec2(ref i) => write!(f, "UniformValue::UnsignedIntVec2({:?})", i),
            UniformValue::UnsignedIntVec3(ref i) => write!(f, "UniformValue::UnsignedIntVec3({:?})", i),
            UniformValue::UnsignedIntVec4(ref i) => write!(f, "UniformValue::UnsignedIntVec4({:?})", i),
            UniformValue::Bool(ref i) => write!(f, "UniformValue::Bool({:?})", i),
            UniformValue::BoolVec2(ref i) => write!(f, "UniformValue::BoolVec2({:?})", i),
            UniformValue::BoolVec3(ref i) => write!(f, "UniformValue::BoolVec3({:?})", i),
            UniformValue::BoolVec4(ref i) => write!(f, "UniformValue::BoolVec4({:?})", i),
            UniformValue::Vec2(ref i) => write!(f, "UniformValue::Vec2({:?})", i),
            UniformValue::Vec3(ref i) => write!(f, "UniformValue::Vec3({:?})", i),
            UniformValue::Vec4(ref i) => write!(f, "UniformValue::Vec4({:?})", i),
            UniformValue::DoubleVec2(ref i) => write!(f, "UniformValue::DoubleVec2({:?})", i),
            UniformValue::DoubleVec3(ref i) => write!(f, "UniformValue::DoubleVec3({:?})", i),
            UniformValue::DoubleVec4(ref i) => write!(f, "UniformValue::DoubleVec4({:?})", i),
            UniformValue::Int64Vec2(ref i) => write!(f, "UniformValue::Int64Vec2({:?})", i),
            UniformValue::Int64Vec3(ref i) => write!(f, "UniformValue::Int64Vec3({:?})", i),
            UniformValue::Int64Vec4(ref i) => write!(f, "UniformValue::Int64Vec4({:?})", i),
            UniformValue::UnsignedInt64Vec2(ref i) => write!(f, "UniformValue::UnsignedInt64Vec2({:?})", i),
            UniformValue::UnsignedInt64Vec3(ref i) => write!(f, "UniformValue::UnsignedInt64Vec3({:?})", i),
            UniformValue::UnsignedInt64Vec4(ref i) => write!(f, "UniformValue::UnsignedInt64Vec4({:?})", i),
            UniformValue::Mat2(ref i) => write!(f, "UniformValue::Mat2({:?})", i),
            UniformValue::Mat3(ref i) => write!(f, "UniformValue::Mat3({:?})", i),
            UniformValue::Mat4(ref i) => write!(f, "UniformValue::Mat4({:?})", i),
            UniformValue::DoubleMat2(ref i) => write!(f, "UniformValue::DoubleMat2({:?})", i),
            UniformValue::DoubleMat3(ref i) => write!(f, "UniformValue::DoubleMat3({:?})", i),
            UniformValue::DoubleMat4(ref i) => write!(f, "UniformValue::DoubleMat4({:?})", i),
            UniformValue::Texture1d(ref i, ref o) => write!(f, "UniformValue::Texture1d({:?}, {:?})", i, o),
            UniformValue::CompressedTexture1d(ref i, ref o) => write!(f, "UniformValue::CompressedTexture1d({:?}, {:?})", i, o),
            UniformValue::SrgbTexture1d(ref i, ref o) => write!(f, "UniformValue::SrgbTexture1d({:?}, {:?})", i, o),
            UniformValue::CompressedSrgbTexture1d(ref i, ref o) => write!(f, "UniformValue::CompressedSrgbTexture1d({:?}, {:?})", i, o),
            UniformValue::IntegralTexture1d(ref i, ref o) => write!(f, "UniformValue::IntegralUnsignedTexture1d({:?}, {:?})", i, o),
            UniformValue::UnsignedTexture1d(ref i, ref o) => write!(f, "UniformValue::IntegralUnsignedTexture1d({:?}, {:?})", i, o),
            UniformValue::DepthTexture1d(ref i, ref o) => write!(f, "UniformValue::DepthTexture1d({:?}, {:?})", i, o),
            UniformValue::Texture2d(ref i, ref o) => write!(f, "UniformValue::Texture2d({:?}, {:?})", i, o),
            UniformValue::CompressedTexture2d(ref i, ref o) => write!(f, "UniformValue::CompressedTexture2d({:?}, {:?})", i, o),
            UniformValue::SrgbTexture2d(ref i, ref o) => write!(f, "UniformValue::SrgbTexture2d({:?}, {:?})", i, o),
            UniformValue::CompressedSrgbTexture2d(ref i, ref o) => write!(f, "UniformValue::CompressedSrgbTexture2d({:?}, {:?})", i, o),
            UniformValue::IntegralTexture2d(ref i, ref o) => write!(f, "UniformValue::IntegralUnsignedTexture2d({:?}, {:?})", i, o),
            UniformValue::UnsignedTexture2d(ref i, ref o) => write!(f, "UniformValue::IntegralUnsignedTexture2d({:?}, {:?})", i, o),
            UniformValue::DepthTexture2d(ref i, ref o) => write!(f, "UniformValue::DepthTexture2d({:?}, {:?})", i, o),
            UniformValue::Texture3d(ref i, ref o) => write!(f, "UniformValue::Texture3d({:?}, {:?})", i, o),
            UniformValue::CompressedTexture3d(ref i, ref o) => write!(f, "UniformValue::CompressedTexture3d({:?}, {:?})", i, o),
            UniformValue::SrgbTexture3d(ref i, ref o) => write!(f, "UniformValue::SrgbTexture3d({:?}, {:?})", i, o),
            UniformValue::CompressedSrgbTexture3d(ref i, ref o) => write!(f, "UniformValue::CompressedSrgbTexture3d({:?}, {:?})", i, o),
            UniformValue::IntegralTexture3d(ref i, ref o) => write!(f, "UniformValue::IntegralUnsignedTexture3d({:?}, {:?})", i, o),
            UniformValue::UnsignedTexture3d(ref i, ref o) => write!(f, "UniformValue::IntegralUnsignedTexture3d({:?}, {:?})", i, o),
            UniformValue::DepthTexture3d(ref i, ref o) => write!(f, "UniformValue::DepthTexture3d({:?}, {:?})", i, o),
            UniformValue::CompressedTexture1dArray(ref i, ref o) => write!(f, "UniformValue::CompressedTexture1dArray({:?}, {:?})", i, o),
            UniformValue::SrgbTexture1dArray(ref i, ref o) => write!(f, "UniformValue::SrgbTexture1dArray({:?}, {:?})", i, o),
            UniformValue::CompressedSrgbTexture1dArray(ref i, ref o) => write!(f, "UniformValue::CompressedSrgbTexture1dArray({:?}, {:?})", i, o),
            UniformValue::IntegralTexture1dArray(ref i, ref o) => write!(f, "UniformValue::IntegralUnsignedTexture1dArray({:?}, {:?})", i, o),
            UniformValue::UnsignedTexture1dArray(ref i, ref o) => write!(f, "UniformValue::IntegralUnsignedTexture1dArray({:?}, {:?})", i, o),
            UniformValue::DepthTexture1dArray(ref i, ref o) => write!(f, "UniformValue::DepthTexture1dArray({:?}, {:?})", i, o),
            UniformValue::Texture2dArray(ref i, ref o) => write!(f, "UniformValue::Texture2dArray({:?}, {:?})", i, o),
            UniformValue::CompressedTexture2dArray(ref i, ref o) => write!(f, "UniformValue::CompressedTexture2dArray({:?}, {:?})", i, o),
            UniformValue::SrgbTexture2dArray(ref i, ref o) => write!(f, "UniformValue::SrgbTexture2dArray({:?}, {:?})", i, o),
            UniformValue::CompressedSrgbTexture2dArray(ref i, ref o) => write!(f, "UniformValue::CompressedSrgbTexture2dArray({:?}, {:?})", i, o),
            UniformValue::IntegralTexture2dArray(ref i, ref o) => write!(f, "UniformValue::IntegralUnsignedTexture2dArray({:?}, {:?})", i, o),
            UniformValue::UnsignedTexture2dArray(ref i, ref o) => write!(f, "UniformValue::IntegralUnsignedTexture2dArray({:?}, {:?})", i, o),
            UniformValue::DepthTexture2dArray(ref i, ref o) => write!(f, "UniformValue::DepthTexture2dArray({:?}, {:?})", i, o),
            UniformValue::Texture2dMultisampleArray(ref i, ref o) => write!(f, "UniformValue::Texture2dMultisampleArray({:?}, {:?})", i, o),
            UniformValue::SrgbTexture2dMultisampleArray(ref i, ref o) => write!(f, "UniformValue::SrgbTexture2dMultisampleArray({:?}, {:?})", i, o),
            UniformValue::IntegralTexture2dMultisampleArray(ref i, ref o) => write!(f, "UniformValue::IntegralUnsignedTexture2dMultisampleArray({:?}, {:?})", i, o),
            UniformValue::UnsignedTexture2dMultisampleArray(ref i, ref o) => write!(f, "UniformValue::IntegralUnsignedTexture2dMultisampleArray({:?}, {:?})", i, o),
            UniformValue::DepthTexture2dMultisampleArray(ref i, ref o) => write!(f, "UniformValue::DepthTexture2dMultisampleArray({:?}, {:?})", i, o),
            UniformValue::Texture2dMultisample(ref i, ref o) => write!(f, "UniformValue::Texture2dMultisample({:?}, {:?})", i, o),
            UniformValue::SrgbTexture2dMultisample(ref i, ref o) => write!(f, "UniformValue::SrgbTexture2dMultisample({:?}, {:?})", i, o),
            UniformValue::IntegralTexture2dMultisample(ref i, ref o) => write!(f, "UniformValue::IntegralUnsignedTexture2dMultisample({:?}, {:?})", i, o),
            UniformValue::UnsignedTexture2dMultisample(ref i, ref o) => write!(f, "UniformValue::IntegralUnsignedTexture2dMultisample({:?}, {:?})", i, o),
            UniformValue::DepthTexture2dMultisample(ref i, ref o) => write!(f, "UniformValue::DepthTexture2dMultisample({:?}, {:?})", i, o),
            UniformValue::Texture1dArray(ref i, ref o) => write!(f, "UniformValue::Texture1dArray({:?}, {:?})", i, o),
            UniformValue::Cubemap(ref i, ref o) => write!(f, "UniformValue::Cubemap({:?}, {:?})", i, o),
            UniformValue::CompressedCubemap(ref i, ref o) => write!(f, "UniformValue::CompressedCubemap({:?}, {:?})", i, o),
            UniformValue::SrgbCubemap(ref i, ref o) => write!(f, "UniformValue::SrgbCubemap({:?}, {:?})", i, o),
            UniformValue::CompressedSrgbCubemap(ref i, ref o) => write!(f, "UniformValue::CompressedSrgbCubemap({:?}, {:?})", i, o),
            UniformValue::IntegralCubemap(ref i, ref o) => write!(f, "UniformValue::IntegralUnsignedCubemap({:?}, {:?})", i, o),
            UniformValue::UnsignedCubemap(ref i, ref o) => write!(f, "UniformValue::IntegralUnsignedCubemap({:?}, {:?})", i, o),
            UniformValue::DepthCubemap(ref i, ref o) => write!(f, "UniformValue::DepthCubemap({:?}, {:?})", i, o),
            UniformValue::CubemapArray(ref i, ref o) => write!(f, "UniformValue::CubemapArray({:?}, {:?})", i, o),
            UniformValue::CompressedCubemapArray(ref i, ref o) => write!(f, "UniformValue::CompressedCubemapArray({:?}, {:?})", i, o),
            UniformValue::SrgbCubemapArray(ref i, ref o) => write!(f, "UniformValue::SrgbCubemapArray({:?}, {:?})", i, o),
            UniformValue::CompressedSrgbCubemapArray(ref i, ref o) => write!(f, "UniformValue::CompressedSrgbCubemapArray({:?}, {:?})", i, o),
            UniformValue::IntegralCubemapArray(ref i, ref o) => write!(f, "UniformValue::IntegralUnsignedCubemapArray({:?}, {:?})", i, o),
            UniformValue::UnsignedCubemapArray(ref i, ref o) => write!(f, "UniformValue::IntegralUnsignedCubemapArray({:?}, {:?})", i, o),
            UniformValue::DepthCubemapArray(ref i, ref o) => write!(f, "UniformValue::DepthCubemapArray({:?}, {:?})", i, o),
            UniformValue::BufferTexture(ref i) => write!(f, "UniformValue::BufferTexture({:?})", i),
        }
    }
}

impl<'a, 'b> cmp::PartialEq<UniformValue<'b>> for UniformValue<'a> {
    fn eq(&self, other: &UniformValue<'b>) -> bool {
        match (self, other) {
            (&UniformValue::Float(i), &UniformValue::Float(j)) => i == j,
            (&UniformValue::Double(i), &UniformValue::Double(j)) => i == j,
            (&UniformValue::Int64(i), &UniformValue::Int64(j)) => i == j,
            (&UniformValue::UnsignedInt64(i), &UniformValue::UnsignedInt64(j)) => i == j,
            (&UniformValue::SignedInt(i), &UniformValue::SignedInt(j)) => i == j,
            (&UniformValue::UnsignedInt(i), &UniformValue::UnsignedInt(j)) => i == j,
            (&UniformValue::IntVec2(i), &UniformValue::IntVec2(j)) => i == j,
            (&UniformValue::IntVec3(i), &UniformValue::IntVec3(j)) => i == j,
            (&UniformValue::IntVec4(i), &UniformValue::IntVec4(j)) => i == j,
            (&UniformValue::UnsignedIntVec2(i), &UniformValue::UnsignedIntVec2(j)) => i == j,
            (&UniformValue::UnsignedIntVec3(i), &UniformValue::UnsignedIntVec3(j)) => i == j,
            (&UniformValue::UnsignedIntVec4(i), &UniformValue::UnsignedIntVec4(j)) => i == j,
            (&UniformValue::Bool(i), &UniformValue::Bool(j)) => i == j,
            (&UniformValue::BoolVec2(i), &UniformValue::BoolVec2(j)) => i == j,
            (&UniformValue::BoolVec3(i), &UniformValue::BoolVec3(j)) => i == j,
            (&UniformValue::BoolVec4(i), &UniformValue::BoolVec4(j)) => i == j,
            (&UniformValue::Vec2(i), &UniformValue::Vec2(j)) => i == j,
            (&UniformValue::Vec3(i), &UniformValue::Vec3(j)) => i == j,
            (&UniformValue::Vec4(i), &UniformValue::Vec4(j)) => i == j,
            (&UniformValue::DoubleVec2(i), &UniformValue::DoubleVec2(j)) => i == j,
            (&UniformValue::DoubleVec3(i), &UniformValue::DoubleVec3(j)) => i == j,
            (&UniformValue::DoubleVec4(i), &UniformValue::DoubleVec4(j)) => i == j,
            (&UniformValue::Int64Vec2(i), &UniformValue::Int64Vec2(j)) => i == j,
            (&UniformValue::Int64Vec3(i), &UniformValue::Int64Vec3(j)) => i == j,
            (&UniformValue::Int64Vec4(i), &UniformValue::Int64Vec4(j)) => i == j,
            (&UniformValue::UnsignedInt64Vec2(i), &UniformValue::UnsignedInt64Vec2(j)) => i == j,
            (&UniformValue::UnsignedInt64Vec3(i), &UniformValue::UnsignedInt64Vec3(j)) => i == j,
            (&UniformValue::UnsignedInt64Vec4(i), &UniformValue::UnsignedInt64Vec4(j)) => i == j,
            (&UniformValue::Mat2(i), &UniformValue::Mat2(j)) => i == j,
            (&UniformValue::Mat3(i), &UniformValue::Mat3(j)) => i == j,
            (&UniformValue::Mat4(i), &UniformValue::Mat4(j)) => i == j,
            (&UniformValue::DoubleMat2(i), &UniformValue::DoubleMat2(j)) => i == j,
            (&UniformValue::DoubleMat3(i), &UniformValue::DoubleMat3(j)) => i == j,
            (&UniformValue::DoubleMat4(i), &UniformValue::DoubleMat4(j)) => i == j,
            _ => false,
        }
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
            (&UniformValue::DepthTexture1d(_, _), UniformType::Sampler1dShadow) => true,
            (&UniformValue::Texture2d(_, _), UniformType::Sampler2d) => true,
            (&UniformValue::CompressedTexture2d(_, _), UniformType::Sampler2d) => true,
            (&UniformValue::SrgbTexture2d(_, _), UniformType::Sampler2d) => true,
            (&UniformValue::CompressedSrgbTexture2d(_, _), UniformType::Sampler2d) => true,
            (&UniformValue::IntegralTexture2d(_, _), UniformType::ISampler2d) => true,
            (&UniformValue::UnsignedTexture2d(_, _), UniformType::USampler2d) => true,
            (&UniformValue::DepthTexture2d(_, _), UniformType::Sampler2d) => true,
            (&UniformValue::DepthTexture2d(_, _), UniformType::Sampler2dShadow) => true,
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
            (&UniformValue::DepthTexture1dArray(_, _), UniformType::Sampler1dArrayShadow) => true,
            (&UniformValue::Texture2dArray(_, _), UniformType::Sampler2dArray) => true,
            (&UniformValue::CompressedTexture2dArray(_, _), UniformType::Sampler2dArray) => true,
            (&UniformValue::SrgbTexture2dArray(_, _), UniformType::Sampler2dArray) => true,
            (&UniformValue::CompressedSrgbTexture2dArray(_, _), UniformType::Sampler2dArray) => true,
            (&UniformValue::IntegralTexture2dArray(_, _), UniformType::ISampler2dArray) => true,
            (&UniformValue::UnsignedTexture2dArray(_, _), UniformType::USampler2dArray) => true,
            (&UniformValue::DepthTexture2dArray(_, _), UniformType::Sampler2dArray) => true,
            (&UniformValue::DepthTexture2dArray(_, _), UniformType::Sampler2dArrayShadow) => true,
            (&UniformValue::Cubemap(_, _), UniformType::SamplerCube) => true,
            (&UniformValue::CompressedCubemap(_, _), UniformType::SamplerCube) => true,
            (&UniformValue::SrgbCubemap(_, _), UniformType::SamplerCube) => true,
            (&UniformValue::CompressedSrgbCubemap(_, _), UniformType::SamplerCube) => true,
            (&UniformValue::IntegralCubemap(_, _), UniformType::ISamplerCube) => true,
            (&UniformValue::UnsignedCubemap(_, _), UniformType::USamplerCube) => true,
            (&UniformValue::DepthCubemap(_, _), UniformType::SamplerCube) => true,
            (&UniformValue::DepthCubemap(_, _), UniformType::SamplerCubeShadow) => true,
            (&UniformValue::CubemapArray(_, _), UniformType::SamplerCubeArray) => true,
            (&UniformValue::CompressedCubemapArray(_, _), UniformType::SamplerCubeArray) => true,
            (&UniformValue::SrgbCubemapArray(_, _), UniformType::SamplerCubeArray) => true,
            (&UniformValue::CompressedSrgbCubemapArray(_, _), UniformType::SamplerCubeArray) => true,
            (&UniformValue::IntegralCubemapArray(_, _), UniformType::ISamplerCubeArray) => true,
            (&UniformValue::UnsignedCubemapArray(_, _), UniformType::USamplerCubeArray) => true,
            (&UniformValue::DepthCubemapArray(_, _), UniformType::SamplerCubeArray) => true,
            (&UniformValue::DepthCubemapArray(_, _), UniformType::SamplerCubeArrayShadow) => true,
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

impl<'a> From<i8> for UniformValue<'a> {
    #[inline]
    fn from(v: i8) -> UniformValue<'a> {
        UniformValue::SignedInt(v as i32)
    }
}

impl<'a> From<u8> for UniformValue<'a> {
    #[inline]
    fn from(v: u8) -> UniformValue<'a> {
        UniformValue::UnsignedInt(v as u32)
    }
}

impl<'a> From<i16> for UniformValue<'a> {
    #[inline]
    fn from(v: i16) -> UniformValue<'a> {
        UniformValue::SignedInt(v as i32)
    }
}

impl<'a> From<u16> for UniformValue<'a> {
    #[inline]
    fn from(v: u16) -> UniformValue<'a> {
        UniformValue::UnsignedInt(v as u32)
    }
}

impl<'a> From<i32> for UniformValue<'a> {
    #[inline]
    fn from(v: i32) -> UniformValue<'a> {
        UniformValue::SignedInt(v as i32)
    }
}

impl<'a> From<u32> for UniformValue<'a> {
    #[inline]
    fn from(v: u32) -> UniformValue<'a> {
        UniformValue::UnsignedInt(v as u32)
    }
}

impl<'a> From<[i32; 2]> for UniformValue<'a> {
    #[inline]
    fn from(v: [i32; 2]) -> UniformValue<'a> {
        UniformValue::IntVec2(v)
    }
}

impl<'a> From<[i32; 3]> for UniformValue<'a> {
    #[inline]
    fn from(v: [i32; 3]) -> UniformValue<'a> {
        UniformValue::IntVec3(v)
    }
}

impl<'a> From<[i32; 4]> for UniformValue<'a> {
    #[inline]
    fn from(v: [i32; 4]) -> UniformValue<'a> {
        UniformValue::IntVec4(v)
    }
}

impl<'a> From<(i32, i32)> for UniformValue<'a> {
    #[inline]
    fn from(v: (i32, i32)) -> UniformValue<'a> {
        [v.0, v.1].into()
    }
}

impl<'a> From<(i32, i32, i32)> for UniformValue<'a> {
    #[inline]
    fn from(v: (i32, i32, i32)) -> UniformValue<'a> {
        [v.0, v.1, v.2].into()
    }
}

impl<'a> From<(i32, i32, i32, i32)> for UniformValue<'a> {
    #[inline]
    fn from(v: (i32, i32, i32, i32)) -> UniformValue<'a> {
        [v.0, v.1, v.2, v.3].into()
    }
}

impl<'a> From<[u32; 2]> for UniformValue<'a> {
    #[inline]
    fn from(v: [u32; 2]) -> UniformValue<'a> {
        UniformValue::UnsignedIntVec2(v)
    }
}

impl<'a> From<[u32; 3]> for UniformValue<'a> {
    #[inline]
    fn from(v: [u32; 3]) -> UniformValue<'a> {
        UniformValue::UnsignedIntVec3(v)
    }
}

impl<'a> From<[u32; 4]> for UniformValue<'a> {
    #[inline]
    fn from(v: [u32; 4]) -> UniformValue<'a> {
        UniformValue::UnsignedIntVec4(v)
    }
}

impl<'a> From<(u32, u32)> for UniformValue<'a> {
    #[inline]
    fn from(v: (u32, u32)) -> UniformValue<'a> {
        [v.0, v.1].into()
    }
}

impl<'a> From<(u32, u32, u32)> for UniformValue<'a> {
    #[inline]
    fn from(v: (u32, u32, u32)) -> UniformValue<'a> {
        [v.0, v.1, v.2].into()
    }
}

impl<'a> From<(u32, u32, u32, u32)> for UniformValue<'a> {
    #[inline]
    fn from(v: (u32, u32, u32, u32)) -> UniformValue<'a> {
        [v.0, v.1, v.2, v.3].into()
    }
}


impl<'a> From<[bool; 2]> for UniformValue<'a> {
    #[inline]
    fn from(v: [bool; 2]) -> UniformValue<'a> {
        UniformValue::BoolVec2(v)
    }
}

impl<'a> From<[bool; 3]> for UniformValue<'a> {
    #[inline]
    fn from(v: [bool; 3]) -> UniformValue<'a> {
        UniformValue::BoolVec3(v)
    }
}

impl<'a> From<[bool; 4]> for UniformValue<'a> {
    #[inline]
    fn from(v: [bool; 4]) -> UniformValue<'a> {
        UniformValue::BoolVec4(v)
    }
}

impl<'a> From<bool> for UniformValue<'a> {
    #[inline]
    fn from(v: bool) -> UniformValue<'a> {
        UniformValue::Bool(v)
    }
}

impl<'a> From<(bool, bool)> for UniformValue<'a> {
    #[inline]
    fn from(v: (bool, bool)) -> UniformValue<'a> {
        [v.0, v.1].into()
    }
}

impl<'a> From<(bool, bool, bool)> for UniformValue<'a> {
    #[inline]
    fn from(v: (bool, bool, bool)) -> UniformValue<'a> {
        [v.0, v.1, v.2].into()
    }
}

impl<'a> From<(bool, bool, bool, bool)> for UniformValue<'a> {
    #[inline]
    fn from(v: (bool, bool, bool, bool)) -> UniformValue<'a> {
        [v.0, v.1, v.2, v.3].into()
    }
}

impl<'a> From<f32> for UniformValue<'a> {
    #[inline]
    fn from(v: f32) -> UniformValue<'a> {
        UniformValue::Float(v)
    }
}

impl<'a> From<[f32; 2]> for UniformValue<'a> {
    #[inline]
    fn from(v: [f32; 2]) -> UniformValue<'a> {
        UniformValue::Vec2(v)
    }
}

impl<'a> From<[f32; 3]> for UniformValue<'a> {
    #[inline]
    fn from(v: [f32; 3]) -> UniformValue<'a> {
        UniformValue::Vec3(v)
    }
}

impl<'a> From<[f32; 4]> for UniformValue<'a> {
    #[inline]
    fn from(v: [f32; 4]) -> UniformValue<'a> {
        UniformValue::Vec4(v)
    }
}

impl<'a> From<(f32, f32)> for UniformValue<'a> {
    #[inline]
    fn from(v: (f32, f32)) -> UniformValue<'a> {
        [v.0, v.1].into()
    }
}

impl<'a> From<(f32, f32, f32)> for UniformValue<'a> {
    #[inline]
    fn from(v: (f32, f32, f32)) -> UniformValue<'a> {
        [v.0, v.1, v.2].into()
    }
}

impl<'a> From<(f32, f32, f32, f32)> for UniformValue<'a> {
    #[inline]
    fn from(v: (f32, f32, f32, f32)) -> UniformValue<'a> {
        [v.0, v.1, v.2, v.3].into()
    }
}

impl<'a> From<f64> for UniformValue<'a> {
    #[inline]
    fn from(v: f64) -> UniformValue<'a> {
        UniformValue::Double(v)
    }
}

impl<'a> From<[f64; 2]> for UniformValue<'a> {
    #[inline]
    fn from(v: [f64; 2]) -> UniformValue<'a> {
        UniformValue::DoubleVec2(v)
    }
}

impl<'a> From<[f64; 3]> for UniformValue<'a> {
    #[inline]
    fn from(v: [f64; 3]) -> UniformValue<'a> {
        UniformValue::DoubleVec3(v)
    }
}

impl<'a> From<[f64; 4]> for UniformValue<'a> {
    #[inline]
    fn from(v: [f64; 4]) -> UniformValue<'a> {
        UniformValue::DoubleVec4(v)
    }
}

impl<'a> From<(f64, f64)> for UniformValue<'a> {
    #[inline]
    fn from(v: (f64, f64)) -> UniformValue<'a> {
        [v.0, v.1].into()
    }
}

impl<'a> From<(f64, f64, f64)> for UniformValue<'a> {
    #[inline]
    fn from(v: (f64, f64, f64)) -> UniformValue<'a> {
        [v.0, v.1, v.2].into()
    }
}

impl<'a> From<(f64, f64, f64, f64)> for UniformValue<'a> {
    #[inline]
    fn from(v: (f64, f64, f64, f64)) -> UniformValue<'a> {
        [v.0, v.1, v.2, v.3].into()
    }
}

impl<'a> From<i64> for UniformValue<'a> {
    #[inline]
    fn from(v: i64) -> UniformValue<'a> {
        UniformValue::Int64(v)
    }
}

impl<'a> From<[i64; 2]> for UniformValue<'a> {
    #[inline]
    fn from(v: [i64; 2]) -> UniformValue<'a> {
        UniformValue::Int64Vec2(v)
    }
}

impl<'a> From<[i64; 3]> for UniformValue<'a> {
    #[inline]
    fn from(v: [i64; 3]) -> UniformValue<'a> {
        UniformValue::Int64Vec3(v)
    }
}

impl<'a> From<[i64; 4]> for UniformValue<'a> {
    #[inline]
    fn from(v: [i64; 4]) -> UniformValue<'a> {
        UniformValue::Int64Vec4(v)
    }
}

impl<'a> From<(i64, i64)> for UniformValue<'a> {
    #[inline]
    fn from(v: (i64, i64)) -> UniformValue<'a> {
        [v.0, v.1].into()
    }
}

impl<'a> From<(i64, i64, i64)> for UniformValue<'a> {
    #[inline]
    fn from(v: (i64, i64, i64)) -> UniformValue<'a> {
        [v.0, v.1, v.2].into()
    }
}

impl<'a> From<(i64, i64, i64, i64)> for UniformValue<'a> {
    #[inline]
    fn from(v: (i64, i64, i64, i64)) -> UniformValue<'a> {
        [v.0, v.1, v.2, v.3].into()
    }
}

impl<'a> From<u64> for UniformValue<'a> {
    #[inline]
    fn from(v: u64) -> UniformValue<'a> {
        UniformValue::UnsignedInt64(v)
    }
}

impl<'a> From<[u64; 2]> for UniformValue<'a> {
    #[inline]
    fn from(v: [u64; 2]) -> UniformValue<'a> {
        UniformValue::UnsignedInt64Vec2(v)
    }
}

impl<'a> From<[u64; 3]> for UniformValue<'a> {
    #[inline]
    fn from(v: [u64; 3]) -> UniformValue<'a> {
        UniformValue::UnsignedInt64Vec3(v)
    }
}

impl<'a> From<[u64; 4]> for UniformValue<'a> {
    #[inline]
    fn from(v: [u64; 4]) -> UniformValue<'a> {
        UniformValue::UnsignedInt64Vec4(v)
    }
}

impl<'a> From<(u64, u64)> for UniformValue<'a> {
    #[inline]
    fn from(v: (u64, u64)) -> UniformValue<'a> {
        [v.0, v.1].into()
    }
}

impl<'a> From<(u64, u64, u64)> for UniformValue<'a> {
    #[inline]
    fn from(v: (u64, u64, u64)) -> UniformValue<'a> {
        [v.0, v.1, v.2].into()
    }
}

impl<'a> From<(u64, u64, u64, u64)> for UniformValue<'a> {
    #[inline]
    fn from(v: (u64, u64, u64, u64)) -> UniformValue<'a> {
        [v.0, v.1, v.2, v.3].into()
    }
}

impl<'a> From<[[f32; 2]; 2]> for UniformValue<'a> {
    #[inline]
    fn from(v: [[f32; 2]; 2]) -> UniformValue<'a> {
        UniformValue::Mat2(v)
    }
}

impl<'a> From<[[f32; 3]; 3]> for UniformValue<'a> {
    #[inline]
    fn from(v: [[f32; 3]; 3]) -> UniformValue<'a> {
        UniformValue::Mat3(v)
    }
}

impl<'a> From<[[f32; 4]; 4]> for UniformValue<'a> {
    #[inline]
    fn from(v: [[f32; 4]; 4]) -> UniformValue<'a> {
        UniformValue::Mat4(v)
    }
}

impl<'a> From<[[f64; 2]; 2]> for UniformValue<'a> {
    #[inline]
    fn from(v: [[f64; 2]; 2]) -> UniformValue<'a> {
        UniformValue::DoubleMat2(v)
    }
}

impl<'a> From<[[f64; 3]; 3]> for UniformValue<'a> {
    #[inline]
    fn from(v: [[f64; 3]; 3]) -> UniformValue<'a> {
        UniformValue::DoubleMat3(v)
    }
}

impl<'a> From<[[f64; 4]; 4]> for UniformValue<'a> {
    #[inline]
    fn from(v: [[f64; 4]; 4]) -> UniformValue<'a> {
        UniformValue::DoubleMat4(v)
    }
}

impl<'a> From<(&'a str, ShaderStage)> for UniformValue<'a> {
    #[inline]
    fn from(v: (&'a str, ShaderStage)) -> UniformValue<'a> {
        UniformValue::Subroutine(v.1, v.0)
    }
}

impl_uniform_block_basic!(u32, UniformType::UnsignedInt);
impl_uniform_block_basic!(i32, UniformType::Int);
impl_uniform_block_basic!((i32, i32), UniformType::IntVec2);
impl_uniform_block_basic!((i32, i32, i32), UniformType::IntVec3);
impl_uniform_block_basic!((i32, i32, i32, i32), UniformType::IntVec4);
impl_uniform_block_basic!([i32; 2], UniformType::IntVec2);
impl_uniform_block_basic!([i32; 3], UniformType::IntVec3);
impl_uniform_block_basic!([i32; 4], UniformType::IntVec4);
impl_uniform_block_basic!((u32, u32), UniformType::UnsignedIntVec2);
impl_uniform_block_basic!((u32, u32, u32), UniformType::UnsignedIntVec3);
impl_uniform_block_basic!((u32, u32, u32, u32), UniformType::UnsignedIntVec4);
impl_uniform_block_basic!([u32; 2], UniformType::UnsignedIntVec2);
impl_uniform_block_basic!([u32; 3], UniformType::UnsignedIntVec3);
impl_uniform_block_basic!([u32; 4], UniformType::UnsignedIntVec4);

impl_uniform_block_basic!(bool, UniformType::Bool);
impl_uniform_block_basic!((bool, bool), UniformType::BoolVec2);
impl_uniform_block_basic!((bool, bool, bool), UniformType::BoolVec3);
impl_uniform_block_basic!((bool, bool, bool, bool), UniformType::BoolVec4);
impl_uniform_block_basic!([bool; 2], UniformType::BoolVec2);
impl_uniform_block_basic!([bool; 3], UniformType::BoolVec3);
impl_uniform_block_basic!([bool; 4], UniformType::BoolVec4);

impl_uniform_block_basic!(f32, UniformType::Float);
impl_uniform_block_basic!((f32, f32), UniformType::FloatVec2);
impl_uniform_block_basic!((f32, f32, f32), UniformType::FloatVec3);
impl_uniform_block_basic!((f32, f32, f32, f32), UniformType::FloatVec4);
impl_uniform_block_basic!([f32; 2], UniformType::FloatVec2);
impl_uniform_block_basic!([f32; 3], UniformType::FloatVec3);
impl_uniform_block_basic!([f32; 4], UniformType::FloatVec4);

impl_uniform_block_basic!([[f32; 2]; 2], UniformType::FloatMat2);
impl_uniform_block_basic!([[f32; 3]; 3], UniformType::FloatMat3);
impl_uniform_block_basic!([[f32; 4]; 4], UniformType::FloatMat4);

impl_uniform_block_basic!([f64; 2], UniformType::DoubleVec2);
impl_uniform_block_basic!([f64; 3], UniformType::DoubleVec3);
impl_uniform_block_basic!([f64; 4], UniformType::DoubleVec4);
impl_uniform_block_basic!(f64, UniformType::Double);
impl_uniform_block_basic!((f64, f64), UniformType::DoubleVec2);
impl_uniform_block_basic!((f64, f64, f64), UniformType::DoubleVec3);
impl_uniform_block_basic!((f64, f64, f64, f64), UniformType::DoubleVec4);

impl_uniform_block_basic!(i64, UniformType::Int);
impl_uniform_block_basic!((i64, i64), UniformType::Int64Vec2);
impl_uniform_block_basic!((i64, i64, i64), UniformType::Int64Vec3);
impl_uniform_block_basic!((i64, i64, i64, i64), UniformType::Int64Vec4);

impl_uniform_block_basic!([i64; 2], UniformType::Int64Vec2);
impl_uniform_block_basic!([i64; 3], UniformType::Int64Vec3);
impl_uniform_block_basic!([i64; 4], UniformType::Int64Vec4);

impl_uniform_block_basic!(u64, UniformType::UnsignedInt64);
impl_uniform_block_basic!((u64, u64), UniformType::UnsignedInt64Vec2);
impl_uniform_block_basic!((u64, u64, u64), UniformType::UnsignedInt64Vec3);
impl_uniform_block_basic!((u64, u64, u64, u64), UniformType::UnsignedInt64Vec4);

impl_uniform_block_basic!([u64; 2], UniformType::UnsignedInt64Vec2);
impl_uniform_block_basic!([u64; 3], UniformType::UnsignedInt64Vec3);
impl_uniform_block_basic!([u64; 4], UniformType::UnsignedInt64Vec4);

impl_uniform_block_basic!([[f64; 2]; 2], UniformType::DoubleMat2);
impl_uniform_block_basic!([[f64; 3]; 3], UniformType::DoubleMat3);
impl_uniform_block_basic!([[f64; 4]; 4], UniformType::DoubleMat4);
