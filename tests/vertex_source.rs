#[macro_use]
extern crate glium;

use glium::Surface;
use glium::index::PrimitiveType;

mod support;

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
            &[
                Vertex { position: [-1.0,  1.0] },
                Vertex { position: [ 1.0,  1.0] },
                Vertex { position: [-1.0, -1.0] },
                Vertex { position: [ 1.0, -1.0] },
            ]
        ).unwrap()
    };

    let buffer2 = {
        #[derive(Copy, Clone)]
        struct Vertex {
            color: [f32; 3],
        }

        implement_vertex!(Vertex, color);

        glium::VertexBuffer::new(&display,
            &[
                Vertex { color: [1.0, 0.0, 0.0] },
                Vertex { color: [1.0, 0.0, 0.0] },
                Vertex { color: [1.0, 0.0, 0.0] },
                Vertex { color: [1.0, 0.0, 0.0] },
            ]
        ).unwrap()
    };

    let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TriangleStrip,
                                               &[0u16, 1, 2, 3]).unwrap();

    let program = program!(&display,
        110 => {
            vertex: "
                #version 110

                attribute vec2 position;
                attribute vec3 color;

                varying vec3 v_color;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                    v_color = color;
                }
            ",
            fragment: "
                #version 110
                varying vec3 v_color;

                void main() {
                    gl_FragColor = vec4(v_color, 1.0);
                }
            ",
        },
        100 => {
            vertex: "
                #version 100

                attribute lowp vec2 position;
                attribute lowp vec3 color;

                varying lowp vec3 v_color;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                    v_color = color;
                }
            ",
            fragment: "
                #version 100
                varying lowp vec3 v_color;

                void main() {
                    gl_FragColor = vec4(v_color, 1.0);
                }
            ",
        }).unwrap();

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw((&buffer1, &buffer2), &index_buffer, &program, &uniform!{},
                              &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 0, 0, 255));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn slice_draw_indices() {
    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }

    implement_vertex!(Vertex, position);

    let display = support::build_display();
    let program = program!(&display,
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
        }).unwrap();

    let vb = glium::VertexBuffer::new(&display, &[
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]).unwrap();

    let indices = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                          &[0u16, 1, 2]).unwrap();

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(vb.slice(1 .. 4).unwrap(), &indices, &program,
                &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    assert_eq!(data.last().unwrap()[0], (0, 0, 0, 0));
    assert_eq!(data[0].last().unwrap(), &(255, 0, 0, 255));

    display.assert_no_error(None);
}

#[test]
fn slice_draw_noindices() {
    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }

    implement_vertex!(Vertex, position);

    let display = support::build_display();
    let program = program!(&display,
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
        }).unwrap();

    let vb = glium::VertexBuffer::new(&display, &[
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]).unwrap();

    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(vb.slice(1 .. 4).unwrap(), &indices, &program,
                &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    assert_eq!(data.last().unwrap()[0], (0, 0, 0, 0));
    assert_eq!(data[0].last().unwrap(), &(255, 0, 0, 255));

    display.assert_no_error(None);
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
    let program = program!(&display,
        110 => {
            vertex: "
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
                attribute lowp vec2 position2;

                void main() {
                    if (position != position2) {
                        gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
                    } else {
                        gl_Position = vec4(position, 0.0, 1.0);
                    }
                }
            ",
            fragment: "
                #version 100

                void main() {
                    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
                }
            ",
        }).unwrap();

    // the 3 last elements will be drawn
    let vb1 = glium::VertexBuffer::new(&display, &[
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [-1.0, 1.0] },
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]).unwrap();

    // the 3 last elements will be drawn
    let vb2 = glium::VertexBuffer::new(&display, &[
        Vertex2 { position2: [-1.0,  1.0] }, Vertex2 { position2: [-1.0, 1.0] },
        Vertex2 { position2: [-1.0,  1.0] }, Vertex2 { position2: [-1.0, 1.0] },
        Vertex2 { position2: [-1.0,  1.0] }, Vertex2 { position2: [1.0,  1.0] },
        Vertex2 { position2: [-1.0, -1.0] }, Vertex2 { position2: [1.0, -1.0] },
    ]).unwrap();

    let indices = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                          &[2u16, 3, 4]).unwrap();

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw((vb1.slice(1 .. 4).unwrap(), vb2.slice(3 .. 6).unwrap()), &indices,
                &program, &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    assert_eq!(data.last().unwrap()[0], (0, 0, 0, 0));
    assert_eq!(data[0].last().unwrap(), &(255, 0, 0, 255));

    display.assert_no_error(None);
}

#[test]
fn attributes_marker() {
    let display = support::build_display();

    let program = match glium::Program::from_source(&display,
        "
            #version 140

            void main() {
                if (gl_VertexID == 0) {
                    gl_Position = vec4(-1.0, 1.0, 0.0, 1.0);
                } else if (gl_VertexID == 1) {
                    gl_Position = vec4(1.0, 1.0, 0.0, 1.0);
                } else if (gl_VertexID == 2) {
                    gl_Position = vec4(-1.0, -1.0, 0.0, 1.0);
                } else if (gl_VertexID == 3) {
                    gl_Position = vec4(1.0, -1.0, 0.0, 1.0);
                }
            }
        ",
        "
            #version 140

            out vec4 color;

            void main() {
                color = vec4(1.0, 0.0, 0.0, 1.0);
            }
        ",
        None) {
        Ok(p) => p,
        _ => return
    };

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(glium::vertex::EmptyVertexAttributes { len: 4 },
                              &glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip),
                              &program, &uniform!{},
                              &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 0, 0, 255));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn attributes_marker_indices() {
    let display = support::build_display();

    let program = match glium::Program::from_source(&display,
        "
            #version 140

            void main() {
                if (gl_VertexID == 0) {
                    gl_Position = vec4(-1.0, 1.0, 0.0, 1.0);
                } else if (gl_VertexID == 1) {
                    gl_Position = vec4(1.0, 1.0, 0.0, 1.0);
                } else if (gl_VertexID == 2) {
                    gl_Position = vec4(-1.0, -1.0, 0.0, 1.0);
                } else if (gl_VertexID == 3) {
                    gl_Position = vec4(1.0, -1.0, 0.0, 1.0);
                }
            }
        ",
        "
            #version 140

            out vec4 color;

            void main() {
                color = vec4(1.0, 0.0, 0.0, 1.0);
            }
        ",
        None) {
        Ok(p) => p,
        _ => return
    };

    let indices = glium::index::IndexBuffer::new(&display, PrimitiveType::TriangleStrip,
                                                 &[0u16, 1, 2, 3]).unwrap();

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(glium::vertex::EmptyVertexAttributes { len: 4 },
                              &indices, &program, &uniform!{},
                              &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 0, 0, 255));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn instancing() {
    let display = support::build_display();

    let buffer1 = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
        }

        implement_vertex!(Vertex, position);

        glium::VertexBuffer::new(&display,
            &[
                Vertex { position: [-1.0,  1.0] },
                Vertex { position: [ 1.0,  1.0] },
                Vertex { position: [-1.0, -1.0] },
                Vertex { position: [ 1.0, -1.0] },
            ]
        ).unwrap()
    };

    let buffer2 = {
        #[derive(Copy, Clone)]
        struct Vertex {
            color: [f32; 3],
        }

        implement_vertex!(Vertex, color);

        glium::vertex::VertexBuffer::new(&display,
            &[
                Vertex { color: [0.0, 0.0, 1.0] },
                Vertex { color: [0.0, 0.0, 1.0] },
                Vertex { color: [0.0, 0.0, 1.0] },
                Vertex { color: [1.0, 0.0, 0.0] },
            ]
        ).unwrap()
    };

    let buffer2 = match buffer2.per_instance() {
        Ok(b) => b,
        Err(_) => return
    };

    let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TriangleStrip,
                                               &[0u16, 1, 2, 3]).unwrap();

    let program = match glium::Program::from_source(&display,
        "
            #version 330

            in vec2 position;
            in vec3 color;

            out vec3 v_color;
            flat out int instance;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
                v_color = color;
                instance = gl_InstanceID;
            }
        ",
        "
            #version 330
            in vec3 v_color;
            flat in int instance;

            void main() {
                if (instance != 3) {
                    discard;
                }

                gl_FragColor = vec4(v_color, 1.0);
            }
        ",
        None) {
        Ok(p) => p,
        _ => return
    };

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw((&buffer1, buffer2), &index_buffer, &program, &uniform!{},
                              &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 0, 0, 255));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn per_instance_length_mismatch() {
    let display = support::build_display();

    let buffer1 = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
        }

        implement_vertex!(Vertex, position);

        glium::VertexBuffer::new(&display,
            &[
                Vertex { position: [-1.0,  1.0] },
                Vertex { position: [ 1.0,  1.0] },
                Vertex { position: [-1.0, -1.0] },
                Vertex { position: [ 1.0, -1.0] },
            ]
        ).unwrap()
    };

    let buffer2 = {
        #[derive(Copy, Clone)]
        struct Vertex {
            color: [f32; 3],
        }

        implement_vertex!(Vertex, color);

        glium::vertex::VertexBuffer::new(&display,
            &[
                Vertex { color: [0.0, 0.0, 1.0] },
                Vertex { color: [0.0, 0.0, 1.0] },
                Vertex { color: [0.0, 0.0, 1.0] },
                Vertex { color: [1.0, 0.0, 0.0] },
            ]
        ).unwrap()
    };

    let buffer2 = match buffer2.per_instance() {
        Ok(b) => b,
        Err(_) => return
    };

    let buffer3 = {
        #[derive(Copy, Clone)]
        struct Vertex {
            color: [f32; 3],
        }

        implement_vertex!(Vertex, color);

        glium::vertex::VertexBuffer::new(&display,
            &[
                Vertex { color: [0.0, 0.0, 1.0] },
                Vertex { color: [0.0, 0.0, 1.0] },
                Vertex { color: [0.0, 0.0, 1.0] },
            ]
        ).unwrap()
    };

    let buffer3 = match buffer3.per_instance() {
        Ok(b) => b,
        Err(_) => return
    };

    let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TriangleStrip,
                                               &[0u16, 1, 2, 3]).unwrap();

    let program = program!(&display,
        110 => {
            vertex: "
                #version 110

                void main() {
                    gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
                }
            ",
            fragment: "
                #version 110

                void main() {
                    gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
                }
            ",
        },
        100 => {
            vertex: "
                #version 100

                void main() {
                    gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
                }
            ",
            fragment: "
                #version 100

                void main() {
                    gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
                }
            ",
        }).unwrap();

    let mut frame = display.draw();
    match frame.draw((&buffer1, buffer2, buffer3), &index_buffer, &program, &uniform!{},
                     &Default::default())
    {
        Err(glium::DrawError::InstancesCountMismatch) => (),
        a => panic!("{:?}", a)
    }

    frame.finish().unwrap();
    display.assert_no_error(None);
}
