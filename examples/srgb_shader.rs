//! This example shows how to configure if a shader outputs linear RGB or sRGB
//! color values.
//!
//! It draws two gradients from black to white. Both uses the exact
//! same vertex position and color values. But one is rendered with outputs_srgb
//! disabled and one with it enabled. This shows the visual difference of having
//! gamma correction applied: bottom gradient, percieved as smooth linear transition
//! from black to white. VS not having any gamma correction: top gradient, looks mostly
//! white, with the blacks squashed into a small portion of the left side.
//!
//! Glium has sRGB enabled by default. This means that when you create a shader
//! program with the `program!` macro, it will expect the shader to output sRGB
//! color values.

#[macro_use]
extern crate glium;
mod support;

use glium::index::{NoIndices, PrimitiveType};
use glium::{Display, Surface};
use glutin::surface::WindowSurface;
use support::{ApplicationContext, State};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}
implement_vertex!(Vertex, position, color);

fn create_program(display: &Display<WindowSurface>, outputs_srgb: bool) -> glium::Program {
    let source = glium::program::ProgramCreationInput::SourceCode {
        tessellation_control_shader: None,
        tessellation_evaluation_shader: None,
        geometry_shader: None,
        outputs_srgb,
        uses_point_size: false,

        vertex_shader: "
            #version 150

            uniform mat4 matrix;
            in vec2 position;
            in vec3 color;
            out vec3 vColor;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0) * matrix;
                vColor = color;
            }
        ",
        fragment_shader: "
            #version 150

            in vec3 vColor;
            out vec4 f_color;

            void main() {
                f_color = vec4(vColor, 1.0);
            }
        ",
        transform_feedback_varyings: None,
    };

    glium::Program::new(display, source).unwrap()
}

struct Application {
    pub vertex_buffer: glium::VertexBuffer<Vertex>,
    pub linear_rgb_program: glium::Program,
    pub srgb_program: glium::Program,
}

impl ApplicationContext for Application {
    const WINDOW_TITLE: &'static str = "Glium sRGB shader example";

    fn new(display: &Display<WindowSurface>) -> Self {
        const RESOLUTION: usize = 100;
        let mut vertices = Vec::new();
        for i in 0..RESOLUTION {
            let x = i as f32 / (RESOLUTION - 1) as f32;
            let color = x;

            vertices.push(Vertex {
                position: [x, 0.0],
                color: [color, color, color],
            });
            vertices.push(Vertex {
                position: [x, 1.0],
                color: [color, color, color],
            });
        }
        let vertex_buffer = { glium::VertexBuffer::new(display, &vertices).unwrap() };

        Self {
            vertex_buffer,
            linear_rgb_program: create_program(display, false),
            srgb_program: create_program(display, true),
        }
    }

    fn draw_frame(&mut self, display: &Display<WindowSurface>) {
        let mut frame = display.draw();

        // Draw band of linear RGB gradient at the top of the window
        let linear_rgb_uniforms = uniform! {
            matrix: [
                [1.9, 0.0, 0.0, -0.95],
                [0.0, 0.85, 0.0, 0.05],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ]
        };
        // Draw band of sRGB gradient at the bottom of the window
        let srgb_uniforms = uniform! {
            matrix: [
                [1.9, 0.0, 0.0, -0.95],
                [0.0, 0.85, 0.0, -0.9],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ]
        };

        // Clear the window with some non-grey color, to make the gradients stand out a bit
        frame.clear_color(0.1, 0.3, 0.1, 1.0);
        
        frame
            .draw(
                &self.vertex_buffer,
                NoIndices(PrimitiveType::TriangleStrip),
                &self.linear_rgb_program,
                &linear_rgb_uniforms,
                &Default::default(),
            )
            .unwrap();
        frame
            .draw(
                &self.vertex_buffer,
                NoIndices(PrimitiveType::TriangleStrip),
                &self.srgb_program,
                &srgb_uniforms,
                &Default::default(),
            )
            .unwrap();
        frame.finish().unwrap();
    }
}

fn main() {
    State::<Application>::run_loop();
}
