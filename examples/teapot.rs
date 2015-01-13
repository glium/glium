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

            attribute vec3 position;
            varying vec3 v_position;

            void main() {
                v_position = position;
            }
        ",

        // fragment shader
        "
            #version 330

            in vec3 g_normal;

            const vec3 LIGHT = vec3(-0.2, 0.8, 0.1);

            void main() {
                float lum = max(dot(g_normal, normalize(LIGHT)), 0.0);
                vec3 color = (0.3 + 0.7 * lum) * vec3(1.0, 1.0, 1.0);
                gl_FragColor = vec4(color, 1.0);
            }
        ",

        // geometry shader
        Some("
            #version 330

            uniform mat4 matrix;

            layout(triangles) in;
            layout(triangle_strip, max_vertices=3) out;

            in vec3 v_position[3];

            out vec3 g_normal;

            void main() {
                // ugly since we don't have adjacency infos
                vec3 normal = normalize(cross(v_position[1].xyz - v_position[0].xyz,
                                              v_position[2].xyz - v_position[0].xyz));

                gl_Position = vec4(v_position[0], 1.0) * matrix;
                g_normal = normal;
                EmitVertex();
                gl_Position = vec4(v_position[1], 1.0) * matrix;
                g_normal = normal;
                EmitVertex();
                gl_Position = vec4(v_position[2], 1.0) * matrix;
                g_normal = normal;
                EmitVertex();
            }
        "))
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
        target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &params);
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
