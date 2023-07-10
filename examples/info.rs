#[macro_use]
extern crate glium;

use glium::{Display, Api, Version, Profile};
use glutin::surface::WindowSurface;
use support::{ApplicationContext, State};

mod support;

struct Application { }
impl ApplicationContext for Application {
    const WINDOW_TITLE:&'static str = "Glium info example";

    fn new(display: &Display<WindowSurface>) -> Self {
        // Now we can query the display for various information
        let version = *display.get_opengl_version();
        let api = match version {
            Version(Api::Gl, _, _) => "OpenGL",
            Version(Api::GlEs, _, _) => "OpenGL ES"
        };

        println!("{} context version: {}", api, display.get_opengl_version_string());

        print!("{} context flags:", api);
        if display.is_forward_compatible() {
            print!(" forward-compatible");
        }
        if display.is_debug() {
            print!(" debug");
        }
        if display.is_robust() {
            print!(" robustness");
        }
        print!("\n");

        if version >= Version(Api::Gl, 3, 2) {
            println!("{} profile mask: {}", api,
                    match display.get_opengl_profile() {
                        Some(Profile::Core) => "core",
                        Some(Profile::Compatibility) => "compatibility",
                        None => "unknown"
                    });
        }

        println!("{} robustness strategy: {}", api,
                if display.is_context_loss_possible() {
                    "lose"
                } else {
                    "none"
                });

        println!("{} context renderer: {}", api, display.get_opengl_renderer_string());
        println!("{} context vendor: {}", api, display.get_opengl_vendor_string());

        Self { }
    }
}

fn main() {
    State::<Application>::run_once(false);
}
