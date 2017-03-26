#[macro_use]
extern crate glium;

use glium::Surface;
use glium::index::PrimitiveType;

mod support;

#[test]
fn color_mask() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        color_mask: (false, true, true, true),
        .. Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(0, 0, 0, 255));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn viewport_too_large() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        viewport: Some(glium::Rect {
            left: 0,
            bottom: 0,
            width: 4294967295,
            height: 4294967295,
        }),
        .. Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let mut frame = display.draw();
    match frame.draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params) {
        Err(glium::DrawError::ViewportTooLarge) => (),
        a => panic!("{:?}", a)
    };
    frame.finish().unwrap();

    display.assert_no_error(None);
}

#[test]
fn wrong_depth_range() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        depth: glium::Depth {
            range: (-0.1, 1.0),
            .. Default::default()
        },
        .. Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let mut frame = display.draw();
    match frame.draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params) {
        Err(glium::DrawError::InvalidDepthRange) => (),
        a => panic!("{:?}", a)
    };
    frame.finish().unwrap();

    display.assert_no_error(None);
}

#[test]
fn scissor() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        scissor: Some(glium::Rect {
            left: 0,
            bottom: 0,
            width: 1,
            height: 1,
        }),
        .. Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    assert_eq!(data[0][0], (255, 0, 0, 255));
    assert_eq!(data[1][0], (0, 0, 0, 0));
    assert_eq!(data[0][1], (0, 0, 0, 0));
    assert_eq!(data[1][1], (0, 0, 0, 0));

    for row in data.iter().skip(1) {
        for pixel in row.iter().skip(1) {
            assert_eq!(pixel, &(0, 0, 0, 0));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn scissor_followed_by_clear() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        scissor: Some(glium::Rect {
            left: 2,
            bottom: 2,
            width: 2,
            height: 2,
        }),
        .. Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms,
                              &params).unwrap();
    texture.as_surface().clear_color(1.0, 0.0, 1.0, 1.0);

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 0, 255, 255));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn viewport_followed_by_clear() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        viewport: Some(glium::Rect {
            left: 2,
            bottom: 2,
            width: 2,
            height: 2,
        }),
        .. Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms,
                              &params).unwrap();
    texture.as_surface().clear_color(1.0, 0.0, 1.0, 1.0);

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 0, 255, 255));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn viewport() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        viewport: Some(glium::Rect {
            left: 0,
            bottom: 0,
            width: 1,
            height: 1,
        }),
        .. Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    assert_eq!(data[0][0], (255, 0, 0, 255));
    assert_eq!(data[1][0], (0, 0, 0, 0));
    assert_eq!(data[0][1], (0, 0, 0, 0));
    assert_eq!(data[1][1], (0, 0, 0, 0));

    for row in data.iter().skip(1) {
        for pixel in row.iter().skip(1) {
            assert_eq!(pixel, &(0, 0, 0, 0));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn dont_draw_primitives() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        draw_primitives: false,
        .. Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 1.0, 0.0, 0.0);
    match texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params) {
        Ok(_) => (),
        Err(glium::DrawError::RasterizerDiscardNotSupported) => return,
        e => e.unwrap()
    }

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(0, 255, 0, 0));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn dont_draw_primitives_then_draw() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        draw_primitives: false,
        .. Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 1.0, 0.0, 0.0);
    match texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params) {
        Ok(_) => (),
        Err(glium::DrawError::RasterizerDiscardNotSupported) => return,
        e => e.unwrap()
    }
    texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 0, 0, 255));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn cull_clockwise() {
    let display = support::build_display();

    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
        }

        implement_vertex!(Vertex, position);

        glium::VertexBuffer::new(&display, &[
            Vertex { position: [-1.0,  1.0] },      // top-left
            Vertex { position: [ 1.0,  1.0] },      // top-right
            Vertex { position: [-1.0, -1.0] },      // bottom-left
            Vertex { position: [ 1.0, -1.0] }       // bottom-right
        ]).unwrap()
    };

    // first triangle covers the top-left side of the screen and is clockwise
    // second triangle covers the bottom-right side of the screen and is ccw
    let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                               &[0u16, 1, 2, 1, 2, 3]).unwrap();

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
            "
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
            "
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
            "
        },
    ).unwrap();

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vertex_buffer, &index_buffer, &program, &glium::uniforms::EmptyUniforms,
        &glium::DrawParameters {
            backface_culling: glium::BackfaceCullingMode::CullClockwise,
            .. Default::default()
        }).unwrap();

    let read_back: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    assert_eq!(read_back[0].last().unwrap(), &(255, 0, 0, 255));
    assert_eq!(read_back.last().unwrap()[0], (0, 0, 0, 0));

    display.assert_no_error(None);
}

#[test]
fn cull_counterclockwise() {
    let display = support::build_display();

    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
        }

        implement_vertex!(Vertex, position);

        glium::VertexBuffer::new(&display, &[
            Vertex { position: [-1.0,  1.0] },      // top-left
            Vertex { position: [ 1.0,  1.0] },      // top-right
            Vertex { position: [-1.0, -1.0] },      // bottom-left
            Vertex { position: [ 1.0, -1.0] }       // bottom-right
        ]).unwrap()
    };

    // first triangle covers the top-left side of the screen and is clockwise
    // second triangle covers the bottom-right side of the screen and is ccw
    let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                               &[0u16, 1, 2, 1, 2, 3]).unwrap();

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
            "
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
            "
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
            "
        },
    ).unwrap();

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vertex_buffer, &index_buffer, &program, &glium::uniforms::EmptyUniforms,
        &glium::DrawParameters {
            backface_culling: glium::BackfaceCullingMode::CullCounterClockwise,
            .. Default::default()
        }).unwrap();

    let read_back: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    assert_eq!(read_back[0].last().unwrap(), &(0, 0, 0, 0));
    assert_eq!(read_back.last().unwrap()[0], (255, 0, 0, 255));

    display.assert_no_error(None);
}

#[test]
fn cull_clockwise_trianglestrip() {
    let display = support::build_display();

    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
        }

        implement_vertex!(Vertex, position);

        glium::VertexBuffer::new(&display, &[
            Vertex { position: [-1.0,  1.0] },      // top-left
            Vertex { position: [ 1.0,  1.0] },      // top-right
            Vertex { position: [-1.0, -1.0] },      // bottom-left
            Vertex { position: [ 1.0, -1.0] }       // bottom-right
        ]).unwrap()
    };

    // both triangles are clockwise
    let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TriangleStrip,
                                               &[0u16, 1, 2, 3]).unwrap();

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
            "
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
            "
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
            "
        },
    ).unwrap();

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vertex_buffer, &index_buffer, &program, &glium::uniforms::EmptyUniforms,
        &glium::DrawParameters {
            backface_culling: glium::BackfaceCullingMode::CullClockwise,
            .. Default::default()
        }).unwrap();

    let read_back: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    assert_eq!(read_back[0][0], (0, 0, 0, 0));
    assert_eq!(read_back.last().unwrap().last().unwrap(), &(0, 0, 0, 0));

    display.assert_no_error(None);
}

#[test]
fn cull_counterclockwise_trianglestrip() {
    let display = support::build_display();

    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
        }

        implement_vertex!(Vertex, position);

        glium::VertexBuffer::new(&display, &[
            Vertex { position: [-1.0,  1.0] },      // top-left
            Vertex { position: [ 1.0,  1.0] },      // top-right
            Vertex { position: [-1.0, -1.0] },      // bottom-left
            Vertex { position: [ 1.0, -1.0] }       // bottom-right
        ]).unwrap()
    };

    // both triangles are clockwise
    let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TriangleStrip,
                                               &[0u16, 1, 2, 3]).unwrap();

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
            "
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
            "
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
            "
        },
    ).unwrap();

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vertex_buffer, &index_buffer, &program, &glium::uniforms::EmptyUniforms,
        &glium::DrawParameters {
            backface_culling: glium::BackfaceCullingMode::CullCounterClockwise,
            .. Default::default()
        }).unwrap();

    let read_back: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    assert_eq!(read_back[0][0], (255, 0, 0, 255));
    assert_eq!(read_back.last().unwrap().last().unwrap(), &(255, 0, 0, 255));

    display.assert_no_error(None);
}

macro_rules! blending_test {
    ($name:ident, $func:expr, $source:expr, $dest:expr, $result:expr) => (
        #[test]
        fn $name() {
            let display = support::build_display();

            let params = glium::DrawParameters {
                blend: glium::Blend {
                    color: $func,
                    alpha: $func,
                    constant_value: (1.0, 1.0, 1.0, 1.0)
                },
                .. Default::default()
            };

            let (vb, ib) = support::build_rectangle_vb_ib(&display);

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
                    "
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
                    "
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
                    "
                },
            ).unwrap();

            let texture = support::build_renderable_texture(&display);
            texture.as_surface().clear(None, Some($source), false, None, None);
            texture.as_surface().draw(&vb, &ib, &program, &uniform!{ color: $dest },
                                      &params).unwrap();

            let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
            for row in data.iter() {
                for pixel in row.iter() {
                    assert_eq!(pixel, &$result);
                }
            }

            display.assert_no_error(None);
        }
    )
}


blending_test!(min_blending, glium::BlendingFunction::Min,
               (0.0, 0.2, 0.3, 0.0), (1.0, 0.0, 0.0, 1.0), (0, 0, 0, 0));

blending_test!(max_blending, glium::BlendingFunction::Max,
               (0.4, 1.0, 1.0, 0.2), (1.0, 0.0, 0.0, 1.0), (255, 255, 255, 255));

blending_test!(one_plus_one, glium::BlendingFunction::Addition {
                   source: glium::LinearBlendingFactor::One,
                   destination: glium::LinearBlendingFactor::One,
               },
               (0.0, 1.0, 1.0, 0.0), (1.0, 0.0, 0.0, 1.0), (255, 255, 255, 255));


#[test]
fn provoking_vertex_last() {
    let display = support::build_display();

    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            value: (f32, f32),
        }

        implement_vertex!(Vertex, value);

        glium::VertexBuffer::new(&display, &[
            Vertex { value: (-1.0, 1.0) },
            Vertex { value: (1.0, -1.0) },
            Vertex { value: (-1.0, -1.0) },
        ]).unwrap()
    };

    let program = glium::Program::from_source(&display, "
            #version 100

            attribute lowp vec2 value;
            varying lowp flat float v_value;

            void main() {
                v_value = value.y;
                gl_Position = vec4(value, 0.0, 1.0);
            }
        ",
        "
            #version 100

            varying lowp flat float v_value;

            void main() {
                gl_FragColor = vec4(1.0, (v_value + 1.0) * 0.5, 0.0, 1.0);
            }
        ", None);
    let program = match program {
        Err(_) => return,
        Ok(p) => p
    };

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vertex_buffer,
        &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList), &program,
        &glium::uniforms::EmptyUniforms,
        &glium::DrawParameters {
            provoking_vertex: glium::draw_parameters::ProvokingVertex::LastVertex,
            .. Default::default()
        }).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    // only the bottom-left half of the screen is filled
    for row in data.iter().take(256) {
        for pixel in row.iter().take(256) {
            assert_eq!(pixel, &(255, 0, 0, 255));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn provoking_vertex_first() {
    let display = support::build_display();

    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            value: (f32, f32),
        }

        implement_vertex!(Vertex, value);

        glium::VertexBuffer::new(&display, &[
            Vertex { value: (-1.0, 1.0) },
            Vertex { value: (1.0, -1.0) },
            Vertex { value: (-1.0, -1.0) },
        ]).unwrap()
    };

    let program = glium::Program::from_source(&display, "
            #version 100

            attribute lowp vec2 value;
            varying lowp flat float v_value;

            void main() {
                v_value = value.y;
                gl_Position = vec4(value, 0.0, 1.0);
            }
        ",
        "
            #version 100

            varying lowp flat float v_value;

            void main() {
                gl_FragColor = vec4(1.0, (v_value + 1.0) * 0.5, 0.0, 1.0);
            }
        ", None);
    let program = match program {
        Err(_) => return,
        Ok(p) => p
    };

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    let res = texture.as_surface().draw(&vertex_buffer,
        &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList), &program,
        &glium::uniforms::EmptyUniforms,
        &glium::DrawParameters {
            provoking_vertex: glium::draw_parameters::ProvokingVertex::FirstVertex,
            .. Default::default()
        });

    match res {
        Ok(_) => (),
        Err(glium::DrawError::ProvokingVertexNotSupported) => {
            display.assert_no_error(None);
            return;
        },
        e => e.unwrap(),
    }

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    // only the bottom-left half of the screen is filled
    for row in data.iter().take(256) {
        for pixel in row.iter().take(256) {
            assert_eq!(pixel, &(255, 255, 0, 255));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn depth_clamp_all() {
    let display = support::build_display();

    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: (f32, f32, f32),
        }

        implement_vertex!(Vertex, position);

        glium::VertexBuffer::new(&display, &[
            Vertex { position: (-1.0, 1.0, -3.0) },
            Vertex { position: (1.0, 1.0, 3.0) },
            Vertex { position: (-1.0, -1.0, -3.0) },
            Vertex { position: (1.0, -1.0, 3.0) },
        ]).unwrap()
    };

    let program = program!(&display,
        140 => {
            vertex: "
                #version 140

                in vec3 position;

                void main() {
                    gl_Position = vec4(position, 1.0);
                }
            ",
            fragment: "
                #version 140

                out vec4 color;
                void main() {
                    color = vec4(1.0, 0.0, 0.0, 1.0);
                }
            "
        },
        110 => {
            vertex: "
                #version 110

                attribute vec3 position;

                void main() {
                    gl_Position = vec4(position, 1.0);
                }
            ",
            fragment: "
                #version 110

                void main() {
                    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
                }
            "
        },
        100 => {
            vertex: "
                #version 100

                attribute lowp vec3 position;

                void main() {
                    gl_Position = vec4(position, 1.0);
                }
            ",
            fragment: "
                #version 100

                void main() {
                    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
                }
            "
        },
    ).unwrap();

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    let res = texture.as_surface().draw(&vertex_buffer,
        &glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip), &program,
        &glium::uniforms::EmptyUniforms,
        &glium::DrawParameters {
            depth: glium::Depth {
                clamp: glium::draw_parameters::DepthClamp::Clamp,
                .. Default::default()
            },
            .. Default::default()
        });

    match res {
        Ok(_) => (),
        Err(glium::DrawError::DepthClampNotSupported) => {
            display.assert_no_error(None);
            return;
        },
        e => e.unwrap(),
    }

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 0, 0, 255));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn depth_clamp_near() {
    let display = support::build_display();

    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: (f32, f32, f32),
        }

        implement_vertex!(Vertex, position);

        glium::VertexBuffer::new(&display, &[
            Vertex { position: (-1.0, 1.0, -3.0) },
            Vertex { position: (1.0, 1.0, 3.0) },
            Vertex { position: (-1.0, -1.0, -3.0) },
            Vertex { position: (1.0, -1.0, 3.0) },
        ]).unwrap()
    };

    let program = program!(&display,
        140 => {
            vertex: "
                #version 140

                in vec3 position;

                void main() {
                    gl_Position = vec4(position, 1.0);
                }
            ",
            fragment: "
                #version 140

                out vec4 color;
                void main() {
                    color = vec4(1.0, 0.0, 0.0, 1.0);
                }
            "
        },
        110 => {
            vertex: "
                #version 110

                attribute vec3 position;

                void main() {
                    gl_Position = vec4(position, 1.0);
                }
            ",
            fragment: "
                #version 110

                void main() {
                    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
                }
            "
        },
        100 => {
            vertex: "
                #version 100

                attribute lowp vec3 position;

                void main() {
                    gl_Position = vec4(position, 1.0);
                }
            ",
            fragment: "
                #version 100

                void main() {
                    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
                }
            "
        },
    ).unwrap();

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    let res = texture.as_surface().draw(&vertex_buffer,
        &glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip), &program,
        &glium::uniforms::EmptyUniforms,
        &glium::DrawParameters {
            depth: glium::Depth {
                clamp: glium::draw_parameters::DepthClamp::ClampNear,
                .. Default::default()
            },
            .. Default::default()
        });

    match res {
        Ok(_) => (),
        Err(glium::DrawError::DepthClampNotSupported) => {
            display.assert_no_error(None);
            return;
        },
        e => e.unwrap(),
    }

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    // the left part of the texture is red
    for row in data.iter() {
        for pixel in row.iter().take(texture.get_width() as usize / 2) {
            assert_eq!(pixel, &(255, 0, 0, 255));
        }
    }

    // the right part of the texture is black
    for row in data.iter() {
        for pixel in row.iter().skip(3 * texture.get_width() as usize / 4) {
            assert_eq!(pixel, &(0, 0, 0, 0));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn depth_clamp_far() {
    let display = support::build_display();

    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: (f32, f32, f32),
        }

        implement_vertex!(Vertex, position);

        glium::VertexBuffer::new(&display, &[
            Vertex { position: (-1.0, 1.0, -3.0) },
            Vertex { position: (1.0, 1.0, 3.0) },
            Vertex { position: (-1.0, -1.0, -3.0) },
            Vertex { position: (1.0, -1.0, 3.0) },
        ]).unwrap()
    };

    let program = program!(&display,
        140 => {
            vertex: "
                #version 140

                in vec3 position;

                void main() {
                    gl_Position = vec4(position, 1.0);
                }
            ",
            fragment: "
                #version 140

                out vec4 color;
                void main() {
                    color = vec4(1.0, 0.0, 0.0, 1.0);
                }
            "
        },
        110 => {
            vertex: "
                #version 110

                attribute vec3 position;

                void main() {
                    gl_Position = vec4(position, 1.0);
                }
            ",
            fragment: "
                #version 110

                void main() {
                    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
                }
            "
        },
        100 => {
            vertex: "
                #version 100

                attribute lowp vec3 position;

                void main() {
                    gl_Position = vec4(position, 1.0);
                }
            ",
            fragment: "
                #version 100

                void main() {
                    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
                }
            "
        },
    ).unwrap();

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    let res = texture.as_surface().draw(&vertex_buffer,
        &glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip), &program,
        &glium::uniforms::EmptyUniforms,
        &glium::DrawParameters {
            depth: glium::Depth {
                clamp: glium::draw_parameters::DepthClamp::ClampFar,
                .. Default::default()
            },
            .. Default::default()
        });

    match res {
        Ok(_) => (),
        Err(glium::DrawError::DepthClampNotSupported) => {
            display.assert_no_error(None);
            return;
        },
        e => e.unwrap(),
    }

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    // the left part of the texture is black
    for row in data.iter() {
        for pixel in row.iter().take(texture.get_width() as usize / 4) {
            assert_eq!(pixel, &(0, 0, 0, 0));
        }
    }

    // the right part of the texture is red
    for row in data.iter() {
        for pixel in row.iter().skip(texture.get_width() as usize / 2) {
            assert_eq!(pixel, &(255, 0, 0, 255));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn primitive_bounding_box() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        primitive_bounding_box: (0.0 .. 1.0, -0.2 .. 0.3, 0.0 .. 1.0, -1.0 .. 1.0),
        .. Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);
    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface()
           .draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params).unwrap();

    display.assert_no_error(None);
}

#[test]
fn primitive_restart_index() {

    let display = support::build_display();

    // make two horizontal lines
    // on failure, they will connect like a z
    // on success, they will connect like a =

    let vertex_buffer = {

        #[derive(Copy, Clone)]
        struct Vertex {
            position: (f32, f32, f32),
        }

        implement_vertex!(Vertex, position);

        glium::VertexBuffer::new(&display, &[
            // lower line
            Vertex { position: (-0.5, -0.5, 0.0) },
            Vertex { position: ( 0.5, -0.5, 0.0) },
            // upper line
            Vertex { position: (-0.5,  0.5, 0.0) },
            Vertex { position: ( 0.5,  0.5, 0.0) },
        ]).unwrap()
    };

    let index_buffer = glium::IndexBuffer::<u8>::new(&display, glium::index::PrimitiveType::LineStrip, 
                                                     &[0, 1, 255, 2, 3]).unwrap();
    let program = program!(&display,
        140 => {
            vertex: "
                #version 140

                in vec3 position;

                void main() {
                    gl_Position = vec4(position, 1.0);
                }
            ",
            fragment: "
                #version 140

                out vec4 color;
                void main() {
                    color = vec4(1.0, 0.0, 0.0, 1.0);
                }
            "
        },
        110 => {
            vertex: "
                #version 110

                attribute vec3 position;

                void main() {
                    gl_Position = vec4(position, 1.0);
                }
            ",
            fragment: "
                #version 110

                void main() {
                    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
                }
            "
        },
        100 => {
            vertex: "
                #version 100

                attribute lowp vec3 position;

                void main() {
                    gl_Position = vec4(position, 1.0);
                }
            ",
            fragment: "
                #version 100

                void main() {
                    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
                }
            "
        },
    ).unwrap();

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(1.0, 1.0, 1.0, 1.0);

    let options = glium::DrawParameters {
           primitive_restart_index: true,
        .. Default::default()
    };

    let res = texture.as_surface().draw(&vertex_buffer,
        &index_buffer, &program,
        &glium::uniforms::EmptyUniforms,
        &options);

    match res {
        Ok(_) => (),
        Err(glium::DrawError::DepthClampNotSupported) => {
            display.assert_no_error(None);
            return;
        },
        e => e.unwrap(),
    }

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    let tex_w = texture.get_width() as usize;
    let tex_h = texture.get_height().unwrap() as usize;

    // midpoint of the texture should be white
    let mid_x = tex_w / 2;
    let mid_y = tex_h / 2;

    // sometimes the sampling can be a few pixels off
    for row in (mid_y - 2)..(mid_y + 2) {
        for pixel in (mid_x - 2)..(mid_x + 2){
            assert_eq!(data[row][pixel], (255, 255, 255, 255));
        }
    }

    display.assert_no_error(None);
}