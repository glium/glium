#[cfg(feature = "image")]
use image;

/// A trait that must be implemented for any type that can represent the value of a pixel.
pub trait PixelValue: Copy + Send + 'static {     // TODO: Clone, but [T, ..N] doesn't impl Clone
    /// Returns corresponding client format.
    fn get_format() -> super::ClientFormat;
}

impl PixelValue for i8 {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I8
    }
}

impl PixelValue for (i8, i8) {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I8I8
    }
}

impl PixelValue for (i8, i8, i8) {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I8I8I8
    }
}

impl PixelValue for (i8, i8, i8, i8) {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I8I8I8I8
    }
}

impl PixelValue for u8 {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U8
    }
}

impl PixelValue for (u8, u8) {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U8U8
    }
}

impl PixelValue for (u8, u8, u8) {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U8U8U8
    }
}

impl PixelValue for (u8, u8, u8, u8) {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U8U8U8U8
    }
}

impl PixelValue for i16 {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I16
    }
}

impl PixelValue for (i16, i16) {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I16I16
    }
}

impl PixelValue for (i16, i16, i16) {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I16I16I16
    }
}

impl PixelValue for (i16, i16, i16, i16) {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I16I16I16I16
    }
}

impl PixelValue for u16 {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U16
    }
}

impl PixelValue for (u16, u16) {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U16U16
    }
}

impl PixelValue for (u16, u16, u16) {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U16U16U16
    }
}

impl PixelValue for (u16, u16, u16, u16) {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U16U16U16U16
    }
}

impl PixelValue for i32 {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I32
    }
}

impl PixelValue for (i32, i32) {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I32I32
    }
}

impl PixelValue for (i32, i32, i32) {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I32I32I32
    }
}

impl PixelValue for (i32, i32, i32, i32) {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I32I32I32I32
    }
}

impl PixelValue for u32 {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U32
    }
}

impl PixelValue for (u32, u32) {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U32U32
    }
}

impl PixelValue for (u32, u32, u32) {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U32U32U32
    }
}

impl PixelValue for (u32, u32, u32, u32) {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U32U32U32U32
    }
}

impl PixelValue for f32 {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::F32
    }
}

impl PixelValue for (f32, f32) {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::F32F32
    }
}

impl PixelValue for (f32, f32, f32) {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::F32F32F32
    }
}

impl PixelValue for (f32, f32, f32, f32) {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::F32F32F32F32
    }
}

#[cfg(feature = "image")]
impl PixelValue for image::Rgb<u8> {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U8U8U8
    }
}

#[cfg(feature = "image")]
impl PixelValue for image::Rgba<u8> {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U8U8U8U8
    }
}

#[cfg(feature = "image")]
impl PixelValue for image::Luma<u8> {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U8
    }
}

#[cfg(feature = "image")]
impl PixelValue for image::Luma<u16> {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U16
    }
}

#[cfg(feature = "image")]
impl PixelValue for image::Luma<f32> {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::F32
    }
}

#[cfg(feature = "image")]
impl PixelValue for image::LumaA<u8> {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U8U8
    }
}

#[cfg(feature = "image")]
impl PixelValue for image::LumaA<u16> {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U16U16
    }
}

#[cfg(feature = "image")]
impl PixelValue for image::LumaA<f32> {
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::F32F32
    }
}
