/*!
A uniform is a global variable in your program. In order to draw something, you will need to
give `glium` the values of all your uniforms. Objects that implement the `Uniform` trait are
here to do that.

There are two primarly ways to do this. The first one is to create your own structure and put
the `#[uniforms]` attribute on it. See the `glium_macros` crate for more infos.

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

let buffer = glium::uniforms::UniformBuffer::new(&display, (0.5f32, 0.5f32, 0.5f32)).unwrap();

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
pub use self::value::{UniformValue, UniformType};

use buffer::Content as BufferContent;
use buffer::Buffer;
use program;
use program::BlockLayout;

mod bind;
mod buffer;
mod sampler;
mod uniforms;
mod value;

/// Object that contains the values of all the uniforms to bind to a program.
///
/// Objects of this type can be passed to the `draw()` function.
pub trait Uniforms {
    /// Calls the parameter once with the name and value of each uniform.
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, F);
}

/// Error about a block layout mismatch.
#[derive(Clone, Debug)]
pub enum LayoutMismatchError {
    /// There is a mismatch in the type of one element.
    TypeMismatch {
        /// Type expected by the shader.
        expected: UniformType,
        /// Type that you gave.
        obtained: UniformType,
    },

    /// The expected layout is totally different from what we have.
    LayoutMismatch {
        /// Layout expected by the shader.
        expected: BlockLayout,
        /// Layout of the input.
        obtained: BlockLayout,
    },

    /// The type of data is good, but there is a misalignment.
    OffsetMismatch {
        /// Expected offset of a member.
        expected: usize,
        /// Offset of the same member in the input.
        obtained: usize,
    },

    /// There is a mismatch in a submember of this layout.
    ///
    /// This is kind of a hierarchy inside the `LayoutMismatchError`s.
    MemberMismatch {
        /// Name of the field.
        member: String,
        /// The sub-error.
        err: Box<LayoutMismatchError>,
    },

    /// A field is missing in either the expected of the input data layout.
    MissingField {
        /// Name of the field.
        name: String,
    },
}

/// Value that can be used as the value of a uniform.
///
/// This includes buffers and textures for example.
pub trait AsUniformValue {
    /// Builds a `UniformValue`.
    fn as_uniform_value(&self) -> UniformValue;
}

// TODO: no way to bind a slice
impl<'a, T: ?Sized> AsUniformValue for &'a Buffer<T> where T: UniformBlock + BufferContent {
    #[inline]
    fn as_uniform_value(&self) -> UniformValue {
        #[inline]
        fn f<T: ?Sized>(block: &program::UniformBlock) -> Result<(), LayoutMismatchError>
            where T: UniformBlock + BufferContent
        {
            // TODO: more checks?
            T::matches(&block.layout, 0)
        }

        UniformValue::Block(self.as_slice_any(), f::<T>)
    }
}

/// Objects that are suitable for being inside a uniform block or a SSBO.
pub trait UniformBlock {        // TODO: `: Copy`, but unsized structs don't impl `Copy`
    /// Checks whether the uniforms' layout matches the given block if `Self` starts at
    /// the given offset.
    fn matches(&BlockLayout, base_offset: usize) -> Result<(), LayoutMismatchError>;

    /// Builds the `BlockLayout` corresponding to the current object.
    fn build_layout(base_offset: usize) -> BlockLayout;
}

impl<T> UniformBlock for [T] where T: UniformBlock {
    fn matches(layout: &BlockLayout, base_offset: usize) -> Result<(), LayoutMismatchError> {
        if let &BlockLayout::DynamicSizedArray { ref content } = layout {
            <T as UniformBlock>::matches(content, base_offset).map_err(|err| {
                LayoutMismatchError::MemberMismatch {
                    member: "<dynamic array content>".to_owned(),
                    err: Box::new(err),
                }
            })

        } else if let &BlockLayout::Array { ref content, .. } = layout {
            <T as UniformBlock>::matches(content, base_offset).map_err(|err| {
                LayoutMismatchError::MemberMismatch {
                    member: "<dynamic array content>".to_owned(),
                    err: Box::new(err),
                }
            })

        } else {
            Err(LayoutMismatchError::LayoutMismatch {
                expected: layout.clone(),
                obtained: <Self as UniformBlock>::build_layout(base_offset),
            })
        }
    }

    #[inline]
    fn build_layout(base_offset: usize) -> BlockLayout {
        BlockLayout::DynamicSizedArray {
            content: Box::new(<T as UniformBlock>::build_layout(base_offset)),
        }
    }
}
