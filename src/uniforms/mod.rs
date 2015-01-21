/*!
A uniform is a global variable in your program. In order to draw something, you will need to
 give `glium` the values of all your uniforms. Objects that implement the `Uniform` trait are 
 here to do that.

The recommended way to is to create your own structure and put the `#[uniforms]` attribute
 on it.

For example:

```no_run
# #![feature(plugin)]
#[plugin]
extern crate glium_macros;
# extern crate glium;
# fn main() {

#[uniforms]
struct Uniforms<'a> {
    texture: &'a glium::Texture2d,
    matrix: [[f32; 4]; 4],
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
pub use self::buffer::UniformBuffer;
pub use self::sampler::{SamplerWrapFunction, MagnifySamplerFilter, MinifySamplerFilter};
pub use self::sampler::{Sampler, SamplerBehavior};
pub use self::uniforms::{EmptyUniforms, UniformsStorage};
pub use self::value::{UniformValue, IntoUniformValue, UniformType};

use program;

mod buffer;
mod sampler;
mod uniforms;
mod value;

/// Object that contains the values of all the uniforms to bind to a program.
pub trait Uniforms {
    /// Calls the parameter once with the name and value of each uniform.
    fn visit_values<F: FnMut(&str, &UniformValue)>(self, F);
}

/// Objects that are suitable for being binded to a uniform block.
pub trait UniformBlock: Copy {
    /// Checks whether the uniforms' layout matches the given block.
    fn matches(&program::UniformBlock) -> bool;
}

// TODO: hacky (see #189)
impl<'a, T: 'a> Uniforms for &'a T where T: Uniforms + Copy {
    fn visit_values<F: FnMut(&str, &UniformValue)>(self, output: F) {
        let me = *self;
        me.visit_values(output);
    }
}
