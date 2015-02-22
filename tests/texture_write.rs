extern crate glutin;
#[macro_use]
extern crate glium;

use glium::{Texture, Surface};

mod support;

#[test]
fn texture_2d_write() {
    let display = support::build_display();

    // we use only powers of two, in order to avoid float rounding errors
    let texture = glium::texture::Texture2d::new(&display, vec![
        vec![(0u8, 1u8, 2u8), (4u8, 8u8, 16u8)],
        vec![(32u8, 64u8, 128u8), (32u8, 16u8, 4u8)],
    ]);

    texture.write(glium::Rect { bottom: 1, left: 1, width: 1, height: 1 },
                  vec![vec![(128u8, 64u8, 2u8)]]);

    let read_back: Vec<Vec<(u8, u8, u8)>> = texture.read();
    assert_eq!(read_back[0][0], (0, 1, 2));
    assert_eq!(read_back[0][1], (4, 8, 16));
    assert_eq!(read_back[1][0], (32, 64, 128));
    assert_eq!(read_back[1][1], (128, 64, 2));

    display.assert_no_error();
}
