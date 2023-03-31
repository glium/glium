#[macro_use]
extern crate glium;
mod support;

use glium::{Display, Surface};
use glutin::surface::WindowSurface;
use support::{ApplicationContext, State};

struct Application {
    pub positions: glium::VertexBuffer<teapot::Vertex>,
    pub normals: glium::VertexBuffer<teapot::Normal>,
    pub indices: glium::IndexBuffer<u16>,
    pub program: glium::Program,
}

#[path = "../book/tuto-07-teapot.rs"]
mod teapot;

impl ApplicationContext for Application {
    const WINDOW_TITLE:&'static str = "Glium tutorial #7";

    fn new(display: &Display<WindowSurface>) -> Self {
        let positions = glium::VertexBuffer::new(display, &teapot::VERTICES).unwrap();
        let normals = glium::VertexBuffer::new(display, &teapot::NORMALS).unwrap();
        let indices = glium::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList,
                                            &teapot::INDICES).unwrap();

        let vertex_shader_src = r#"
            #version 140

            in vec3 position;
            in vec3 normal;

            uniform mat4 matrix;

            void main() {
                gl_Position = matrix * vec4(position, 1.0);
            }
        "#;

        let fragment_shader_src = r#"
            #version 140

            out vec4 color;

            void main() {
                color = vec4(1.0, 0.0, 0.0, 1.0);
            }
        "#;

        let program = glium::Program::from_source(display, vertex_shader_src, fragment_shader_src,
                                                None).unwrap();

        Self {
            positions,
            normals,
            indices,
            program,
        }
    }

    fn draw_frame(&mut self, display: &Display<WindowSurface>) {
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        let matrix = [
            [0.01, 0.0, 0.0, 0.0],
            [0.0, 0.01, 0.0, 0.0],
            [0.0, 0.0, 0.01, 0.0],
            [0.0, 0.0, 0.0, 1.0f32]
        ];

        target.draw((&self.positions, &self.normals), &self.indices, &self.program, &uniform! { matrix: matrix },
                    &Default::default()).unwrap();
        target.finish().unwrap();
    }
}

fn main() {
    State::<Application>::run_loop();
}
