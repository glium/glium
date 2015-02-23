/*!
A uniform is a global variable in your program. In order to draw something, you will need to
give `glium` the values of all your uniforms. Objects that implement the `Uniform` trait are 
here to do that.

There are two primarly ways to do this. The first one is to create your own structure and put
the `#[uniforms]` attribute on it.

For example:

```no_run
# #![feature(plugin)]
# #![plugin(glium_macros)]
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

The second way is to use the `uniform!` macro provided by glium:

```no_run
#[macro_use]
extern crate glium;

# fn main() {
# let display: glium::Display = unsafe { std::mem::uninitialized() };
# let tex: f32 = unsafe { std::mem::uninitialized() };
# let matrix: f32 = unsafe { std::mem::uninitialized() };
let uniforms = uniform! {
    texture: tex,
    matrix: matrix
};
# }
```

In both situations, each field must implement the `UniformValue` trait.

## Samplers

In order to customize the way a texture is being sampled, you must use a `Sampler`.

```no_run
#[macro_use]
extern crate glium;

# fn main() {
use std::default::Default;
# let display: glium::Display = unsafe { std::mem::uninitialized() };
# let texture: glium::texture::Texture2d = unsafe { std::mem::uninitialized() };
let uniforms = uniform! {
    texture: glium::uniforms::Sampler::new(&texture)
                        .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
};
# }
```

## Blocks

In GLSL, you can choose to use a uniform *block*. When you use a block, you first need to
upload the content of this block in the video memory thanks to a `UniformBuffer`. Then you
can link the buffer to the name of the block, just like any other uniform.

```no_run
#[macro_use]
extern crate glium;
# fn main() {
# let display: glium::Display = unsafe { std::mem::uninitialized() };
# let texture: glium::texture::Texture2d = unsafe { std::mem::uninitialized() };

let program = glium::Program::from_source(&display,
    "
        #version 110

        attribute vec2 position;

        void main() {
            gl_Position = vec4(position, 0.0, 1.0);
        }
    ",
    "
        #version 330
        uniform layout(std140);

        uniform MyBlock {
            vec3 color;
        };

        void main() {
            gl_FragColor = vec4(color, 1.0);
        }
    ",
    None);

let buffer = glium::uniforms::UniformBuffer::new_if_supported(&display,
                                                              (0.5f32, 0.5f32, 0.5f32)).unwrap();

let uniforms = uniform! {
    MyBlock: &buffer
};
# }
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
