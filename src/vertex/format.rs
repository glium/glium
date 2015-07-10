use std::borrow::Cow;
use std::mem;

use vertex::Attribute;
use version::Api;
use version::Version;
use CapabilitiesSource;

#[cfg(feature = "cgmath")]
use cgmath;

#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AttributeType {
    I8,
    I8I8,
    I8I8I8,
    I8I8I8I8,
    U8,
    U8U8,
    U8U8U8,
    U8U8U8U8,
    I16,
    I16I16,
    I16I16I16,
    I16I16I16I16,
    U16,
    U16U16,
    U16U16U16,
    U16U16U16U16,
    I32,
    I32I32,
    I32I32I32,
    I32I32I32I32,
    U32,
    U32U32,
    U32U32U32,
    U32U32U32U32,
    F32,
    F32F32,
    F32F32F32,
    F32F32F32F32,
    /// 2x2 matrix of `f32`s
    F32x2x2,
    /// 2x3 matrix of `f32`s
    F32x2x3,
    /// 2x3 matrix of `f32`s
    F32x2x4,
    /// 3x2 matrix of `f32`s
    F32x3x2,
    /// 3x3 matrix of `f32`s
    F32x3x3,
    /// 3x4 matrix of `f32`s
    F32x3x4,
    /// 4x2 matrix of `f32`s
    F32x4x2,
    /// 4x3 matrix of `f32`s
    F32x4x3,
    /// 4x4 matrix of `f32`s
    F32x4x4,
    /// Warning: using `f64`s can be very slow.
    F64,
    /// Warning: using `f64`s can be very slow.
    F64F64,
    /// Warning: using `f64`s can be very slow.
    F64F64F64,
    /// Warning: using `f64`s can be very slow.
    F64F64F64F64,
    /// 2x2 matrix of `f64`s
    /// Warning: using `f64`s can be very slow.
    F64x2x2,
    /// 2x3 matrix of `f64`s
    /// Warning: using `f64`s can be very slow.
    F64x2x3,
    /// 2x3 matrix of `f64`s
    /// Warning: using `f64`s can be very slow.
    F64x2x4,
    /// 3x2 matrix of `f64`s
    /// Warning: using `f64`s can be very slow.
    F64x3x2,
    /// 3x3 matrix of `f64`s
    /// Warning: using `f64`s can be very slow.
    F64x3x3,
    /// 3x4 matrix of `f64`s
    /// Warning: using `f64`s can be very slow.
    F64x3x4,
    /// 4x2 matrix of `f64`s
    /// Warning: using `f64`s can be very slow.
    F64x4x2,
    /// 4x3 matrix of `f64`s
    /// Warning: using `f64`s can be very slow.
    F64x4x3,
    /// 4x4 matrix of `f64`s
    /// Warning: using `f64`s can be very slow.
    F64x4x4,
}

impl AttributeType {
    /// Returns true if the backend supports this type of attribute.
    pub fn is_supported<C>(&self, caps: &C) -> bool where C: CapabilitiesSource {
        match self {    
            &AttributeType::I8 | &AttributeType::I8I8 | &AttributeType::I8I8I8 |
            &AttributeType::I8I8I8I8 | &AttributeType::U8 | &AttributeType::U8U8 |
            &AttributeType::U8U8U8 | &AttributeType::U8U8U8U8 | &AttributeType::I16 |
            &AttributeType::I16I16 | &AttributeType::I16I16I16 | &AttributeType::I16I16I16I16 |
            &AttributeType::U16 | &AttributeType::U16U16 | &AttributeType::U16U16U16 |
            &AttributeType::U16U16U16U16 | &AttributeType::F32 |
            &AttributeType::F32F32 | &AttributeType::F32F32F32 | &AttributeType::F32F32F32F32 |
            &AttributeType::F32x2x2 | &AttributeType::F32x2x3 | &AttributeType::F32x2x4 |
            &AttributeType::F32x3x2 | &AttributeType::F32x3x3 | &AttributeType::F32x3x4 |
            &AttributeType::F32x4x2 | &AttributeType::F32x4x3 | &AttributeType::F32x4x4 => 
            {
                true
            },

            &AttributeType::I32 | &AttributeType::I32I32 | &AttributeType::I32I32I32 |
            &AttributeType::I32I32I32I32 | &AttributeType::U32 | &AttributeType::U32U32 |
            &AttributeType::U32U32U32 | &AttributeType::U32U32U32U32 =>
            {
                caps.get_version() >= &Version(Api::Gl, 1, 0) ||
                caps.get_version() >= &Version(Api::GlEs, 3, 0)
            },

            &AttributeType::F64 | &AttributeType::F64F64 | &AttributeType::F64F64F64 |
            &AttributeType::F64F64F64F64 | &AttributeType::F64x2x2 | &AttributeType::F64x2x3 |
            &AttributeType::F64x2x4 | &AttributeType::F64x3x2 | &AttributeType::F64x3x3 |
            &AttributeType::F64x3x4 | &AttributeType::F64x4x2 | &AttributeType::F64x4x3 |
            &AttributeType::F64x4x4 =>
            {
                caps.get_version() >= &Version(Api::Gl, 1, 0)
            },
        }
    }

    /// Returns the size in bytes of a value of this type.
    pub fn get_size_bytes(&self) -> usize {
        match *self {
            AttributeType::I8 => 1 * mem::size_of::<i8>(),
            AttributeType::I8I8 => 2 * mem::size_of::<i8>(),
            AttributeType::I8I8I8 => 3 * mem::size_of::<i8>(),
            AttributeType::I8I8I8I8 => 4 * mem::size_of::<i8>(),
            AttributeType::U8 => 1 * mem::size_of::<u8>(),
            AttributeType::U8U8 => 2 * mem::size_of::<u8>(),
            AttributeType::U8U8U8 => 3 * mem::size_of::<u8>(),
            AttributeType::U8U8U8U8 => 4 * mem::size_of::<u8>(),
            AttributeType::I16 => 1 * mem::size_of::<i16>(),
            AttributeType::I16I16 => 2 * mem::size_of::<i16>(),
            AttributeType::I16I16I16 => 3 * mem::size_of::<i16>(),
            AttributeType::I16I16I16I16 => 4 * mem::size_of::<i16>(),
            AttributeType::U16 => 1 * mem::size_of::<u16>(),
            AttributeType::U16U16 => 2 * mem::size_of::<u16>(),
            AttributeType::U16U16U16 => 3 * mem::size_of::<u16>(),
            AttributeType::U16U16U16U16 => 4 * mem::size_of::<u16>(),
            AttributeType::I32 => 1 * mem::size_of::<i32>(),
            AttributeType::I32I32 => 2 * mem::size_of::<i32>(),
            AttributeType::I32I32I32 => 3 * mem::size_of::<i32>(),
            AttributeType::I32I32I32I32 => 4 * mem::size_of::<i32>(),
            AttributeType::U32 => 1 * mem::size_of::<u32>(),
            AttributeType::U32U32 => 2 * mem::size_of::<u32>(),
            AttributeType::U32U32U32 => 3 * mem::size_of::<u32>(),
            AttributeType::U32U32U32U32 => 4 * mem::size_of::<u32>(),
            AttributeType::F32 => 1 * mem::size_of::<f32>(),
            AttributeType::F32F32 => 2 * mem::size_of::<f32>(),
            AttributeType::F32F32F32 => 3 * mem::size_of::<f32>(),
            AttributeType::F32F32F32F32 => 4 * mem::size_of::<f32>(),
            AttributeType::F32x2x2 => 4 * mem::size_of::<f32>(),
            AttributeType::F32x2x3 => 6 * mem::size_of::<f32>(),
            AttributeType::F32x2x4 => 8 * mem::size_of::<f32>(),
            AttributeType::F32x3x2 => 6 * mem::size_of::<f32>(),
            AttributeType::F32x3x3 => 9 * mem::size_of::<f32>(),
            AttributeType::F32x3x4 => 12 * mem::size_of::<f32>(),
            AttributeType::F32x4x2 => 8 * mem::size_of::<f32>(),
            AttributeType::F32x4x3 => 12 * mem::size_of::<f32>(),
            AttributeType::F32x4x4 => 16 * mem::size_of::<f32>(),
            AttributeType::F64 => 1 * mem::size_of::<f64>(),
            AttributeType::F64F64 => 2 * mem::size_of::<f64>(),
            AttributeType::F64F64F64 => 3 * mem::size_of::<f64>(),
            AttributeType::F64F64F64F64 => 4 * mem::size_of::<f64>(),
            AttributeType::F64x2x2 => 4 * mem::size_of::<f64>(),
            AttributeType::F64x2x3 => 6 * mem::size_of::<f64>(),
            AttributeType::F64x2x4 => 8 * mem::size_of::<f64>(),
            AttributeType::F64x3x2 => 6 * mem::size_of::<f64>(),
            AttributeType::F64x3x3 => 9 * mem::size_of::<f64>(),
            AttributeType::F64x3x4 => 12 * mem::size_of::<f64>(),
            AttributeType::F64x4x2 => 8 * mem::size_of::<f64>(),
            AttributeType::F64x4x3 => 12 * mem::size_of::<f64>(),
            AttributeType::F64x4x4 => 16 * mem::size_of::<f64>(),
        }
    }

    /// Returns the number of values for this type.
    pub fn get_num_components(&self) -> usize {
        match *self {
            AttributeType::I8 => 1,
            AttributeType::I8I8 => 2,
            AttributeType::I8I8I8 => 3,
            AttributeType::I8I8I8I8 => 4,
            AttributeType::U8 => 1,
            AttributeType::U8U8 => 2,
            AttributeType::U8U8U8 => 3,
            AttributeType::U8U8U8U8 => 4,
            AttributeType::I16 => 1,
            AttributeType::I16I16 => 2,
            AttributeType::I16I16I16 => 3,
            AttributeType::I16I16I16I16 => 4,
            AttributeType::U16 => 1,
            AttributeType::U16U16 => 2,
            AttributeType::U16U16U16 => 3,
            AttributeType::U16U16U16U16 => 4,
            AttributeType::I32 => 1,
            AttributeType::I32I32 => 2,
            AttributeType::I32I32I32 => 3,
            AttributeType::I32I32I32I32 => 4,
            AttributeType::U32 => 1,
            AttributeType::U32U32 => 2,
            AttributeType::U32U32U32 => 3,
            AttributeType::U32U32U32U32 => 4,
            AttributeType::F32 => 1,
            AttributeType::F32F32 => 2,
            AttributeType::F32F32F32 => 3,
            AttributeType::F32F32F32F32 => 4,
            AttributeType::F32x2x2 => 4,
            AttributeType::F32x2x3 => 6,
            AttributeType::F32x2x4 => 8,
            AttributeType::F32x3x2 => 6,
            AttributeType::F32x3x3 => 9,
            AttributeType::F32x3x4 => 12,
            AttributeType::F32x4x2 => 8,
            AttributeType::F32x4x3 => 12,
            AttributeType::F32x4x4 => 16,
            AttributeType::F64 => 1,
            AttributeType::F64F64 => 2,
            AttributeType::F64F64F64 => 3,
            AttributeType::F64F64F64F64 => 4,
            AttributeType::F64x2x2 => 4,
            AttributeType::F64x2x3 => 6,
            AttributeType::F64x2x4 => 8,
            AttributeType::F64x3x2 => 6,
            AttributeType::F64x3x3 => 9,
            AttributeType::F64x3x4 => 12,
            AttributeType::F64x4x2 => 8,
            AttributeType::F64x4x3 => 12,
            AttributeType::F64x4x4 => 16,
        }
    }
}

/// Describes the layout of each vertex in a vertex buffer.
///
/// The first element is the name of the binding, the second element is the offset
/// from the start of each vertex to this element, and the third element is the type.
pub type VertexFormat = Cow<'static, [(Cow<'static, str>, usize, AttributeType)]>;

unsafe impl Attribute for i8 {
    fn get_type() -> AttributeType {
        AttributeType::I8
    }
}

unsafe impl Attribute for (i8, i8) {
    fn get_type() -> AttributeType {
        AttributeType::I8I8
    }
}

unsafe impl Attribute for [i8; 2] {
    fn get_type() -> AttributeType {
        AttributeType::I8I8
    }
}

unsafe impl Attribute for (i8, i8, i8) {
    fn get_type() -> AttributeType {
        AttributeType::I8I8I8
    }
}

unsafe impl Attribute for [i8; 3] {
    fn get_type() -> AttributeType {
        AttributeType::I8I8I8
    }
}

unsafe impl Attribute for (i8, i8, i8, i8) {
    fn get_type() -> AttributeType {
        AttributeType::I8I8I8I8
    }
}

unsafe impl Attribute for [i8; 4] {
    fn get_type() -> AttributeType {
        AttributeType::I8I8I8I8
    }
}

unsafe impl Attribute for u8 {
    fn get_type() -> AttributeType {
        AttributeType::U8
    }
}

unsafe impl Attribute for (u8, u8) {
    fn get_type() -> AttributeType {
        AttributeType::U8U8
    }
}

unsafe impl Attribute for [u8; 2] {
    fn get_type() -> AttributeType {
        AttributeType::U8U8
    }
}

unsafe impl Attribute for (u8, u8, u8) {
    fn get_type() -> AttributeType {
        AttributeType::U8U8U8
    }
}

unsafe impl Attribute for [u8; 3] {
    fn get_type() -> AttributeType {
        AttributeType::U8U8U8
    }
}

unsafe impl Attribute for (u8, u8, u8, u8) {
    fn get_type() -> AttributeType {
        AttributeType::U8U8U8U8
    }
}

unsafe impl Attribute for [u8; 4] {
    fn get_type() -> AttributeType {
        AttributeType::U8U8U8U8
    }
}

unsafe impl Attribute for i16 {
    fn get_type() -> AttributeType {
        AttributeType::I16
    }
}

unsafe impl Attribute for (i16, i16) {
    fn get_type() -> AttributeType {
        AttributeType::I16I16
    }
}

unsafe impl Attribute for [i16; 2] {
    fn get_type() -> AttributeType {
        AttributeType::I16I16
    }
}

unsafe impl Attribute for (i16, i16, i16) {
    fn get_type() -> AttributeType {
        AttributeType::I16I16I16
    }
}

unsafe impl Attribute for [i16; 3] {
    fn get_type() -> AttributeType {
        AttributeType::I16I16I16
    }
}

unsafe impl Attribute for (i16, i16, i16, i16) {
    fn get_type() -> AttributeType {
        AttributeType::I16I16I16I16
    }
}

unsafe impl Attribute for [i16; 4] {
    fn get_type() -> AttributeType {
        AttributeType::I16I16I16I16
    }
}

unsafe impl Attribute for u16 {
    fn get_type() -> AttributeType {
        AttributeType::U16
    }
}

unsafe impl Attribute for (u16, u16) {
    fn get_type() -> AttributeType {
        AttributeType::U16U16
    }
}

unsafe impl Attribute for [u16; 2] {
    fn get_type() -> AttributeType {
        AttributeType::U16U16
    }
}

unsafe impl Attribute for (u16, u16, u16) {
    fn get_type() -> AttributeType {
        AttributeType::U16U16U16
    }
}

unsafe impl Attribute for [u16; 3] {
    fn get_type() -> AttributeType {
        AttributeType::U16U16U16
    }
}

unsafe impl Attribute for (u16, u16, u16, u16) {
    fn get_type() -> AttributeType {
        AttributeType::U16U16U16U16
    }
}

unsafe impl Attribute for [u16; 4] {
    fn get_type() -> AttributeType {
        AttributeType::U16U16U16U16
    }
}

unsafe impl Attribute for i32 {
    fn get_type() -> AttributeType {
        AttributeType::I32
    }
}

unsafe impl Attribute for (i32, i32) {
    fn get_type() -> AttributeType {
        AttributeType::I32I32
    }
}

unsafe impl Attribute for [i32; 2] {
    fn get_type() -> AttributeType {
        AttributeType::I32I32
    }
}

unsafe impl Attribute for (i32, i32, i32) {
    fn get_type() -> AttributeType {
        AttributeType::I32I32I32
    }
}

unsafe impl Attribute for [i32; 3] {
    fn get_type() -> AttributeType {
        AttributeType::I32I32I32
    }
}

unsafe impl Attribute for (i32, i32, i32, i32) {
    fn get_type() -> AttributeType {
        AttributeType::I32I32I32I32
    }
}

unsafe impl Attribute for [i32; 4] {
    fn get_type() -> AttributeType {
        AttributeType::I32I32I32I32
    }
}

unsafe impl Attribute for u32 {
    fn get_type() -> AttributeType {
        AttributeType::U32
    }
}

unsafe impl Attribute for (u32, u32) {
    fn get_type() -> AttributeType {
        AttributeType::U32U32
    }
}

unsafe impl Attribute for [u32; 2] {
    fn get_type() -> AttributeType {
        AttributeType::U32U32
    }
}

unsafe impl Attribute for (u32, u32, u32) {
    fn get_type() -> AttributeType {
        AttributeType::U32U32U32
    }
}

unsafe impl Attribute for [u32; 3] {
    fn get_type() -> AttributeType {
        AttributeType::U32U32U32
    }
}

unsafe impl Attribute for (u32, u32, u32, u32) {
    fn get_type() -> AttributeType {
        AttributeType::U32U32U32U32
    }
}

unsafe impl Attribute for [u32; 4] {
    fn get_type() -> AttributeType {
        AttributeType::U32U32U32U32
    }
}

unsafe impl Attribute for f32 {
    fn get_type() -> AttributeType {
        AttributeType::F32
    }
}

unsafe impl Attribute for (f32, f32) {
    fn get_type() -> AttributeType {
        AttributeType::F32F32
    }
}

unsafe impl Attribute for [f32; 2] {
    fn get_type() -> AttributeType {
        AttributeType::F32F32
    }
}

unsafe impl Attribute for (f32, f32, f32) {
    fn get_type() -> AttributeType {
        AttributeType::F32F32F32
    }
}

unsafe impl Attribute for [f32; 3] {
    fn get_type() -> AttributeType {
        AttributeType::F32F32F32
    }
}

unsafe impl Attribute for (f32, f32, f32, f32) {
    fn get_type() -> AttributeType {
        AttributeType::F32F32F32F32
    }
}

unsafe impl Attribute for [f32; 4] {
    fn get_type() -> AttributeType {
        AttributeType::F32F32F32F32
    }
}

unsafe impl Attribute for [[f32; 2]; 2] {
    fn get_type() -> AttributeType {
        AttributeType::F32x2x2
    }
}

unsafe impl Attribute for [[f32; 3]; 3] {
    fn get_type() -> AttributeType {
        AttributeType::F32x3x3
    }
}

unsafe impl Attribute for [[f32; 4]; 4] {
    fn get_type() -> AttributeType {
        AttributeType::F32x4x4
    }
}

unsafe impl Attribute for f64 {
    fn get_type() -> AttributeType {
        AttributeType::F64
    }
}

unsafe impl Attribute for (f64, f64) {
    fn get_type() -> AttributeType {
        AttributeType::F64F64
    }
}

unsafe impl Attribute for [f64; 2] {
    fn get_type() -> AttributeType {
        AttributeType::F64F64
    }
}

unsafe impl Attribute for (f64, f64, f64) {
    fn get_type() -> AttributeType {
        AttributeType::F64F64F64
    }
}

unsafe impl Attribute for [f64; 3] {
    fn get_type() -> AttributeType {
        AttributeType::F64F64F64
    }
}

unsafe impl Attribute for (f64, f64, f64, f64) {
    fn get_type() -> AttributeType {
        AttributeType::F64F64F64F64
    }
}

unsafe impl Attribute for [f64; 4] {
    fn get_type() -> AttributeType {
        AttributeType::F64F64F64F64
    }
}

unsafe impl Attribute for [[f64; 2]; 2] {
    fn get_type() -> AttributeType {
        AttributeType::F64x2x2
    }
}

unsafe impl Attribute for [[f64; 3]; 3] {
    fn get_type() -> AttributeType {
        AttributeType::F64x3x3
    }
}

unsafe impl Attribute for [[f64; 4]; 4] {
    fn get_type() -> AttributeType {
        AttributeType::F64x4x4
    }
}


#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point2<i8> {
    fn get_type() -> AttributeType {
        AttributeType::I8I8
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point3<i8> {
    fn get_type() -> AttributeType {
        AttributeType::I8I8I8
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector2<i8> {
    fn get_type() -> AttributeType {
        AttributeType::I8I8
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector3<i8> {
    fn get_type() -> AttributeType {
        AttributeType::I8I8I8
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector4<i8> {
    fn get_type() -> AttributeType {
        AttributeType::I8I8I8I8
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point2<u8> {
    fn get_type() -> AttributeType {
        AttributeType::U8U8
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point3<u8> {
    fn get_type() -> AttributeType {
        AttributeType::U8U8U8
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector2<u8> {
    fn get_type() -> AttributeType {
        AttributeType::U8U8
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector3<u8> {
    fn get_type() -> AttributeType {
        AttributeType::U8U8U8
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector4<u8> {
    fn get_type() -> AttributeType {
        AttributeType::U8U8U8U8
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point2<i16> {
    fn get_type() -> AttributeType {
        AttributeType::I16I16
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point3<i16> {
    fn get_type() -> AttributeType {
        AttributeType::I16I16I16
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector2<i16> {
    fn get_type() -> AttributeType {
        AttributeType::I16I16
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector3<i16> {
    fn get_type() -> AttributeType {
        AttributeType::I16I16I16
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector4<i16> {
    fn get_type() -> AttributeType {
        AttributeType::I16I16I16I16
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point2<u16> {
    fn get_type() -> AttributeType {
        AttributeType::U16U16
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point3<u16> {
    fn get_type() -> AttributeType {
        AttributeType::U16U16U16
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector2<u16> {
    fn get_type() -> AttributeType {
        AttributeType::U16U16
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector3<u16> {
    fn get_type() -> AttributeType {
        AttributeType::U16U16U16
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector4<u16> {
    fn get_type() -> AttributeType {
        AttributeType::U16U16U16U16
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point2<i32> {
    fn get_type() -> AttributeType {
        AttributeType::I32I32
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point3<i32> {
    fn get_type() -> AttributeType {
        AttributeType::I32I32I32
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector2<i32> {
    fn get_type() -> AttributeType {
        AttributeType::I32I32
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector3<i32> {
    fn get_type() -> AttributeType {
        AttributeType::I32I32I32
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector4<i32> {
    fn get_type() -> AttributeType {
        AttributeType::I32I32I32I32
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point2<u32> {
    fn get_type() -> AttributeType {
        AttributeType::U32U32
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point3<u32> {
    fn get_type() -> AttributeType {
        AttributeType::U32U32U32
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector2<u32> {
    fn get_type() -> AttributeType {
        AttributeType::U32U32
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector3<u32> {
    fn get_type() -> AttributeType {
        AttributeType::U32U32U32
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector4<u32> {
    fn get_type() -> AttributeType {
        AttributeType::U32U32U32U32
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point2<f32> {
    fn get_type() -> AttributeType {
        AttributeType::F32F32
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point3<f32> {
    fn get_type() -> AttributeType {
        AttributeType::F32F32F32
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector2<f32> {
    fn get_type() -> AttributeType {
        AttributeType::F32F32
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector3<f32> {
    fn get_type() -> AttributeType {
        AttributeType::F32F32F32
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector4<f32> {
    fn get_type() -> AttributeType {
        AttributeType::F32F32F32F32
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Matrix2<f32> {
    fn get_type() -> AttributeType {
        AttributeType::F32x2x2
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Matrix3<f32> {
    fn get_type() -> AttributeType {
        AttributeType::F32x3x3
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Matrix4<f32> {
    fn get_type() -> AttributeType {
        AttributeType::F32x4x4
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point2<f64> {
    fn get_type() -> AttributeType {
        AttributeType::F64F64
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point3<f64> {
    fn get_type() -> AttributeType {
        AttributeType::F64F64F64
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector2<f64> {
    fn get_type() -> AttributeType {
        AttributeType::F64F64
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector3<f64> {
    fn get_type() -> AttributeType {
        AttributeType::F64F64F64
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector4<f64> {
    fn get_type() -> AttributeType {
        AttributeType::F64F64F64F64
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Matrix2<f64> {
    fn get_type() -> AttributeType {
        AttributeType::F64x2x2
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Matrix3<f64> {
    fn get_type() -> AttributeType {
        AttributeType::F64x3x3
    }
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Matrix4<f64> {
    fn get_type() -> AttributeType {
        AttributeType::F64x4x4
    }
}




#[cfg(test)]
mod tests {
    use std::mem;

    #[cfg(feature="cgmath")]
    #[test]
    fn test_cgmath_layout() {
        use cgmath::{self, FixedArray};

        macro_rules! test_cgmath_layout {
            ($from_fixed_ref:path, $ety:ty, $ncomps:expr, $literal:expr) => {{
                // from_fixed_ref is used instead of from_fixed because the later is not yet
                // implemented due to rust-lang/rust#16418
                let vaa = $literal.clone();
                let val = $from_fixed_ref(&vaa);
                let arr: &[$ety; $ncomps] = unsafe { mem::transmute(val) };
                assert_eq!(*arr, $literal);
            }}
        }

        test_cgmath_layout!(cgmath::Vector2::from_fixed_ref, u8, 2, [0u8, 1]);
        test_cgmath_layout!(cgmath::Vector3::from_fixed_ref, u8, 3, [0u8, 1, 2]);
        test_cgmath_layout!(cgmath::Vector4::from_fixed_ref, u8, 4, [0u8, 1, 2, 3]);
        test_cgmath_layout!(cgmath::Vector2::from_fixed_ref, i8, 2, [0i8, 1]);
        test_cgmath_layout!(cgmath::Vector3::from_fixed_ref, i8, 3, [0i8, 1, 2]);
        test_cgmath_layout!(cgmath::Vector4::from_fixed_ref, i8, 4, [0i8, 1, 2, 3]);
        test_cgmath_layout!(cgmath::Point2::from_fixed_ref, u8, 2, [0u8, 1]);
        test_cgmath_layout!(cgmath::Point3::from_fixed_ref, u8, 3, [0u8, 1, 2]);
        test_cgmath_layout!(cgmath::Point2::from_fixed_ref, i8, 2, [0i8, 1]);
        test_cgmath_layout!(cgmath::Point3::from_fixed_ref, i8, 3, [0i8, 1, 2]);

        test_cgmath_layout!(cgmath::Vector2::from_fixed_ref, u16, 2, [0u16, 1]);
        test_cgmath_layout!(cgmath::Vector3::from_fixed_ref, u16, 3, [0u16, 1, 2]);
        test_cgmath_layout!(cgmath::Vector4::from_fixed_ref, u16, 4, [0u16, 1, 2, 3]);
        test_cgmath_layout!(cgmath::Vector2::from_fixed_ref, i16, 2, [0i16, 1]);
        test_cgmath_layout!(cgmath::Vector3::from_fixed_ref, i16, 3, [0i16, 1, 2]);
        test_cgmath_layout!(cgmath::Vector4::from_fixed_ref, i16, 4, [0i16, 1, 2, 3]);
        test_cgmath_layout!(cgmath::Point2::from_fixed_ref, u16, 2, [0u16, 1]);
        test_cgmath_layout!(cgmath::Point3::from_fixed_ref, u16, 3, [0u16, 1, 2]);
        test_cgmath_layout!(cgmath::Point2::from_fixed_ref, i16, 2, [0i16, 1]);
        test_cgmath_layout!(cgmath::Point3::from_fixed_ref, i16, 3, [0i16, 1, 2]);

        test_cgmath_layout!(cgmath::Vector2::from_fixed_ref, u32, 2, [0u32, 1]);
        test_cgmath_layout!(cgmath::Vector3::from_fixed_ref, u32, 3, [0u32, 1, 2]);
        test_cgmath_layout!(cgmath::Vector4::from_fixed_ref, u32, 4, [0u32, 1, 2, 3]);
        test_cgmath_layout!(cgmath::Vector2::from_fixed_ref, i32, 2, [0i32, 1]);
        test_cgmath_layout!(cgmath::Vector3::from_fixed_ref, i32, 3, [0i32, 1, 2]);
        test_cgmath_layout!(cgmath::Vector4::from_fixed_ref, i32, 4, [0i32, 1, 2, 3]);
        test_cgmath_layout!(cgmath::Point2::from_fixed_ref, u32, 2, [0u32, 1]);
        test_cgmath_layout!(cgmath::Point3::from_fixed_ref, u32, 3, [0u32, 1, 2]);
        test_cgmath_layout!(cgmath::Point2::from_fixed_ref, i32, 2, [0i32, 1]);
        test_cgmath_layout!(cgmath::Point3::from_fixed_ref, i32, 3, [0i32, 1, 2]);

        test_cgmath_layout!(cgmath::Vector2::from_fixed_ref, f32, 2, [0.0f32, 1.0]);
        test_cgmath_layout!(cgmath::Vector3::from_fixed_ref, f32, 3, [0.0f32, 1.0, 2.0]);
        test_cgmath_layout!(cgmath::Vector4::from_fixed_ref, f32, 4, [0.0f32, 1.0, 2.0, 3.0]);
        test_cgmath_layout!(cgmath::Vector2::from_fixed_ref, f64, 2, [0.0f64, 1.0]);
        test_cgmath_layout!(cgmath::Vector3::from_fixed_ref, f64, 3, [0.0f64, 1.0, 2.0]);
        test_cgmath_layout!(cgmath::Vector4::from_fixed_ref, f64, 4, [0.0f64, 1.0, 2.0, 3.0]);
        test_cgmath_layout!(cgmath::Point2::from_fixed_ref, f32, 2, [0.0f32, 1.0]);
        test_cgmath_layout!(cgmath::Point3::from_fixed_ref, f32, 3, [0.0f32, 1.0, 2.0]);
        test_cgmath_layout!(cgmath::Point2::from_fixed_ref, f64, 2, [0.0f64, 1.0]);
        test_cgmath_layout!(cgmath::Point3::from_fixed_ref, f64, 3, [0.0f64, 1.0, 2.0]);
        test_cgmath_layout!(cgmath::Matrix2::from_fixed_ref, [f32; 2], 2, [[0.0f32, 1.0],
                                                                           [2.0f32, 3.0]]);
        test_cgmath_layout!(cgmath::Matrix3::from_fixed_ref, [f32; 3], 3, [[0.0f32, 1.0, 2.0],
                                                                           [3.0f32, 4.0, 5.0],
                                                                           [6.0f32, 7.0, 8.0]]);
        test_cgmath_layout!(cgmath::Matrix4::from_fixed_ref, [f32; 4], 4, [[0.0f32, 1.0, 2.0, 3.0],
                                                                           [4.0f32, 5.0, 6.0, 7.0],
                                                                           [8.0f32, 9.0, 10.0, 11.0],
                                                                           [12.0f32, 13.0, 14.0, 15.0]]);
        test_cgmath_layout!(cgmath::Matrix2::from_fixed_ref, [f64; 2], 2, [[0.0f64, 1.0],
                                                                           [2.0f64, 3.0]]);
        test_cgmath_layout!(cgmath::Matrix3::from_fixed_ref, [f64; 3], 3, [[0.0f64, 1.0, 2.0],
                                                                           [3.0f64, 4.0, 5.0],
                                                                           [6.0f64, 7.0, 8.0]]);
        test_cgmath_layout!(cgmath::Matrix4::from_fixed_ref, [f64; 4], 4, [[0.0f64, 1.0, 2.0, 3.0],
                                                                           [4.0f64, 5.0, 6.0, 7.0],
                                                                           [8.0f64, 9.0, 10.0, 11.0],
                                                                           [12.0f64, 13.0, 14.0, 15.0]]);
    }
}
