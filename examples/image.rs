#[macro_use]
extern crate glium;

use glium::index::PrimitiveType;
#[allow(unused_imports)]
use glium::{Display, Frame, Surface};
use glutin::surface::WindowSurface;
use std::io::Cursor;
use support::{ApplicationContext, State};

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
}

impl ApplicationContext for Application {
    const WINDOW_TITLE:&'static str = "Glium image example";

    fn new(display: &Display<WindowSurface>) -> Self {
        // building a texture with "OpenGL" drawn on it
        let image = image::load(
            Cursor::new(&include_bytes!("../tests/fixture/opengl.png")[..]),
            image::ImageFormat::Png,
        )
        .unwrap()
        .to_rgba8();
        let image_dimensions = image.dimensions();
        let image =
            glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        let opengl_texture = glium::texture::CompressedTexture2d::new(display, image).unwrap();

        // building the vertex buffer, which contains all the vertices that we will draw
        let vertex_buffer = {
            glium::VertexBuffer::new(
                display,
                &[
                    Vertex {
                        position: [-1.0, -1.0],
                        tex_coords: [0.0, 0.0],
                    },
                    Vertex {
                        position: [-1.0, 1.0],
                        tex_coords: [0.0, 1.0],
                    },
                    Vertex {
                        position: [1.0, 1.0],
                        tex_coords: [1.0, 1.0],
                    },
                    Vertex {
                        position: [1.0, -1.0],
                        tex_coords: [1.0, 0.0],
                    },
                ],
            )
            .unwrap()
        };

        // building the index buffer
        let index_buffer =
            glium::IndexBuffer::new(display, PrimitiveType::TriangleStrip, &[1 as u16, 2, 0, 3])
                .unwrap();

        // compiling shaders and linking them together
        let program = program!(display,
            140 => {
                vertex: "
                    #version 140
                    uniform mat4 matrix;
                    in vec2 position;
                    in vec2 tex_coords;
                    out vec2 v_tex_coords;
                    void main() {
                        gl_Position = matrix * vec4(position, 0.0, 1.0);
                        v_tex_coords = tex_coords;
                    }
                ",

                fragment: "
                    #version 140
                    uniform sampler2D tex;
                    in vec2 v_tex_coords;
                    out vec4 f_color;
                    void main() {
                        f_color = texture(tex, v_tex_coords);
                    }
                "
            },

            110 => {
                vertex: "
                    #version 110
                    uniform mat4 matrix;
                    attribute vec2 position;
                    attribute vec2 tex_coords;
                    varying vec2 v_tex_coords;
                    void main() {
                        gl_Position = matrix * vec4(position, 0.0, 1.0);
                        v_tex_coords = tex_coords;
                    }
                ",

                fragment: "
                    #version 110
                    uniform sampler2D tex;
                    varying vec2 v_tex_coords;
                    void main() {
                        gl_FragColor = texture2D(tex, v_tex_coords);
                    }
                ",
            },

            100 => {
                vertex: "
                    #version 100
                    uniform lowp mat4 matrix;
                    attribute lowp vec2 position;
                    attribute lowp vec2 tex_coords;
                    varying lowp vec2 v_tex_coords;
                    void main() {
                        gl_Position = matrix * vec4(position, 0.0, 1.0);
                        v_tex_coords = tex_coords;
                    }
                ",

                fragment: "
                    #version 100
                    uniform lowp sampler2D tex;
                    varying lowp vec2 v_tex_coords;
                    void main() {
                        gl_FragColor = texture2D(tex, v_tex_coords);
                    }
                ",
            },
        )
        .unwrap();

        Self {
            vertex_buffer,
            index_buffer,
            opengl_texture,
            program,
        }
    }

    fn draw_frame(&mut self, display: &Display<WindowSurface>) {
        let mut frame = display.draw();
        // building the uniforms
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ],
            tex: &self.opengl_texture
        };

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
}

fn main() {
    State::<Application>::run_loop();
}
