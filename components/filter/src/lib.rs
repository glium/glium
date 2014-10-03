/*!

This crate allows you to easily create filter for your scenes.

## Fragment shader

The fragment shader must have a uniform named `uTexture`, and texture coordinates are
 passed with the `vTexCoords` varying.

*/

#![feature(if_let)]
#![feature(phase)]
#![feature(tuple_indexing)]
#![deny(warnings)]
#![deny(missing_doc)]

#[phase(plugin)]
extern crate glium_core_macros;

extern crate glium_core;

use std::sync::Mutex;

#[vertex_format]
#[allow(dead_code)]
#[allow(non_snake_case)]
struct VertexFormat {
    iPosition: [f32, ..2],
    iTexCoords: [f32, ..2],
}

/// 
pub struct Filter<'d> {
    display: &'d glium_core::Display,
    texture: Mutex<Option<glium_core::Texture>>,
    vertex_buffer: glium_core::VertexBuffer<VertexFormat>,
    index_buffer: glium_core::IndexBuffer,
    program: glium_core::Program,
}

impl<'d> Filter<'d> {
    /// Builds a new `Filter`.
    pub fn new(display: &'d glium_core::Display, fragment_shader: &str) -> Filter<'d> {
        Filter {
            display: display,
            texture: Mutex::new(None),

            vertex_buffer: glium_core::VertexBuffer::new(display, 
                vec![
                    VertexFormat { iPosition: [-1.0,  1.0], iTexCoords: [0.0, 1.0] },
                    VertexFormat { iPosition: [ 1.0,  1.0], iTexCoords: [1.0, 1.0] },
                    VertexFormat { iPosition: [-1.0, -1.0], iTexCoords: [0.0, 0.0] },
                    VertexFormat { iPosition: [ 1.0, -1.0], iTexCoords: [1.0, 0.0] },
                ]
            ),

            index_buffer: glium_core::IndexBuffer::new(display, glium_core::TrianglesList,
                &[ 0u16, 1, 2, 1, 3, 2 ]),

            program: glium_core::Program::new(display, "
                #version 110

                attribute vec2 iPosition;
                attribute vec2 iTexCoords;

                varying vec2 vTexCoords;

                void main() {
                    gl_Position = vec4(iPosition, 0.0, 1.0);
                    vTexCoords = iTexCoords;
                }", fragment_shader, None).unwrap(),
        }
    }
}

/// Draw command to use a filter.
///
/// The closure takes a `Target` which must be used to draw the objects to filter.
pub struct WithFilter<'f, 'd: 'f, 'c>(pub &'f Filter<'d>, pub |&mut glium_core::Target|:'c);

impl<'f, 'd, 'c> glium_core::DrawCommand for WithFilter<'f, 'd, 'c> {
    fn draw(self, target: &mut glium_core::Target) {
        let mut texture = self.0.texture.lock();

        let dimensions = target.get_dimensions();

        {
            let mut clear = false;
            if let Some(tex) = texture.as_mut() {
                if tex.get_width() != dimensions.0 || tex.get_height() != dimensions.1 {
                    clear = true;
                }
            }
            if clear {
                *texture = None;
            }
        }

        if texture.is_none() {
            let tmp_data = Vec::from_elem(dimensions.0 * dimensions.1, (0u8, 0u8, 0u8, 0u8));
            *texture = Some(glium_core::Texture::new(self.0.display, tmp_data.as_slice(),
                                                     dimensions.0, dimensions.1, 1, 1));
        }

        // drawing on the texture
        self.1(&mut texture.as_mut().unwrap().draw());

        // building the uniforms
        let mut uniforms = self.0.program.build_uniforms();
        uniforms.set_texture("uTexture", texture.as_ref().unwrap());

        glium_core::BasicDraw(&self.0.vertex_buffer, &self.0.index_buffer, &uniforms)
            .draw(target);
    }
}
