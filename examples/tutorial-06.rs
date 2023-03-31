#[macro_use]
extern crate glium;
mod support;

use std::io::Cursor;
use glium::{Display, Surface};
use glutin::surface::WindowSurface;
use support::{ApplicationContext, State};

struct Application {
    pub vertex_buffer: glium::VertexBuffer<Vertex>,
    pub program: glium::Program,
    pub texture: glium::texture::SrgbTexture2d,
    pub t:f32,
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);

impl ApplicationContext for Application {
    const WINDOW_TITLE:&'static str = "Glium tutorial #6";

    fn new(display: &Display<WindowSurface>) -> Self {
        let image = image::load(Cursor::new(&include_bytes!("../tests/fixture/opengl.png")),
                            image::ImageFormat::Png).unwrap().to_rgba8();
        let image_dimensions = image.dimensions();
        let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        let texture = glium::texture::SrgbTexture2d::new(display, image).unwrap();

        let vertex1 = Vertex { position: [-0.5, -0.5], tex_coords: [0.0, 0.0] };
        let vertex2 = Vertex { position: [ 0.0,  0.5], tex_coords: [0.0, 1.0] };
        let vertex3 = Vertex { position: [ 0.5, -0.25], tex_coords: [1.0, 0.0] };
        let shape = vec![vertex1, vertex2, vertex3];

        let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();


        let vertex_shader_src = r#"
            #version 140

            in vec2 position;
            in vec2 tex_coords;
            out vec2 v_tex_coords;

            uniform mat4 matrix;

            void main() {
                v_tex_coords = tex_coords;
                gl_Position = matrix * vec4(position, 0.0, 1.0);
            }
        "#;

        let fragment_shader_src = r#"
            #version 140

            in vec2 v_tex_coords;
            out vec4 color;

            uniform sampler2D tex;

            void main() {
                color = texture(tex, v_tex_coords);
            }
        "#;

        let t = -0.5;

        let program = glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap();

        Self {
            vertex_buffer,
            texture,
            program,
            t,
        }
    }

    fn draw_frame(&mut self, display: &Display<WindowSurface>) {
        // we update `t`
        self.t += 0.0002;
        if self.t > 0.5 {
            self.t = -0.5;
        }

        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [ self.t , 0.0, 0.0, 1.0f32],
            ],
            tex: &self.texture,
        };

        target.draw(&self.vertex_buffer, &indices, &self.program, &uniforms,
                    &Default::default()).unwrap();
        target.finish().unwrap();
    }
}

fn main() {
    State::<Application>::run_loop();
}
