/// A trait that must be implemented for any type that can represent the value of a pixel.
pub unsafe trait PixelValue: Copy + Clone + Send + 'static {
    /// Returns corresponding client format.
    fn get_format() -> super::ClientFormat;
}

unsafe impl PixelValue for i8 {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I8
    }
}

unsafe impl PixelValue for (i8, i8) {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I8I8
    }
}

unsafe impl PixelValue for (i8, i8, i8) {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I8I8I8
    }
}

unsafe impl PixelValue for (i8, i8, i8, i8) {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I8I8I8I8
    }
}

unsafe impl PixelValue for u8 {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U8
    }
}

unsafe impl PixelValue for (u8, u8) {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U8U8
    }
}

unsafe impl PixelValue for (u8, u8, u8) {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U8U8U8
    }
}

unsafe impl PixelValue for (u8, u8, u8, u8) {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U8U8U8U8
    }
}

unsafe impl PixelValue for i16 {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I16
    }
}

unsafe impl PixelValue for (i16, i16) {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I16I16
    }
}

unsafe impl PixelValue for (i16, i16, i16) {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I16I16I16
    }
}

unsafe impl PixelValue for (i16, i16, i16, i16) {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I16I16I16I16
    }
}

unsafe impl PixelValue for u16 {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U16
    }
}

unsafe impl PixelValue for (u16, u16) {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U16U16
    }
}

unsafe impl PixelValue for (u16, u16, u16) {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U16U16U16
    }
}

unsafe impl PixelValue for (u16, u16, u16, u16) {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U16U16U16U16
    }
}

unsafe impl PixelValue for i32 {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I32
    }
}

unsafe impl PixelValue for (i32, i32) {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I32I32
    }
}

unsafe impl PixelValue for (i32, i32, i32) {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I32I32I32
    }
}

unsafe impl PixelValue for (i32, i32, i32, i32) {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::I32I32I32I32
    }
}

unsafe impl PixelValue for u32 {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U32
    }
}

unsafe impl PixelValue for (u32, u32) {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U32U32
    }
}

unsafe impl PixelValue for (u32, u32, u32) {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U32U32U32
    }
}

unsafe impl PixelValue for (u32, u32, u32, u32) {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::U32U32U32U32
    }
}

unsafe impl PixelValue for f32 {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::F32
    }
}

unsafe impl PixelValue for (f32, f32) {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::F32F32
    }
}

unsafe impl PixelValue for (f32, f32, f32) {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::F32F32F32
    }
}

unsafe impl PixelValue for (f32, f32, f32, f32) {
    #[inline]
    fn get_format() -> super::ClientFormat {
        super::ClientFormat::F32F32F32F32
    }
}

