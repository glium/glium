#[macro_use]
extern crate glium;

#[allow(unused_imports)]
use glium::index::PrimitiveType;
use winit::event_loop::EventLoopBuilder;
use std::num::NonZeroU32;
use winit::window::WindowBuilder;
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;

fn main() {
    let event_loop = EventLoopBuilder::new().build();
    // First we need to create a display, which wraps a context/window pair.
    let window_builder = WindowBuilder::new().with_visible(false);
    let config_template_builder = ConfigTemplateBuilder::new();
    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

    // First we need to create a window, this is mainly because on Windows/WGL we need a window in order to obtain a context
    // On other platforms we could provide None as the raw_window_handle
    let (window, gl_config) = display_builder
        .build(&event_loop, config_template_builder, |mut configs| {
            // Just use the first configuration since we don't have any special preferences here
            configs.next().unwrap()
        })
        .unwrap();
    let window = window.unwrap();

    // Then the configuration which decides which OpenGL version we'll end up using, here we just use the default which is currently 3.3 core
    // When this fails we'll try and create an ES context, this is mainly used on mobile devices or various ARM SBC's
    // If you depend on features available in modern OpenGL Versions you need to request a specific, modern, version. Otherwise things will very likely fail.
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

    // Now we can create our surface, use it to make our context current and finally create our display
    let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        raw_window_handle,
        NonZeroU32::new(512).unwrap(),
        NonZeroU32::new(512).unwrap(),
    );
    let surface = unsafe { gl_config.display().create_window_surface(&gl_config, &attrs).unwrap() };
    let current_context = not_current_gl_context.unwrap().make_current(&surface).unwrap();
    let display = glium::Display::from_context_surface(current_context, surface).unwrap();


    let program = glium::program::ComputeShader::from_source(&display, r#"\
            #version 430
            layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

            layout(std140) buffer MyBlock {
                float power;
                vec4 values[4096/4];
            };

            void main() {
                vec4 val = values[gl_GlobalInvocationID.x];

                values[gl_GlobalInvocationID.x] = pow(val, vec4(power));
            }

        "#).unwrap();

    const NUM_VALUES: usize = 4096;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Data {
        power: f32,
        _padding: [f32; 3],
        values: [[f32; 4]; NUM_VALUES / 4],
    }

    implement_uniform_block!(Data, power, values);

    let mut buffer: glium::uniforms::UniformBuffer<Data> =
              glium::uniforms::UniformBuffer::empty(&display).unwrap();

    {
        let mut mapping = buffer.map();
        mapping.power = rand::random();
        for val in mapping.values.iter_mut() {
            *val = [rand::random::<f32>(),rand::random::<f32>(),rand::random::<f32>(),rand::random::<f32>()];
        }
    }

    program.execute(uniform! { MyBlock: &*buffer }, NUM_VALUES as u32 / 4, 1, 1);

    {
        let mapping = buffer.map();
        println!("Power is: {:?}", mapping.power);
        for val in mapping.values.iter().take(3) {
            println!("{:?}", val[0]);
            println!("{:?}", val[1]);
            println!("{:?}", val[2]);
            println!("{:?}", val[3]);
        }
        println!("...");
    }
}
