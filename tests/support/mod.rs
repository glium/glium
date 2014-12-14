/*!
Test supports module.

*/

#![allow(dead_code)]

use glutin;
use glium::{mod, DisplayBuild};

use std::os;

/// Returns true if we are executed headless tests.
pub fn is_headless() -> bool {
    os::getenv("HEADLESS_TESTS").is_some()
}

/// Builds a headless display for tests.
#[cfg(feature = "headless")]
pub fn build_display() -> glium::Display {
    let display = if is_headless() {
        glutin::HeadlessRendererBuilder::new(1024, 768).build_glium().unwrap()
    } else {
        glutin::WindowBuilder::new().with_visibility(false).build_glium().unwrap()
    };

    unsafe {
        display.set_debug_callback_sync(|&mut: msg: String, _, _, severity: glium::debug::Severity| {
            if severity == glium::debug::Severity::Medium ||
               severity == glium::debug::Severity::High
            {
                panic!("{}", msg);
            }
        })
    };

    display
}

/// Builds a headless display for tests.
#[cfg(not(feature = "headless"))]
pub fn build_display() -> glium::Display {
    assert!(!is_headless());

    let display = glutin::WindowBuilder::new().with_visibility(false).build_glium().unwrap();

    unsafe {
        display.set_debug_callback_sync(|&mut: msg: String, _, _, severity: glium::debug::Severity| {
            if severity == glium::debug::Severity::Medium ||
               severity == glium::debug::Severity::High
            {
                panic!("{}", msg);
            }
        })
    };

    display
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

#[vertex_format]
#[deriving(Copy)]
struct Vertex {
    position: [f32, ..2],
}

/// Builds a VB, IB and program that draw the red color `(1.0, 0.0, 0.0, 1.0)` on the whole screen.
pub fn build_fullscreen_red_pipeline(display: &glium::Display) -> (glium::VertexBuffer<Vertex>,
    glium::IndexBuffer, glium::Program)
{
    (
        glium::VertexBuffer::new(display, vec![
            Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
            Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
        ]),

        glium::IndexBuffer::new(display, glium::index_buffer::TriangleStrip(vec![0u8, 1, 2, 3])),

        glium::Program::new(display,
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

/// Builds a VB and an IB corresponding to a rectangle.
///
/// The VB has the "position" attribute of type "vec2".
pub fn build_rectangle_vb_ib(display: &glium::Display)
    -> (glium::VertexBuffer<Vertex>, glium::IndexBuffer)
{
    (
        glium::VertexBuffer::new(display, vec![
            Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
            Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
        ]),

        glium::IndexBuffer::new(display, glium::index_buffer::TriangleStrip(vec![0u8, 1, 2, 3])),
    )
}
