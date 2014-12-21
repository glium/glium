use std::os;

use {glium, glutin};
use glium::DisplayBuild;

/// Builds a headless display for benches.
pub fn build_display() -> glium::Display {
    if os::getenv("TRAVIS").is_some() {
        glutin::HeadlessRendererBuilder::new(1024, 768).build_glium().unwrap()
    } else {
        glutin::WindowBuilder::new().with_visibility(false).build_glium().unwrap()
    }
}
