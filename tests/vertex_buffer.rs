#![feature(phase)]
#![feature(unboxed_closures)]

#[phase(plugin)]
extern crate glium_macros;

extern crate glutin;
extern crate glium;

mod support;

#[test]
fn vertex_buffer_creation() {
    let display = support::build_display();

    #[vertex_format]
    #[allow(dead_code)]
    #[deriving(Copy)]
    struct Vertex {
        field1: [f32, ..3],
        field2: [f32, ..3],
    }

    glium::VertexBuffer::new(&display, 
        vec![
            Vertex { field1: [-0.5, -0.5, 0.0], field2: [0.0, 1.0, 0.0] },
            Vertex { field1: [ 0.0,  0.5, 1.0], field2: [0.0, 0.0, 1.0] },
            Vertex { field1: [ 0.5, -0.5, 0.0], field2: [1.0, 0.0, 0.0] },
        ]
    );

    display.assert_no_error();
}

#[test]
fn vertex_buffer_mapping_read() {
    let display = support::build_display();

    #[vertex_format]
    #[deriving(Copy)]
    struct Vertex {
        field1: [u8, ..2],
        field2: [u8, ..2],
    }

    let mut vb = glium::VertexBuffer::new(&display, 
        vec![
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [12, 13], field2: [15, 17] },
        ]
    );

    let mapping = vb.map();
    assert_eq!(mapping[0].field1.as_slice(), [2, 3].as_slice());
    assert_eq!(mapping[1].field2.as_slice(), [15, 17].as_slice());

    display.assert_no_error();
}

#[test]
fn vertex_buffer_mapping_write() {
    let display = support::build_display();
    
    #[vertex_format]
    #[deriving(Copy)]
    struct Vertex {
        field1: [u8, ..2],
        field2: [u8, ..2],
    }

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
    assert_eq!(mapping[0].field1.as_slice(), [0, 1].as_slice());
    assert_eq!(mapping[1].field2.as_slice(), [15, 17].as_slice());

    display.assert_no_error();
}

#[test]
#[cfg(feature = "gl_extensions")]
fn vertex_buffer_read() {
    let display = support::build_display();

    #[vertex_format]
    #[deriving(Copy)]
    struct Vertex {
        field1: [u8, ..2],
        field2: [u8, ..2],
    }

    let vb = glium::VertexBuffer::new(&display, 
        vec![
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [12, 13], field2: [15, 17] },
        ]
    );

    let data = vb.read();
    assert_eq!(data[0].field1.as_slice(), [2, 3].as_slice());
    assert_eq!(data[1].field2.as_slice(), [15, 17].as_slice());

    display.assert_no_error();
}

#[test]
#[cfg(feature = "gl_extensions")]
fn vertex_buffer_read_slice() {
    let display = support::build_display();

    #[vertex_format]
    #[deriving(Copy)]
    struct Vertex {
        field1: [u8, ..2],
        field2: [u8, ..2],
    }

    let vb = glium::VertexBuffer::new(&display, 
        vec![
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [12, 13], field2: [15, 17] },
        ]
    );

    let data = vb.read_slice(1, 1);
    assert_eq!(data[0].field2.as_slice(), [15, 17].as_slice());
    
    display.assert_no_error();
}

#[test]
#[should_fail]
#[cfg(feature = "gl_extensions")]
fn vertex_buffer_read_slice_out_of_bounds() {
    let display = support::build_display();

    #[vertex_format]
    #[deriving(Copy)]
    struct Vertex {
        field1: [u8, ..2],
        field2: [u8, ..2],
    }

    let vb = glium::VertexBuffer::new(&display, 
        vec![
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [12, 13], field2: [15, 17] },
        ]
    );

    vb.read_slice(0, 3);
}

#[test]
fn vertex_buffer_any() {
    let display = support::build_display();

    #[vertex_format]
    #[allow(dead_code)]
    #[deriving(Copy)]
    struct Vertex {
        field1: [f32, ..3],
        field2: [f32, ..3],
    }

    glium::VertexBuffer::new(&display, 
        vec![
            Vertex { field1: [-0.5, -0.5, 0.0], field2: [0.0, 1.0, 0.0] },
            Vertex { field1: [ 0.0,  0.5, 1.0], field2: [0.0, 0.0, 1.0] },
            Vertex { field1: [ 0.5, -0.5, 0.0], field2: [1.0, 0.0, 0.0] },
        ]
    ).into_vertex_buffer_any();

    display.assert_no_error();
}
