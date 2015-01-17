use vertex::Attribute;

#[allow(missing_docs)]
#[derive(Copy, Clone, Show, PartialEq, Eq)]
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
}

/// Describes the layout of each vertex in a vertex buffer.
///
/// The first element is the name of the binding, the second element is the offset
/// from the start of each vertex to this element, and the third element is the type.
pub type VertexFormat = Vec<(String, usize, AttributeType)>;

unsafe impl Attribute for i8 {
    fn get_type(_: Option<i8>) -> AttributeType {
        AttributeType::I8
    }
}

unsafe impl Attribute for (i8, i8) {
    fn get_type(_: Option<(i8, i8)>) -> AttributeType {
        AttributeType::I8I8
    }
}

unsafe impl Attribute for [i8; 2] {
    fn get_type(_: Option<[i8; 2]>) -> AttributeType {
        AttributeType::I8I8
    }
}

unsafe impl Attribute for (i8, i8, i8) {
    fn get_type(_: Option<(i8, i8, i8)>) -> AttributeType {
        AttributeType::I8I8I8
    }
}

unsafe impl Attribute for [i8; 3] {
    fn get_type(_: Option<[i8; 3]>) -> AttributeType {
        AttributeType::I8I8I8
    }
}

unsafe impl Attribute for (i8, i8, i8, i8) {
    fn get_type(_: Option<(i8, i8, i8, i8)>) -> AttributeType {
        AttributeType::I8I8I8I8
    }
}

unsafe impl Attribute for [i8; 4] {
    fn get_type(_: Option<[i8; 4]>) -> AttributeType {
        AttributeType::I8I8I8I8
    }
}

unsafe impl Attribute for u8 {
    fn get_type(_: Option<u8>) -> AttributeType {
        AttributeType::U8
    }
}

unsafe impl Attribute for (u8, u8) {
    fn get_type(_: Option<(u8, u8)>) -> AttributeType {
        AttributeType::U8U8
    }
}

unsafe impl Attribute for [u8; 2] {
    fn get_type(_: Option<[u8; 2]>) -> AttributeType {
        AttributeType::U8U8
    }
}

unsafe impl Attribute for (u8, u8, u8) {
    fn get_type(_: Option<(u8, u8, u8)>) -> AttributeType {
        AttributeType::U8U8U8
    }
}

unsafe impl Attribute for [u8; 3] {
    fn get_type(_: Option<[u8; 3]>) -> AttributeType {
        AttributeType::U8U8U8
    }
}

unsafe impl Attribute for (u8, u8, u8, u8) {
    fn get_type(_: Option<(u8, u8, u8, u8)>) -> AttributeType {
        AttributeType::U8U8U8U8
    }
}

unsafe impl Attribute for [u8; 4] {
    fn get_type(_: Option<[u8; 4]>) -> AttributeType {
        AttributeType::U8U8U8U8
    }
}

unsafe impl Attribute for i16 {
    fn get_type(_: Option<i16>) -> AttributeType {
        AttributeType::I16
    }
}

unsafe impl Attribute for (i16, i16) {
    fn get_type(_: Option<(i16, i16)>) -> AttributeType {
        AttributeType::I16I16
    }
}

unsafe impl Attribute for [i16; 2] {
    fn get_type(_: Option<[i16; 2]>) -> AttributeType {
        AttributeType::I16I16
    }
}

unsafe impl Attribute for (i16, i16, i16) {
    fn get_type(_: Option<(i16, i16, i16)>) -> AttributeType {
        AttributeType::I16I16I16
    }
}

unsafe impl Attribute for [i16; 3] {
    fn get_type(_: Option<[i16; 3]>) -> AttributeType {
        AttributeType::I16I16I16
    }
}

unsafe impl Attribute for (i16, i16, i16, i16) {
    fn get_type(_: Option<(i16, i16, i16, i16)>) -> AttributeType {
        AttributeType::I16I16I16I16
    }
}

unsafe impl Attribute for [i16; 4] {
    fn get_type(_: Option<[i16; 4]>) -> AttributeType {
        AttributeType::I16I16I16I16
    }
}

unsafe impl Attribute for u16 {
    fn get_type(_: Option<u16>) -> AttributeType {
        AttributeType::U16
    }
}

unsafe impl Attribute for (u16, u16) {
    fn get_type(_: Option<(u16, u16)>) -> AttributeType {
        AttributeType::U16U16
    }
}

unsafe impl Attribute for [u16; 2] {
    fn get_type(_: Option<[u16; 2]>) -> AttributeType {
        AttributeType::U16U16
    }
}

unsafe impl Attribute for (u16, u16, u16) {
    fn get_type(_: Option<(u16, u16, u16)>) -> AttributeType {
        AttributeType::U16U16U16
    }
}

unsafe impl Attribute for [u16; 3] {
    fn get_type(_: Option<[u16; 3]>) -> AttributeType {
        AttributeType::U16U16U16
    }
}

unsafe impl Attribute for (u16, u16, u16, u16) {
    fn get_type(_: Option<(u16, u16, u16, u16)>) -> AttributeType {
        AttributeType::U16U16U16U16
    }
}

unsafe impl Attribute for [u16; 4] {
    fn get_type(_: Option<[u16; 4]>) -> AttributeType {
        AttributeType::U16U16U16U16
    }
}

unsafe impl Attribute for i32 {
    fn get_type(_: Option<i32>) -> AttributeType {
        AttributeType::I32
    }
}

unsafe impl Attribute for (i32, i32) {
    fn get_type(_: Option<(i32, i32)>) -> AttributeType {
        AttributeType::I32I32
    }
}

unsafe impl Attribute for [i32; 2] {
    fn get_type(_: Option<[i32; 2]>) -> AttributeType {
        AttributeType::I32I32
    }
}

unsafe impl Attribute for (i32, i32, i32) {
    fn get_type(_: Option<(i32, i32, i32)>) -> AttributeType {
        AttributeType::I32I32I32
    }
}

unsafe impl Attribute for [i32; 3] {
    fn get_type(_: Option<[i32; 3]>) -> AttributeType {
        AttributeType::I32I32I32
    }
}

unsafe impl Attribute for (i32, i32, i32, i32) {
    fn get_type(_: Option<(i32, i32, i32, i32)>) -> AttributeType {
        AttributeType::I32I32I32I32
    }
}

unsafe impl Attribute for [i32; 4] {
    fn get_type(_: Option<[i32; 4]>) -> AttributeType {
        AttributeType::I32I32I32I32
    }
}

unsafe impl Attribute for u32 {
    fn get_type(_: Option<u32>) -> AttributeType {
        AttributeType::U32
    }
}

unsafe impl Attribute for (u32, u32) {
    fn get_type(_: Option<(u32, u32)>) -> AttributeType {
        AttributeType::U32U32
    }
}

unsafe impl Attribute for [u32; 2] {
    fn get_type(_: Option<[u32; 2]>) -> AttributeType {
        AttributeType::U32U32
    }
}

unsafe impl Attribute for (u32, u32, u32) {
    fn get_type(_: Option<(u32, u32, u32)>) -> AttributeType {
        AttributeType::U32U32U32
    }
}

unsafe impl Attribute for [u32; 3] {
    fn get_type(_: Option<[u32; 3]>) -> AttributeType {
        AttributeType::U32U32U32
    }
}

unsafe impl Attribute for (u32, u32, u32, u32) {
    fn get_type(_: Option<(u32, u32, u32, u32)>) -> AttributeType {
        AttributeType::U32U32U32U32
    }
}

unsafe impl Attribute for [u32; 4] {
    fn get_type(_: Option<[u32; 4]>) -> AttributeType {
        AttributeType::U32U32U32U32
    }
}

unsafe impl Attribute for f32 {
    fn get_type(_: Option<f32>) -> AttributeType {
        AttributeType::F32
    }
}

unsafe impl Attribute for (f32, f32) {
    fn get_type(_: Option<(f32, f32)>) -> AttributeType {
        AttributeType::F32F32
    }
}

unsafe impl Attribute for [f32; 2] {
    fn get_type(_: Option<[f32; 2]>) -> AttributeType {
        AttributeType::F32F32
    }
}

unsafe impl Attribute for (f32, f32, f32) {
    fn get_type(_: Option<(f32, f32, f32)>) -> AttributeType {
        AttributeType::F32F32F32
    }
}

unsafe impl Attribute for [f32; 3] {
    fn get_type(_: Option<[f32; 3]>) -> AttributeType {
        AttributeType::F32F32F32
    }
}

unsafe impl Attribute for (f32, f32, f32, f32) {
    fn get_type(_: Option<(f32, f32, f32, f32)>) -> AttributeType {
        AttributeType::F32F32F32F32
    }
}

unsafe impl Attribute for [f32; 4] {
    fn get_type(_: Option<[f32; 4]>) -> AttributeType {
        AttributeType::F32F32F32F32
    }
}
