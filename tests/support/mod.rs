/*!
Test supports module.

*/

#![allow(dead_code)]

use glutin;
use glium::{self, DisplayBuild};

use std::env;

/// Returns true if we are executing headless tests.
pub fn is_headless() -> bool {
    env::var("HEADLESS_TESTS").is_ok()
}

/// Builds a headless display for tests.
#[cfg(feature = "headless")]
pub fn build_display() -> glium::Display {
    let display = if is_headless() {
        glutin::HeadlessRendererBuilder::new(1024, 768).with_gl_debug_flag(true)
                                                       .build_glium().unwrap()
    } else {
        glutin::WindowBuilder::new().with_gl_debug_flag(true).with_visibility(false)
                                    .build_glium().unwrap()
    };

    display
}

/// Builds a headless display for tests.
#[cfg(not(feature = "headless"))]
pub fn build_display() -> glium::Display {
    assert!(!is_headless());
    glutin::WindowBuilder::new().with_gl_debug_flag(true).with_visibility(false)
                                .build_glium().unwrap()
}

/// Builds a 2x2 unicolor texture.
pub fn build_unicolor_texture2d(display: &glium::Display, red: f32, green: f32, blue: f32)
    -> glium::Texture2d
{
    let color = ((red * 255.0) as u8, (green * 255.0) as u8, (blue * 255.0) as u8);

    glium::texture::Texture2d::new(display, vec![
        vec![color, color],
        vec![color, color],
    ])
}

/// Builds a vertex buffer, index buffer, and program, to draw red `(1.0, 0.0, 0.0, 1.0)` to the whole screen.
pub fn build_fullscreen_red_pipeline(display: &glium::Display) -> (glium::vertex::VertexBufferAny,
    glium::IndexBuffer, glium::Program)
{
    #[derive(Copy)]
    struct Vertex {
        position: [f32; 2],
    }

    implement_vertex!(Vertex, position);

    (
        glium::VertexBuffer::new(display, vec![
            Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
            Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
        ]).into_vertex_buffer_any(),

        glium::IndexBuffer::new(display, glium::index::TriangleStrip(vec![0u8, 1, 2, 3])),

        glium::Program::from_source(display,
            "
                #version 110

                attribute vec2 position;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                }
            ",
            "
                #version 110

                void main() {
                    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
                }
            ",
            None).unwrap()
    )
}

/// Builds a vertex buffer and an index buffer corresponding to a rectangle.
///
/// The vertex buffer has the "position" attribute of type "vec2".
pub fn build_rectangle_vb_ib(display: &glium::Display)
    -> (glium::vertex::VertexBufferAny, glium::IndexBuffer)
{
    #[derive(Copy)]
    struct Vertex {
        position: [f32; 2],
    }

    implement_vertex!(Vertex, position);

    (
        glium::VertexBuffer::new(display, vec![
            Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
            Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
        ]).into_vertex_buffer_any(),

        glium::IndexBuffer::new(display, glium::index::TriangleStrip(vec![0u8, 1, 2, 3])),
    )
}

/// Builds a texture suitable for rendering.
pub fn build_renderable_texture(display: &glium::Display) -> glium::Texture2d {
    glium::Texture2d::empty(display, 1024, 1024)
}
