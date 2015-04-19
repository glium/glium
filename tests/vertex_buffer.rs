extern crate glutin;

#[macro_use]
extern crate glium;

use glium::Surface;
use std::default::Default;

mod support;

#[test]
fn vertex_buffer_creation() {
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

    display.assert_no_error();
}

#[test]
fn vertex_buffer_empty() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [f32; 3],
        field2: [f32; 3],
    }

    implement_vertex!(Vertex, field1, field2);

    let vb: glium::VertexBuffer<Vertex> = glium::VertexBuffer::empty(&display, 12);
    assert_eq!(vb.len(), 12);

    display.assert_no_error();
}

#[test]
fn vertex_buffer_mapping_read() {
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

    display.assert_no_error();
}

#[test]
fn vertex_buffer_mapping_write() {
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

    display.assert_no_error();
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

    display.assert_no_error();
}*/

#[test]
fn vertex_buffer_read() {
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

    display.assert_no_error();
}

#[test]
fn vertex_buffer_read_slice() {
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
    
    display.assert_no_error();
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

    display.assert_no_error();
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

    display.assert_no_error();
}

#[test]
fn vertex_buffer_write() {
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

    display.assert_no_error();
}

#[test]
fn vertex_buffer_write_slice() {
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

    display.assert_no_error();
}

#[test]
fn multiple_buffers_source() {
    let display = support::build_display();

    let buffer1 = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
        }

        implement_vertex!(Vertex, position);

        glium::VertexBuffer::new(&display, 
            vec![
                Vertex { position: [-1.0,  1.0] },
                Vertex { position: [ 1.0,  1.0] },
                Vertex { position: [-1.0, -1.0] },
                Vertex { position: [ 1.0, -1.0] },
            ]
        )
    };

    let buffer2 = {
        #[derive(Copy, Clone)]
        struct Vertex {
            color: [f32; 3],
        }

        implement_vertex!(Vertex, color);

        glium::VertexBuffer::new(&display, 
            vec![
                Vertex { color: [1.0, 0.0, 0.0] },
                Vertex { color: [1.0, 0.0, 0.0] },
                Vertex { color: [1.0, 0.0, 0.0] },
                Vertex { color: [1.0, 0.0, 0.0] },
            ]
        )
    };

    let index_buffer = glium::IndexBuffer::new(&display,
        glium::index::TriangleStrip(vec![0u16, 1, 2, 3]));

    let program = glium::Program::from_source(&display,
        "
            #version 110

            attribute vec2 position;
            attribute vec3 color;

            varying vec3 v_color;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
                v_color = color;
            }
        ",
        "
            #version 110
            varying vec3 v_color;

            void main() {
                gl_FragColor = vec4(v_color, 1.0);
            }
        ",
        None)
        .unwrap();

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw((&buffer1, &buffer2), &index_buffer, &program, &uniform!{},
                              &std::default::Default::default()).unwrap();

    let data: Vec<Vec<(f32, f32, f32, f32)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(1.0, 0.0, 0.0, 1.0));
        }
    }
    
    display.assert_no_error();
}

#[test]
fn zero_sized_vertex_buffer() {
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        field1: [f32; 3],
        field2: [f32; 3],
    }

    implement_vertex!(Vertex, field1, field2);

    glium::VertexBuffer::new(&display, Vec::<Vertex>::new());

    display.assert_no_error();
}

#[test]
fn slice_draw_indices() {
    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }

    implement_vertex!(Vertex, position);

    let display = support::build_display();
    let program = glium::Program::from_source(&display,
        "
            #version 110

            attribute vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        ",
        "
            #version 110

            void main() {
                gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
            }
        ",
        None).unwrap();

    let vb = glium::VertexBuffer::new(&display, vec![
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]);

    let indices = glium::index::TrianglesList(vec![0u16, 1, 2]);
    let indices = glium::IndexBuffer::new(&display, indices);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(vb.slice(1 .. 4).unwrap(), &indices, &program,
                &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8)>> = texture.read();
    assert_eq!(data.last().unwrap()[0], (0, 0, 0));
    assert_eq!(data[0].last().unwrap(), &(255, 0, 0));

    display.assert_no_error();
}

#[test]
fn slice_draw_noindices() {
    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }

    implement_vertex!(Vertex, position);

    let display = support::build_display();
    let program = glium::Program::from_source(&display,
        "
            #version 110

            attribute vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        ",
        "
            #version 110

            void main() {
                gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
            }
        ",
        None).unwrap();

    let vb = glium::VertexBuffer::new(&display, vec![
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]);

    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(vb.slice(1 .. 4).unwrap(), &indices, &program,
                &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8)>> = texture.read();
    assert_eq!(data.last().unwrap()[0], (0, 0, 0));
    assert_eq!(data[0].last().unwrap(), &(255, 0, 0));

    display.assert_no_error();
}

#[test]
fn slice_draw_multiple() {
    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }

    implement_vertex!(Vertex, position);

    #[derive(Copy, Clone)]
    struct Vertex2 {
        position2: [f32; 2],
    }

    implement_vertex!(Vertex2, position2);

    let display = support::build_display();
    let program = glium::Program::from_source(&display,
        "
            #version 110

            attribute vec2 position;
            attribute vec2 position2;

            void main() {
                if (position != position2) {
                    gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
                } else {
                    gl_Position = vec4(position, 0.0, 1.0);
                }
            }
        ",
        "
            #version 110

            void main() {
                gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
            }
        ",
        None).unwrap();

    // the 3 last elements will be drawn
    let vb1 = glium::VertexBuffer::new(&display, vec![
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [-1.0, 1.0] },
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]);

    // the 3 last elements will be drawn
    let vb2 = glium::VertexBuffer::new(&display, vec![
        Vertex2 { position2: [-1.0,  1.0] }, Vertex2 { position2: [-1.0, 1.0] },
        Vertex2 { position2: [-1.0,  1.0] }, Vertex2 { position2: [-1.0, 1.0] },
        Vertex2 { position2: [-1.0,  1.0] }, Vertex2 { position2: [1.0,  1.0] },
        Vertex2 { position2: [-1.0, -1.0] }, Vertex2 { position2: [1.0, -1.0] },
    ]);

    let indices = glium::index::TrianglesList(vec![2u16, 3, 4]);
    let indices = glium::IndexBuffer::new(&display, indices);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw((vb1.slice(1 .. 4).unwrap(), vb2.slice(3 .. 6).unwrap()), &indices,
                &program, &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8)>> = texture.read();
    assert_eq!(data.last().unwrap()[0], (0, 0, 0));
    assert_eq!(data[0].last().unwrap(), &(255, 0, 0));

    display.assert_no_error();
}
