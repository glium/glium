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

    fn draw<V, I, U>(&mut self, vb: &::VertexBuffer<V>, ib: &I, program: &::Program,
        uniforms: &U, draw_parameters: &::DrawParameters) where I: ::IndicesSource,
        U: ::uniforms::Uniforms
    {
        if draw_parameters.depth_function.requires_depth_buffer() && !self.has_depth_buffer() {
            panic!("Requested a depth function but no depth buffer is attached");
        }

        fbo::draw(&self.display, Some(&self.attachments), vb, ib, program, uniforms, draw_parameters)
    }

    fn get_blit_helper(&self) -> ::BlitHelper {
        ::BlitHelper(&self.display, Some(&self.attachments))
    }
}

pub struct MultiOutputFrameBuffer<'a> {
    display: Arc<DisplayImpl>,
    marker: ContravariantLifetime<'a>,
    dimensions: (u32, u32),
    color_attachments: Vec<(String, gl::types::GLuint)>,
}

impl<'a> MultiOutputFrameBuffer<'a> {
    /// Creates a new `MultiOutputFramebuffer`.
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
