#[macro_use]
extern crate glium;

mod support;

use glium::Surface;
use glium::buffer::BufferMode;
use std::mem;

#[test]
fn buffer_immutable_creation() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [f32; 3],
        field2: [f32; 3],
    }

    implement_vertex!(Vertex, field1, field2);

    glium::VertexBuffer::new(&display,
        &[
            Vertex { field1: [-0.5, -0.5, 0.0], field2: [0.0, 1.0, 0.0] },
            Vertex { field1: [ 0.0,  0.5, 1.0], field2: [0.0, 0.0, 1.0] },
            Vertex { field1: [ 0.5, -0.5, 0.0], field2: [1.0, 0.0, 0.0] },
        ]
    ).unwrap();

    display.assert_no_error(None);
}

#[test]
fn buffer_dynamic_creation() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [f32; 3],
        field2: [f32; 3],
    }

    implement_vertex!(Vertex, field1, field2);

    glium::VertexBuffer::dynamic(&display,
        &[
            Vertex { field1: [-0.5, -0.5, 0.0], field2: [0.0, 1.0, 0.0] },
            Vertex { field1: [ 0.0,  0.5, 1.0], field2: [0.0, 0.0, 1.0] },
            Vertex { field1: [ 0.5, -0.5, 0.0], field2: [1.0, 0.0, 0.0] },
        ]
    ).unwrap();

    display.assert_no_error(None);
}

#[test]
fn buffer_immutable_empty_creation() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [f32; 3],
        field2: [f32; 3],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb: glium::VertexBuffer<Vertex> = glium::VertexBuffer::empty(&display, 12).unwrap();
    assert_eq!(vb.len(), 12);

    display.assert_no_error(None);
}

#[test]
fn buffer_dynamic_empty_creation() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [f32; 3],
        field2: [f32; 3],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb: glium::VertexBuffer<Vertex> = glium::VertexBuffer::empty_dynamic(&display, 12).unwrap();
    assert_eq!(vb.len(), 12);

    display.assert_no_error(None);
}

#[test]
fn buffer_immutable_mapping_read() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let mut vb = glium::VertexBuffer::new(&display,
        &[
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [12, 13], field2: [15, 17] },
        ]
    ).unwrap();

    let mapping = vb.map();
    assert_eq!(mapping[0].field1, [2, 3]);
    assert_eq!(mapping[1].field2, [15, 17]);

    display.assert_no_error(None);
}

#[test]
fn buffer_dynamic_mapping_read() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let mut vb = glium::VertexBuffer::dynamic(&display,
        &[
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [12, 13], field2: [15, 17] },
        ]
    ).unwrap();

    let mapping = vb.map();
    assert_eq!(mapping[0].field1, [2, 3]);
    assert_eq!(mapping[1].field2, [15, 17]);

    display.assert_no_error(None);
}

#[test]
fn buffer_immutable_mapping_write() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let mut vb = glium::VertexBuffer::new(&display,
        &[
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [12, 13], field2: [15, 17] },
        ]
    ).unwrap();

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
fn buffer_dynamic_mapping_write() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let mut vb = glium::VertexBuffer::dynamic(&display,
        &[
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [12, 13], field2: [15, 17] },
        ]
    ).unwrap();

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
fn buffer_mapping_multithread() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let mut vb = glium::VertexBuffer::empty(&display, 2).unwrap();

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
fn buffer_immutable_read() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb = glium::VertexBuffer::new(&display,
        &[
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [12, 13], field2: [15, 17] },
        ]
    ).unwrap();

    let data = match vb.read() {
        Ok(r) => r,
        Err(glium::buffer::ReadError::NotSupported) => return,
        e => e.unwrap()
    };

    assert_eq!(data[0].field1, [2, 3]);
    assert_eq!(data[1].field2, [15, 17]);

    display.assert_no_error(None);
}

#[test]
fn buffer_dynamic_read() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb = glium::VertexBuffer::dynamic(&display,
        &[
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [12, 13], field2: [15, 17] },
        ]
    ).unwrap();

    let data = match vb.read() {
        Ok(r) => r,
        Err(glium::buffer::ReadError::NotSupported) => return,
        e => e.unwrap()
    };

    assert_eq!(data[0].field1, [2, 3]);
    assert_eq!(data[1].field2, [15, 17]);

    display.assert_no_error(None);
}

#[test]
fn buffer_immutable_read_slice() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb = glium::VertexBuffer::new(&display,
        &[
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [12, 13], field2: [15, 17] },
        ]
    ).unwrap();

    let data = match vb.slice(1 .. 2).unwrap().read() {
        Ok(r) => r,
        Err(glium::buffer::ReadError::NotSupported) => return,
        e => e.unwrap()
    };

    assert_eq!(data[0].field2, [15, 17]);

    display.assert_no_error(None);
}

#[test]
fn buffer_dynamic_read_slice() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb = glium::VertexBuffer::dynamic(&display,
        &[
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [12, 13], field2: [15, 17] },
        ]
    ).unwrap();

    let data = match vb.slice(1 .. 2).unwrap().read() {
        Ok(r) => r,
        Err(glium::buffer::ReadError::NotSupported) => return,
        e => e.unwrap()
    };

    assert_eq!(data[0].field2, [15, 17]);

    display.assert_no_error(None);
}

#[test]
fn buffer_slice_out_of_bounds() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb = glium::VertexBuffer::new(&display,
        &[
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [12, 13], field2: [15, 17] },
        ]
    ).unwrap();

    assert!(vb.slice(0 .. 3).is_none());

    display.assert_no_error(None);
}

#[test]
fn buffer_immutable_write() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb = glium::VertexBuffer::new(&display,
        &[
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [ 0,  0], field2: [ 0,  0] },
        ]
    ).unwrap();

    vb.write(&[
        Vertex { field1: [ 2,  3], field2: [ 5,  7] },
        Vertex { field1: [12, 13], field2: [15, 17] }
    ]);

    let data = match vb.read() {
        Ok(r) => r,
        Err(glium::buffer::ReadError::NotSupported) => return,
        e => e.unwrap()
    };

    assert_eq!(data[0].field1, [2, 3]);
    assert_eq!(data[0].field2, [5, 7]);
    assert_eq!(data[1].field1, [12, 13]);
    assert_eq!(data[1].field2, [15, 17]);

    display.assert_no_error(None);
}

#[test]
fn buffer_dynamic_write() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb = glium::VertexBuffer::dynamic(&display,
        &[
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [ 0,  0], field2: [ 0,  0] },
        ]
    ).unwrap();

    vb.write(&[
        Vertex { field1: [ 2,  3], field2: [ 5,  7] },
        Vertex { field1: [12, 13], field2: [15, 17] }
    ]);

    let data = match vb.read() {
        Ok(r) => r,
        Err(glium::buffer::ReadError::NotSupported) => return,
        e => e.unwrap()
    };

    assert_eq!(data[0].field1, [2, 3]);
    assert_eq!(data[0].field2, [5, 7]);
    assert_eq!(data[1].field1, [12, 13]);
    assert_eq!(data[1].field2, [15, 17]);

    display.assert_no_error(None);
}

#[test]
fn buffer_immutable_write_slice() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb = glium::VertexBuffer::new(&display,
        &[
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [ 0,  0], field2: [ 0,  0] },
        ]
    ).unwrap();

    vb.slice(1 .. 2).unwrap().write(&[Vertex { field1: [12, 13], field2: [15, 17] }]);

    let data = match vb.read() {
        Ok(r) => r,
        Err(glium::buffer::ReadError::NotSupported) => return,
        e => e.unwrap()
    };

    assert_eq!(data[0].field1, [2, 3]);
    assert_eq!(data[0].field2, [5, 7]);
    assert_eq!(data[1].field1, [12, 13]);
    assert_eq!(data[1].field2, [15, 17]);

    display.assert_no_error(None);
}

#[test]
fn buffer_dynamic_write_slice() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [u8; 2],
        field2: [u8; 2],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb = glium::VertexBuffer::dynamic(&display,
        &[
            Vertex { field1: [ 2,  3], field2: [ 5,  7] },
            Vertex { field1: [ 0,  0], field2: [ 0,  0] },
        ]
    ).unwrap();

    vb.slice(1 .. 2).unwrap().write(&[Vertex { field1: [12, 13], field2: [15, 17] }]);

    let data = match vb.read() {
        Ok(r) => r,
        Err(glium::buffer::ReadError::NotSupported) => return,
        e => e.unwrap()
    };

    assert_eq!(data[0].field1, [2, 3]);
    assert_eq!(data[0].field2, [5, 7]);
    assert_eq!(data[1].field1, [12, 13]);
    assert_eq!(data[1].field2, [15, 17]);

    display.assert_no_error(None);
}

#[test]
fn zero_sized_immutable_buffer() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [f32; 3],
        field2: [f32; 3],
    }

    implement_vertex!(Vertex, field1, field2);

    glium::VertexBuffer::new(&display, &Vec::<Vertex>::new()).unwrap();

    display.assert_no_error(None);
}

#[test]
fn zero_sized_dynamic_buffer() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [f32; 3],
        field2: [f32; 3],
    }

    implement_vertex!(Vertex, field1, field2);

    glium::VertexBuffer::new(&display, &Vec::<Vertex>::new()).unwrap();

    display.assert_no_error(None);
}

#[test]
fn invalidate() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex { field: f32 }
    implement_vertex!(Vertex, field);

    let buffer = glium::VertexBuffer::new(&display,
        &[ Vertex { field: 2.0 } ]
    ).unwrap();

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
    ).unwrap();

    buffer.slice(1 .. 2).unwrap().invalidate();

    display.assert_no_error(None);
}

#[test]
fn immutable_mapping_forget_then_remap() {
    let display = support::build_display();

    let mut buf = glium::buffer::BufferView::new(&display, &[1, 2, 3],
                                                 glium::buffer::BufferType::ArrayBuffer,
                                                 BufferMode::Immutable)
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
                                                 glium::buffer::BufferType::ArrayBuffer,
                                                 BufferMode::Immutable)
                                                 .unwrap();

    {
        let mut mapping = buf.map();
        mapping[0] = 3;
        mem::forget(mapping);
    }

    let data = match buf.read() {
        Ok(r) => r,
        Err(glium::buffer::ReadError::NotSupported) => return,
        e => e.unwrap()
    };

    assert_eq!(data[1], 2);
    assert_eq!(data[2], 3);

    display.assert_no_error(None);
}

#[test]
fn immutable_mapping_forget_then_invalidate() {
    let display = support::build_display();

    let mut buf = glium::buffer::BufferView::new(&display, &[1, 2, 3],
                                                 glium::buffer::BufferType::ArrayBuffer,
                                                 BufferMode::Immutable)
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
        &[
            Vertex { position: [-1.0,  1.0] },
            Vertex { position: [ 1.0,  1.0] },
            Vertex { position: [-1.0, -1.0] },
            Vertex { position: [ 1.0, -1.0] },
        ]
    ).unwrap();

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
fn persistent_mapping_forget_then_remap() {
    let display = support::build_display();

    let mut buf = glium::buffer::BufferView::new(&display, &[1, 2, 3],
                                                 glium::buffer::BufferType::ArrayBuffer,
                                                 BufferMode::Persistent)
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
fn persistent_mapping_forget_then_read() {
    let display = support::build_display();

    let mut buf = glium::buffer::BufferView::new(&display, &[1, 2, 3],
                                                 glium::buffer::BufferType::ArrayBuffer,
                                                 BufferMode::Persistent)
                                                 .unwrap();

    {
        let mut mapping = buf.map();
        mapping[0] = 3;
        mem::forget(mapping);
    }

    let data = match buf.read() {
        Ok(r) => r,
        Err(glium::buffer::ReadError::NotSupported) => return,
        e => e.unwrap()
    };

    assert_eq!(data[1], 2);
    assert_eq!(data[2], 3);

    display.assert_no_error(None);
}

#[test]
fn persistent_mapping_forget_then_invalidate() {
    let display = support::build_display();

    let mut buf = glium::buffer::BufferView::new(&display, &[1, 2, 3],
                                                 glium::buffer::BufferType::ArrayBuffer,
                                                 BufferMode::Persistent)
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
        &[
            Vertex { position: [-1.0,  1.0] },
            Vertex { position: [ 1.0,  1.0] },
            Vertex { position: [-1.0, -1.0] },
            Vertex { position: [ 1.0, -1.0] },
        ]
    ).unwrap();

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
fn copy_to() {
    let display = support::build_display();

    let buf1 = glium::buffer::BufferView::new(&display, &[1, 2, 3],
                                              glium::buffer::BufferType::ArrayBuffer,
                                              BufferMode::Persistent);
    let buf1 = if let Ok(buf) = buf1 { buf } else { return };

    let buf2 = glium::buffer::BufferView::new(&display, &[0, 0, 0],
                                              glium::buffer::BufferType::ArrayBuffer,
                                              BufferMode::Persistent);
    let buf2 = if let Ok(buf) = buf2 { buf } else { return };

    if let Err(_) = buf1.copy_to(buf2.as_slice()) {
        return;
    }

    let result = match buf2.read() {
        Ok(r) => r,
        Err(_) => return
    };

    assert_eq!(result, [1, 2, 3]);

    display.assert_no_error(None);
}

#[test]
fn copy_to_slice() {
    let display = support::build_display();

    let buf1 = glium::buffer::BufferView::<[u8]>::new(&display, &[1, 2],
                                                      glium::buffer::BufferType::ArrayBuffer,
                                                      BufferMode::Persistent);
    let buf1 = if let Ok(buf) = buf1 { buf } else { return };

    let buf2 = glium::buffer::BufferView::<[u8]>::new(&display, &[0, 0, 0],
                                                      glium::buffer::BufferType::ArrayBuffer,
                                                      BufferMode::Persistent);
    let buf2 = if let Ok(buf) = buf2 { buf } else { return };

    if let Err(_) = buf1.copy_to(buf2.slice(1 .. 3).unwrap()) {
        return;
    }

    let result = match buf2.read() {
        Ok(r) => r,
        Err(_) => return
    };

    assert_eq!(result, [0, 1, 2]);

    display.assert_no_error(None);
}
