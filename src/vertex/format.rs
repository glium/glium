use std::borrow::Cow;
use std::mem;

use crate::version::Api;
use crate::version::Version;
use crate::vertex::Attribute;
use crate::CapabilitiesSource;

#[cfg(feature = "cgmath")]
use cgmath;
#[cfg(feature = "nalgebra")]
use nalgebra;

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
    I64,
    I64I64,
    I64I64I64,
    I64I64I64I64,
    U64,
    U64U64,
    U64U64U64,
    U64U64U64U64,
    F16,
    F16F16,
    F16F16F16,
    F16F16F16F16,
    /// 2x2 matrix of `f16`s
    F16x2x2,
    /// 2x3 matrix of `f16`s
    F16x2x3,
    /// 2x3 matrix of `f16`s
    F16x2x4,
    /// 3x2 matrix of `f16`s
    F16x3x2,
    /// 3x3 matrix of `f16`s
    F16x3x3,
    /// 3x4 matrix of `f16`s
    F16x3x4,
    /// 4x2 matrix of `f16`s
    F16x4x2,
    /// 4x3 matrix of `f16`s
    F16x4x3,
    /// 4x4 matrix of `f16`s
    F16x4x4,
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
    /// From MSB to LSB: two bits for the alpha, ten bits for the blue, ten bits for the green,
    /// ten bits for the red.
    ///
    /// Corresponds to `GL_INT_2_10_10_10_REV`.
    I2I10I10I10Reversed,
    /// From MSB to LSB: two bits for the alpha, ten bits for the blue, ten bits for the green,
    /// ten bits for the red.
    ///
    /// Corresponds to `GL_UNSIGNED_INT_2_10_10_10_REV`.
    U2U10U10U10Reversed,
    /// Corresponds to `GL_INT_10_10_10_2`.
    I10I10I10I2,
    /// Corresponds to `GL_UNSIGNED_INT_10_10_10_2`.
    U10U10U10U2,
    /// Three floating points values turned into unsigned integers./
    ///
    /// Corresponds to `GL_UNSIGNED_INT_10F_11F_11F_REV`.
    F10F11F11UnsignedIntReversed,
    /// Fixed floating points. A 16bits signed value followed by the 16bits unsigned exponent.
    ///
    /// Corresponds to `GL_FIXED`.
    FixedFloatI16U16,
}

impl AttributeType {
    /// Returns true if the backend supports this type of attribute.
    pub fn is_supported<C: ?Sized>(&self, caps: &C) -> bool where C: CapabilitiesSource {
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

            &AttributeType::I64 | &AttributeType::I64I64 | &AttributeType::I64I64I64 |
            &AttributeType::I64I64I64I64 =>
            {
                caps.get_extensions().gl_nv_vertex_attrib_integer_64bit
            },

            &AttributeType::U64 | &AttributeType::U64U64 |
            &AttributeType::U64U64U64 | &AttributeType::U64U64U64U64 =>
            {
                caps.get_extensions().gl_arb_bindless_texture ||
                caps.get_extensions().gl_nv_vertex_attrib_integer_64bit
            },

            &AttributeType::F64 | &AttributeType::F64F64 | &AttributeType::F64F64F64 |
            &AttributeType::F64F64F64F64 | &AttributeType::F64x2x2 | &AttributeType::F64x2x3 |
            &AttributeType::F64x2x4 | &AttributeType::F64x3x2 | &AttributeType::F64x3x3 |
            &AttributeType::F64x3x4 | &AttributeType::F64x4x2 | &AttributeType::F64x4x3 |
            &AttributeType::F64x4x4 =>
            {
                caps.get_version() >= &Version(Api::Gl, 1, 0)
            },

            &AttributeType::F16 | &AttributeType::F16F16 | &AttributeType::F16F16F16 |
            &AttributeType::F16F16F16F16 |
            &AttributeType::F16x2x2 | &AttributeType::F16x2x3 | &AttributeType::F16x2x4 |
            &AttributeType::F16x3x2 | &AttributeType::F16x3x3 | &AttributeType::F16x3x4 |
            &AttributeType::F16x4x2 | &AttributeType::F16x4x3 | &AttributeType::F16x4x4 =>
            {
                caps.get_version() >= &Version(Api::GlEs, 3, 0) ||
                caps.get_version() >= &Version(Api::Gl, 4, 0) ||
                caps.get_extensions().gl_arb_es3_compatibility ||
                caps.get_extensions().gl_oes_vertex_half_float ||
                caps.get_extensions().gl_arb_vertex_half_float ||
                caps.get_extensions().gl_nv_half_float
            },

            &AttributeType::FixedFloatI16U16 => {
                caps.get_version() >= &Version(Api::GlEs, 2, 0) ||
                caps.get_version() >= &Version(Api::Gl, 4, 0) ||
                caps.get_extensions().gl_arb_es2_compatibility ||
                caps.get_extensions().gl_oes_fixed_point
            },

            &AttributeType::I2I10I10I10Reversed | &AttributeType::U2U10U10U10Reversed => {
                caps.get_version() >= &Version(Api::Gl, 3, 0) ||
                caps.get_version() >= &Version(Api::GlEs, 3, 0) ||
                caps.get_extensions().gl_arb_vertex_type_2_10_10_10_rev ||
                caps.get_extensions().gl_arb_es3_compatibility
            },

            &AttributeType::I10I10I10I2 | &AttributeType::U10U10U10U2 => {
                caps.get_extensions().gl_oes_vertex_type_10_10_10_2
            },

            &AttributeType::F10F11F11UnsignedIntReversed => {
                caps.get_version() >= &Version(Api::Gl, 4, 0) ||
                caps.get_extensions().gl_arb_vertex_type_10f_11f_11f_rev
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
            AttributeType::I64 => 1 * mem::size_of::<i64>(),
            AttributeType::I64I64 => 2 * mem::size_of::<i64>(),
            AttributeType::I64I64I64 => 3 * mem::size_of::<i64>(),
            AttributeType::I64I64I64I64 => 4 * mem::size_of::<i64>(),
            AttributeType::U64 => 1 * mem::size_of::<u64>(),
            AttributeType::U64U64 => 2 * mem::size_of::<u64>(),
            AttributeType::U64U64U64 => 3 * mem::size_of::<u64>(),
            AttributeType::U64U64U64U64 => 4 * mem::size_of::<u64>(),
            AttributeType::F16 => 1 * 2,
            AttributeType::F16F16 => 2 * 2,
            AttributeType::F16F16F16 => 3 * 2,
            AttributeType::F16F16F16F16 => 4 * 2,
            AttributeType::F16x2x2 => 4 * 2,
            AttributeType::F16x2x3 => 6 * 2,
            AttributeType::F16x2x4 => 8 * 2,
            AttributeType::F16x3x2 => 6 * 2,
            AttributeType::F16x3x3 => 9 * 2,
            AttributeType::F16x3x4 => 12 * 2,
            AttributeType::F16x4x2 => 8 * 2,
            AttributeType::F16x4x3 => 12 * 2,
            AttributeType::F16x4x4 => 16 * 2,
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
            AttributeType::I2I10I10I10Reversed => 4,
            AttributeType::U2U10U10U10Reversed => 4,
            AttributeType::I10I10I10I2 => 4,
            AttributeType::U10U10U10U2 => 4,
            AttributeType::F10F11F11UnsignedIntReversed => 4,
            AttributeType::FixedFloatI16U16 => 4,
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
            AttributeType::I64 => 1,
            AttributeType::I64I64 => 2,
            AttributeType::I64I64I64 => 3,
            AttributeType::I64I64I64I64 => 4,
            AttributeType::U64 => 1,
            AttributeType::U64U64 => 2,
            AttributeType::U64U64U64 => 3,
            AttributeType::U64U64U64U64 => 4,
            AttributeType::F16 => 1,
            AttributeType::F16F16 => 2,
            AttributeType::F16F16F16 => 3,
            AttributeType::F16F16F16F16 => 4,
            AttributeType::F16x2x2 => 4,
            AttributeType::F16x2x3 => 6,
            AttributeType::F16x2x4 => 8,
            AttributeType::F16x3x2 => 6,
            AttributeType::F16x3x3 => 9,
            AttributeType::F16x3x4 => 12,
            AttributeType::F16x4x2 => 8,
            AttributeType::F16x4x3 => 12,
            AttributeType::F16x4x4 => 16,
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
            AttributeType::I2I10I10I10Reversed => 4,
            AttributeType::U2U10U10U10Reversed => 4,
            AttributeType::I10I10I10I2 => 4,
            AttributeType::U10U10U10U2 => 4,
            AttributeType::F10F11F11UnsignedIntReversed => 3,
            AttributeType::FixedFloatI16U16 => 1,
        }
    }
}

/// Describes the layout of each vertex in a vertex buffer.
///
/// The first element is the name of the binding, the second element
/// is the offset from the start of each vertex to this element, the
/// third element is the layout location specified in the shader, the
/// fourth element is the type and the fifth element indicates whether
/// or not the element should use fixed-point normalization when
/// binding in a VAO.
pub type VertexFormat = &'static [(Cow<'static, str>, usize, i32, AttributeType, bool)];

unsafe impl Attribute for i8 {
    const TYPE: AttributeType = AttributeType::I8;
}

unsafe impl Attribute for (i8, i8) {
    const TYPE: AttributeType = AttributeType::I8I8;
}

unsafe impl Attribute for [i8; 2] {
    const TYPE: AttributeType = AttributeType::I8I8;
}

unsafe impl Attribute for (i8, i8, i8) {
    const TYPE: AttributeType = AttributeType::I8I8I8;
}

unsafe impl Attribute for [i8; 3] {
    const TYPE: AttributeType = AttributeType::I8I8I8;
}

unsafe impl Attribute for (i8, i8, i8, i8) {
    const TYPE: AttributeType = AttributeType::I8I8I8I8;
}

unsafe impl Attribute for [i8; 4] {
    const TYPE: AttributeType = AttributeType::I8I8I8I8;
}

unsafe impl Attribute for u8 {
    const TYPE: AttributeType = AttributeType::U8;
}

unsafe impl Attribute for (u8, u8) {
    const TYPE: AttributeType = AttributeType::U8U8;
}

unsafe impl Attribute for [u8; 2] {
    const TYPE: AttributeType = AttributeType::U8U8;
}

unsafe impl Attribute for (u8, u8, u8) {
    const TYPE: AttributeType = AttributeType::U8U8U8;
}

unsafe impl Attribute for [u8; 3] {
    const TYPE: AttributeType = AttributeType::U8U8U8;
}

unsafe impl Attribute for (u8, u8, u8, u8) {
    const TYPE: AttributeType = AttributeType::U8U8U8U8;
}

unsafe impl Attribute for [u8; 4] {
    const TYPE: AttributeType = AttributeType::U8U8U8U8;
}

unsafe impl Attribute for i16 {
    const TYPE: AttributeType = AttributeType::I16;
}

unsafe impl Attribute for (i16, i16) {
    const TYPE: AttributeType = AttributeType::I16I16;
}

unsafe impl Attribute for [i16; 2] {
    const TYPE: AttributeType = AttributeType::I16I16;
}

unsafe impl Attribute for (i16, i16, i16) {
    const TYPE: AttributeType = AttributeType::I16I16I16;
}

unsafe impl Attribute for [i16; 3] {
    const TYPE: AttributeType = AttributeType::I16I16I16;
}

unsafe impl Attribute for (i16, i16, i16, i16) {
    const TYPE: AttributeType = AttributeType::I16I16I16I16;
}

unsafe impl Attribute for [i16; 4] {
    const TYPE: AttributeType = AttributeType::I16I16I16I16;
}

unsafe impl Attribute for u16 {
    const TYPE: AttributeType = AttributeType::U16;
}

unsafe impl Attribute for (u16, u16) {
    const TYPE: AttributeType = AttributeType::U16U16;
}

unsafe impl Attribute for [u16; 2] {
    const TYPE: AttributeType = AttributeType::U16U16;
}

unsafe impl Attribute for (u16, u16, u16) {
    const TYPE: AttributeType = AttributeType::U16U16U16;
}

unsafe impl Attribute for [u16; 3] {
    const TYPE: AttributeType = AttributeType::U16U16U16;
}

unsafe impl Attribute for (u16, u16, u16, u16) {
    const TYPE: AttributeType = AttributeType::U16U16U16U16;
}

unsafe impl Attribute for [u16; 4] {
    const TYPE: AttributeType = AttributeType::U16U16U16U16;
}

unsafe impl Attribute for i32 {
    const TYPE: AttributeType = AttributeType::I32;
}

unsafe impl Attribute for (i32, i32) {
    const TYPE: AttributeType = AttributeType::I32I32;
}

unsafe impl Attribute for [i32; 2] {
    const TYPE: AttributeType = AttributeType::I32I32;
}

unsafe impl Attribute for (i32, i32, i32) {
    const TYPE: AttributeType = AttributeType::I32I32I32;
}

unsafe impl Attribute for [i32; 3] {
    const TYPE: AttributeType = AttributeType::I32I32I32;
}

unsafe impl Attribute for (i32, i32, i32, i32) {
    const TYPE: AttributeType = AttributeType::I32I32I32I32;
}

unsafe impl Attribute for [i32; 4] {
    const TYPE: AttributeType = AttributeType::I32I32I32I32;
}

unsafe impl Attribute for u32 {
    const TYPE: AttributeType = AttributeType::U32;
}

unsafe impl Attribute for (u32, u32) {
    const TYPE: AttributeType = AttributeType::U32U32;
}

unsafe impl Attribute for [u32; 2] {
    const TYPE: AttributeType = AttributeType::U32U32;
}

unsafe impl Attribute for (u32, u32, u32) {
    const TYPE: AttributeType = AttributeType::U32U32U32;
}

unsafe impl Attribute for [u32; 3] {
    const TYPE: AttributeType = AttributeType::U32U32U32;
}

unsafe impl Attribute for (u32, u32, u32, u32) {
    const TYPE: AttributeType = AttributeType::U32U32U32U32;
}

unsafe impl Attribute for [u32; 4] {
    const TYPE: AttributeType = AttributeType::U32U32U32U32;
}

unsafe impl Attribute for i64 {
    const TYPE: AttributeType = AttributeType::I64;
}

unsafe impl Attribute for (i64, i64) {
    const TYPE: AttributeType = AttributeType::I64I64;
}

unsafe impl Attribute for [i64; 2] {
    const TYPE: AttributeType = AttributeType::I64I64;
}

unsafe impl Attribute for (i64, i64, i64) {
    const TYPE: AttributeType = AttributeType::I64I64I64;
}

unsafe impl Attribute for [i64; 3] {
    const TYPE: AttributeType = AttributeType::I64I64I64;
}

unsafe impl Attribute for (i64, i64, i64, i64) {
    const TYPE: AttributeType = AttributeType::I64I64I64I64;
}

unsafe impl Attribute for [i64; 4] {
    const TYPE: AttributeType = AttributeType::I64I64I64I64;
}

unsafe impl Attribute for u64 {
    const TYPE: AttributeType = AttributeType::U64;
}

unsafe impl Attribute for (u64, u64) {
    const TYPE: AttributeType = AttributeType::U64U64;
}

unsafe impl Attribute for [u64; 2] {
    const TYPE: AttributeType = AttributeType::U64U64;
}

unsafe impl Attribute for (u64, u64, u64) {
    const TYPE: AttributeType = AttributeType::U64U64U64;
}

unsafe impl Attribute for [u64; 3] {
    const TYPE: AttributeType = AttributeType::U64U64U64;
}

unsafe impl Attribute for (u64, u64, u64, u64) {
    const TYPE: AttributeType = AttributeType::U64U64U64U64;
}

unsafe impl Attribute for [u64; 4] {
    const TYPE: AttributeType = AttributeType::U64U64U64U64;
}

unsafe impl Attribute for f32 {
    const TYPE: AttributeType = AttributeType::F32;
}

unsafe impl Attribute for (f32, f32) {
    const TYPE: AttributeType = AttributeType::F32F32;
}

unsafe impl Attribute for [f32; 2] {
    const TYPE: AttributeType = AttributeType::F32F32;
}

unsafe impl Attribute for (f32, f32, f32) {
    const TYPE: AttributeType = AttributeType::F32F32F32;
}

unsafe impl Attribute for [f32; 3] {
    const TYPE: AttributeType = AttributeType::F32F32F32;
}

unsafe impl Attribute for (f32, f32, f32, f32) {
    const TYPE: AttributeType = AttributeType::F32F32F32F32;
}

unsafe impl Attribute for [f32; 4] {
    const TYPE: AttributeType = AttributeType::F32F32F32F32;
}

unsafe impl Attribute for [[f32; 2]; 2] {
    const TYPE: AttributeType = AttributeType::F32x2x2;
}

unsafe impl Attribute for [[f32; 3]; 3] {
    const TYPE: AttributeType = AttributeType::F32x3x3;
}

unsafe impl Attribute for [[f32; 4]; 4] {
    const TYPE: AttributeType = AttributeType::F32x4x4;
}

unsafe impl Attribute for f64 {
    const TYPE: AttributeType = AttributeType::F64;
}

unsafe impl Attribute for (f64, f64) {
    const TYPE: AttributeType = AttributeType::F64F64;
}

unsafe impl Attribute for [f64; 2] {
    const TYPE: AttributeType = AttributeType::F64F64;
}

unsafe impl Attribute for (f64, f64, f64) {
    const TYPE: AttributeType = AttributeType::F64F64F64;
}

unsafe impl Attribute for [f64; 3] {
    const TYPE: AttributeType = AttributeType::F64F64F64;
}

unsafe impl Attribute for (f64, f64, f64, f64) {
    const TYPE: AttributeType = AttributeType::F64F64F64F64;
}

unsafe impl Attribute for [f64; 4] {
    const TYPE: AttributeType = AttributeType::F64F64F64F64;
}

unsafe impl Attribute for [[f64; 2]; 2] {
    const TYPE: AttributeType = AttributeType::F64x2x2;
}

unsafe impl Attribute for [[f64; 3]; 3] {
    const TYPE: AttributeType = AttributeType::F64x3x3;
}

unsafe impl Attribute for [[f64; 4]; 4] {
    const TYPE: AttributeType = AttributeType::F64x4x4;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point2<i8> {
    const TYPE: AttributeType = AttributeType::I8I8;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point3<i8> {
    const TYPE: AttributeType = AttributeType::I8I8I8;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector2<i8> {
    const TYPE: AttributeType = AttributeType::I8I8;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector3<i8> {
    const TYPE: AttributeType = AttributeType::I8I8I8;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector4<i8> {
    const TYPE: AttributeType = AttributeType::I8I8I8I8;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point2<u8> {
    const TYPE: AttributeType = AttributeType::U8U8;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point3<u8> {
    const TYPE: AttributeType = AttributeType::U8U8U8;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector2<u8> {
    const TYPE: AttributeType = AttributeType::U8U8;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector3<u8> {
    const TYPE: AttributeType = AttributeType::U8U8U8;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector4<u8> {
    const TYPE: AttributeType = AttributeType::U8U8U8U8;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point2<i16> {
    const TYPE: AttributeType = AttributeType::I16I16;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point3<i16> {
    const TYPE: AttributeType = AttributeType::I16I16I16;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector2<i16> {
    const TYPE: AttributeType = AttributeType::I16I16;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector3<i16> {
    const TYPE: AttributeType = AttributeType::I16I16I16;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector4<i16> {
    const TYPE: AttributeType = AttributeType::I16I16I16I16;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point2<u16> {
    const TYPE: AttributeType = AttributeType::U16U16;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point3<u16> {
    const TYPE: AttributeType = AttributeType::U16U16U16;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector2<u16> {
    const TYPE: AttributeType = AttributeType::U16U16;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector3<u16> {
    const TYPE: AttributeType = AttributeType::U16U16U16;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector4<u16> {
    const TYPE: AttributeType = AttributeType::U16U16U16U16;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point2<i32> {
    const TYPE: AttributeType = AttributeType::I32I32;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point3<i32> {
    const TYPE: AttributeType = AttributeType::I32I32I32;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector2<i32> {
    const TYPE: AttributeType = AttributeType::I32I32;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector3<i32> {
    const TYPE: AttributeType = AttributeType::I32I32I32;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector4<i32> {
    const TYPE: AttributeType = AttributeType::I32I32I32I32;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point2<u32> {
    const TYPE: AttributeType = AttributeType::U32U32;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point3<u32> {
    const TYPE: AttributeType = AttributeType::U32U32U32;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector2<u32> {
    const TYPE: AttributeType = AttributeType::U32U32;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector3<u32> {
    const TYPE: AttributeType = AttributeType::U32U32U32;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector4<u32> {
    const TYPE: AttributeType = AttributeType::U32U32U32U32;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point2<i64> {
    const TYPE: AttributeType = AttributeType::I64I64;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point3<i64> {
    const TYPE: AttributeType = AttributeType::I64I64I64;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector2<i64> {
    const TYPE: AttributeType = AttributeType::I64I64;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector3<i64> {
    const TYPE: AttributeType = AttributeType::I64I64I64;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector4<i64> {
    const TYPE: AttributeType = AttributeType::I64I64I64I64;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point2<u64> {
    const TYPE: AttributeType = AttributeType::U64U64;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point3<u64> {
    const TYPE: AttributeType = AttributeType::U64U64U64;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector2<u64> {
    const TYPE: AttributeType = AttributeType::U64U64;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector3<u64> {
    const TYPE: AttributeType = AttributeType::U64U64U64;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector4<u64> {
    const TYPE: AttributeType = AttributeType::U64U64U64U64;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point2<f32> {
    const TYPE: AttributeType = AttributeType::F32F32;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point3<f32> {
    const TYPE: AttributeType = AttributeType::F32F32F32;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector2<f32> {
    const TYPE: AttributeType = AttributeType::F32F32;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector3<f32> {
    const TYPE: AttributeType = AttributeType::F32F32F32;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector4<f32> {
    const TYPE: AttributeType = AttributeType::F32F32F32F32;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Matrix2<f32> {
    const TYPE: AttributeType = AttributeType::F32x2x2;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Matrix3<f32> {
    const TYPE: AttributeType = AttributeType::F32x3x3;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Matrix4<f32> {
    const TYPE: AttributeType = AttributeType::F32x4x4;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point2<f64> {
    const TYPE: AttributeType = AttributeType::F64F64;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Point3<f64> {
    const TYPE: AttributeType = AttributeType::F64F64F64;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector2<f64> {
    const TYPE: AttributeType = AttributeType::F64F64;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector3<f64> {
    const TYPE: AttributeType = AttributeType::F64F64F64;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Vector4<f64> {
    const TYPE: AttributeType = AttributeType::F64F64F64F64;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Matrix2<f64> {
    const TYPE: AttributeType = AttributeType::F64x2x2;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Matrix3<f64> {
    const TYPE: AttributeType = AttributeType::F64x3x3;
}

#[cfg(feature="cgmath")]
unsafe impl Attribute for cgmath::Matrix4<f64> {
    const TYPE: AttributeType = AttributeType::F64x4x4;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt1<i8> {
    const TYPE: AttributeType = AttributeType::I8;
}
#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt2<i8> {
    const TYPE: AttributeType = AttributeType::I8I8;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt3<i8> {
    const TYPE: AttributeType = AttributeType::I8I8I8;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt4<i8> {
    const TYPE: AttributeType = AttributeType::I8I8I8I8;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec1<i8> {
    const TYPE: AttributeType = AttributeType::I8;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec2<i8> {
    const TYPE: AttributeType = AttributeType::I8I8;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec3<i8> {
    const TYPE: AttributeType = AttributeType::I8I8I8;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec4<i8> {
    const TYPE: AttributeType = AttributeType::I8I8I8I8;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt1<u8> {
    const TYPE: AttributeType = AttributeType::U8;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt2<u8> {
    const TYPE: AttributeType = AttributeType::U8U8;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt3<u8> {
    const TYPE: AttributeType = AttributeType::U8U8U8;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt4<u8> {
    const TYPE: AttributeType = AttributeType::U8U8U8U8;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec1<u8> {
    const TYPE: AttributeType = AttributeType::U8;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec2<u8> {
    const TYPE: AttributeType = AttributeType::U8U8;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec3<u8> {
    const TYPE: AttributeType = AttributeType::U8U8U8;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec4<u8> {
    const TYPE: AttributeType = AttributeType::U8U8U8U8;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt1<i16> {
    const TYPE: AttributeType = AttributeType::I16;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt2<i16> {
    const TYPE: AttributeType = AttributeType::I16I16;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt3<i16> {
    const TYPE: AttributeType = AttributeType::I16I16I16;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt4<i16> {
    const TYPE: AttributeType = AttributeType::I16I16I16I16;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec1<i16> {
    const TYPE: AttributeType = AttributeType::I16;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec2<i16> {
    const TYPE: AttributeType = AttributeType::I16I16;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec3<i16> {
    const TYPE: AttributeType = AttributeType::I16I16I16;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec4<i16> {
    const TYPE: AttributeType = AttributeType::I16I16I16I16;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt1<u16> {
    const TYPE: AttributeType = AttributeType::U16;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt2<u16> {
    const TYPE: AttributeType = AttributeType::U16U16;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt3<u16> {
    const TYPE: AttributeType = AttributeType::U16U16U16;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt4<u16> {
    const TYPE: AttributeType = AttributeType::U16U16U16U16;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec1<u16> {
    const TYPE: AttributeType = AttributeType::U16;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec2<u16> {
    const TYPE: AttributeType = AttributeType::U16U16;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec3<u16> {
    const TYPE: AttributeType = AttributeType::U16U16U16;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec4<u16> {
    const TYPE: AttributeType = AttributeType::U16U16U16U16;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt1<i32> {
    const TYPE: AttributeType = AttributeType::I32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt2<i32> {
    const TYPE: AttributeType = AttributeType::I32I32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt3<i32> {
    const TYPE: AttributeType = AttributeType::I32I32I32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt4<i32> {
    const TYPE: AttributeType = AttributeType::I32I32I32I32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec1<i32> {
    const TYPE: AttributeType = AttributeType::I32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec2<i32> {
    const TYPE: AttributeType = AttributeType::I32I32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec3<i32> {
    const TYPE: AttributeType = AttributeType::I32I32I32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec4<i32> {
    const TYPE: AttributeType = AttributeType::I32I32I32I32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt1<u32> {
    const TYPE: AttributeType = AttributeType::U32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt2<u32> {
    const TYPE: AttributeType = AttributeType::U32U32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt3<u32> {
    const TYPE: AttributeType = AttributeType::U32U32U32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt4<u32> {
    const TYPE: AttributeType = AttributeType::U32U32U32U32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec1<u32> {
    const TYPE: AttributeType = AttributeType::U32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec2<u32> {
    const TYPE: AttributeType = AttributeType::U32U32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec3<u32> {
    const TYPE: AttributeType = AttributeType::U32U32U32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec4<u32> {
    const TYPE: AttributeType = AttributeType::U32U32U32U32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt1<i64> {
    const TYPE: AttributeType = AttributeType::I64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt2<i64> {
    const TYPE: AttributeType = AttributeType::I64I64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt3<i64> {
    const TYPE: AttributeType = AttributeType::I64I64I64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt4<i64> {
    const TYPE: AttributeType = AttributeType::I64I64I64I64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec1<i64> {
    const TYPE: AttributeType = AttributeType::I64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec2<i64> {
    const TYPE: AttributeType = AttributeType::I64I64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec3<i64> {
    const TYPE: AttributeType = AttributeType::I64I64I64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec4<i64> {
    const TYPE: AttributeType = AttributeType::I64I64I64I64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt1<u64> {
    const TYPE: AttributeType = AttributeType::U64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt2<u64> {
    const TYPE: AttributeType = AttributeType::U64U64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt3<u64> {
    const TYPE: AttributeType = AttributeType::U64U64U64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt4<u64> {
    const TYPE: AttributeType = AttributeType::U64U64U64U64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec1<u64> {
    const TYPE: AttributeType = AttributeType::U64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec2<u64> {
    const TYPE: AttributeType = AttributeType::U64U64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec3<u64> {
    const TYPE: AttributeType = AttributeType::U64U64U64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec4<u64> {
    const TYPE: AttributeType = AttributeType::U64U64U64U64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt1<f32> {
    const TYPE: AttributeType = AttributeType::F32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt2<f32> {
    const TYPE: AttributeType = AttributeType::F32F32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt3<f32> {
    const TYPE: AttributeType = AttributeType::F32F32F32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt4<f32> {
    const TYPE: AttributeType = AttributeType::F32F32F32F32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec1<f32> {
    const TYPE: AttributeType = AttributeType::F32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec2<f32> {
    const TYPE: AttributeType = AttributeType::F32F32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec3<f32> {
    const TYPE: AttributeType = AttributeType::F32F32F32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec4<f32> {
    const TYPE: AttributeType = AttributeType::F32F32F32F32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Mat1<f32> {
    const TYPE: AttributeType = AttributeType::F32;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Mat2<f32> {
    const TYPE: AttributeType = AttributeType::F32x2x2;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Mat3<f32> {
    const TYPE: AttributeType = AttributeType::F32x3x3;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Mat4<f32> {
    const TYPE: AttributeType = AttributeType::F32x4x4;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt1<f64> {
    const TYPE: AttributeType = AttributeType::F64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt2<f64> {
    const TYPE: AttributeType = AttributeType::F64F64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt3<f64> {
    const TYPE: AttributeType = AttributeType::F64F64F64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Pnt4<f64> {
    const TYPE: AttributeType = AttributeType::F64F64F64F64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec1<f64> {
    const TYPE: AttributeType = AttributeType::F64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec2<f64> {
    const TYPE: AttributeType = AttributeType::F64F64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec3<f64> {
    const TYPE: AttributeType = AttributeType::F64F64F64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Vec4<f64> {
    const TYPE: AttributeType = AttributeType::F64F64F64F64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Mat1<f64> {
    const TYPE: AttributeType = AttributeType::F64;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Mat2<f64> {
    const TYPE: AttributeType = AttributeType::F64x2x2;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Mat3<f64> {
    const TYPE: AttributeType = AttributeType::F64x3x3;
}

#[cfg(feature="nalgebra")]
unsafe impl Attribute for nalgebra::Mat4<f64> {
    const TYPE: AttributeType = AttributeType::F64x4x4;
}


#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use std::mem;

    #[cfg(feature="cgmath")]
    macro_rules! test_layout_val {
        ($from_val:path, $ety:ty, $ncomps:expr, $literal:expr) => {{
            let arr: [$ety; $ncomps] = unsafe { mem::transmute($from_val($literal)) };
            assert_eq!(arr, $literal);
        }}
    }

    #[cfg(feature = "nalgebra")]
    macro_rules! test_layout_ref {
        ($from_ref:path, $ety:ty, $ncomps:expr, $literal:expr) => {{
            let arr: &[$ety; $ncomps] = unsafe { mem::transmute($from_ref(&$literal)) };
            assert_eq!(*arr, $literal);
        }}
    }

    #[cfg(feature="cgmath")]
    #[test]
    fn test_cgmath_layout() {
        use cgmath;

        test_layout_val!(cgmath::Vector2::from, u8, 2, [0u8, 1]);
        test_layout_val!(cgmath::Vector3::from, u8, 3, [0u8, 1, 2]);
        test_layout_val!(cgmath::Vector4::from, u8, 4, [0u8, 1, 2, 3]);
        test_layout_val!(cgmath::Vector2::from, i8, 2, [0i8, 1]);
        test_layout_val!(cgmath::Vector3::from, i8, 3, [0i8, 1, 2]);
        test_layout_val!(cgmath::Vector4::from, i8, 4, [0i8, 1, 2, 3]);
        test_layout_val!(cgmath::Point2::from, u8, 2, [0u8, 1]);
        test_layout_val!(cgmath::Point3::from, u8, 3, [0u8, 1, 2]);
        test_layout_val!(cgmath::Point2::from, i8, 2, [0i8, 1]);
        test_layout_val!(cgmath::Point3::from, i8, 3, [0i8, 1, 2]);

        test_layout_val!(cgmath::Vector2::from, u16, 2, [0u16, 1]);
        test_layout_val!(cgmath::Vector3::from, u16, 3, [0u16, 1, 2]);
        test_layout_val!(cgmath::Vector4::from, u16, 4, [0u16, 1, 2, 3]);
        test_layout_val!(cgmath::Vector2::from, i16, 2, [0i16, 1]);
        test_layout_val!(cgmath::Vector3::from, i16, 3, [0i16, 1, 2]);
        test_layout_val!(cgmath::Vector4::from, i16, 4, [0i16, 1, 2, 3]);
        test_layout_val!(cgmath::Point2::from, u16, 2, [0u16, 1]);
        test_layout_val!(cgmath::Point3::from, u16, 3, [0u16, 1, 2]);
        test_layout_val!(cgmath::Point2::from, i16, 2, [0i16, 1]);
        test_layout_val!(cgmath::Point3::from, i16, 3, [0i16, 1, 2]);

        test_layout_val!(cgmath::Vector2::from, u32, 2, [0u32, 1]);
        test_layout_val!(cgmath::Vector3::from, u32, 3, [0u32, 1, 2]);
        test_layout_val!(cgmath::Vector4::from, u32, 4, [0u32, 1, 2, 3]);
        test_layout_val!(cgmath::Vector2::from, i32, 2, [0i32, 1]);
        test_layout_val!(cgmath::Vector3::from, i32, 3, [0i32, 1, 2]);
        test_layout_val!(cgmath::Vector4::from, i32, 4, [0i32, 1, 2, 3]);
        test_layout_val!(cgmath::Point2::from, u32, 2, [0u32, 1]);
        test_layout_val!(cgmath::Point3::from, u32, 3, [0u32, 1, 2]);
        test_layout_val!(cgmath::Point2::from, i32, 2, [0i32, 1]);
        test_layout_val!(cgmath::Point3::from, i32, 3, [0i32, 1, 2]);

        test_layout_val!(cgmath::Vector2::from, f32, 2, [0.0f32, 1.0]);
        test_layout_val!(cgmath::Vector3::from, f32, 3, [0.0f32, 1.0, 2.0]);
        test_layout_val!(cgmath::Vector4::from, f32, 4, [0.0f32, 1.0, 2.0, 3.0]);
        test_layout_val!(cgmath::Vector2::from, f64, 2, [0.0f64, 1.0]);
        test_layout_val!(cgmath::Vector3::from, f64, 3, [0.0f64, 1.0, 2.0]);
        test_layout_val!(cgmath::Vector4::from, f64, 4, [0.0f64, 1.0, 2.0, 3.0]);
        test_layout_val!(cgmath::Point2::from, f32, 2, [0.0f32, 1.0]);
        test_layout_val!(cgmath::Point3::from, f32, 3, [0.0f32, 1.0, 2.0]);
        test_layout_val!(cgmath::Point2::from, f64, 2, [0.0f64, 1.0]);
        test_layout_val!(cgmath::Point3::from, f64, 3, [0.0f64, 1.0, 2.0]);

        test_layout_val!(cgmath::Matrix2::from, [f32; 2], 2, [[0.0f32, 1.0],
                                                              [2.0f32, 3.0]]);
        test_layout_val!(cgmath::Matrix3::from, [f32; 3], 3, [[0.0f32, 1.0, 2.0],
                                                              [3.0f32, 4.0, 5.0],
                                                              [6.0f32, 7.0, 8.0]]);
        test_layout_val!(cgmath::Matrix4::from, [f32; 4], 4, [[0.0f32, 1.0, 2.0, 3.0],
                                                              [4.0f32, 5.0, 6.0, 7.0],
                                                              [8.0f32, 9.0, 10.0, 11.0],
                                                              [12.0f32, 13.0, 14.0, 15.0]]);

        test_layout_val!(cgmath::Matrix2::from, [f64; 2], 2, [[0.0f64, 1.0],
                                                              [2.0f64, 3.0]]);
        test_layout_val!(cgmath::Matrix3::from, [f64; 3], 3, [[0.0f64, 1.0, 2.0],
                                                              [3.0f64, 4.0, 5.0],
                                                              [6.0f64, 7.0, 8.0]]);
        test_layout_val!(cgmath::Matrix4::from, [f64; 4], 4, [[0.0f64, 1.0, 2.0, 3.0],
                                                              [4.0f64, 5.0, 6.0, 7.0],
                                                              [8.0f64, 9.0, 10.0, 11.0],
                                                              [12.0f64, 13.0, 14.0, 15.0]]);
    }

    #[cfg(feature = "nalgebra")]
    #[test]
    fn test_nalgebra_layout() {
        use nalgebra;

        test_layout_ref!(nalgebra::Vec1::from_array_ref, u8, 1, [0u8]);
        test_layout_ref!(nalgebra::Vec2::from_array_ref, u8, 2, [0u8, 1]);
        test_layout_ref!(nalgebra::Vec3::from_array_ref, u8, 3, [0u8, 1, 2]);
        test_layout_ref!(nalgebra::Vec4::from_array_ref, u8, 4, [0u8, 1, 2, 3]);
        test_layout_ref!(nalgebra::Vec1::from_array_ref, i8, 1, [0i8]);
        test_layout_ref!(nalgebra::Vec2::from_array_ref, i8, 2, [0i8, 1]);
        test_layout_ref!(nalgebra::Vec3::from_array_ref, i8, 3, [0i8, 1, 2]);
        test_layout_ref!(nalgebra::Vec4::from_array_ref, i8, 4, [0i8, 1, 2, 3]);
        test_layout_ref!(nalgebra::Pnt1::from_array_ref, u8, 1, [0u8]);
        test_layout_ref!(nalgebra::Pnt2::from_array_ref, u8, 2, [0u8, 1]);
        test_layout_ref!(nalgebra::Pnt3::from_array_ref, u8, 3, [0u8, 1, 2]);
        test_layout_ref!(nalgebra::Pnt4::from_array_ref, u8, 4, [0u8, 1, 2, 3]);
        test_layout_ref!(nalgebra::Pnt1::from_array_ref, i8, 1, [0i8]);
        test_layout_ref!(nalgebra::Pnt2::from_array_ref, i8, 2, [0i8, 1]);
        test_layout_ref!(nalgebra::Pnt3::from_array_ref, i8, 3, [0i8, 1, 2]);
        test_layout_ref!(nalgebra::Pnt4::from_array_ref, i8, 4, [0i8, 1, 2, 3]);

        test_layout_ref!(nalgebra::Vec1::from_array_ref, u16, 1, [0u16]);
        test_layout_ref!(nalgebra::Vec2::from_array_ref, u16, 2, [0u16, 1]);
        test_layout_ref!(nalgebra::Vec3::from_array_ref, u16, 3, [0u16, 1, 2]);
        test_layout_ref!(nalgebra::Vec4::from_array_ref, u16, 4, [0u16, 1, 2, 3]);
        test_layout_ref!(nalgebra::Vec1::from_array_ref, i16, 1, [0i16]);
        test_layout_ref!(nalgebra::Vec2::from_array_ref, i16, 2, [0i16, 1]);
        test_layout_ref!(nalgebra::Vec3::from_array_ref, i16, 3, [0i16, 1, 2]);
        test_layout_ref!(nalgebra::Vec4::from_array_ref, i16, 4, [0i16, 1, 2, 3]);
        test_layout_ref!(nalgebra::Pnt1::from_array_ref, u16, 1, [0u16]);
        test_layout_ref!(nalgebra::Pnt2::from_array_ref, u16, 2, [0u16, 1]);
        test_layout_ref!(nalgebra::Pnt3::from_array_ref, u16, 3, [0u16, 1, 2]);
        test_layout_ref!(nalgebra::Pnt4::from_array_ref, u16, 4, [0u16, 1, 2, 3]);
        test_layout_ref!(nalgebra::Pnt1::from_array_ref, i16, 1, [0i16]);
        test_layout_ref!(nalgebra::Pnt2::from_array_ref, i16, 2, [0i16, 1]);
        test_layout_ref!(nalgebra::Pnt3::from_array_ref, i16, 3, [0i16, 1, 2]);
        test_layout_ref!(nalgebra::Pnt4::from_array_ref, i16, 4, [0i16, 1, 2, 3]);

        test_layout_ref!(nalgebra::Vec1::from_array_ref, u32, 1, [0u32]);
        test_layout_ref!(nalgebra::Vec2::from_array_ref, u32, 2, [0u32, 1]);
        test_layout_ref!(nalgebra::Vec3::from_array_ref, u32, 3, [0u32, 1, 2]);
        test_layout_ref!(nalgebra::Vec4::from_array_ref, u32, 4, [0u32, 1, 2, 3]);
        test_layout_ref!(nalgebra::Vec1::from_array_ref, i32, 1, [0i32]);
        test_layout_ref!(nalgebra::Vec2::from_array_ref, i32, 2, [0i32, 1]);
        test_layout_ref!(nalgebra::Vec3::from_array_ref, i32, 3, [0i32, 1, 2]);
        test_layout_ref!(nalgebra::Vec4::from_array_ref, i32, 4, [0i32, 1, 2, 3]);
        test_layout_ref!(nalgebra::Pnt1::from_array_ref, u32, 1, [0u32]);
        test_layout_ref!(nalgebra::Pnt2::from_array_ref, u32, 2, [0u32, 1]);
        test_layout_ref!(nalgebra::Pnt3::from_array_ref, u32, 3, [0u32, 1, 2]);
        test_layout_ref!(nalgebra::Pnt4::from_array_ref, u32, 4, [0u32, 1, 2, 3]);
        test_layout_ref!(nalgebra::Pnt1::from_array_ref, i32, 1, [0i32]);
        test_layout_ref!(nalgebra::Pnt2::from_array_ref, i32, 2, [0i32, 1]);
        test_layout_ref!(nalgebra::Pnt3::from_array_ref, i32, 3, [0i32, 1, 2]);
        test_layout_ref!(nalgebra::Pnt4::from_array_ref, i32, 4, [0i32, 1, 2, 3]);

        test_layout_ref!(nalgebra::Vec1::from_array_ref, f32, 1, [0.0f32]);
        test_layout_ref!(nalgebra::Vec2::from_array_ref, f32, 2, [0.0f32, 1.0]);
        test_layout_ref!(nalgebra::Vec3::from_array_ref, f32, 3, [0.0f32, 1.0, 2.0]);
        test_layout_ref!(nalgebra::Vec4::from_array_ref, f32, 4, [0.0f32, 1.0, 2.0, 3.0]);
        test_layout_ref!(nalgebra::Vec1::from_array_ref, f64, 1, [0.0f64]);
        test_layout_ref!(nalgebra::Vec2::from_array_ref, f64, 2, [0.0f64, 1.0]);
        test_layout_ref!(nalgebra::Vec3::from_array_ref, f64, 3, [0.0f64, 1.0, 2.0]);
        test_layout_ref!(nalgebra::Vec4::from_array_ref, f64, 4, [0.0f64, 1.0, 2.0, 3.0]);
        test_layout_ref!(nalgebra::Pnt1::from_array_ref, f32, 1, [0.0f32]);
        test_layout_ref!(nalgebra::Pnt2::from_array_ref, f32, 2, [0.0f32, 1.0]);
        test_layout_ref!(nalgebra::Pnt3::from_array_ref, f32, 3, [0.0f32, 1.0, 2.0]);
        test_layout_ref!(nalgebra::Pnt4::from_array_ref, f32, 4, [0.0f32, 1.0, 2.0, 3.0]);
        test_layout_ref!(nalgebra::Pnt1::from_array_ref, f64, 1, [0.0f64]);
        test_layout_ref!(nalgebra::Pnt2::from_array_ref, f64, 2, [0.0f64, 1.0]);
        test_layout_ref!(nalgebra::Pnt3::from_array_ref, f64, 3, [0.0f64, 1.0, 2.0]);
        test_layout_ref!(nalgebra::Pnt4::from_array_ref, f64, 4, [0.0f64, 1.0, 2.0, 3.0]);

        test_layout_ref!(nalgebra::Mat1::from_array_ref, [f32; 1], 1, [[0.0f32]]);
        test_layout_ref!(nalgebra::Mat2::from_array_ref, [f32; 2], 2, [[0.0f32, 1.0],
                                                                       [2.0f32, 3.0]]);
        test_layout_ref!(nalgebra::Mat3::from_array_ref, [f32; 3], 3, [[0.0f32, 1.0, 2.0],
                                                                       [3.0f32, 4.0, 5.0],
                                                                       [6.0f32, 7.0, 8.0]]);
        test_layout_ref!(nalgebra::Mat4::from_array_ref, [f32; 4], 4, [[0.0f32, 1.0, 2.0, 3.0],
                                                                       [4.0f32, 5.0, 6.0, 7.0],
                                                                       [8.0f32, 9.0, 10.0, 11.0],
                                                                       [12.0f32, 13.0, 14.0, 15.0]]);

        test_layout_ref!(nalgebra::Mat1::from_array_ref, [f64; 1], 1, [[0.0f64]]);
        test_layout_ref!(nalgebra::Mat2::from_array_ref, [f64; 2], 2, [[0.0f64, 1.0],
                                                                       [2.0f64, 3.0]]);
        test_layout_ref!(nalgebra::Mat3::from_array_ref, [f64; 3], 3, [[0.0f64, 1.0, 2.0],
                                                                       [3.0f64, 4.0, 5.0],
                                                                       [6.0f64, 7.0, 8.0]]);
        test_layout_ref!(nalgebra::Mat4::from_array_ref, [f64; 4], 4, [[0.0f64, 1.0, 2.0, 3.0],
                                                                       [4.0f64, 5.0, 6.0, 7.0],
                                                                       [8.0f64, 9.0, 10.0, 11.0],
                                                                       [12.0f64, 13.0, 14.0, 15.0]]);
    }
}
