#![feature(phase)]

#[phase(plugin)]
extern crate glium_macros;

extern crate glutin;
extern crate glium;

#[cfg(feature = "image")]
extern crate image;

use std::io::BufReader;

use glium::{DisplayBuild, Surface};

#[cfg(not(feature = "image"))]
fn main() {
    println!("This example requires the `image` feature to be enabled");
}

#[cfg(feature = "image")]
fn main() {
    // building the display, ie. the main object
    let display = glutin::WindowBuilder::new()
        .with_vsync()
        .build_glium()
        .unwrap();

    // building a texture with "OpenGL" drawn on it
    let image = image::load(BufReader::new(include_bytes!("../tests/fixture/opengl.png")),
        image::PNG).unwrap();
    let opengl_texture = glium::texture::CompressedTexture2d::new(&display, image);

    // building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {
        #[vertex_format]
        #[deriving(Copy)]
        struct Vertex {
            position: [f32, ..2],
            tex_coords: [f32, ..2],
        }

        glium::VertexBuffer::new(&display, 
            vec![
                Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 0.0] },
                Vertex { position: [-1.0,  1.0], tex_coords: [0.0, 1.0] },
                Vertex { position: [ 1.0,  1.0], tex_coords: [1.0, 1.0] },
                Vertex { position: [ 1.0, -1.0], tex_coords: [1.0, 0.0] }
            ]
        )
    };

    // building the index buffer
    let index_buffer = glium::IndexBuffer::new(&display,
        glium::index_buffer::TriangleStrip(vec![1 as u16, 2, 0, 3]));

    // compiling shaders and linking them together
    let program = glium::Program::from_source(&display, r"
        #version 110

        uniform mat4 matrix;

        attribute vec2 position;
        attribute vec2 tex_coords;

        varying vec2 v_tex_coords;

        void main() {
            gl_Position = matrix * vec4(position, 0.0, 1.0);
            v_tex_coords = tex_coords;
        }
    ", r"
        #version 110
        uniform sampler2D texture;
        varying vec2 v_tex_coords;

        void main() {
            gl_FragColor = texture2D(texture, v_tex_coords);
        }
    ", None).unwrap();

    // creating the uniforms structure
    #[uniforms]
    struct Uniforms<'a> {
        matrix: [[f32, ..4], ..4],
        texture: &'a glium::texture::CompressedTexture2d,
    }
    
    // the main loop
    // each cycle will draw once
    'main: loop {
        use std::io::timer;
        use std::time::Duration;

        // building the uniforms
        let uniforms = Uniforms {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ],
            texture: &opengl_texture,
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &std::default::Default::default());
        target.finish();

        // sleeping for some time in order not to use up too much CPU
        timer::sleep(Duration::milliseconds(17));

        // polling and handling the events received by the window
        for event in display.poll_events().into_iter() {
            match event {
                glutin::Event::Closed => break 'main,
                _ => ()
            }
        }
    }
}
