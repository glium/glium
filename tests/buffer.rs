#[macro_use]
extern crate glium;

mod support;

#[test]
fn single_buffer_builder() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [f32; 3],
        field2: [f32; 3],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb = glium::buffer::Builder::new(&display)
                        .add_empty(12)
                        .build();
    let vb: glium::vertex::VertexBuffer<Vertex> = vb;

    assert_eq!(vb.len(), 12);

    display.assert_no_error(None);
}

#[test]
fn multiple_buffers_creation() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [f32; 3],
        field2: [f32; 3],
    }

    implement_vertex!(Vertex, field1, field2);

    let (vb1, vb2) = glium::buffer::Builder::new(&display)
                        .add_empty(12)
                        .add_empty(16)
                        .build();

    let vb1: glium::vertex::VertexBuffer<Vertex> = vb1;
    let vb2: glium::vertex::VertexBuffer<Vertex> = vb2;

    assert_eq!(vb1.len(), 12);
    assert_eq!(vb2.len(), 16);

    display.assert_no_error(None);
}

#[test]
fn multiple_buffers_data() {
    let display = support::build_display();

    #[derive(Debug, Copy, Clone, PartialEq)]
    struct Vertex {
        field: f32,
    }

    implement_vertex!(Vertex, field);

    let (vb1, vb2) = glium::buffer::Builder::new(&display)
                        .add_data(&[Vertex { field: 1.0 }, Vertex { field: 2.0 }])
                        .add_data(&[Vertex { field: 3.0 }])
                        .build();

    let vb1: glium::vertex::VertexBuffer<Vertex> = vb1;
    let vb2: glium::vertex::VertexBuffer<Vertex> = vb2;

    assert_eq!(vb1.len(), 2);
    if let Some(data) = vb1.read_if_supported() {
        assert_eq!(data, &[Vertex { field: 1.0 }, Vertex { field: 2.0 }]);
    }

    assert_eq!(vb2.len(), 1);
    if let Some(data) = vb2.read_if_supported() {
        assert_eq!(data, &[Vertex { field: 3.0 }]);
    }

    display.assert_no_error(None);
}
