#![feature(plugin)]
#![feature(unboxed_closures)]
#![plugin(glium_macros)]

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
fn empty_texture1d_u8u8u8u8() {
    let display = support::build_display();

    let texture = glium::texture::Texture1d::new_empty(&display,
                                                       glium::texture::UncompressedFloatFormat::
                                                           U8U8U8U8, 128);

    display.assert_no_error();
    drop(texture);
    display.assert_no_error();
}

#[test]
fn depth_texture_1d_creation() {    
    let display = support::build_display();

    let texture = glium::texture::DepthTexture1d::new(&display, vec![0.0, 0.0, 0.0, 0.0f32]);

    assert_eq!(texture.get_width(), 4);
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
fn empty_texture2d_u8u8u8u8() {
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
fn depth_texture_2d_creation() {    
    let display = support::build_display();

    let texture = glium::texture::DepthTexture2d::new(&display, vec![
        vec![0.0, 0.0, 0.0, 0.0f32],
        vec![0.0, 0.0, 0.0, 0.0f32],
        vec![0.0, 0.0, 0.0, 0.0f32],
    ]);

    assert_eq!(texture.get_width(), 4);
    assert_eq!(texture.get_height(), Some(3));
    assert_eq!(texture.get_depth(), None);
    assert_eq!(texture.get_array_size(), None);

    display.assert_no_error();
}

#[test]
#[ignore]   // `thread 'empty_depth_texture2d_f32' panicked at 'assertion failed: version >= &GlVersion(3, 0)'`
fn empty_depth_texture2d_f32() {
    let display = support::build_display();

    let texture = glium::texture::DepthTexture2d::new_empty(&display,
                                                            glium::texture::DepthFormat::F32,
                                                            128, 128);

    display.assert_no_error();
    drop(texture);
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
