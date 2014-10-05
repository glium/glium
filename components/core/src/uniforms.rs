use {gl, texture};

/// Represents a value that can be used as the value of a uniform.
///
/// You can implement this trait for your own types by redirecting the call to another
///  implementation.
pub trait UniformValue {
    /// Builds a new `UniformValueBinder`.
    fn to_binder(&self) -> UniformValueBinder;
}

/// The actual content of this object is hidden outside of this library.
///
/// The proc takes as parameter the `Gl` object, the binding location, and a `&mut GLenum` that
///  represents the current value of `glActiveTexture`.
/// It must call `glUniform*`.
pub struct UniformValueBinder(proc(&gl::Gl, gl::types::GLint, &mut gl::types::GLenum):Send);

impl UniformValueBinder {
    /// This method exists because we need to access it from `glium_core_macros`.
    #[doc(hidden)]
    pub fn get_proc(self) -> proc(&gl::Gl, gl::types::GLint, &mut gl::types::GLenum):Send {
        self.0
    }
}

/// Object that contains all the uniforms of a program with their bindings points.
///
/// It is more or less a collection of `UniformValue`s.
///
/// You can implement this trait for your own types by redirecting the call to another
///  implementation.
pub trait Uniforms {
    /// Builds a new `UniformsBinder`.
    fn to_binder(&self) -> UniformsBinder;
}

/// Object that can be used when you don't have any uniform.
pub struct EmptyUniforms;

impl Uniforms for EmptyUniforms {
    fn to_binder(&self) -> UniformsBinder {
        UniformsBinder(proc(_, _) {})
    }
}

/// The actual content of this object is hidden outside of this library.
///
/// The content is however `pub` because we need to access it from `glium_core_macros`.
#[doc(hidden)]
pub struct UniformsBinder(pub proc(&gl::Gl, |&str| -> Option<gl::types::GLint>):Send);


impl UniformValue for i8 {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.clone();
        UniformValueBinder(proc(gl, location, _) {
            gl.Uniform1i(location, my_value as gl::types::GLint)
        })
    }
}

impl UniformValue for u8 {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.clone();
        UniformValueBinder(proc(gl, location, _) {
            gl.Uniform1ui(location, my_value as gl::types::GLuint)
        })
    }
}

impl UniformValue for i16 {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.clone();
        UniformValueBinder(proc(gl, location, _) {
            gl.Uniform1i(location, my_value as gl::types::GLint)
        })
    }
}

impl UniformValue for u16 {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.clone();
        UniformValueBinder(proc(gl, location, _) {
            gl.Uniform1ui(location, my_value as gl::types::GLuint)
        })
    }
}

impl UniformValue for i32 {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.clone();
        UniformValueBinder(proc(gl, location, _) {
            gl.Uniform1i(location, my_value as gl::types::GLint)
        })
    }
}

impl UniformValue for u32 {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.clone();
        UniformValueBinder(proc(gl, location, _) {
            gl.Uniform1ui(location, my_value as gl::types::GLuint)
        })
    }
}

impl UniformValue for f32 {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.clone();
        UniformValueBinder(proc(gl, location, _) {
            gl.Uniform1f(location, my_value)
        })
    }
}

impl UniformValue for [[f32, ..2], ..2] {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = *self;
        UniformValueBinder(proc(gl, location, _) {
            unsafe { gl.UniformMatrix2fv(location, 1, 0, my_value.as_ptr() as *const f32) }
        })
    }
}

impl UniformValue for [[f32, ..3], ..3] {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = *self;
        UniformValueBinder(proc(gl, location, _) {
            unsafe { gl.UniformMatrix3fv(location, 1, 0, my_value.as_ptr() as *const f32) }
        })
    }
}

impl UniformValue for [[f32, ..4], ..4] {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = *self;
        UniformValueBinder(proc(gl, location, _) {
            unsafe { gl.UniformMatrix4fv(location, 1, 0, my_value.as_ptr() as *const f32) }
        })
    }
}

impl UniformValue for (f32, f32, f32, f32) {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.clone();
        UniformValueBinder(proc(gl, location, _) {
            let my_value = [ my_value.0, my_value.1, my_value.2, my_value.3 ];
            unsafe { gl.Uniform4fv(location, 1, my_value.as_ptr() as *const f32) }
        })
    }
}

impl UniformValue for [f32, ..4] {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = *self;
        UniformValueBinder(proc(gl, location, _) {
            unsafe { gl.Uniform4fv(location, 1, my_value.as_ptr() as *const f32) }
        })
    }
}

impl<'a> UniformValue for &'a texture::Texture {
    fn to_binder(&self) -> UniformValueBinder {
        let my_id = texture::get_impl(*self).id.clone();
        UniformValueBinder(proc(gl, location, active_texture) {
            gl.BindTexture(gl::TEXTURE_2D, my_id);      // FIXME: check bind point
            unsafe { gl.Uniform1i(location, (*active_texture - gl::TEXTURE0) as gl::types::GLint) };
            *active_texture += 1;
        })
    }
}
