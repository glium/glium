#[macro_use]
extern crate glium;

use glium::Surface;

mod support;

#[test]
fn vertex_buffer_immutable_creation() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [f32; 3],
        field2: [f32; 3],
    }

    implement_vertex!(Vertex, field1, field2);

    glium::VertexBuffer::new(&display,
        vec![
            Vertex { field1: [-0.5, -0.5, 0.0], field2: [0.0, 1.0, 0.0] },
            Vertex { field1: [ 0.0,  0.5, 1.0], field2: [0.0, 0.0, 1.0] },
            Vertex { field1: [ 0.5, -0.5, 0.0], field2: [1.0, 0.0, 0.0] },
        ]
    );

    display.assert_no_error(None);
}

#[test]
fn vertex_buffer_dynamic_creation() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [f32; 3],
        field2: [f32; 3],
    }

    implement_vertex!(Vertex, field1, field2);

    glium::VertexBuffer::dynamic(&display,
        vec![
            Vertex { field1: [-0.5, -0.5, 0.0], field2: [0.0, 1.0, 0.0] },
            Vertex { field1: [ 0.0,  0.5, 1.0], field2: [0.0, 0.0, 1.0] },
            Vertex { field1: [ 0.5, -0.5, 0.0], field2: [1.0, 0.0, 0.0] },
        ]
    );

    display.assert_no_error(None);
}

#[test]
fn vertex_buffer_immutable_empty_creation() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [f32; 3],
        field2: [f32; 3],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb: glium::VertexBuffer<Vertex> = glium::VertexBuffer::empty(&display, 12);
    assert_eq!(vb.len(), 12);

    display.assert_no_error(None);
}

#[test]
fn vertex_buffer_dynamic_empty_creation() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [f32; 3],
        field2: [f32; 3],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb: glium::VertexBuffer<Vertex> = glium::VertexBuffer::empty_dynamic(&display, 12);
    assert_eq!(vb.len(), 12);

    display.assert_no_error(None);
}

#[test]
fn vertex_buffer_immutable_mapping_read() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let mut vb = glium::VertexBuffer::new(&display,
        vec![
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [12, 13], field2: [15, 17] },
        ]
    );

    let mapping = vb.map();
    assert_eq!(mapping[0].field1, [2, 3]);
    assert_eq!(mapping[1].field2, [15, 17]);

    display.assert_no_error(None);
}

#[test]
fn vertex_buffer_dynamic_mapping_read() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let mut vb = glium::VertexBuffer::dynamic(&display,
        vec![
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [12, 13], field2: [15, 17] },
        ]
    );

    let mapping = vb.map();
    assert_eq!(mapping[0].field1, [2, 3]);
    assert_eq!(mapping[1].field2, [15, 17]);

    display.assert_no_error(None);
}

#[test]
fn vertex_buffer_immutable_mapping_write() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let mut vb = glium::VertexBuffer::new(&display,
        vec![
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [12, 13], field2: [15, 17] },
        ]
    );

    {
        let mut mapping = vb.map();
        mapping[0].field1 = [0, 1];
    }

    let mapping = vb.map();
    assert_eq!(mapping[0].field1, [0, 1]);
    assert_eq!(mapping[1].field2, [15, 17]);

    display.assert_no_error(None);
}

#[test]
fn vertex_buffer_dynamic_mapping_write() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let mut vb = glium::VertexBuffer::dynamic(&display,
        vec![
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [12, 13], field2: [15, 17] },
        ]
    );

    {
        let mut mapping = vb.map();
        mapping[0].field1 = [0, 1];
    }

    let mapping = vb.map();
    assert_eq!(mapping[0].field1, [0, 1]);
    assert_eq!(mapping[1].field2, [15, 17]);

    display.assert_no_error(None);
}

// TODO: uncomment after std::thread::scoped has been stabilized
/*#[test]
fn vertex_buffer_mapping_multithread() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let mut vb = glium::VertexBuffer::empty(&display, 2);

    {
        let mut mapping = vb.map();
        std::thread::scoped(|| {
            mapping[0].field1 = [0, 1];
            mapping[1].field2 = [15, 17];
        });
    }

    let mapping = vb.map();
    assert_eq!(mapping[0].field1, [0, 1]);
    assert_eq!(mapping[1].field2, [15, 17]);

    display.assert_no_error(None);
}*/

#[test]
fn vertex_buffer_immutable_read() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb = glium::VertexBuffer::new(&display,
        vec![
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [12, 13], field2: [15, 17] },
        ]
    );

    let data = match vb.read_if_supported() {
        Some(d) => d,
        None => return
    };

    assert_eq!(data[0].field1, [2, 3]);
    assert_eq!(data[1].field2, [15, 17]);

    display.assert_no_error(None);
}

#[test]
fn vertex_buffer_dynamic_read() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb = glium::VertexBuffer::dynamic(&display,
        vec![
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [12, 13], field2: [15, 17] },
        ]
    );

    let data = match vb.read_if_supported() {
        Some(d) => d,
        None => return
    };

    assert_eq!(data[0].field1, [2, 3]);
    assert_eq!(data[1].field2, [15, 17]);

    display.assert_no_error(None);
}

#[test]
fn vertex_buffer_immutable_read_slice() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb = glium::VertexBuffer::new(&display,
        vec![
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [12, 13], field2: [15, 17] },
        ]
    );

    let data = match vb.slice(1 .. 2).unwrap().read_if_supported() {
        Some(d) => d,
        None => return
    };

    assert_eq!(data[0].field2, [15, 17]);

    display.assert_no_error(None);
}

#[test]
fn vertex_buffer_dynamic_read_slice() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb = glium::VertexBuffer::dynamic(&display,
        vec![
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [12, 13], field2: [15, 17] },
        ]
    );

    let data = match vb.slice(1 .. 2).unwrap().read_if_supported() {
        Some(d) => d,
        None => return
    };

    assert_eq!(data[0].field2, [15, 17]);

    display.assert_no_error(None);
}

#[test]
fn vertex_buffer_slice_out_of_bounds() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb = glium::VertexBuffer::new(&display,
        vec![
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [12, 13], field2: [15, 17] },
        ]
    );

    assert!(vb.slice(0 .. 3).is_none());

    display.assert_no_error(None);
}

#[test]
fn vertex_buffer_any() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [f32; 3],
        field2: [f32; 3],
    }

    implement_vertex!(Vertex, field1, field2);

    glium::VertexBuffer::new(&display,
        vec![
            Vertex { field1: [-0.5, -0.5, 0.0], field2: [0.0, 1.0, 0.0] },
            Vertex { field1: [ 0.0,  0.5, 1.0], field2: [0.0, 0.0, 1.0] },
            Vertex { field1: [ 0.5, -0.5, 0.0], field2: [1.0, 0.0, 0.0] },
        ]
    ).into_vertex_buffer_any();

    display.assert_no_error(None);
}

#[test]
fn vertex_buffer_immutable_write() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb = glium::VertexBuffer::new(&display,
        vec![
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [ 0,  0], field2: [ 0,  0] },
        ]
    );

    vb.write(vec![
        Vertex { field1: [ 2,  3], field2: [ 5,  7] },
        Vertex { field1: [12, 13], field2: [15, 17] }
    ]);

    let data = match vb.read_if_supported() {
        Some(d) => d,
        None => return
    };

    assert_eq!(data[0].field1, [2, 3]);
    assert_eq!(data[0].field2, [5, 7]);
    assert_eq!(data[1].field1, [12, 13]);
    assert_eq!(data[1].field2, [15, 17]);

    display.assert_no_error(None);
}

#[test]
fn vertex_buffer_dynamic_write() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb = glium::VertexBuffer::dynamic(&display,
        vec![
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [ 0,  0], field2: [ 0,  0] },
        ]
    );

    vb.write(vec![
        Vertex { field1: [ 2,  3], field2: [ 5,  7] },
        Vertex { field1: [12, 13], field2: [15, 17] }
    ]);

    let data = match vb.read_if_supported() {
        Some(d) => d,
        None => return
    };

    assert_eq!(data[0].field1, [2, 3]);
    assert_eq!(data[0].field2, [5, 7]);
    assert_eq!(data[1].field1, [12, 13]);
    assert_eq!(data[1].field2, [15, 17]);

    display.assert_no_error(None);
}

#[test]
fn vertex_buffer_immutable_write_slice() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb = glium::VertexBuffer::new(&display,
        vec![
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [ 0,  0], field2: [ 0,  0] },
        ]
    );

    vb.slice(1 .. 2).unwrap().write(vec![Vertex { field1: [12, 13], field2: [15, 17] }]);

    let data = match vb.read_if_supported() {
        Some(d) => d,
        None => return
    };

    assert_eq!(data[0].field1, [2, 3]);
    assert_eq!(data[0].field2, [5, 7]);
    assert_eq!(data[1].field1, [12, 13]);
    assert_eq!(data[1].field2, [15, 17]);

    display.assert_no_error(None);
}

#[test]
fn vertex_buffer_dynamic_write_slice() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb = glium::VertexBuffer::dynamic(&display,
        vec![
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [ 0,  0], field2: [ 0,  0] },
        ]
    );

    vb.slice(1 .. 2).unwrap().write(vec![Vertex { field1: [12, 13], field2: [15, 17] }]);

    let data = match vb.read_if_supported() {
        Some(d) => d,
        None => return
    };

    assert_eq!(data[0].field1, [2, 3]);
    assert_eq!(data[0].field2, [5, 7]);
    assert_eq!(data[1].field1, [12, 13]);
    assert_eq!(data[1].field2, [15, 17]);

    display.assert_no_error(None);
}

#[test]
fn zero_sized_immutable_vertex_buffer() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [f32; 3],
        field2: [f32; 3],
    }

    implement_vertex!(Vertex, field1, field2);

    glium::VertexBuffer::new(&display, Vec::<Vertex>::new());

    display.assert_no_error(None);
}

#[test]
fn zero_sized_dynamic_vertex_buffer() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [f32; 3],
        field2: [f32; 3],
    }

    implement_vertex!(Vertex, field1, field2);

    glium::VertexBuffer::new(&display, Vec::<Vertex>::new());

    display.assert_no_error(None);
}

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

    let mut out_buffer: glium::VertexBuffer<Vertex> = glium::VertexBuffer::empty(&display, 6);

    {
        let session = glium::vertex::TransformFeedbackSession::new(&display, &program,
                                                                   &mut out_buffer).unwrap();

        let params = glium::DrawParameters {
            transform_feedback: Some(&session),
            .. Default::default()
        };

        display.draw().draw(&vb, &ib, &program, &uniform!{}, &params).unwrap();
    }

    let result = match out_buffer.read_if_supported() {
        Some(r) => r,
        None => return
    };

    assert_eq!(result[0].output_val, (-1.0, 1.0));
    assert_eq!(result[1].output_val, (1.0, 1.0));
    assert_eq!(result[2].output_val, (-1.0, -1.0));
    assert_eq!(result[3].output_val, (-1.0, -1.0));
    assert_eq!(result[4].output_val, (1.0, 1.0));
    assert_eq!(result[5].output_val, (1.0, -1.0));

    display.assert_no_error(None);
}
