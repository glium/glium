#[macro_use]
extern crate glium;

mod support;

use glium::Surface;
use std::mem;

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

#[test]
fn immutable_mapping_forget_then_remap() {
    let display = support::build_display();

    let mut buf = glium::buffer::BufferView::new(&display, &[1, 2, 3],
                                                 glium::buffer::BufferType::ArrayBuffer, false)
                                                 .unwrap();

    {
        let mut mapping = buf.map();
        mapping[0] = 3;
        mem::forget(mapping);
    }

    let mapping = buf.map();
    assert_eq!(mapping[1], 2);
    assert_eq!(mapping[2], 3);

    display.assert_no_error(None);
}

#[test]
fn immutable_mapping_forget_then_read() {
    let display = support::build_display();

    let mut buf = glium::buffer::BufferView::new(&display, &[1, 2, 3],
                                                 glium::buffer::BufferType::ArrayBuffer, false)
                                                 .unwrap();

    {
        let mut mapping = buf.map();
        mapping[0] = 3;
        mem::forget(mapping);
    }

    let data = match buf.read_if_supported() {
        Some(d) => d,
        None => return,
    };

    assert_eq!(data[1], 2);
    assert_eq!(data[2], 3);

    display.assert_no_error(None);
}

#[test]
fn immutable_mapping_forget_then_invalidate() {
    let display = support::build_display();

    let mut buf = glium::buffer::BufferView::new(&display, &[1, 2, 3],
                                                 glium::buffer::BufferType::ArrayBuffer, false)
                                                 .unwrap();

    {
        let mut mapping = buf.map();
        mapping[0] = 3;
        mem::forget(mapping);
    }

    buf.invalidate();
    display.assert_no_error(None);
}

#[test]
fn immutable_mapping_forget_then_draw() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2]
    }

    implement_vertex!(Vertex, position);

    let mut vb = glium::VertexBuffer::new(&display,
        vec![
            Vertex { position: [-1.0,  1.0] },
            Vertex { position: [ 1.0,  1.0] },
            Vertex { position: [-1.0, -1.0] },
            Vertex { position: [ 1.0, -1.0] },
        ]
    );

    let program = program!(&display,
        140 => {
            vertex: "
                #version 140

                in vec2 position;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                }
            ",
            fragment: "
                #version 140

                out vec4 color;
                void main() {
                    color = vec4(1.0, 0.0, 0.0, 1.0);
                }
            ",
        },
        110 => {
            vertex: "
                #version 110

                attribute vec2 position;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                }
            ",
            fragment: "
                #version 110

                void main() {
                    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
                }
            ",
        },
        100 => {
            vertex: "
                #version 100

                attribute lowp vec2 position;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                }
            ",
            fragment: "
                #version 100

                void main() {
                    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
                }
            ",
        },
    ).unwrap();

    {
        let mapping = vb.map();
        mem::forget(mapping);
    }

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip),
                              &program, &uniform!{}, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    for pixel in data.iter().flat_map(|e| e.iter()) {
        assert_eq!(*pixel, (255, 0, 0, 255));
    }

    display.assert_no_error(None);
}

#[test]
fn dynamic_mapping_forget_then_remap() {
    let display = support::build_display();

    let mut buf = glium::buffer::BufferView::new(&display, &[1, 2, 3],
                                                 glium::buffer::BufferType::ArrayBuffer, true)
                                                 .unwrap();

    {
        let mut mapping = buf.map();
        mapping[0] = 3;
        mem::forget(mapping);
    }

    let mapping = buf.map();
    assert_eq!(mapping[1], 2);
    assert_eq!(mapping[2], 3);

    display.assert_no_error(None);
}

#[test]
#[ignore]       // TODO: mystic rust-related bug
fn dynamic_mapping_forget_then_read() {
    let display = support::build_display();

    let mut buf = glium::buffer::BufferView::new(&display, &[1, 2, 3],
                                                 glium::buffer::BufferType::ArrayBuffer, true)
                                                 .unwrap();

    {
        let mut mapping = buf.map();
        mapping[0] = 3;
        mem::forget(mapping);
    }

    let data = match buf.read_if_supported() {
        Some(d) => d,
        None => return,
    };

    assert_eq!(data[1], 2);
    assert_eq!(data[2], 3);

    display.assert_no_error(None);
}

#[test]
fn dynamic_mapping_forget_then_invalidate() {
    let display = support::build_display();

    let mut buf = glium::buffer::BufferView::new(&display, &[1, 2, 3],
                                                 glium::buffer::BufferType::ArrayBuffer, true)
                                                 .unwrap();

    {
        let mut mapping = buf.map();
        mapping[0] = 3;
        mem::forget(mapping);
    }

    buf.invalidate();
    display.assert_no_error(None);
}

#[test]
fn dynamic_mapping_forget_then_draw() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2]
    }

    implement_vertex!(Vertex, position);

    let mut vb = glium::VertexBuffer::dynamic(&display,
        vec![
            Vertex { position: [-1.0,  1.0] },
            Vertex { position: [ 1.0,  1.0] },
            Vertex { position: [-1.0, -1.0] },
            Vertex { position: [ 1.0, -1.0] },
        ]
    );

    let program = program!(&display,
        140 => {
            vertex: "
                #version 140

                in vec2 position;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                }
            ",
            fragment: "
                #version 140

                out vec4 color;
                void main() {
                    color = vec4(1.0, 0.0, 0.0, 1.0);
                }
            ",
        },
        110 => {
            vertex: "
                #version 110

                attribute vec2 position;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                }
            ",
            fragment: "
                #version 110

                void main() {
                    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
                }
            ",
        },
        100 => {
            vertex: "
                #version 100

                attribute lowp vec2 position;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                }
            ",
            fragment: "
                #version 100

                void main() {
                    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
                }
            ",
        },
    ).unwrap();

    {
        let mapping = vb.map();
        mem::forget(mapping);
    }

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip),
                              &program, &uniform!{}, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    for pixel in data.iter().flat_map(|e| e.iter()) {
        assert_eq!(*pixel, (255, 0, 0, 255));
    }

    display.assert_no_error(None);
}
