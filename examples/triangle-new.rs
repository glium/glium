#[macro_use]
extern crate glium;

use std::num::NonZeroU32;
use winit::event_loop::{EventLoopBuilder, EventLoopWindowTarget};
use winit::window::WindowBuilder;
use glium::{glutin, Surface};
use glium::index::PrimitiveType;
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}
implement_vertex!(Vertex, position, color);

struct State {
    pub display: glium::Display<WindowSurface>,
    pub _window: winit::window::Window,

    pub vertex_buffer: glium::VertexBuffer<Vertex>,
    pub index_buffer: glium::IndexBuffer<u16>,
    pub program: glium::Program,
}

impl State {
    pub fn new<W>(
        event_loop: &EventLoopWindowTarget<W>
    ) -> Self {
        let window_builder = WindowBuilder::new();
        let config_template_builder = ConfigTemplateBuilder::new();
        let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

        let (window, gl_config) = display_builder
            .build(event_loop, config_template_builder, |mut configs| {
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

        Self::from_display_window(display, window)
    }

    pub fn from_display_window(
        display: glium::Display<WindowSurface>,
        window: winit::window::Window,
    ) -> Self {
        let vertex_buffer = {
            glium::VertexBuffer::new(&display,
                &[
                    Vertex { position: [-0.5, -0.5], color: [0.0, 1.0, 0.0] },
                    Vertex { position: [ 0.0,  0.5], color: [0.0, 0.0, 1.0] },
                    Vertex { position: [ 0.5, -0.5], color: [1.0, 0.0, 0.0] },
                ]
            ).unwrap()
        };

        // building the index buffer
        let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                                    &[0u16, 1, 2]).unwrap();

        // compiling shaders and linking them together
        let program = program!(&display,
            100 => {
                vertex: "
                    #version 100

                    uniform lowp mat4 matrix;

                    attribute lowp vec2 position;
                    attribute lowp vec3 color;

                    varying lowp vec3 vColor;

                    void main() {
                        gl_Position = vec4(position, 0.0, 1.0) * matrix;
                        vColor = color;
                    }
                ",

                fragment: "
                    #version 100
                    varying lowp vec3 vColor;

                    void main() {
                        gl_FragColor = vec4(vColor, 1.0);
                    }
                ",
            },
        ).unwrap();

        Self {
            display,
            _window: window,

            vertex_buffer,
            index_buffer,
            program
        }
    }

    pub fn draw(&self) {
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
        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.draw(&self.vertex_buffer, &self.index_buffer, &self.program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();
    }
}

fn main() {
    let event_loop = EventLoopBuilder::new().build();
    let mut state = None;

    // the main loop
    event_loop.run(move |event, window_target, control_flow| {
        match event {
            winit::event::Event::Resumed => {
                control_flow.set_poll();
                state = Some(State::new(window_target));
            },
            winit::event::Event::Suspended => state = None,
            winit::event::Event::RedrawRequested(_) => {
                if let Some(state) = &state {
                    state.draw();
                }
            }
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => control_flow.set_exit(),
                winit::event::WindowEvent::Resized(new_size) => {
                    if let Some(state) = &state {
                        state.display.context_surface_pair().resize(new_size.into());
                    }
                },
                _ => (),
            },
            _ => (),
        };
    });
}
