/*!
Framebuffers allows you to customize the color, depth and stencil buffers you will draw on.

In order to draw on a texture, use a `SimpleFrameBuffer`. This framebuffer is compatible with
shaders that write to `gl_FragColor`.

```no_run
# let display: glium::Display = unsafe { ::std::mem::uninitialized() };
# let texture: glium::texture::Texture2d = unsafe { ::std::mem::uninitialized() };
let framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display, &texture);
// framebuffer.draw(...);    // draws over `texture`
```

Instead if your shader wants to write to multiple color buffers at once, you must use
a `MultiOutputFrameBuffer`.

```no_run
# let display: glium::Display = unsafe { ::std::mem::uninitialized() };
# let texture1: glium::texture::Texture2d = unsafe { ::std::mem::uninitialized() };
# let texture2: glium::texture::Texture2d = unsafe { ::std::mem::uninitialized() };
let output = &[ ("output1", &texture1), ("output2", &texture2) ];
let framebuffer = glium::framebuffer::MultiOutputFrameBuffer::new(&display, output);
// framebuffer.draw(...);

// example shader:
// 
//     out vec4 output1;
//     out vec4 output2;
//
//     void main() {
//         output1 = vec4(0.0, 0.0, 0.5, 1.0);
//         output2 = vec4(1.0, 0.7, 1.0, 1.0);
//     }
```

**Note**: depth and stencil attachments are not yet implemented.

*/
#![experimental]

use std::kinds::marker::ContravariantLifetime;
use std::sync::Arc;

use texture::{Texture, Texture2d};
use fbo::FramebufferAttachments;

use {DisplayImpl, Program, Surface, GlObject};

use {fbo, gl};

/// A framebuffer which has only one color attachment.
pub struct SimpleFrameBuffer<'a> {
    display: Arc<DisplayImpl>,
    attachments: FramebufferAttachments,
    marker: ContravariantLifetime<'a>,
    dimensions: (u32, u32),
}

impl<'a> SimpleFrameBuffer<'a> {
    /// Creates a `SimpleFrameBuffer`.
    pub fn new(display: &::Display, color: &'a Texture2d) -> SimpleFrameBuffer<'a> {
        let dimensions = (color.get_width(), color.get_height().unwrap());

        SimpleFrameBuffer {
            display: display.context.clone(),
            attachments: FramebufferAttachments {
                colors: vec![(0, color.get_id())],
                depth: None,
                stencil: None
            },
            marker: ContravariantLifetime,
            dimensions: dimensions,
        }
    }
}

impl<'a> Surface for SimpleFrameBuffer<'a> {
    fn clear_color(&mut self, red: f32, green: f32, blue: f32, alpha: f32) {
        fbo::clear_color(&self.display, Some(&self.attachments), red, green, blue, alpha)
    }

    fn clear_depth(&mut self, value: f32) {
        fbo::clear_depth(&self.display, Some(&self.attachments), value)
    }

    fn clear_stencil(&mut self, value: int) {
        fbo::clear_stencil(&self.display, Some(&self.attachments), value)
    }

    fn get_dimensions(&self) -> (uint, uint) {
        (self.dimensions.0 as uint, self.dimensions.1 as uint)
    }

    fn get_depth_buffer_bits(&self) -> Option<u16> {
        None
    }

    fn get_stencil_buffer_bits(&self) -> Option<u16> {
        None
    }

    fn draw<V, I, ID, U>(&mut self, vb: &::VertexBuffer<V>, ib: &I, program: &::Program,
        uniforms: &U, draw_parameters: &::DrawParameters) where I: ::index_buffer::ToIndicesSource<ID>,
        U: ::uniforms::Uniforms, ID: ::index_buffer::Index
    {
        use index_buffer::ToIndicesSource;

        if draw_parameters.depth_function.requires_depth_buffer() && !self.has_depth_buffer() {
            panic!("Requested a depth function but no depth buffer is attached");
        }

        fbo::draw(&self.display, Some(&self.attachments), vb, &ib.to_indices_source(),
                  program, uniforms, draw_parameters)
    }

    fn get_blit_helper(&self) -> ::BlitHelper {
        ::BlitHelper(&self.display, Some(&self.attachments))
    }
}

/// This struct is useless for the moment.
pub struct MultiOutputFrameBuffer<'a> {
    display: Arc<DisplayImpl>,
    marker: ContravariantLifetime<'a>,
    dimensions: (u32, u32),
    color_attachments: Vec<(String, gl::types::GLuint)>,
}

impl<'a> MultiOutputFrameBuffer<'a> {
    /// Creates a new `MultiOutputFramebuffer`.
    ///
    /// # Panic
    ///
    /// Panics if all attachments don't have the same dimensions.
    ///
    pub fn new(display: &::Display, color_attachments: &[(&str, &'a Texture2d)])
               -> MultiOutputFrameBuffer<'a>
    {
        let mut attachments = Vec::new();
        let mut dimensions = None;

        for &(name, texture) in color_attachments.iter() {
            let tex_dims = (texture.get_width(), texture.get_height().unwrap());

            if let Some(ref dimensions) = dimensions {
                if dimensions != &tex_dims {
                    panic!("All textures of a MultiOutputFrameBuffer must have \
                            the same dimensions");
                }
            }

            dimensions = Some(tex_dims);
            attachments.push((name.to_string(), texture.get_id()));
        }

        if dimensions.is_none() {
            panic!("Cannot pass an empty color_attachments when \
                    creating a MultiOutputFrameBuffer");
        }

        MultiOutputFrameBuffer {
            display: display.context.clone(),
            marker: ContravariantLifetime,
            dimensions: dimensions.unwrap(),
            color_attachments: attachments,
        }
    }

    fn build_attachments(&self, program: &Program) -> FramebufferAttachments {
        let mut colors = Vec::new();

        for &(ref name, texture) in self.color_attachments.iter() {
            let location = match program.get_frag_data_location(name.as_slice()) {
                Some(l) => l,
                None => panic!("The fragment output `{}` was not found in the program", name)
            };

            colors.push((location, texture));
        }

        FramebufferAttachments {
            colors: colors,
            depth: None,
            stencil: None,
        }
    }
}
