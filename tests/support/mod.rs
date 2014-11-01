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
