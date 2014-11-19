/*!
A uniform is a global variable in your program. In order to draw something, you will need to
 tell `glium` what the values of all your uniforms are. Objects that implement the `Uniform`
 trait are here to do that.

The recommended way to is to create your own structure and put the `#[uniforms]` attribute
 to it.

For example:

```no_run
# #![feature(phase)]
#[phase(plugin)]
extern crate glium_macros;
# extern crate glium;
# fn main() {

#[uniforms]
struct Uniforms<'a> {
    texture: &'a glium::Texture2d,
    matrix: [[f32, ..4], ..4],
}

# let display: glium::Display = unsafe { std::mem::uninitialized() };
# let tex = unsafe { std::mem::uninitialized() };
# let matrix = unsafe { std::mem::uninitialized() };
let uniforms = Uniforms {
    texture: tex,
    matrix: matrix,
};
# }
```

Each field must implement the `UniformValue` trait.

## Textures and samplers

To use a texture, write a `&Texture2d` like a regular uniform value.

To use a texture with a sampler, write a `Sampler` object.

Example:

```no_run
# #![feature(phase)]
# #[phase(plugin)]
# extern crate glium_macros;
# extern crate glium;
# fn main() {
#[uniforms]
struct Uniforms<'a> {
    texture: glium::uniforms::Sampler<'a, glium::Texture2d>,
    matrix: [[f32, ..4], ..4],
}

# let display: glium::Display = unsafe { std::mem::uninitialized() };
# let tex = unsafe { std::mem::uninitialized() };
# let matrix = unsafe { std::mem::uninitialized() };
let uniforms = Uniforms {
    texture: glium::uniforms::Sampler(&tex, glium::uniforms::SamplerBehavior {
        wrap_function: (
            glium::uniforms::Repeat,
            glium::uniforms::Repeat,
            glium::uniforms::Repeat
        ),
        minify_filter: glium::uniforms::Linear,
        .. std::default::Default::default()
    }),
    matrix: matrix,
};
# }
```


*/

use {gl, texture};
use cgmath;
//use nalgebra;
use std::sync::Arc;

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
        UniformsBinder(proc(_, _, _) {})
    }
}

/// Stores uniforms in an efficient way.
///
/// # Example
///
/// ```ignore   // TODO: CRASHES RUSTDOC OTHERWISE
/// use glium::uniforms::UniformsStorage;
///
/// // `name1` will contain 2.0
/// let uniforms = UniformsStorage::new("name1", 2.0f32);
///
/// // `name2` will contain -0.5
/// let uniforms = uniforms.add("name2", -0.5f32);
///
/// // `name3` will contain `texture`
/// # let texture: glium::Texture2d = unsafe { ::std::mem::uninitialized() };
/// let uniforms = uniforms.add("name3", &texture);
///
/// // the final type is `UniformsStorage<&Texture2d, UniformsStorage<f32, UniformsStorage<f32, EmptyUniforms>>>`
/// // but you shouldn't care about it
/// ```
///
pub struct UniformsStorage<'a, T, R>(&'a str, T, R);

impl<'a, T> UniformsStorage<'a, T, EmptyUniforms> where T: UniformValue {
    /// Builds a new storage with a value.
    pub fn new(name: &'a str, value: T) -> UniformsStorage<'a, T, EmptyUniforms> {
        UniformsStorage(name, value, EmptyUniforms)
    }
}

impl<'a, T, R> UniformsStorage<'a, T, R> {
    /// Adds a value to the storage.
    pub fn add<'b, V: UniformValue>(self, name: &'b str, value: V)
        -> UniformsStorage<'b, V, UniformsStorage<'a, T, R>>
    {
        UniformsStorage(name, value, self)
    }
}

impl<'a, T, R> Uniforms for UniformsStorage<'a, T, R> where T: UniformValue, R: Uniforms {
    fn to_binder(&self) -> UniformsBinder {
        let name = self.0.to_string();
        let value_binder = self.1.to_binder().0;
        let rest = self.2.to_binder().0;

        UniformsBinder(proc(gl, symbols, active_texture) {
            if let Some(loc) = symbols(name.as_slice()) {
                value_binder(gl, loc, active_texture);
            }   // note: ignoring if the uniform was not found in the program

            rest(gl, symbols, active_texture);
        })
    }
}

/// The actual content of this object is hidden outside of this library.
// The field is "pub" because of framebuffer.
// TODO: remove this hack
#[doc(hidden)]
pub struct UniformsBinder(pub proc(&gl::Gl, |&str| -> Option<gl::types::GLint>,
                          &mut gl::types::GLenum):Send);


/// Function to use for out-of-bounds samples.
///
/// This is how GL must handle samples that are outside the texture.
#[deriving(Show, Clone, Hash, PartialEq, Eq)]
pub enum SamplerWrapFunction {
    /// Samples at coord `x + 1` are mapped to coord `x`.
    Repeat,

    /// Samples at coord `x + 1` are mapped to coord `1 - x`.
    Mirror,

    /// Samples at coord `x + 1` are mapped to coord `1`.
    Clamp
}

impl SamplerWrapFunction {
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            SamplerWrapFunction::Repeat => gl::REPEAT,
            SamplerWrapFunction::Mirror => gl::MIRRORED_REPEAT,
            SamplerWrapFunction::Clamp => gl::CLAMP_TO_EDGE,
        }
    }
} 

/// The function that the GPU will use when loading the value of a texel.
#[deriving(Show, Clone, Hash, PartialEq, Eq)]
pub enum SamplerFilter {
    /// The nearest texel will be loaded.
    Nearest,

    /// All nearby texels will be loaded and their values will be merged.
    Linear
}

impl SamplerFilter {
    #[doc(hidden)]      // TODO: hacky
    pub fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            SamplerFilter::Nearest => gl::NEAREST,
            SamplerFilter::Linear => gl::LINEAR,
        }
    }
}

/// A sampler.
pub struct Sampler<'t, T: 't>(pub &'t T, pub SamplerBehavior);

impl<'t, T: texture::Texture + 't> UniformValue for Sampler<'t, T> {
    fn to_binder(&self) -> UniformValueBinder {
        // TODO: use the behavior too
        self.0.get_implementation().to_binder()
    }
}

/// Behavior of a sampler.
// TODO: GL_TEXTURE_BORDER_COLOR, GL_TEXTURE_MIN_LOD, GL_TEXTURE_MAX_LOD, GL_TEXTURE_LOD_BIAS,
//       GL_TEXTURE_COMPARE_MODE, GL_TEXTURE_COMPARE_FUNC
#[deriving(Show, Clone, Hash, PartialEq, Eq)]
pub struct SamplerBehavior {
    /// Functions to use for the X, Y, and Z coordinates.
    pub wrap_function: (SamplerWrapFunction, SamplerWrapFunction, SamplerWrapFunction),
    /// Filter to use when mignifying the texture.
    pub minify_filter: SamplerFilter,
    /// Filter to use when magnifying the texture.
    pub magnify_filter: SamplerFilter,
}

impl ::std::default::Default for SamplerBehavior {
    fn default() -> SamplerBehavior {
        SamplerBehavior {
            wrap_function: (
                SamplerWrapFunction::Mirror,
                SamplerWrapFunction::Mirror,
                SamplerWrapFunction::Mirror
            ),
            minify_filter: SamplerFilter::Linear,
            magnify_filter: SamplerFilter::Linear,
        }
    }
}

/// An OpenGL sampler object.
// TODO: cache parameters set in the sampler
struct SamplerObject {
    display: Arc<super::DisplayImpl>,
    id: gl::types::GLuint,
}

impl SamplerObject {
    pub fn new(display: &super::Display) -> SamplerObject {
        let (tx, rx) = channel();

        display.context.context.exec(proc(gl, _, _, _) {
            let sampler = unsafe {
                use std::mem;
                let mut sampler: gl::types::GLuint = mem::uninitialized();
                gl.GenSamplers(1, &mut sampler);
                sampler
            };

            tx.send(sampler);
        });

        SamplerObject {
            display: display.context.clone(),
            id: rx.recv(),
        }
    }

    pub fn bind(&self, gl: gl::Gl, sampler: SamplerBehavior) {
        let id = self.id;
        self.display.context.exec(proc(gl, _, _, _) {
            unsafe {
                gl.SamplerParameteri(id, gl::TEXTURE_WRAP_S,
                    sampler.wrap_function.0.to_glenum() as gl::types::GLint);
                gl.SamplerParameteri(id, gl::TEXTURE_WRAP_T,
                    sampler.wrap_function.1.to_glenum() as gl::types::GLint);
                gl.SamplerParameteri(id, gl::TEXTURE_WRAP_R,
                    sampler.wrap_function.2.to_glenum() as gl::types::GLint);
                gl.SamplerParameteri(id, gl::TEXTURE_MIN_FILTER,
                    sampler.minify_filter.to_glenum() as gl::types::GLint);
                gl.SamplerParameteri(id, gl::TEXTURE_MAG_FILTER,
                    sampler.magnify_filter.to_glenum() as gl::types::GLint);
            }
        });
    }

    pub fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for SamplerObject {
    fn drop(&mut self) {
        let id = self.id;
        self.display.context.exec(proc(gl, _, _, _) {
            unsafe {
                gl.DeleteSamplers(1, [id].as_ptr());
            }
        });
    }
}


impl UniformValue for i8 {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.clone();
        UniformValueBinder(proc(gl, location, _) {
            unsafe {
                gl.Uniform1i(location, my_value as gl::types::GLint)
            }
        })
    }
}

impl UniformValue for u8 {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.clone();
        UniformValueBinder(proc(gl, location, _) {
            unsafe {
                gl.Uniform1ui(location, my_value as gl::types::GLuint)
            }
        })
    }
}

impl UniformValue for i16 {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.clone();
        UniformValueBinder(proc(gl, location, _) {
            unsafe {
                gl.Uniform1i(location, my_value as gl::types::GLint)
            }
        })
    }
}

impl UniformValue for u16 {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.clone();
        UniformValueBinder(proc(gl, location, _) {
            unsafe {
                gl.Uniform1ui(location, my_value as gl::types::GLuint)
            }
        })
    }
}

impl UniformValue for i32 {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.clone();
        UniformValueBinder(proc(gl, location, _) {
            unsafe {
                gl.Uniform1i(location, my_value as gl::types::GLint)
            }
        })
    }
}

impl UniformValue for u32 {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.clone();
        UniformValueBinder(proc(gl, location, _) {
            unsafe {
                gl.Uniform1ui(location, my_value as gl::types::GLuint)
            }
        })
    }
}

impl UniformValue for f32 {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.clone();
        UniformValueBinder(proc(gl, location, _) {
            unsafe {
                gl.Uniform1f(location, my_value)
            }
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

impl UniformValue for (f32, f32) {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.clone();
        UniformValueBinder(proc(gl, location, _) {
            let my_value = [ my_value.0, my_value.1 ];
            unsafe { gl.Uniform2fv(location, 1, my_value.as_ptr() as *const f32) }
        })
    }
}

impl UniformValue for (f32, f32, f32) {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.clone();
        UniformValueBinder(proc(gl, location, _) {
            let my_value = [ my_value.0, my_value.1, my_value.2 ];
            unsafe { gl.Uniform3fv(location, 1, my_value.as_ptr() as *const f32) }
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

impl UniformValue for [f32, ..2] {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = *self;
        UniformValueBinder(proc(gl, location, _) {
            unsafe { gl.Uniform2fv(location, 1, my_value.as_ptr() as *const f32) }
        })
    }
}

impl UniformValue for [f32, ..3] {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = *self;
        UniformValueBinder(proc(gl, location, _) {
            unsafe { gl.Uniform3fv(location, 1, my_value.as_ptr() as *const f32) }
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

impl<'a> UniformValue for &'a texture::TextureImplementation {
    fn to_binder(&self) -> UniformValueBinder {
        let my_id = texture::get_id(*self);
        UniformValueBinder(proc(gl, location, active_texture) {
            unsafe {
                gl.ActiveTexture(*active_texture as u32);
                gl.BindTexture(gl::TEXTURE_2D, my_id);      // FIXME: check bind point
                gl.Uniform1i(location, (*active_texture - gl::TEXTURE0) as gl::types::GLint);
                *active_texture += 1;
            }
        })
    }
}

// TODO: no method to get a slice?
/*impl UniformValue for nalgebra::na::Vec1<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.into_fixed();
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::na::Vec2<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.into_fixed();
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::na::Vec3<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.into_fixed();
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::na::Vec4<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.into_fixed();
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::na::Mat1<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.into_fixed();
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::na::Mat2<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.into_fixed();
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::na::Mat3<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.into_fixed();
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::na::Mat4<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.into_fixed();
        my_value.to_binder()
    }
}*/

impl UniformValue for cgmath::Matrix2<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        my_value.to_binder()
    }
}

impl UniformValue for cgmath::Matrix3<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        my_value.to_binder()
    }
}

impl UniformValue for cgmath::Matrix4<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        my_value.to_binder()
    }
}

impl UniformValue for cgmath::Vector2<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        my_value.to_binder()
    }
}

impl UniformValue for cgmath::Vector3<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        my_value.to_binder()
    }
}

impl UniformValue for cgmath::Vector4<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        use cgmath::FixedArray;
        let my_value = self.into_fixed();
        my_value.to_binder()
    }
}
