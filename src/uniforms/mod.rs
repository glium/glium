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

## Sampler

In order to customize the way a texture is being sampled, you must use a `Sampler`.

```no_run
use std::default::Default;
# let display: glium::Display = unsafe { std::mem::uninitialized() };
# let texture: glium::texture::Texture2d = unsafe { std::mem::uninitialized() };
let uniforms = glium::uniforms::UniformsStorage::new("texture",
    glium::uniforms::Sampler(&texture, glium::uniforms::SamplerBehavior {
        magnify_filter: glium::uniforms::MagnifySamplerFilter::Nearest,
        .. Default::default()
    }));
```

*/
pub use self::value::{UniformValue, IntoUniformValue, UniformType};
pub use self::sampler::*;

mod sampler;
mod value;

/// Object that contains all the uniforms of a program with their bindings points.
///
/// It is more or less a collection of `UniformValue`s.
///
/// You can implement this trait for your own types by redirecting the call to another
///  implementation.
pub trait Uniforms {
    /// Returns a list of the values
    fn visit_values<F: FnMut(&str, &UniformValue)>(self, F);
}

impl<'a, T: 'a> Uniforms for &'a T where T: Uniforms + Copy {
    fn visit_values<F: FnMut(&str, &UniformValue)>(self, output: F) {
        let me = *self;
        me.visit_values(output);
    }
}

/// Object that can be used when you don't have any uniform.
#[deriving(Show, Copy, Clone)]
pub struct EmptyUniforms;

impl Uniforms for EmptyUniforms {
    fn visit_values<F: FnMut(&str, &UniformValue)>(self, _: F) {
    }
}

/// Stores uniforms.
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
pub struct UniformsStorage<'a> {
    uniforms: Vec<(&'a str, UniformValue<'a>)>,
}

impl<'a> UniformsStorage<'a> {
    /// Builds a new storage with a value.
    pub fn new<T>(name: &'a str, value: T) -> UniformsStorage<'a> where T: IntoUniformValue<'a> {
        UniformsStorage {
            uniforms: vec![(name, value.into_uniform_value())]
        }
    }

    /// Adds a value to the storage.
    pub fn add<T>(mut self, name: &'a str, value: T) -> UniformsStorage<'a>
                  where T: IntoUniformValue<'a>
    {
        self.uniforms.push((name, value.into_uniform_value()));
        self
    }
}

impl<'a: 'b, 'b> Uniforms for &'b UniformsStorage<'a> {
    fn visit_values<F: FnMut(&str, &UniformValue)>(self, mut output: F) {
        for &(n, ref v) in self.uniforms.iter() {
            output(n, v)
        }
    }
}
