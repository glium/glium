#![feature(phase)]

#[phase(plugin)]
extern crate glium_macros;

extern crate glutin;
extern crate glium;

fn main() {
    use glium::DisplayBuild;

    // building the display, ie. the main object
    let display = glutin::WindowBuilder::new()
        .build_glium()
        .unwrap();

    // building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {
        #[vertex_format]
        struct Vertex {
            position: [f32, ..2],
            color: [f32, ..3],
        }

        glium::VertexBuffer::new(&display, 
            vec![
                Vertex { position: [-0.5, -0.5], color: [0.0, 1.0, 0.0] },
                Vertex { position: [ 0.0,  0.5], color: [0.0, 0.0, 1.0] },
                Vertex { position: [ 0.5, -0.5], color: [1.0, 0.0, 0.0] },
            ]
        )
    };

    // building the index buffer
    let index_buffer = glium::IndexBuffer::new(&display, glium::TrianglesList,
        &[ 0u16, 1, 2 ]);

    // compiling shaders and linking them together
    let program = glium::Program::new(&display,
        // vertex shader
        "
            #version 110

            uniform mat4 matrix;

            attribute vec2 position;
            attribute vec3 color;

            varying vec3 vColor;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0) * matrix;
                vColor = color;
            }
        ",

        // fragment shader
        "
            #version 110
            varying vec3 vColor;

            void main() {
                gl_FragColor = vec4(vColor, 1.0);
            }
        ",

        // geometry shader
        None)
        .unwrap();

    // creating the uniforms structure
    #[uniforms]
    struct Uniforms {
        matrix: [[f32, ..4], ..4],
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
            ]
        };

        // drawing a frame
        let mut target = display.draw();
        target.draw(glium::BasicDraw(&vertex_buffer, &index_buffer, &program, &uniforms, &std::default::Default::default()));
        target.finish();

        // sleeping for some time in order not to use up too much CPU
        timer::sleep(Duration::milliseconds(17));

        // polling and handling the events received by the window
        for event in display.poll_events().into_iter() {
            match event {
                glutin::Closed => break 'main,
                _ => ()
            }
        }
    }
}
