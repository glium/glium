#![feature(phase)]
#![feature(unboxed_closures)]

#[phase(plugin)]
extern crate glium_macros;

extern crate glutin;
extern crate glium;

use glium::Surface;

mod support;

#[test]
fn attribute_types_matching() {
    let display = support::build_display();

    #[vertex_format]
    #[deriving(Copy)]
    struct Vertex {
        field1: [f32, ..2],
    }

    let vertex_buffer = glium::VertexBuffer::new(&display, vec![
            Vertex { field1: [0.0, 0.0] }
        ]);
    let index_buffer = glium::IndexBuffer::new(&display,
                            glium::index_buffer::PointsList(vec![0u16]));

    let program = glium::Program::new(&display,
        // vertex shader
        "
            #version 110

            attribute vec2 field1;

            void main() {
                gl_Position = vec4(field1, 0.0, 1.0);
            }
        ",
        "
            #version 110
            void main() {
                gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
            }
        ",

        // geometry shader
        None)
        .unwrap();

    // drawing a frame
    let mut target = display.draw();
    target.draw(&vertex_buffer, &index_buffer, &program, &glium::uniforms::EmptyUniforms,
                &std::default::Default::default());
    target.finish();
    
    display.assert_no_error();
}

#[test]
#[should_fail(expected = "The program attribute `field1` does not match the vertex format")]
fn attribute_types_mismatch() {
    let display = support::build_display();

    #[vertex_format]
    #[deriving(Copy)]
    struct Vertex {
        field1: [f32, ..4],
    }

    let vertex_buffer = glium::VertexBuffer::new(&display, Vec::<Vertex>::new());
    let index_buffer = glium::IndexBuffer::new(&display,
                            glium::index_buffer::PointsList(Vec::<u16>::new()));

    let program = glium::Program::new(&display,
        // vertex shader
        "
            #version 110

            attribute vec2 field1;

            void main() {
                gl_Position = vec4(field1, 0.0, 1.0);
            }
        ",
        "
            #version 110
            void main() {
                gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
            }
        ",

        // geometry shader
        None)
        .unwrap();

    // drawing a frame
    let mut target = display.draw();
    target.draw(&vertex_buffer, &index_buffer, &program, &glium::uniforms::EmptyUniforms,
                &std::default::Default::default());
    target.finish();
    
    display.assert_no_error();
}

#[test]
#[should_fail(expected = "The program attribute `field2` is missing in the vertex bindings")]
fn missing_attribute() {
    let display = support::build_display();

    #[vertex_format]
    #[deriving(Copy)]
    struct Vertex {
        field1: [f32, ..4],
    }

    let vertex_buffer = glium::VertexBuffer::new(&display, Vec::<Vertex>::new());
    let index_buffer = glium::IndexBuffer::new(&display,
                            glium::index_buffer::PointsList(Vec::<u16>::new()));

    let program = glium::Program::new(&display,
        // vertex shader
        "
            #version 110

            attribute vec2 field2;

            void main() {
                gl_Position = vec4(field2, 0.0, 1.0);
            }
        ",
        "
            #version 110
            void main() {
                gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
            }
        ",

        // geometry shader
        None)
        .unwrap();

    // drawing a frame
    let mut target = display.draw();
    target.draw(&vertex_buffer, &index_buffer, &program, &glium::uniforms::EmptyUniforms,
                &std::default::Default::default());
    target.finish();
    
    display.assert_no_error();
}
