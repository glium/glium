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
            glium::uniforms::SamplerWrapFunction::Repeat,
            glium::uniforms::SamplerWrapFunction::Repeat,
            glium::uniforms::SamplerWrapFunction::Repeat
        ),
        minify_filter: glium::uniforms::SamplerFilter::Linear,
        .. std::default::Default::default()
    }),
    matrix: matrix,
};
# }
```


*/

use {gl, context, texture};
use cgmath;
use nalgebra;
use std::sync::Arc;

use GlObject;

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
pub struct UniformValueBinder(UniformValueBinderImpl);

impl UniformValueBinder {
    unsafe fn bind(&self, ctxt: &mut context::CommandContext, location: gl::types::GLint,
                   active_texture: &mut gl::types::GLenum)
    {
        match self.0 {
            UniformValueBinderImpl::SignedInt(val) => {
                ctxt.gl.Uniform1i(location, val)
            },
            UniformValueBinderImpl::UnsignedInt(val) => {
                ctxt.gl.Uniform1ui(location, val)
            },
            UniformValueBinderImpl::Float(val) => {
                ctxt.gl.Uniform1f(location, val)
            },
            UniformValueBinderImpl::Mat2(val) => {
                ctxt.gl.UniformMatrix2fv(location, 1, 0, val.as_ptr() as *const f32)
            },
            UniformValueBinderImpl::Mat3(val) => {
                ctxt.gl.UniformMatrix3fv(location, 1, 0, val.as_ptr() as *const f32)
            },
            UniformValueBinderImpl::Mat4(val) => {
                ctxt.gl.UniformMatrix4fv(location, 1, 0, val.as_ptr() as *const f32)
            },
            UniformValueBinderImpl::Vec2(val) => {
                ctxt.gl.Uniform2fv(location, 1, val.as_ptr() as *const f32)
            },
            UniformValueBinderImpl::Vec3(val) => {
                ctxt.gl.Uniform3fv(location, 1, val.as_ptr() as *const f32)
            },
            UniformValueBinderImpl::Vec4(val) => {
                ctxt.gl.Uniform4fv(location, 1, val.as_ptr() as *const f32)
            },
            UniformValueBinderImpl::Texture(id) => {
                ctxt.gl.ActiveTexture(*active_texture as u32);
                ctxt.gl.BindTexture(gl::TEXTURE_2D, id);      // FIXME: check bind point
                ctxt.gl.Uniform1i(location, (*active_texture - gl::TEXTURE0) as gl::types::GLint);
                *active_texture += 1;
            }
        }
    }
}

enum UniformValueBinderImpl {
    SignedInt(gl::types::GLint),
    UnsignedInt(gl::types::GLuint),
    Float(f32),
    Mat2([[f32, ..2], ..2]),
    Mat3([[f32, ..3], ..3]),
    Mat4([[f32, ..4], ..4]),
    Vec2([f32, ..2]),
    Vec3([f32, ..3]),
    Vec4([f32, ..4]),
    Texture(gl::types::GLuint),
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
#[deriving(Show, Copy, Clone)]
pub struct EmptyUniforms;

impl Uniforms for EmptyUniforms {
    fn to_binder(&self) -> UniformsBinder {
        UniformsBinder(box move |_, _, _| {})
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
        let value = self.1.to_binder();
        let rest = self.2.to_binder().0;

        UniformsBinder(box move |ctxt, symbols, active_texture| {
            if let Some(loc) = symbols.call((name.as_slice(),)) {
                unsafe { value.bind(ctxt, loc, active_texture) };
            }   // note: ignoring if the uniform was not found in the program

            rest.call((ctxt, symbols, active_texture));
        })
    }
}

/// The actual content of this object is hidden outside of this library.
// The field is "pub" because of framebuffer.
// TODO: remove this hack
#[doc(hidden)]
pub struct UniformsBinder(pub Box<Fn(&mut context::CommandContext, Box<Fn(&str) -> Option<gl::types::GLint>>,
                          &mut gl::types::GLenum)+Send>);


/// Function to use for out-of-bounds samples.
///
/// This is how GL must handle samples that are outside the texture.
#[deriving(Show, Clone, Copy, Hash, PartialEq, Eq)]
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
#[deriving(Show, Clone, Copy, Hash, PartialEq, Eq)]
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
#[deriving(Show, Clone, Copy, Hash, PartialEq, Eq)]
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

        display.context.context.exec(move |: ctxt| {
            let sampler = unsafe {
                use std::mem;
                let mut sampler: gl::types::GLuint = mem::uninitialized();
                ctxt.gl.GenSamplers(1, &mut sampler);
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
        self.display.context.exec(move |: ctxt| {
            unsafe {
                ctxt.gl.SamplerParameteri(id, gl::TEXTURE_WRAP_S,
                    sampler.wrap_function.0.to_glenum() as gl::types::GLint);
                ctxt.gl.SamplerParameteri(id, gl::TEXTURE_WRAP_T,
                    sampler.wrap_function.1.to_glenum() as gl::types::GLint);
                ctxt.gl.SamplerParameteri(id, gl::TEXTURE_WRAP_R,
                    sampler.wrap_function.2.to_glenum() as gl::types::GLint);
                ctxt.gl.SamplerParameteri(id, gl::TEXTURE_MIN_FILTER,
                    sampler.minify_filter.to_glenum() as gl::types::GLint);
                ctxt.gl.SamplerParameteri(id, gl::TEXTURE_MAG_FILTER,
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
        self.display.context.exec(move |: ctxt| {
            unsafe {
                ctxt.gl.DeleteSamplers(1, [id].as_ptr());
            }
        });
    }
}


impl UniformValue for i8 {
    fn to_binder(&self) -> UniformValueBinder {
        UniformValueBinder(UniformValueBinderImpl::SignedInt(*self as gl::types::GLint))
    }
}

impl UniformValue for u8 {
    fn to_binder(&self) -> UniformValueBinder {
        UniformValueBinder(UniformValueBinderImpl::UnsignedInt(*self as gl::types::GLuint))
    }
}

impl UniformValue for i16 {
    fn to_binder(&self) -> UniformValueBinder {
        UniformValueBinder(UniformValueBinderImpl::SignedInt(*self as gl::types::GLint))
    }
}

impl UniformValue for u16 {
    fn to_binder(&self) -> UniformValueBinder {
        UniformValueBinder(UniformValueBinderImpl::UnsignedInt(*self as gl::types::GLuint))
    }
}

impl UniformValue for i32 {
    fn to_binder(&self) -> UniformValueBinder {
        UniformValueBinder(UniformValueBinderImpl::SignedInt(*self as gl::types::GLint))
    }
}

impl UniformValue for u32 {
    fn to_binder(&self) -> UniformValueBinder {
        UniformValueBinder(UniformValueBinderImpl::UnsignedInt(*self as gl::types::GLuint))
    }
}

impl UniformValue for f32 {
    fn to_binder(&self) -> UniformValueBinder {
        UniformValueBinder(UniformValueBinderImpl::Float(*self))
    }
}

impl UniformValue for [[f32, ..2], ..2] {
    fn to_binder(&self) -> UniformValueBinder {
        UniformValueBinder(UniformValueBinderImpl::Mat2(*self))
    }
}

impl UniformValue for [[f32, ..3], ..3] {
    fn to_binder(&self) -> UniformValueBinder {
        UniformValueBinder(UniformValueBinderImpl::Mat3(*self))
    }
}

impl UniformValue for [[f32, ..4], ..4] {
    fn to_binder(&self) -> UniformValueBinder {
        UniformValueBinder(UniformValueBinderImpl::Mat4(*self))
    }
}

impl UniformValue for (f32, f32) {
    fn to_binder(&self) -> UniformValueBinder {
        UniformValueBinder(UniformValueBinderImpl::Vec2([self.0, self.1]))
    }
}

impl UniformValue for (f32, f32, f32) {
    fn to_binder(&self) -> UniformValueBinder {
        UniformValueBinder(UniformValueBinderImpl::Vec3([self.0, self.1, self.2]))
    }
}

impl UniformValue for (f32, f32, f32, f32) {
    fn to_binder(&self) -> UniformValueBinder {
        UniformValueBinder(UniformValueBinderImpl::Vec4([self.0, self.1, self.2, self.3]))
    }
}

impl UniformValue for [f32, ..2] {
    fn to_binder(&self) -> UniformValueBinder {
        UniformValueBinder(UniformValueBinderImpl::Vec2(*self))
    }
}

impl UniformValue for [f32, ..3] {
    fn to_binder(&self) -> UniformValueBinder {
        UniformValueBinder(UniformValueBinderImpl::Vec3(*self))
    }
}

impl UniformValue for [f32, ..4] {
    fn to_binder(&self) -> UniformValueBinder {
        UniformValueBinder(UniformValueBinderImpl::Vec4(*self))
    }
}

impl<'a> UniformValue for &'a texture::TextureImplementation {
    fn to_binder(&self) -> UniformValueBinder {
        UniformValueBinder(UniformValueBinderImpl::Texture(self.get_id()))
    }
}

impl UniformValue for nalgebra::Mat2<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.as_array();
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::Mat3<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.as_array();
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::Mat4<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.as_array();
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::Ortho3<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.to_mat(); // Bind to a Mat4
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::OrthoMat3<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.as_mat(); // Bind to a Mat4
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::Persp3<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.to_mat(); // Bind to a Mat4
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::PerspMat3<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.as_mat(); // Bind to a Mat4
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::Pnt2<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.as_array();
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::Pnt3<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.as_array();
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::Pnt4<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.as_array();
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::Quat<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.as_array();
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::Rot2<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.submat(); // Bind to a Mat2
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::Rot3<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.submat(); // Bind to a Mat3
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::Rot4<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.submat(); // Bind to a Mat4
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::UnitQuat<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.quat(); // Bind to a Quat
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::Vec2<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.as_array();
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::Vec3<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.as_array();
        my_value.to_binder()
    }
}

impl UniformValue for nalgebra::Vec4<f32> {
    fn to_binder(&self) -> UniformValueBinder {
        let my_value = self.as_array();
        my_value.to_binder()
    }
}


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
