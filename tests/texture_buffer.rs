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
                                                                        BufferTextureType::Float).unwrap();

    display.assert_no_error(None);
    drop(texture);
    display.assert_no_error(None);
}
