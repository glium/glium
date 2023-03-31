#[macro_use]
extern crate glium;
mod support;

use glium::{Display, Surface};
use glutin::surface::WindowSurface;
use support::{ApplicationContext, State};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}
implement_vertex!(Vertex, position);

struct Application {
    pub vertex_buffer: glium::VertexBuffer<Vertex>,
    pub program: glium::Program,
    pub t:f32,
}

impl ApplicationContext for Application {
    const WINDOW_TITLE:&'static str = "Glium tutorial #3";

    fn new(display: &Display<WindowSurface>) -> Self {
        let vertex1 = Vertex { position: [-0.5, -0.5] };
        let vertex2 = Vertex { position: [ 0.0,  0.5] };
        let vertex3 = Vertex { position: [ 0.5, -0.25] };
        let shape = vec![vertex1, vertex2, vertex3];

        let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();

        let vertex_shader_src = r#"
            #version 140

            in vec2 position;
            uniform float t;

            void main() {
                vec2 pos = position;
                pos.x += t;
                gl_Position = vec4(pos, 0.0, 1.0);
            }
        "#;

        let fragment_shader_src = r#"
            #version 140

            out vec4 color;

            void main() {
                color = vec4(1.0, 0.0, 0.0, 1.0);
            }
        "#;

        let program = glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap();

        let t: f32 = -0.5;

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
        let uniforms = uniform! { t: self.t };
        target.draw(&self.vertex_buffer, &indices, &self.program, &uniforms,
                    &Default::default()).unwrap();
        target.finish().unwrap();
    }
}

fn main() {
    State::<Application>::run_loop();
}
