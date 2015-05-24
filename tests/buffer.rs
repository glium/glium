#[macro_use]
extern crate glium;

mod support;

#[test]
fn invalidate() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex { field: f32 }
    implement_vertex!(Vertex, field);

    let buffer = glium::VertexBuffer::new(&display,
        &[ Vertex { field: 2.0 } ]
    );

    buffer.invalidate();

    display.assert_no_error(None);
}

#[test]
fn invalidate_range() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex { field: f32 }
    implement_vertex!(Vertex, field);

    let buffer = glium::VertexBuffer::new(&display,
        &[ Vertex { field: 1.0 }, Vertex { field: 2.0 }, Vertex { field: 3.0 } ]
    );

    buffer.slice(1 .. 2).unwrap().invalidate();

    display.assert_no_error(None);
}
