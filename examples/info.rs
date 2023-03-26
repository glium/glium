#![allow(dead_code)]
use std::num::NonZeroU32;
use glium::{self, Api, Version, Profile};
use winit::window::WindowBuilder;
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;

fn main() {
    // building the display, ie. the main object
    let event_loop = winit::event_loop::EventLoop::new();
    let window_builder = WindowBuilder::new();
    let config_template_builder = ConfigTemplateBuilder::new();
    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

    let (window, gl_config) = display_builder
        .build(&event_loop, config_template_builder, |mut configs| {
            // Just use the first configuration since we don't have any special preferences here
            configs.next().unwrap()
        })
        .unwrap();
    let window = window.unwrap();
    let raw_window_handle = window.raw_window_handle();

    let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));
    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(Some(raw_window_handle));

    let not_current_gl_context = Some(unsafe {
        gl_config.display().create_context(&gl_config, &context_attributes).unwrap_or_else(|_| {
            gl_config.display()
                .create_context(&gl_config, &fallback_context_attributes)
                .expect("failed to create context")
        })
    });

    let (width, height): (u32, u32) = window.inner_size().into();
    let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        raw_window_handle,
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
    );

    let surface = unsafe { gl_config.display().create_window_surface(&gl_config, &attrs).unwrap() };
    let current_context = not_current_gl_context.unwrap().make_current(&surface).unwrap();
    let display = glium::Display::from_context_surface(current_context, surface).unwrap();


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
}
