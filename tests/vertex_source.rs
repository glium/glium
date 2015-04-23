extern crate glutin;

#[macro_use]
extern crate glium;

use glium::Surface;
use std::default::Default;

mod support;

#[test]
fn attributes_marker() {
    let display = support::build_display();

    let program = match glium::Program::from_source(&display,
        "
            #version 140

            void main() {
                if (gl_VertexID == 0) {
                    gl_Position = vec4(-1.0, 1.0, 0.0, 1.0);
                } else if (gl_VertexID == 1) {
                    gl_Position = vec4(1.0, 1.0, 0.0, 1.0);
                } else if (gl_VertexID == 2) {
                    gl_Position = vec4(-1.0, -1.0, 0.0, 1.0);
                } else if (gl_VertexID == 3) {
                    gl_Position = vec4(1.0, -1.0, 0.0, 1.0);
                }
            }
        ",
        "
            #version 140

            out vec4 color;

            void main() {
                color = vec4(1.0, 0.0, 0.0, 1.0);
            }
        ",
        None) {
        Ok(p) => p,
        _ => return
    };

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(glium::vertex::EmptyVertexAttributes { len: 4 },
                              &glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip),
                              &program, &uniform!{},
                              &std::default::Default::default()).unwrap();

    let data: Vec<Vec<(f32, f32, f32, f32)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(1.0, 0.0, 0.0, 1.0));
        }
    }
    
    display.assert_no_error();
}

#[test]
fn attributes_marker_indices() {
    let display = support::build_display();

    let program = match glium::Program::from_source(&display,
        "
            #version 140

            void main() {
                if (gl_VertexID == 0) {
                    gl_Position = vec4(-1.0, 1.0, 0.0, 1.0);
                } else if (gl_VertexID == 1) {
                    gl_Position = vec4(1.0, 1.0, 0.0, 1.0);
                } else if (gl_VertexID == 2) {
                    gl_Position = vec4(-1.0, -1.0, 0.0, 1.0);
                } else if (gl_VertexID == 3) {
                    gl_Position = vec4(1.0, -1.0, 0.0, 1.0);
                }
            }
        ",
        "
            #version 140

            out vec4 color;

            void main() {
                color = vec4(1.0, 0.0, 0.0, 1.0);
            }
        ",
        None) {
        Ok(p) => p,
        _ => return
    };

    let indices = glium::index::TriangleStrip(vec![0u16, 1, 2, 3]);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(glium::vertex::EmptyVertexAttributes { len: 4 },
                              &indices, &program, &uniform!{},
                              &std::default::Default::default()).unwrap();

    let data: Vec<Vec<(f32, f32, f32, f32)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(1.0, 0.0, 0.0, 1.0));
        }
    }
    
    display.assert_no_error();
}
