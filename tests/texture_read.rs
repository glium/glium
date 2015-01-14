#![feature(plugin)]
#![feature(unboxed_closures)]

#[plugin]
extern crate glium_macros;

extern crate glutin;
extern crate glium;

use glium::{Texture, Surface};

mod support;

#[test]
fn texture_2d_read() {
    let display = support::build_display();

    // we use only powers of two, in order to avoid float rounding errors
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
#[should_fail]
fn empty_pixel_buffer() {
    let display = support::build_display();

    let pixel_buffer = glium::pixel_buffer::PixelBuffer::new_empty(&display, 128 * 128);
    display.assert_no_error();

    let _: Vec<Vec<(u8, u8, u8)>> = pixel_buffer.read_if_supported().unwrap();
}

#[test]
fn texture_2d_read_pixelbuffer() {
    let display = support::build_display();

    // we use only powers of two, in order to avoid float rounding errors
    let texture = glium::texture::Texture2d::new(&display, vec![
        vec![(0u8, 1u8, 2u8), (4u8, 8u8, 16u8)],
        vec![(32u8, 64u8, 128u8), (32u8, 16u8, 4u8)],
    ]);

    let read_back: Vec<Vec<(u8, u8, u8)>> = match texture.read_to_pixel_buffer()
                                                         .read_if_supported() {
        Some(d) => d,
        None => return
    };

    assert_eq!(read_back[0][0], (0, 1, 2));
    assert_eq!(read_back[0][1], (4, 8, 16));
    assert_eq!(read_back[1][0], (32, 64, 128));
    assert_eq!(read_back[1][1], (32, 16, 4));

    display.assert_no_error();
}
