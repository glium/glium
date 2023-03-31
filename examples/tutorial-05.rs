#[macro_use]
extern crate glium;
mod support;

use glium::{Display, Surface};
use glutin::surface::WindowSurface;
use support::{ApplicationContext, State};

struct Application {
    pub vertex_buffer: glium::VertexBuffer<Vertex>,
    pub program: glium::Program,
    pub t:f32,
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}
implement_vertex!(Vertex, position);

impl ApplicationContext for Application {
    const WINDOW_TITLE:&'static str = "Glium tutorial #5";

    fn new(display: &Display<WindowSurface>) -> Self {
        let vertex1 = Vertex { position: [-0.5, -0.5] };
        let vertex2 = Vertex { position: [ 0.0,  0.5] };
        let vertex3 = Vertex { position: [ 0.5, -0.25] };
        let shape = vec![vertex1, vertex2, vertex3];

        let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();

        let vertex_shader_src = r#"
            #version 140

            in vec2 position;
            out vec2 my_attr;      // our new attribute

            uniform mat4 matrix;

            void main() {
                my_attr = position;     // we need to set the value of each `out` variable.
                gl_Position = matrix * vec4(position, 0.0, 1.0);
            }
        "#;

        let fragment_shader_src = r#"
            #version 140

            in vec2 my_attr;
            out vec4 color;

            void main() {
                color = vec4(my_attr, 0.0, 1.0);   // we build a vec4 from a vec2 and two floats
            }
        "#;

        let program = glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap();

        let t = -0.5;

        Self {
            vertex_buffer,
            program,
            t,
        }
    }

    fn draw_frame(&mut self, display: &Display<WindowSurface>) {
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
        // we update `t`
        self.t += 0.0002;
        if self.t > 0.5 {
            self.t = -0.5;
        }

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [ self.t , 0.0, 0.0, 1.0f32],
            ]
        };

        target.draw(&self.vertex_buffer, &indices, &self.program, &uniforms,
                    &Default::default()).unwrap();
        target.finish().unwrap();
    }
}

fn main() {
    State::<Application>::run_loop();
}
