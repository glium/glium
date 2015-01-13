#![feature(plugin)]

#[plugin]
extern crate glium_macros;

extern crate glutin;
extern crate glium;

use glium::Surface;

mod teapot_model;

fn main() {
    use glium::DisplayBuild;

    // building the display, ie. the main object
    let display = glutin::WindowBuilder::new()
        .build_glium()
        .unwrap();

    // building the vertex and index buffers
    let vertex_buffer = glium::VertexBuffer::new(&display, teapot_model::build_vertices());
    let index_buffer = glium::IndexBuffer::new(&display, teapot_model::build_indices());

    // the program
    let program = glium::Program::from_source(&display,
        // vertex shader
        "
            #version 110

            uniform mat4 matrix;

            attribute vec3 position;

            void main() {
                gl_Position = vec4(position, 1.0) * matrix;
            }
        ",

        // fragment shader
        "
            #version 110

            void main() {
                gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
            }
        ",

        // geometry shader
        None)
        .unwrap();

    // creating the uniforms structure
    #[uniforms]
    #[derive(Copy)]
    struct Uniforms {
        matrix: [[f32; 4]; 4],
    }
    
    // the main loop
    // each cycle will draw once
    'main: loop {
        use std::io::timer;
        use std::time::Duration;

        // building the uniforms
        let uniforms = Uniforms {
            matrix: [
                [0.05, 0.0, 0.0, 0.0],
                [0.0, 0.05, 0.0, 0.0],
                [0.0, 0.0, 0.05, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ]
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
