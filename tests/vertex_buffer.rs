#[macro_use]
extern crate glium;

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
