#[macro_use]
extern crate glium;

use std::io::Cursor;

use glium::index::PrimitiveType;
#[allow(unused_imports)]
use glium::{Display, Frame, Surface};
use glutin::surface::WindowSurface;
use support::{ApplicationContext, State};
use glium::winit::{window::Fullscreen, keyboard::{PhysicalKey, KeyCode}};

mod support;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

struct Application {
    pub vertex_buffer: glium::VertexBuffer<Vertex>,
    pub index_buffer: glium::IndexBuffer<u16>,
    pub opengl_texture: glium::texture::CompressedTexture2d,
    pub program: glium::Program,
    pub fullscreen: bool,
}

impl Application {
    fn toggle_fullscreen(&mut self, window: &glium::winit::window::Window) {
        if self.fullscreen {
            window.set_fullscreen(None);
            self.fullscreen = false;
        } else {
            let monitor_handle = window.available_monitors().next().unwrap();
            let fs = Fullscreen::Borderless(Some(monitor_handle));
            window.set_fullscreen(Some(fs));

            self.fullscreen = true;
        }
    }
}

impl ApplicationContext for Application {
    const WINDOW_TITLE:&'static str = "Glium fullscreen example";

    fn new(display: &Display<WindowSurface>) -> Self {
        // building a texture with "OpenGL" drawn on it
        let image = image::load(Cursor::new(&include_bytes!("../tests/fixture/opengl.png")[..]),
                                image::ImageFormat::Png).unwrap().to_rgba8();
        let image_dimensions = image.dimensions();
        let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        let opengl_texture = glium::texture::CompressedTexture2d::new(display, image).unwrap();

        // building the vertex buffer, which contains all the vertices that we will draw
        let vertex_buffer = {
            glium::VertexBuffer::new(display,
                &[
                    Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 0.0] },
                    Vertex { position: [-1.0,  1.0], tex_coords: [0.0, 1.0] },
                    Vertex { position: [ 1.0,  1.0], tex_coords: [1.0, 1.0] },
                    Vertex { position: [ 1.0, -1.0], tex_coords: [1.0, 0.0] }
                ]
            ).unwrap()
        };

        // building the index buffer
        let index_buffer = glium::IndexBuffer::new(display, PrimitiveType::TriangleStrip,
                                                &[1 as u16, 2, 0, 3]).unwrap();

        // compiling shaders and linking them together
        let program = glium::Program::from_source(display, r"
            #version 140

            uniform mat4 matrix;

            in vec2 position;
            in vec2 tex_coords;

            out vec2 v_tex_coords;

            void main() {
                gl_Position = matrix * vec4(position, 0.0, 1.0);
                v_tex_coords = tex_coords;
            }
        ", r"
            #version 140
            uniform sampler2D tex;
            in vec2 v_tex_coords;
            out vec4 color;

            void main() {
                color = texture(tex, v_tex_coords);
            }
        ", None).unwrap();


        let fullscreen = false;

        Self {
            vertex_buffer,
            index_buffer,
            opengl_texture,
            fullscreen,
            program,
        }
    }

    fn draw_frame(&mut self, display: &Display<WindowSurface>) {
        let mut frame = display.draw();
        frame.clear_color(0.0, 1.0, 0.0, 1.0);
        frame.draw(&self.vertex_buffer, &self.index_buffer, &self.program, &uniform! {
                matrix: [
                    [0.5, 0.0, 0.0, 0.0],
                    [0.0, 0.5, 0.0, 0.0],
                    [0.0, 0.0, 0.5, 0.0],
                    [0.0, 0.0, 0.0, 1.0f32]
                ],
                tex: &self.opengl_texture
            }, &Default::default()).unwrap();
        frame.finish().unwrap();
    }

    fn handle_window_event(&mut self, event: &glium::winit::event::WindowEvent, window: &glium::winit::window::Window) {
        match event {
            glium::winit::event::WindowEvent::KeyboardInput { event, .. } => match event.state {
                glium::winit::event::ElementState::Pressed => match event.physical_key {
                    PhysicalKey::Code(KeyCode::Enter) => {
                        self.toggle_fullscreen(window);
                    }
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        }
    }
}

fn main() {
    println!("Press Enter to switch fullscreen mode");
    State::<Application>::run_loop();
}
