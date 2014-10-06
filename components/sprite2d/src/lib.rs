/*!
# glium-sprite2d

Usage:

```no_run
# extern crate glium_core;
# extern crate glium_sprite2d;
# fn main() {
# let display: glium_core::Display = unsafe { std::mem::uninitialized() };
# let texture = unsafe { std::mem::uninitialized() };
let sprite2d_sys = glium_sprite2d::Sprite2DSystem::new(&display);

let mut target = display.draw();

target.draw(glium_sprite2d::SpriteDisplay {
    sprite: &sprite2d_sys,
    texture: &texture,
    matrix: &[
        [ 1.0, 0.0, 0.0, 0.0 ],
        [ 0.0, 1.0, 0.0, 0.0 ],
        [ 0.0, 0.0, 1.0, 0.0 ],
        [ 0.0, 0.0, 0.0, 1.0 ]
    ]
});

target.finish();
# }
```

*/

#![feature(phase)]
#![deny(missing_doc)]
#![deny(warnings)]

#[phase(plugin)]
extern crate glium_core_macros;
extern crate glium_core;

#[vertex_format]
#[allow(non_snake_case)]
struct SpriteVertex {
    #[allow(dead_code)]
    iPosition: [f32, ..2],
    #[allow(dead_code)]
    iTexCoords: [f32, ..2],
}

#[uniforms]
#[allow(non_snake_case)]
struct Uniforms<'a> {
    uTexture: &'a glium_core::Texture,
    uMatrix: [[f32, ..4], ..4],
}

/// Object that will allow you to draw 2D sprites with `glium_core`.
pub struct Sprite2DSystem<'d> {
    vertex_buffer: glium_core::VertexBuffer<SpriteVertex>,
    index_buffer: glium_core::IndexBuffer,
    program: glium_core::Program,
}

impl<'d> Sprite2DSystem<'d> {
    /// Builds a new `Sprite2D`.
    pub fn new(display: &'d glium_core::Display) -> Sprite2DSystem {
        Sprite2DSystem {
            vertex_buffer: glium_core::VertexBuffer::new(display,
                vec![
                    SpriteVertex { iPosition: [-1.0, -1.0], iTexCoords: [0.0, 1.0] },
                    SpriteVertex { iPosition: [-1.0,  1.0], iTexCoords: [0.0, 0.0] },
                    SpriteVertex { iPosition: [ 1.0,  1.0], iTexCoords: [1.0, 0.0] },
                    SpriteVertex { iPosition: [ 1.0, -1.0], iTexCoords: [1.0, 1.0] }
                ]
            ),

            index_buffer: glium_core::IndexBuffer::new(display, glium_core::TriangleStrip,
                &[ 1 as u16, 2, 0, 3 ]),

            program: glium_core::Program::new(display, r"
                #version 110

                uniform mat4 uMatrix;

                attribute vec2 iPosition;
                attribute vec2 iTexCoords;

                varying vec2 vTexCoords;

                void main() {
                    gl_Position = uMatrix * vec4(iPosition, 0.0, 1.0);
                    vTexCoords = iTexCoords;
                }
            ", r"
                #version 110
                uniform sampler2D uTexture;
                varying vec2 vTexCoords;

                void main() {
                    gl_FragColor = texture2D(uTexture, vTexCoords);
                }
            ", None).unwrap()
        }
    }
}

/// Represents a command that can be drawn on a target.
///
/// Draw the texture from coordinates (-1, -1) to (1, 1), which means that it covers the whole
///  viewport.
///
/// Using a matrix allows you to control the coordinates.
pub struct SpriteDisplay<'s, 'd: 's, 't, 'm> {
    /// The `Sprite2D` object.
    pub sprite: &'s Sprite2DSystem<'d>,
    /// The texture that you want to draw.
    pub texture: &'t glium_core::Texture,
    /// The matrix that will be used when drawing.
    pub matrix: &'m [[f32, ..4], ..4],
}

impl<'s, 'd, 't, 'm> glium_core::DrawCommand for SpriteDisplay<'s, 'd, 't, 'm> {
    fn draw(self, target: &mut glium_core::Target) {
        let SpriteDisplay { ref sprite, ref texture, ref matrix } = self;
        let &&Sprite2DSystem { ref vertex_buffer, ref index_buffer, ref program, .. } = sprite;

        let uniforms = Uniforms {
            uMatrix: **matrix,
            uTexture: *texture
        };

        target.draw(glium_core::BasicDraw(vertex_buffer, index_buffer, program, &uniforms, &std::default::Default::default()));
    }
}
