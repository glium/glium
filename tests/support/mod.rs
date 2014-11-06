/*!
Test supports module.

*/
use glutin;
use glium::{mod, DisplayBuild};

use std::os;

/// Builds a headless display for tests.
pub fn build_display() -> glium::Display {
    if os::getenv("HEADLESS_TESTS").is_some() {
        glutin::HeadlessRendererBuilder::new(1024, 768).build_glium().unwrap()
    } else {
        glutin::WindowBuilder::new().with_visibility(false).build_glium().unwrap()
    }
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
