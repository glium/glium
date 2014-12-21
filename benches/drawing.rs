#![feature(phase)]

extern crate glutin;
extern crate glium;
extern crate test;

#[phase(plugin)]
extern crate glium_macros;

use glium::Surface;

mod support;

#[bench]
fn clear_color(bencher: &mut test::Bencher) {
    let display = support::build_display();

    bencher.iter(|| {
        let mut target = display.draw();
        target.clear_color(1.0, 0.0, 0.0, 1.0);
        target.finish();

        display.synchronize();
    });
}

#[bench]
fn triangle(bencher: &mut test::Bencher) {
    let display = support::build_display();

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
    let index_buffer = glium::IndexBuffer::new(&display,
        glium::index_buffer::TrianglesList(vec![0u16, 1, 2]));

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
    
    bencher.iter(|| {
        // building the uniforms
        let uniforms = Uniforms {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ]
        };

        let mut target = display.draw();
        target.draw(&vertex_buffer, &index_buffer, &program, &uniforms,
                    &std::default::Default::default());
        target.finish();

        display.synchronize();
    });
}
