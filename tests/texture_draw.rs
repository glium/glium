#![feature(plugin)]
#![feature(unboxed_closures)]

#[plugin]
extern crate glium_macros;

extern crate glutin;

#[macro_use]
extern crate glium;

use std::default::Default;
use glium::Surface;

mod support;

#[test]
fn texture_2d_draw() {
    let display = support::build_display();
    let (vb, ib) = support::build_rectangle_vb_ib(&display);

    let texture = glium::texture::Texture2d::new(&display, &vec![
        vec![(255, 0, 0, 255), (255, 0, 0, 255)],
        vec![(255, 0, 0, 255), (255, 0, 0, 255u8)],
    ]);

    let program = glium::Program::from_source(&display,
        "
            #version 110

            attribute vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        ",
        "
            #version 110

            uniform sampler2D texture;

            void main() {
                gl_FragColor = texture2D(texture, vec2(0.5, 0.5));
            }
        ",
        None).unwrap();

    let output = support::build_renderable_texture(&display);
    output.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    output.as_surface().draw(&vb, &ib, &program, &uniform!{ texture: &texture },
                             &Default::default()).unwrap();

    let data: Vec<Vec<(f32, f32, f32, f32)>> = output.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(1.0, 0.0, 0.0, 1.0));
        }
    }

    display.assert_no_error();
}

#[test]
fn compressed_texture_2d_draw() {
    let display = support::build_display();
    let (vb, ib) = support::build_rectangle_vb_ib(&display);

    let texture = glium::texture::CompressedTexture2d::new(&display, &vec![
        vec![(255, 0, 0, 255), (255, 0, 0, 255)],
        vec![(255, 0, 0, 255), (255, 0, 0, 255u8)],
    ]);

    let program = glium::Program::from_source(&display,
        "
            #version 110

            attribute vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        ",
        "
            #version 110

            uniform sampler2D texture;

            void main() {
                gl_FragColor = texture2D(texture, vec2(0.5, 0.5));
            }
        ",
        None).unwrap();

    let output = support::build_renderable_texture(&display);
    output.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    output.as_surface().draw(&vb, &ib, &program, &uniform!{ texture: &texture },
                             &Default::default()).unwrap();

    let data: Vec<Vec<(f32, f32, f32, f32)>> = output.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(1.0, 0.0, 0.0, 1.0));
        }
    }

    display.assert_no_error();
}
