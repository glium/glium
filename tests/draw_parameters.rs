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

    let params = glium::DrawParameters::new(&display)
                    .with_viewport(glium::Rect {
                        left: 0,
                        bottom: 0,
                        width: 4294967295,
                        height: 4294967295,
                    });

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
        depth_range: (-0.1, 1.0),
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
        Err(glium::DrawError::TransformFeedbackNotSupported) => return,
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
        Err(glium::DrawError::TransformFeedbackNotSupported) => return,
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
            backface_culling: glium::BackfaceCullingMode::CullClockWise,
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
            backface_culling: glium::BackfaceCullingMode::CullCounterClockWise,
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
            backface_culling: glium::BackfaceCullingMode::CullClockWise,
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
            backface_culling: glium::BackfaceCullingMode::CullCounterClockWise,
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
                blending_function: Some($func),
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
            texture.as_surface().clear(None, Some($source), None, None);
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
