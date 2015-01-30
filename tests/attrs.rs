#![feature(plugin)]
#![feature(unboxed_closures)]

#[plugin]
extern crate glium_macros;

extern crate glutin;
extern crate glium;

use glium::Surface;

mod support;

#[test]
#[should_fail(expected = "The program attribute `field1` does not match the vertex format")]
fn attribute_types_mismatch() {
    let display = support::build_display();

    #[vertex_format]
    #[derive(Copy)]
    struct Vertex {
        field1: [f32; 4],
    }

    let vertex_buffer = glium::VertexBuffer::new(&display, Vec::<Vertex>::new());
    let index_buffer = glium::IndexBuffer::new(&display,
                            glium::index::PointsList(Vec::<u16>::new()));

    let program = glium::Program::from_source(&display,
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
                &std::default::Default::default()).unwrap();
    target.finish();
    
    display.assert_no_error();
}

#[test]
#[should_fail(expected = "The program attribute `field2` is missing in the vertex bindings")]
fn missing_attribute() {
    let display = support::build_display();

    #[vertex_format]
    #[derive(Copy)]
    struct Vertex {
        field1: [f32; 4],
    }

    let vertex_buffer = glium::VertexBuffer::new(&display, Vec::<Vertex>::new());
    let index_buffer = glium::IndexBuffer::new(&display,
                            glium::index::PointsList(Vec::<u16>::new()));

    let program = glium::Program::from_source(&display,
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
                &std::default::Default::default()).unwrap();
    target.finish();
    
    display.assert_no_error();
}

macro_rules! attribute_test(
    ($name:ident, $attr_ty:ty, $glsl_ty:expr, $value:expr, $gl_pos:expr) => (
        #[test]
        fn $name() {
            let display = support::build_display();

            #[vertex_format]
            #[derive(Copy)]
            struct Vertex {
                field1: $attr_ty,
            }

            let vertex_buffer = glium::VertexBuffer::new(&display, vec![
                    Vertex { field1: $value }
                ]);
            let index_buffer = glium::IndexBuffer::new(&display,
                                    glium::index::PointsList(vec![0u16]));

            let program = glium::Program::from_source(&display,
                format!("
                    #version 110

                    attribute {} field1;

                    void main() {{
                        gl_Position = {};
                    }}
                ", $glsl_ty, $gl_pos).as_slice(),
                "
                    #version 110
                    void main() {
                        gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
                    }
                ",
                None)
                .unwrap();

            // drawing a frame
            let mut target = display.draw();
            target.draw(&vertex_buffer, &index_buffer, &program, &glium::uniforms::EmptyUniforms,
                        &std::default::Default::default()).unwrap();
            target.finish();
            
            display.assert_no_error();
        }

    )
);

attribute_test!(attribute_float, f32, "float", 0.0, "vec4(field1, 0.0, 0.0, 1.0)");
attribute_test!(attribute_vec2, [f32; 2], "vec2", [0.0, 0.0], "vec4(field1, 0.0, 1.0)");
attribute_test!(attribute_vec2_tuple, (f32, f32), "vec2", (0.0, 0.0), "vec4(field1, 0.0, 1.0)");
attribute_test!(attribute_vec3, [f32; 3], "vec3", [0.0, 0.0, 0.0], "vec4(field1, 1.0)");
attribute_test!(attribute_vec3_tuple, (f32, f32, f32), "vec3", (0.0, 0.0, 0.0), "vec4(field1, 1.0)");
attribute_test!(attribute_vec4, [f32; 4], "vec4", [0.0, 0.0, 0.0, 0.0], "field1");
attribute_test!(attribute_vec4_tuple, (f32, f32, f32, f32), "vec4", (0.0, 0.0, 0.0, 0.0), "field1");
