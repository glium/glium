/*!
Test supports module.

*/
use glutin;

use glium::{mod, DisplayBuild};

/// Builds a headless display for tests.
pub fn build_display() -> glium::Display {
    glutin::HeadlessRendererBuilder::new(1024, 768).build_glium().unwrap()
}
