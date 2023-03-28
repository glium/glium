#[macro_use]
extern crate glium;

use glium::index::PrimitiveType;
use glium::program::ShaderStage;
use glium::{Display, Surface};
use glutin::surface::WindowSurface;
use support::{ApplicationContext, State};

mod support;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}
implement_vertex!(Vertex, position);

struct Application {
    pub vertex_buffer: glium::VertexBuffer<Vertex>,
    pub index_buffer: glium::IndexBuffer<u16>,
    pub program: glium::Program,
    pub i: i32,
}

impl ApplicationContext for Application {
    const WINDOW_TITLE:&'static str = "Glium subroutines example";

    fn new(display: &Display<WindowSurface>) -> Self {
        // building the vertex buffer, which contains all the vertices that we will draw
        let vertex_buffer = {
            glium::VertexBuffer::new(
                display,
                &[
                    Vertex {
                        position: [-0.5, -0.5],
                    },
                    Vertex {
                        position: [0.0, 0.5],
                    },
                    Vertex {
                        position: [0.5, -0.5],
                    },
                ],
            )
            .unwrap()
        };

        // building the index buffer
        let index_buffer =
            glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &[0u16, 1, 2]).unwrap();

        // compiling shaders and linking them together
        let program = program!(display,
            150 => {
                vertex: "
                    #version 150

                    uniform mat4 matrix;

                    in vec2 position;

                    void main() {
                        gl_Position = vec4(position, 0.0, 1.0) * matrix;
                    }
                ",

                fragment: "
                    #version 150
                    #extension GL_ARB_shader_subroutine : require

                    out vec4 fragColor;
                    subroutine vec4 color_t();

                    subroutine uniform color_t Color;

                    subroutine(color_t)
                    vec4 ColorRed()
                    {
                    return vec4(1, 0, 0, 1);
                    }

                    subroutine(color_t)
                    vec4 ColorBlue()
                    {
                    return vec4(0, 0.4, 1, 1);
                    }

                    subroutine(color_t)
                    vec4 ColorYellow()
                    {
                    return vec4(1, 1, 0, 1);
                    }

                    void main()
                    {
                        fragColor = Color();
                    }
                "
            },
        )
        .unwrap();

        let i = 0;

        Self {
            vertex_buffer,
            index_buffer,
            program,
            i,
        }
    }

    fn draw_frame(&mut self, display: &Display<WindowSurface>) {
        let mut frame = display.draw();
        let subroutine = if self.i < 40 {
            "ColorYellow"
        } else if self.i < 80 {
            "ColorBlue"
        } else {
            "ColorRed"
        };

        // building the uniforms
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ],
            Color: (subroutine, ShaderStage::Fragment)
        };

        // drawing a frame
        frame.clear_color(0.0, 0.0, 0.0, 0.0);
        frame
            .draw(
                &self.vertex_buffer,
                &self.index_buffer,
                &self.program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
        frame.finish().unwrap();
    }

    fn update(&mut self) {
        self.i = (self.i + 1) % 120;
    }
}

fn main() {
    State::<Application>::run_loop();
}
