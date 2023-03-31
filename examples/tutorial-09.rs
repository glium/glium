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
    const WINDOW_TITLE:&'static str = "Glium tutorial #9";

    fn new(display: &Display<WindowSurface>) -> Self {
        let positions = glium::VertexBuffer::new(display, &teapot::VERTICES).unwrap();
        let normals = glium::VertexBuffer::new(display, &teapot::NORMALS).unwrap();
        let indices = glium::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList,
                                            &teapot::INDICES).unwrap();

        let vertex_shader_src = r#"
            #version 150

            in vec3 position;
            in vec3 normal;

            out vec3 v_normal;

            uniform mat4 matrix;

            void main() {
                v_normal = transpose(inverse(mat3(matrix))) * normal;
                gl_Position = matrix * vec4(position, 1.0);
            }
        "#;

        let fragment_shader_src = r#"
            #version 150

            in vec3 v_normal;
            out vec4 color;
            uniform vec3 u_light;

            void main() {
                float brightness = dot(normalize(v_normal), normalize(u_light));
                vec3 dark_color = vec3(0.6, 0.0, 0.0);
                vec3 regular_color = vec3(1.0, 0.0, 0.0);
                color = vec4(mix(dark_color, regular_color, brightness), 1.0);
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
        target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

        let matrix = [
            [0.01, 0.0, 0.0, 0.0],
            [0.0, 0.01, 0.0, 0.0],
            [0.0, 0.0, 0.01, 0.0],
            [0.0, 0.0, 0.0, 1.0f32]
        ];

        let light = [-1.0, 0.4, 0.9f32];

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            .. Default::default()
        };

        target.draw((&self.positions, &self.normals), &self.indices, &self.program,
                    &uniform! { matrix: matrix, u_light: light }, &params).unwrap();
        target.finish().unwrap();
    }
}

fn main() {
    State::<Application>::run_loop();
}
