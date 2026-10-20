#[macro_use]
extern crate glium;

use glium::Surface;

mod support;

#[test]
#[should_panic(expected = "rectangle out of bounds")]
fn test_read() {
    let display = support::build_display();

    let texture =
        glium::texture::Texture2d::empty(&display, 512, 512).expect("fail when creating texture");

    let image = texture.main_level().first_layer().into_image(None).unwrap();

    let rect = glium::Rect {
        left: 0,
        bottom: 4_286_579_200,
        width: 512,
        height: 8_388_608,
    };

    let real_pixels = rect.width as u64 * rect.height as u64;

    let pixels: Vec<Vec<(u8, u8, u8, u8)>> = image.raw_read(&rect);
}
