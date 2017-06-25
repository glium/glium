#[macro_use]
extern crate glium;

extern crate image;

use std::io::Cursor;

use glium::Surface;
use glium::glutin::{self, winit};
use glium::index::PrimitiveType;
use glium::glutin::winit::{ElementState, VirtualKeyCode, Event, WindowEvent};

mod support;

fn main() {
    // building the display, ie. the main object
    let mut events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new().build(&events_loop).unwrap();
    let context = glutin::ContextBuilder::new().build(&window).unwrap();
    let display = glium::Display::new(window, context).unwrap();

    // building a texture with "OpenGL" drawn on it
    let image = image::load(Cursor::new(&include_bytes!("../tests/fixture/opengl.png")[..]),
                            image::PNG).unwrap().to_rgba();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let opengl_texture = glium::texture::CompressedTexture2d::new(&display, image).unwrap();

    // building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
            tex_coords: [f32; 2],
        }

        implement_vertex!(Vertex, position, tex_coords);

        glium::VertexBuffer::new(&display, 
            &[
                Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 0.0] },
                Vertex { position: [-1.0,  1.0], tex_coords: [0.0, 1.0] },
                Vertex { position: [ 1.0,  1.0], tex_coords: [1.0, 1.0] },
                Vertex { position: [ 1.0, -1.0], tex_coords: [1.0, 0.0] }
            ]
        ).unwrap()
    };

    // building the index buffer
    let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TriangleStrip,
                                               &[1 as u16, 2, 0, 3]).unwrap();

    // compiling shaders and linking them together
    let program = glium::Program::from_source(&display, r"
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


    let mut fullscreen = false;

    println!("Press Enter to switch fullscreen mode");

    support::start_loop(|| {
        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.0, 1.0, 0.0, 1.0);
        target.draw(&vertex_buffer, &index_buffer, &program, &uniform! { 
                matrix: [
                    [0.5, 0.0, 0.0, 0.0],
                    [0.0, 0.5, 0.0, 0.0],
                    [0.0, 0.0, 0.5, 0.0],
                    [0.0, 0.0, 0.0, 1.0f32]
                ],
                tex: &opengl_texture
            }, &Default::default()).unwrap();
        target.finish().unwrap();

        let mut action = support::Action::Continue;

        // polling and handling the events received by the window
        let mut enter_pressed = false;
        events_loop.poll_events(|event| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Closed => action = support::Action::Stop,
                WindowEvent::KeyboardInput { input, .. } => {
                    if let ElementState::Pressed = input.state {
                        if let Some(VirtualKeyCode::Return) = input.virtual_keycode {
                            enter_pressed = true;
                        }
                    }
                },
                _ => ()
            },
            _ => (),
        });

        // If enter was pressed toggle fullscreen.
        if enter_pressed {
            if fullscreen {
                let window = winit::WindowBuilder::new()
                    .build(&events_loop)
                    .unwrap();
                let context_builder = glutin::ContextBuilder::new();
                display.rebuild_window(window, context_builder).unwrap();
                fullscreen = false;
            } else {
                let window = winit::WindowBuilder::new()
                    .with_fullscreen(winit::get_primary_monitor())
                    .build(&events_loop)
                    .unwrap();
                let context_builder = glutin::ContextBuilder::new();
                display.rebuild_window(window, context_builder).unwrap();
                fullscreen = true;
            }
        }

        action
    });
}
