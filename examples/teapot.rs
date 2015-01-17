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
    let (vertex_buffer, index_buffer) = match teapot_model::build_model() {
        (vertices, indices) => (
            glium::VertexBuffer::new(&display, vertices),
            glium::IndexBuffer::new(&display, indices)
        )
    };

    // the program
    let program = glium::Program::from_source(&display,
        // vertex shader
        "
            #version 110

            uniform mat4 matrix;

            attribute vec3 position;
            attribute vec3 normal;
            varying vec3 v_position;
            varying vec3 v_normal;

            void main() {
                v_position = position;
                v_normal = normal;
                gl_Position = vec4(v_position, 1.0) * matrix;
            }
        ",

        // fragment shader
        "
            #version 110

            varying vec3 v_normal;

            const vec3 LIGHT = vec3(-0.2, 0.8, 0.1);

            void main() {
                float lum = max(dot(v_normal, normalize(LIGHT)), 0.0);
                vec3 color = (0.3 + 0.7 * lum) * vec3(1.0, 1.0, 1.0);
                gl_FragColor = vec4(color, 1.0);
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

        // draw parameters
        let params = glium::DrawParameters {
            //depth_function: glium::DepthFunction::IfLess,
            .. std::default::Default::default()
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &params).unwrap();
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
