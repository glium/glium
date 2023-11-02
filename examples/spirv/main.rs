// This example demonstrates SPIR-V shader loading.
// It's based on the screenshot example but uses an empty vertex buffer.

#[macro_use]
extern crate glium;

use std::num::NonZeroU32;
use glium::index::PrimitiveType;
use glium::program::{SpirvEntryPoint, ProgramCreationInput, SpirvProgram};
use glium::{Surface};
use glutin::context::Version;
use glutin::prelude::*;
use glutin::display::GetGlDisplay;
use glutin::surface::WindowSurface;
use raw_window_handle::HasRawWindowHandle;

fn main() {
    // We start by creating the main EventLoop
    let event_loop = winit::event_loop::EventLoopBuilder::new()
        .build()
        .expect("event loop building");
    let window_builder = winit::window::WindowBuilder::new().with_title("Glium SPIR-V example");
    let config_template_builder = glutin::config::ConfigTemplateBuilder::new();
    let display_builder = glutin_winit::DisplayBuilder::new().with_window_builder(Some(window_builder));

    // Now we need to create a window
    let (window, gl_config) = display_builder
        .build(&event_loop, config_template_builder, |mut configs| {
            // Just use the first configuration since we don't have any special preferences
            configs.next().unwrap()
        })
        .unwrap();
    let window = window.unwrap();

    // Then the configuration which decides which OpenGL version we'll end up using, here we just use the default which is currently 3.3 core
    let raw_window_handle = window.raw_window_handle();
    // We need a 4.6 context for SPIR-V support
    let context_attributes = glutin::context::ContextAttributesBuilder::new()
        .with_context_api(glutin::context::ContextApi::OpenGl(Some(Version::new(4, 6))))
        .build(Some(raw_window_handle));

    let not_current_gl_context = Some(unsafe {
        gl_config.display().create_context(&gl_config, &context_attributes).expect("failed to create context")
    });

    // Determine our framebuffer size based on the window size
    let (width, height): (u32, u32) = window.inner_size().into();
    let attrs = glutin::surface::SurfaceAttributesBuilder::<WindowSurface>::new().build(
        raw_window_handle,
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
    );
    // Now we can create our surface, use it to make our context current and finally create our display
    let surface = unsafe { gl_config.display().create_window_surface(&gl_config, &attrs).unwrap() };
    let current_context = not_current_gl_context.unwrap().make_current(&surface).unwrap();
    let display = glium::Display::from_context_surface(current_context, surface).unwrap();

    // building the vertex buffer with no vertices
    // https://www.saschawillems.de/blog/2016/08/13/vulkan-tutorial-on-rendering-a-fullscreen-quad-without-buffers/
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            dummy: f32,
        }

        implement_vertex!(Vertex, dummy);

        glium::VertexBuffer::<Vertex>::empty(&display, 0).unwrap()
    };

    // building the index buffer
    let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                               &[0u8, 1, 2]).unwrap();

    // loading SPIR-V module that contains fragment and vertex shader entry points both called "main"
    let spirv = SpirvEntryPoint { binary: include_bytes!("shader.spv"), entry_point: "main" };
    let program = glium::Program::new(
        &display,
        ProgramCreationInput::SpirV(SpirvProgram::from_vs_and_fs(spirv, spirv))
    ).unwrap();

    // drawing once

    // building the uniforms
    let uniforms = uniform! {
        matrix: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0f32]
        ]
    };

    // drawing a frame
    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &Default::default()).unwrap();
    target.finish().unwrap();

    // reading the front buffer into an image
    let image: glium::texture::RawImage2d<'_, u8> = display.read_front_buffer().unwrap();
    let image = image::ImageBuffer::from_raw(image.width, image.height, image.data.into_owned()).unwrap();
    let image = image::DynamicImage::ImageRgba8(image).flipv();
    image.save("glium-example-screenshot.png").unwrap();
}
