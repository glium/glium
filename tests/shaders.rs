//! This file is not named `program.rs`, because executables that contain the string `program`
//! are treated in a special way by Windows.

#[macro_use]
extern crate glium;

use glium::Surface;

mod support;

#[test]
fn program_creation() {
    let display = support::build_display();

    // compiling shaders and linking them together
    glium::Program::from_source(&display,
        // vertex shader
        "
            #version 110

            uniform mat4 matrix;

            attribute vec2 position;
            attribute vec3 color;

            varying vec3 vColor;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0) * matrix;
                vColor = color;
            }
        ",

        // fragment shader
        "
            #version 110
            varying vec3 vColor;

            void main() {
                gl_FragColor = vec4(vColor, 1.0);
            }
        ",

        // geometry shader
        None).unwrap();

    display.assert_no_error(None);
}

#[test]
fn program_compilation_error() {
    let display = support::build_display();

    // compiling shaders and linking them together
    let program = glium::Program::from_source(&display,
        // vertex shader
        "invalid glsl code",

        // fragment shader
        "
            #version 110
            varying vec3 vColor;

            void main() {
                gl_FragColor = vec4(vColor, 1.0);
            }
        ",

        // geometry shader
        None);

    match program {
        Err(glium::CompilationError(..)) => (),
        _ => panic!()
    };

    display.assert_no_error(None);
}

// This test is disabled because some OpenGL drivers don't catch
// the linking error (even though they are supposed to)
#[test]
#[ignore]
fn program_linking_error() {
    let display = support::build_display();

    // compiling shaders and linking them together
    let program = glium::Program::from_source(&display,
        // vertex shader
        "
            #version 110

            varying vec3 output1;

            void main() {
                gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
                output1 = vec3(0.0, 0.0, 0.0);
            }
        ",

        // fragment shader
        "
            #version 110
            varying vec3 output2;

            void main() {
                gl_FragColor = vec4(output2, 1.0);
            }
        ",

        // geometry shader
        None);

    match program {
        Err(glium::LinkingError(_)) => (),
        _ => panic!()
    };

    display.assert_no_error(None);
}

#[test]
fn get_frag_data_location() {
    let display = support::build_display();

    let program = glium::Program::from_source(&display,
        "
            #version 110

            void main() {
                gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
            }
        ",
        "
            #version 130

            out vec4 color;

            void main() {
                color = vec4(1.0, 1.0, 1.0, 1.0);
            }
        ",
        None);

    // ignoring test in case of compilation error (version 1.30 may not be supported)
    let program = match program {
        Ok(p) => p,
        Err(_) => return
    };

    assert!(program.get_frag_data_location("color").is_some());
    assert!(program.get_frag_data_location("unexisting").is_none());

    display.assert_no_error(None);
}

#[test]
fn get_uniform_blocks() {
    let display = support::build_display();

    let program = glium::Program::from_source(&display,
        "
            #version 330

            uniform MyBlock {
                vec3 position;
                float color[12];
            };

            void main() {
                gl_Position = vec4(position, color[2]);
            }
        ",
        "
            #version 130

            out vec4 color;

            void main() {
                color = vec4(1.0, 1.0, 1.0, 1.0);
            }
        ",
        None);

    // ignoring test in case of compilation error (version may not be supported)
    let program = match program {
        Ok(p) => p,
        Err(_) => return
    };

    let blocks = program.get_uniform_blocks();

    assert_eq!(blocks.len(), 1);
    assert!(blocks.get("MyBlock").is_some());

    let my_block = blocks.get("MyBlock").unwrap();
    assert!(my_block.size >= 3 * 4 + 4 * 12);

    if let glium::program::BlockLayout::Struct { ref members } = my_block.layout {
        assert_eq!(members.len(), 2);

        let mut members = members.clone();
        members.sort_by(|a, b| a.0.cmp(&b.0));

        assert_eq!(members[1].0, "position");
        if let glium::program::BlockLayout::BasicType { ty, offset_in_buffer } = members[1].1 {
            assert_eq!(ty, glium::uniforms::UniformType::FloatVec3);
            assert_eq!(offset_in_buffer, 0);
        } else {
            panic!();
        }

        assert_eq!(members[0].0, "color");
        if let glium::program::BlockLayout::Array { ref content, length } = members[0].1 {
            assert_eq!(length, 12);
            if let glium::program::BlockLayout::BasicType { ty, offset_in_buffer } = **content {
                assert_eq!(ty, glium::uniforms::UniformType::Float);
                assert!(offset_in_buffer >= 4 * 3);
            } else {
                panic!();
            }
        } else {
            panic!();
        }

    } else {
        panic!();
    }

    display.assert_no_error(None);
}

#[test]
fn get_program_binary() {
    let display = support::build_display();

    let program = glium::Program::from_source(&display,
        "
            #version 110

            uniform mat4 matrix;

            attribute vec2 position;
            attribute vec3 color;

            varying vec3 vColor;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0) * matrix;
                vColor = color;
            }
        ",
        "
            #version 110
            varying vec3 vColor;

            void main() {
                gl_FragColor = vec4(vColor, 1.0);
            }
        ",
        None).unwrap();

    let binary = match program.get_binary() {
        Err(_) => return,
        Ok(bin) => bin
    };

    assert!(binary.content.len() >= 1);

    display.assert_no_error(None);
}

#[test]
fn program_binary_reload() {
    let display = support::build_display();

    let program = glium::Program::from_source(&display,
        "
            #version 110

            uniform mat4 matrix;

            attribute vec2 position;
            attribute vec3 color;

            varying vec3 vColor;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0) * matrix;
                vColor = color;
            }
        ",
        "
            #version 110
            varying vec3 vColor;

            void main() {
                gl_FragColor = vec4(vColor, 1.0);
            }
        ",
        None).unwrap();

    let binary = match program.get_binary() {
        Err(_) => return,
        Ok(bin) => bin
    };

    let _program2 = glium::Program::new(&display, binary).unwrap();

    display.assert_no_error(None);
}

#[test]
fn program_binary_working() {
    let display = support::build_display();
    let (vb, ib) = support::build_rectangle_vb_ib(&display);

    let program_src = glium::Program::from_source(&display,
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

    let binary = match program_src.get_binary() {
        Err(_) => return,
        Ok(bin) => bin
    };

    let program = glium::Program::new(&display, binary).unwrap();

    let output = support::build_renderable_texture(&display);
    output.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    output.as_surface().draw(&vb, &ib, &program, &uniform!{}, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = output.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 0, 0, 255));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn get_transform_feedback_varyings() {
    let display = support::build_display();

    let source = glium::program::ProgramCreationInput::SourceCode {
        tessellation_control_shader: None,
        tessellation_evaluation_shader: None,
        geometry_shader: None,
        outputs_srgb: false,
        uses_point_size: false,

        vertex_shader: "
            #version 110

            varying vec2 normal;
            varying float color;

            void main() {
                normal = vec2(0.0, 0.0);
                color = 5.0;

                gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
            }
        ",
        fragment_shader: "
            #version 130

            out vec4 color;

            void main() {
                color = vec4(1.0, 1.0, 1.0, 1.0);
            }
        ",

        transform_feedback_varyings: Some((
            vec!["normal".to_string(), "color".to_string()],
            glium::program::TransformFeedbackMode::Separate
        )),
    };

    let program = match glium::Program::new(&display, source) {
        Ok(p) => p,
        Err(glium::program::ProgramCreationError::TransformFeedbackNotSupported) => return,
        Err(e) => panic!("{:?}", e)
    };

    assert_eq!(program.get_transform_feedback_buffers()[0],
                glium::program::TransformFeedbackBuffer {
                    id: 0,
                    stride: 2 * 4,
                    elements: vec![glium::program::TransformFeedbackVarying {
                        name: "normal".to_string(),
                        offset: 0,
                        size: 2 * 4,
                        ty: glium::vertex::AttributeType::F32F32,
                    }]
                });

    assert_eq!(program.get_transform_feedback_buffers()[1],
                glium::program::TransformFeedbackBuffer {
                    id: 1,
                    stride: 4,
                    elements: vec![glium::program::TransformFeedbackVarying {
                        name: "color".to_string(),
                        offset: 0,
                        size: 4,
                        ty: glium::vertex::AttributeType::F32,
                    }]
                });

    assert_eq!(program.get_transform_feedback_buffers().len(), 2);

    display.assert_no_error(None);
}

#[test]
fn get_output_primitives_simple() {
    let display = support::build_display();

    let program = glium::Program::from_source(&display,
        "
            #version 110

            void main() {
                gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
            }
        ",
        "
            #version 110

            void main() {
                gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
            }
        ",
        None);

    // ignoring test in case of compilation error
    let program = match program {
        Ok(p) => p,
        Err(_) => return
    };

    assert!(program.get_output_primitives().is_none());
    display.assert_no_error(None);
}

// TODO: add tests for get_output_primitives with geometry shader, TES, and both

#[test]
fn ssbos() {
    let display = support::build_display();

    let program = glium::Program::from_source(&display,
        "
            #version 430

            buffer MyBuffer {
                vec3 position;
                float color[12];
            };

            void main() {
                gl_Position = vec4(position, color[2]);
            }
        ",
        "
            #version 130

            out vec4 color;

            void main() {
                color = vec4(1.0, 1.0, 1.0, 1.0);
            }
        ",
        None);

    // ignoring test in case of compilation error (version may not be supported)
    let program = match program {
        Ok(p) => p,
        Err(_) => return
    };

    let blocks = program.get_shader_storage_blocks();

    assert_eq!(blocks.len(), 1);
    assert!(blocks.get("MyBuffer").is_some());

    let my_block = blocks.get("MyBuffer").unwrap();
    assert!(my_block.size >= 3 * 4 + 4 * 12);

    if let glium::program::BlockLayout::Struct { ref members } = my_block.layout {
        assert_eq!(members.len(), 2);

        let mut members = members.clone();
        members.sort_by(|a, b| a.0.cmp(&b.0));

        assert_eq!(members[1].0, "position");
        if let glium::program::BlockLayout::BasicType { ty, offset_in_buffer } = members[1].1 {
            assert_eq!(ty, glium::uniforms::UniformType::FloatVec3);
            assert_eq!(offset_in_buffer, 0);
        } else {
            panic!();
        }

        assert_eq!(members[0].0, "color");
        if let glium::program::BlockLayout::Array { ref content, length } = members[0].1 {
            assert_eq!(length, 12);
            if let glium::program::BlockLayout::BasicType { ty, offset_in_buffer } = **content {
                assert_eq!(ty, glium::uniforms::UniformType::Float);
                assert!(offset_in_buffer >= 4 * 3);
            } else {
                panic!();
            }
        } else {
            panic!();
        }

    } else {
        panic!();
    }

    display.assert_no_error(None);
}

#[test]
fn complex_layout() {
    let display = support::build_display();

    let program = glium::Program::from_source(&display,
        "
            #version 330
            uniform layout(std140);

            struct Foo {
                ivec3 a[3];
                int b;
            };

            uniform MyBlock {
                vec3 position;
                float color[12];
                Foo foo[2];
                Foo bar;
            };

            void main() {
                gl_Position = vec4(position, 1.0);
            }
        ",
        "
            #version 130

            out vec4 color;

            void main() {
                color = vec4(1.0, 1.0, 1.0, 1.0);
            }
        ",
        None);

    // ignoring test in case of compilation error (version may not be supported)
    let program = match program {
        Ok(p) => p,
        Err(_) => return
    };

    let blocks = program.get_uniform_blocks();

    assert_eq!(blocks.len(), 1);
    assert!(blocks.get("MyBlock").is_some());

    let my_block = blocks.get("MyBlock").unwrap();
    //assert_eq!(my_block.size, 3 * 4 + 4 * 12 + (3 * 4 + 4) * 3 + );      // TODO: sort offsets out

    if let glium::program::BlockLayout::Struct { ref members } = my_block.layout {
        assert_eq!(members.len(), 4);

        let mut members = members.clone();
        members.sort_by(|a, b| a.0.cmp(&b.0));

        assert_eq!(members[0].0, "bar");
        if let glium::program::BlockLayout::Struct { ref members } = members[0].1 {
            assert_eq!(members.len(), 2);
            let mut members = members.clone();
            members.sort_by(|a, b| a.0.cmp(&b.0));

            assert_eq!(members[0].0, "a");
            if let glium::program::BlockLayout::Array { ref content, length } = members[0].1 {
                assert_eq!(length, 3);
                if let glium::program::BlockLayout::BasicType { ty, .. } = **content {
                    assert_eq!(ty, glium::uniforms::UniformType::IntVec3);
                    //assert_eq!(offset_in_buffer, 4 * 3);      // TODO: sort offsets out
                } else {
                    panic!();
                }
            }

            assert_eq!(members[1].0, "b");
            if let glium::program::BlockLayout::BasicType { ty, .. } = members[1].1 {
                assert_eq!(ty, glium::uniforms::UniformType::Int);
                //assert_eq!(offset_in_buffer, 0);      // TODO: sort offsets out
            } else {
                panic!();
            }

        } else {
            panic!();
        }

        assert_eq!(members[1].0, "color");
        if let glium::program::BlockLayout::Array { ref content, length } = members[1].1 {
            assert_eq!(length, 12);
            if let glium::program::BlockLayout::BasicType { ty, .. } = **content {
                assert_eq!(ty, glium::uniforms::UniformType::Float);
                //assert_eq!(offset_in_buffer, 4 * 3);      // TODO: sort offsets out
            } else {
                panic!();
            }
        } else {
            panic!();
        }

        assert_eq!(members[2].0, "foo");
        if let glium::program::BlockLayout::Array { ref content, length } = members[2].1 {
            assert_eq!(length, 2);

            if let glium::program::BlockLayout::Struct { ref members } = **content {
                assert_eq!(members.len(), 2);
                let mut members = members.clone();
                members.sort_by(|a, b| a.0.cmp(&b.0));

                assert_eq!(members[0].0, "a");
                if let glium::program::BlockLayout::Array { ref content, length } = members[0].1 {
                    assert_eq!(length, 3);
                    if let glium::program::BlockLayout::BasicType { ty, .. } = **content {
                        assert_eq!(ty, glium::uniforms::UniformType::IntVec3);
                        //assert!(offset_in_buffer >= 4 * 3);       // TODO: sort offsets out
                    } else {
                        panic!();
                    }
                }

                assert_eq!(members[1].0, "b");
                if let glium::program::BlockLayout::BasicType { ty, .. } = members[1].1 {
                    assert_eq!(ty, glium::uniforms::UniformType::Int);
                    //assert_eq!(offset_in_buffer, 0);      // TODO: sort offsets out
                } else {
                    panic!();
                }

            } else {
                panic!();
            }
        } else {
            panic!();
        }

        assert_eq!(members[3].0, "position");
        if let glium::program::BlockLayout::BasicType { ty, .. } = members[3].1 {
            assert_eq!(ty, glium::uniforms::UniformType::FloatVec3);
            //assert_eq!(offset_in_buffer, 0);      // TODO: sort offsets out
        } else {
            panic!();
        }

    } else {
        panic!();
    }

    display.assert_no_error(None);
}

#[test]
fn unsized_array() {
    let display = support::build_display();

    let program = glium::Program::from_source(&display,
        "
            #version 430

            layout(std140)
            struct Foo {
                ivec3 a[3];
                int b;
            };

            buffer MyBlock {
                vec3 position;
                Foo foo[1];
                Foo bar[];
            };

            void main() {
                gl_Position = vec4(position, 1.0);
            }
        ",
        "
            #version 140

            out vec4 color;

            void main() {
                color = vec4(1.0, 1.0, 1.0, 1.0);
            }
        ",
        None);

    // ignoring test in case of compilation error (version may not be supported)
    let program = match program {
        Ok(p) => p,
        Err(_) => return
    };

    let blocks = program.get_shader_storage_blocks();

    assert_eq!(blocks.len(), 1);
    let my_block = blocks.get("MyBlock").unwrap();

    assert_eq!(my_block.layout, glium::program::BlockLayout::Struct {
        members: vec![
            ("position".to_string(), glium::program::BlockLayout::BasicType {
                ty: glium::uniforms::UniformType::FloatVec3,
                offset_in_buffer: 0,
            }),

            ("foo".to_string(), glium::program::BlockLayout::Array {
                content: Box::new(glium::program::BlockLayout::Struct {
                    members: vec![
                        ("a".to_string(), glium::program::BlockLayout::Array {
                            content: Box::new(glium::program::BlockLayout::BasicType {
                                ty: glium::uniforms::UniformType::IntVec3,
                                offset_in_buffer: 16,
                            }),
                            length: 3,
                        }),

                        ("b".to_string(), glium::program::BlockLayout::BasicType {
                            ty: glium::uniforms::UniformType::Int,
                            offset_in_buffer: 64,
                        }),
                    ],
                }),
                length: 1,
            }),

            ("bar".to_string(), glium::program::BlockLayout::DynamicSizedArray {
                content: Box::new(glium::program::BlockLayout::Struct {
                    members: vec![
                        ("a".to_string(), glium::program::BlockLayout::Array {
                            content: Box::new(glium::program::BlockLayout::BasicType {
                                ty: glium::uniforms::UniformType::IntVec3,
                                offset_in_buffer: 80,
                            }),
                            length: 3,
                        }),

                        ("b".to_string(), glium::program::BlockLayout::BasicType {
                            ty: glium::uniforms::UniformType::Int,
                            offset_in_buffer: 128,
                        }),
                    ],
                })
            }),
        ]
    });

    display.assert_no_error(None);
}

#[test]
fn array_layout_offsets() {
    let display = support::build_display();

    let program = glium::Program::from_source(&display,
        "
            #version 330

            layout(std140)
            struct Foo {
                vec2 pos;
                vec2 dir;
                float speed;
            };

            layout(std140)
            uniform MyBlock {
                Foo data[256];
            };

            void main() {
                gl_Position = vec4(data[0].pos, 0.0, 1.0);
            }
        ",
        "
            #version 140

            out vec4 color;

            void main() {
                color = vec4(1.0, 1.0, 1.0, 1.0);
            }
        ",
        None);

    // ignoring test in case of compilation error (version may not be supported)
    let program = match program {
        Ok(p) => p,
        Err(_) => return
    };

    let blocks = program.get_uniform_blocks();

    assert_eq!(blocks.len(), 1);
    let my_block = blocks.get("MyBlock").unwrap();

    assert_eq!(my_block.layout, glium::program::BlockLayout::Struct {
        members: vec![
            ("data".to_string(), glium::program::BlockLayout::Array {
                content: Box::new(glium::program::BlockLayout::Struct {
                    members: vec![
                        ("pos".to_string(), glium::program::BlockLayout::BasicType {
                            ty: glium::uniforms::UniformType::FloatVec2,
                            offset_in_buffer: 0,
                        }),

                        ("dir".to_string(), glium::program::BlockLayout::BasicType {
                            ty: glium::uniforms::UniformType::FloatVec2,
                            offset_in_buffer: 8,
                        }),

                        ("speed".to_string(), glium::program::BlockLayout::BasicType {
                            ty: glium::uniforms::UniformType::Float,
                            offset_in_buffer: 16,
                        }),
                    ],
                }),
                length: 256,
            }),
        ]
    });

    display.assert_no_error(None);
}
