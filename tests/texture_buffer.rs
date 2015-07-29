#[macro_use]
extern crate glium;

use glium::Surface;
use glium::texture::buffer_texture::BufferTexture;
use glium::texture::buffer_texture::BufferTextureType;

mod support;

#[test]
fn empty() {
    let display = support::build_display();

    let texture: BufferTexture<(u8, u8, u8, u8)> = BufferTexture::empty(&display, 32,
                                                                        BufferTextureType::Float)
                                                                        .unwrap();

    display.assert_no_error(None);
    drop(texture);
    display.assert_no_error(None);
}

#[test]
fn sample() {
    let display = support::build_display();

    let data = &[(255, 0, 255, 255)];
    let buf_tex: BufferTexture<(u8, u8, u8, u8)> = BufferTexture::new(&display, data,
                                                                      BufferTextureType::Float)
                                                                      .unwrap();

    let (vb, ib) = support::build_rectangle_vb_ib(&display);

    let program = glium::Program::from_source(&display,
        "
            #version 110

            attribute vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        ",
        "
            #version 140

            uniform samplerBuffer tex;

            void main() {
                gl_FragColor = texelFetch(tex, 0);
            }
        ",
        None).unwrap();

    let output = support::build_renderable_texture(&display);
    output.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    output.as_surface().draw(&vb, &ib, &program, &uniform!{ tex: &buf_tex },
                             &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = output.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 0, 255, 255));
        }
    }

    display.assert_no_error(None);
}
