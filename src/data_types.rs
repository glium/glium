#![allow(missing_doc)]

use gl;

pub trait GLDataType: Num + Copy {
    /// Returns the OpenGL enumeration corresponding to this type.
    fn get_gl_type(Option<Self>) -> gl::types::GLenum;
}

impl GLDataType for i8 {
    fn get_gl_type(_: Option<i8>) -> gl::types::GLenum {
        gl::BYTE
    }
}

impl GLDataType for u8 {
    fn get_gl_type(_: Option<u8>) -> gl::types::GLenum {
        gl::UNSIGNED_BYTE
    }
}

impl GLDataType for i16 {
    fn get_gl_type(_: Option<i16>) -> gl::types::GLenum {
        gl::SHORT
    }
}

impl GLDataType for u16 {
    fn get_gl_type(_: Option<u16>) -> gl::types::GLenum {
        gl::UNSIGNED_SHORT
    }
}

impl GLDataType for i32 {
    fn get_gl_type(_: Option<i32>) -> gl::types::GLenum {
        gl::INT
    }
}

impl GLDataType for u32 {
    fn get_gl_type(_: Option<u32>) -> gl::types::GLenum {
        gl::UNSIGNED_INT
    }
}

impl GLDataType for f32 {
    fn get_gl_type(_: Option<f32>) -> gl::types::GLenum {
        gl::FLOAT
    }
}

#[cfg(not(target_os = "android"))]
impl GLDataType for f64 {
    fn get_gl_type(_: Option<f64>) -> gl::types::GLenum {
        gl::DOUBLE
    }
}

#[doc(hidden)]
pub trait GLDataTuple {
    fn get_gl_type(Option<Self>) -> gl::types::GLenum;
    fn get_num_elems(Option<Self>) -> gl::types::GLint;
}

impl GLDataTuple for i8 {
    fn get_gl_type(_: Option<(i8)>) -> gl::types::GLenum { gl::BYTE }
    fn get_num_elems(_: Option<(i8)>) -> gl::types::GLint { 1 }
}

impl GLDataTuple for u8 {
    fn get_gl_type(_: Option<(u8)>) -> gl::types::GLenum { gl::UNSIGNED_BYTE }
    fn get_num_elems(_: Option<(u8)>) -> gl::types::GLint { 1 }
}

impl GLDataTuple for f32 {
    fn get_gl_type(_: Option<(f32)>) -> gl::types::GLenum { gl::FLOAT }
    fn get_num_elems(_: Option<(f32)>) -> gl::types::GLint { 1 }
}

impl<T: GLDataTuple> GLDataTuple for (T, T) {
    fn get_gl_type(_: Option<(T, T)>) -> gl::types::GLenum {
        GLDataTuple::get_gl_type(None::<T>)
    }
    fn get_num_elems(_: Option<(T, T)>) -> gl::types::GLint {
        GLDataTuple::get_num_elems(None::<T>) * 2
    }
}

impl<T: GLDataTuple> GLDataTuple for (T, T, T) {
    fn get_gl_type(_: Option<(T, T, T)>) -> gl::types::GLenum {
        GLDataTuple::get_gl_type(None::<T>)
    }
    fn get_num_elems(_: Option<(T, T, T)>) -> gl::types::GLint {
        GLDataTuple::get_num_elems(None::<T>) * 3
    }
}

impl<T: GLDataTuple> GLDataTuple for (T, T, T, T) {
    fn get_gl_type(_: Option<(T, T, T, T)>) -> gl::types::GLenum {
        GLDataTuple::get_gl_type(None::<T>)
    }
    fn get_num_elems(_: Option<(T, T, T, T)>) -> gl::types::GLint {
        GLDataTuple::get_num_elems(None::<T>) * 4
    }
}

impl<T: GLDataTuple> GLDataTuple for [T, ..2] {
    fn get_gl_type(_: Option<[T, ..2]>) -> gl::types::GLenum {
        GLDataTuple::get_gl_type(None::<T>)
    }
    fn get_num_elems(_: Option<[T, ..2]>) -> gl::types::GLint {
        GLDataTuple::get_num_elems(None::<T>) * 2
    }
}

impl<T: GLDataTuple> GLDataTuple for [T, ..3] {
    fn get_gl_type(_: Option<[T, ..3]>) -> gl::types::GLenum {
        GLDataTuple::get_gl_type(None::<T>)
    }
    fn get_num_elems(_: Option<[T, ..3]>) -> gl::types::GLint {
        GLDataTuple::get_num_elems(None::<T>) * 3
    }
}

impl<T: GLDataTuple> GLDataTuple for [T, ..4] {
    fn get_gl_type(_: Option<[T, ..4]>) -> gl::types::GLenum {
        GLDataTuple::get_gl_type(None::<T>)
    }
    fn get_num_elems(_: Option<[T, ..4]>) -> gl::types::GLint {
        GLDataTuple::get_num_elems(None::<T>) * 4
    }
}
