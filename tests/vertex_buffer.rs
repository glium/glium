#[macro_use]
extern crate glium;

use glium::Surface;

mod support;

#[test]
fn transform_feedback() {
    let display = support::build_display();

    #[derive(Copy, Clone, PartialEq)]
    struct Vertex {
        output_val: (f32, f32),
    }

    implement_vertex!(Vertex, output_val);

    let (vb, ib) = support::build_rectangle_vb_ib(&display);

    let source = glium::program::ProgramCreationInput::SourceCode {
        tessellation_control_shader: None,
        tessellation_evaluation_shader: None,
        geometry_shader: None,
        outputs_srgb: false,
        uses_point_size: false,

        vertex_shader: "
            #version 110

            attribute vec2 position;

            varying vec2 output_val;

            void main() {
                output_val = position;
                gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
            }
        ",
        fragment_shader: "
            #version 130

            out vec4 color;

            void main() {
                color = vec4(1.0, 1.0, 1.0, 1.0);
            }
        ",

        transform_feedback_varyings: Some((
            vec!["output_val".to_string()],
            glium::program::TransformFeedbackMode::Separate
        )),
    };

    let program = match glium::Program::new(&display, source) {
        Ok(p) => p,
        Err(glium::program::ProgramCreationError::TransformFeedbackNotSupported) => return,
        Err(e) => panic!("{:?}", e)
    };

    let mut out_buffer: glium::VertexBuffer<Vertex> = glium::VertexBuffer::empty(&display, 6).unwrap();

    {
        let session = glium::vertex::TransformFeedbackSession::new(&display, &program,
                                                                   &mut out_buffer).unwrap();

        let params = glium::DrawParameters {
            transform_feedback: Some(&session),
            .. Default::default()
        };

        let mut frame = display.draw();
        frame.draw(&vb, &ib, &program, &uniform!{}, &params).unwrap();
        frame.finish().unwrap();
    }

    let result = match out_buffer.read() {
        Ok(r) => r,
        Err(glium::buffer::ReadError::NotSupported) => return,
        e => e.unwrap()
    };

    assert_eq!(result[0].output_val, (-1.0, 1.0));
    assert_eq!(result[1].output_val, (1.0, 1.0));
    assert_eq!(result[2].output_val, (-1.0, -1.0));
    assert_eq!(result[3].output_val, (-1.0, -1.0));
    assert_eq!(result[4].output_val, (1.0, 1.0));
    assert_eq!(result[5].output_val, (1.0, -1.0));

    display.assert_no_error(None);
}
