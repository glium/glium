#![cfg(feature = "unstable")]
#![feature(test)]

#[macro_use]
extern crate glium;
extern crate test;

use glium::Surface;

use test::Bencher;

mod support;

#[bench]
fn init(b: &mut Bencher) {
    b.iter(|| support::build_context());
}

#[bench]
fn clear(b: &mut Bencher) {
    let display = support::build_context();

    b.iter(|| {
        let mut target = glium::Frame::new(display.clone(), (800, 600));
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.finish()
    });
}

#[bench]
fn create_program(b: &mut Bencher) {
    let display = support::build_context();

    b.iter(|| {
        program!(&display,
            140 => {
                vertex: "
                    #version 140

                    in vec2 position;
                    in vec3 color;

                    out vec3 v_color;

                    void main() {
                        gl_Position = vec4(position, 0.0, 1.0);
                        v_color = color;
                    }
                ",

                fragment: "
                    #version 140

                    in vec3 v_color;
                    out vec4 f_color;

                    void main() {
                        f_color = vec4(v_color, 1.0);
                    }
                ",
            },
        )
    });
}

#[bench]
#[ignore]       // TODO: segfaults
fn draw_triangle(b: &mut Bencher) {
    let display = support::build_context();

    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
            color: [f32; 3],
        }

        implement_vertex!(Vertex, position, color);

        glium::VertexBuffer::new(&display,
            &[
                Vertex { position: [-0.5, -0.5], color: [1.0, 0.0, 0.0] },
                Vertex { position: [ 0.0,  0.5], color: [0.0, 1.0, 0.0] },
                Vertex { position: [ 0.5, -0.5], color: [0.0, 0.0, 1.0] },
            ]
        ).unwrap()
    };

    let program = program!(&display,
        140 => {
            vertex: "
                #version 140

                in vec2 position;
                in vec3 color;

                out vec3 v_color;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                    v_color = color;
                }
            ",

            fragment: "
                #version 140

                in vec3 v_color;
                out vec4 f_color;

                void main() {
                    f_color = vec4(v_color, 1.0);
                }
            ",
        },
    ).unwrap();

    b.iter(|| {
        let mut target = glium::Frame::new(display.clone(), (800, 600));
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.draw(&vertex_buffer, &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                    &program, &uniform!{}, &Default::default()).unwrap();
        target.finish().unwrap();
    });
}

#[bench]
fn build_buffer(b: &mut Bencher) {
    let display = support::build_context();

    b.iter(|| {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
            color: [f32; 3],
        }

        implement_vertex!(Vertex, position, color);

        glium::VertexBuffer::new(&display,
            &[
                Vertex { position: [-0.5, -0.5], color: [1.0, 0.0, 0.0] },
                Vertex { position: [ 0.0,  0.5], color: [0.0, 1.0, 0.0] },
                Vertex { position: [ 0.5, -0.5], color: [0.0, 0.0, 1.0] },
            ]
        ).unwrap()
    });
}
