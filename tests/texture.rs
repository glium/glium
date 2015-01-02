#![feature(phase)]
#![feature(unboxed_closures)]

#[phase(plugin)]
extern crate glium_macros;

extern crate glutin;
extern crate glium;

use glium::{Texture, Surface};

mod support;

#[test]
fn texture_1d_creation() {    
    let display = support::build_display();

    let texture = glium::texture::Texture1d::new(&display, vec![
        (0, 0, 0, 0),
        (0, 0, 0, 0),
        (0, 0, 0, 0u8),
    ]);

    assert_eq!(texture.get_width(), 3);
    assert_eq!(texture.get_height(), None);
    assert_eq!(texture.get_depth(), None);
    assert_eq!(texture.get_array_size(), None);

    display.assert_no_error();
}

#[test]
fn texture_2d_creation() {    
    let display = support::build_display();

    let texture = glium::texture::Texture2d::new(&display, vec![
        vec![(0, 0, 0, 0), (0, 0, 0, 0)],
        vec![(0, 0, 0, 0), (0, 0, 0, 0)],
        vec![(0, 0, 0, 0), (0, 0, 0, 0u8)],
    ]);

    assert_eq!(texture.get_width(), 2);
    assert_eq!(texture.get_height(), Some(3));
    assert_eq!(texture.get_depth(), None);
    assert_eq!(texture.get_array_size(), None);

    display.assert_no_error();
}

#[test]
fn texture_3d_creation() {    
    let display = support::build_display();

    let texture = glium::texture::Texture3d::new(&display, vec![
        vec![
            vec![(0, 0, 0, 0)],
            vec![(0, 0, 0, 0)],
        ],
        vec![
            vec![(0, 0, 0, 0)],
            vec![(0, 0, 0, 0)],
        ],
        vec![
            vec![(0, 0, 0, 0)],
            vec![(0, 0, 0, 0u8)],
        ],
    ]);

    assert_eq!(texture.get_width(), 1);
    assert_eq!(texture.get_height(), Some(2));
    assert_eq!(texture.get_depth(), Some(3));
    assert_eq!(texture.get_array_size(), None);

    display.assert_no_error();
}

#[test]
#[cfg(feature = "gl_extensions")]
fn texture_2d_read() {
    let display = support::build_display();

    // we use only powers of two in order to avoid float rounding errors
    let texture = glium::texture::Texture2d::new(&display, vec![
        vec![(0u8, 1u8, 2u8), (4u8, 8u8, 16u8)],
        vec![(32u8, 64u8, 128u8), (32u8, 16u8, 4u8)],
    ]);

    let read_back: Vec<Vec<(u8, u8, u8)>> = texture.read();

    assert_eq!(read_back[0][0], (0, 1, 2));
    assert_eq!(read_back[0][1], (4, 8, 16));
    assert_eq!(read_back[1][0], (32, 64, 128));
    assert_eq!(read_back[1][1], (32, 16, 4));

    display.assert_no_error();
}

#[test]
fn compressed_texture_2d_creation() {
    let display = support::build_display();

    let texture = glium::texture::CompressedTexture2d::new(&display, vec![
        vec![(0, 0, 0, 0), (0, 0, 0, 0)],
        vec![(0, 0, 0, 0), (0, 0, 0, 0)],
        vec![(0, 0, 0, 0), (0, 0, 0, 0u8)],
    ]);

    assert_eq!(texture.get_width(), 2);
    assert_eq!(texture.get_height(), Some(3));
    assert_eq!(texture.get_depth(), None);
    assert_eq!(texture.get_array_size(), None);

    display.assert_no_error();
}

#[test]
fn empty_texture2d() {
    let display = support::build_display();

    let texture = glium::texture::Texture2d::new_empty(&display,
                                                       glium::texture::UncompressedFloatFormat::
                                                           U8U8U8U8,
                                                       128, 128);

    display.assert_no_error();

    drop(texture);
    
    display.assert_no_error();
}

#[test]
#[ignore]  // FIXME: FAILING TEST
#[cfg(feature = "gl_extensions")]
fn render_to_texture2d() {
    use std::default::Default;

    let display = support::build_display();
    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = glium::Texture2d::new_empty(&display,
                                              glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                              1024, 1024);
    let params = Default::default();
    texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params);

    let read_back: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    assert_eq!(read_back[0][0], (255, 0, 0, 255));
    assert_eq!(read_back[512][512], (255, 0, 0, 255));
    assert_eq!(read_back[1023][1023], (255, 0, 0, 255));
    
    display.assert_no_error();
}
