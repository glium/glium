#![allow(dead_code)]
use std::num::NonZeroU32;
use glium::{self, Display, Frame};
use glium::vertex::VertexBufferAny;
use winit::event::WindowEvent;
use winit::event_loop::{EventLoopBuilder, EventLoopWindowTarget};
use winit::window::WindowBuilder;
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;

pub mod camera;

/// Returns a vertex buffer that should be rendered as `TrianglesList`.
pub fn load_wavefront(display: &Display<WindowSurface>, data: &[u8]) -> VertexBufferAny {
    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 3],
        normal: [f32; 3],
        texture: [f32; 2],
    }

    implement_vertex!(Vertex, position, normal, texture);

    let mut data = ::std::io::BufReader::new(data);
    let data = obj::ObjData::load_buf(&mut data).unwrap();

    let mut vertex_data = Vec::new();

    for object in data.objects.iter() {
        for polygon in object.groups.iter().flat_map(|g| g.polys.iter()) {
            match polygon {
                obj::SimplePolygon(indices) => {
                    for v in indices.iter() {
                        let position = data.position[v.0];
                        let texture = v.1.map(|index| data.texture[index]);
                        let normal = v.2.map(|index| data.normal[index]);

                        let texture = texture.unwrap_or([0.0, 0.0]);
                        let normal = normal.unwrap_or([0.0, 0.0, 0.0]);

                        vertex_data.push(Vertex {
                            position,
                            normal,
                            texture,
                        })
                    }
                },
            }
        }
    }

    glium::vertex::VertexBuffer::new(display, &vertex_data).unwrap().into()
}

pub trait ApplicationContext {
    fn draw_frame(&self, frame: Frame) -> Frame;
    fn new(display: &Display<WindowSurface>) -> Self;
    fn update(&mut self);
    fn handle_window_event(&mut self, event: &WindowEvent);
}

pub struct State<T> {
    pub display: glium::Display<WindowSurface>,
    pub window: winit::window::Window,
    pub context: T,
}

impl<T: ApplicationContext + 'static> State<T> {
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

    pub fn draw(&self) {
        self.context.draw_frame(self.display.draw()).finish().unwrap();
    }

    pub fn from_display_window(
        display: glium::Display<WindowSurface>,
        window: winit::window::Window,
    ) -> Self {
        let context = T::new(&display);
        Self {
            display,
            window,
            context,
        }
    }

    pub fn run_loop() {
        let event_loop = EventLoopBuilder::new().build();
        let mut state: Option<State<T>> = None;

        // the main loop
        event_loop.run(move |event, window_target, control_flow| {
            match event {
                winit::event::Event::Resumed => {
                    state = Some(State::new(window_target));
                },
                winit::event::Event::Suspended => state = None,
                winit::event::Event::RedrawRequested(_) => {
                    if let Some(state) = &mut state {
                        state.context.update();
                        state.draw();
                    }
                }
                winit::event::Event::RedrawEventsCleared => {
                    if let Some(state) = &state {
                        state.window.request_redraw();
                    }
                }
                winit::event::Event::WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::CloseRequested => control_flow.set_exit(),
                    winit::event::WindowEvent::Resized(new_size) => {
                        if let Some(state) = &state {
                            state.display.context_surface_pair().resize(new_size.into());
                        }
                    },
                    ev => {
                        if let Some(state) = &mut state {
                            state.context.handle_window_event(&ev);
                        }
                    },
                },
                _ => (),
            };
        });
    }
}