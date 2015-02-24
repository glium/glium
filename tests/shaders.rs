//! This file is not named `program.rs`, because executables that contain the string `program`
//! are treated in a special way by Windows.

extern crate glutin;

#[macro_use]
extern crate glium;

use std::default::Default;
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

    display.assert_no_error();
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
        Err(glium::CompilationError(_)) => (),
        _ => panic!()
    };

    display.assert_no_error();
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
    
    display.assert_no_error();
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
    
    display.assert_no_error();
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
    assert_eq!(my_block.members.len(), 2);

    let mut members = my_block.members.clone();
    members.sort_by(|a, b| a.offset.cmp(&b.offset));

    assert_eq!(members[0].name, "position");
    assert_eq!(members[0].ty, glium::uniforms::UniformType::FloatVec3);
    assert_eq!(members[0].size, None);
    assert_eq!(members[0].offset, 0);

    //assert_eq!(members[1].name, "color");     // FIXME: "color[0]" is returned
    assert_eq!(members[1].ty, glium::uniforms::UniformType::Float);
    assert_eq!(members[1].size, Some(12));
    assert!(members[1].offset >= 4 * 3);

    display.assert_no_error();
}

#[test]
#[ignore]       // TODO: doesn't work with some versions of MESA
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

    let binary = match program.get_binary_if_supported() {
        None => return,
        Some(bin) => bin
    };

    assert!(binary.content.len() >= 1);

    display.assert_no_error();
}

#[test]
#[ignore]       // TODO: doesn't work with some versions of MESA
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

    let binary = match program.get_binary_if_supported() {
        None => return,
        Some(bin) => bin
    };

    let program2 = glium::Program::new(&display, binary).unwrap();

    display.assert_no_error();
}

#[test]
#[ignore]       // TODO: doesn't work with some versions of MESA
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

    let binary = match program_src.get_binary_if_supported() {
        None => return,
        Some(bin) => bin
    };

    let program = glium::Program::new(&display, binary).unwrap();

    let output = support::build_renderable_texture(&display);
    output.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    output.as_surface().draw(&vb, &ib, &program, &uniform!{}, &Default::default()).unwrap();

    let data: Vec<Vec<(f32, f32, f32, f32)>> = output.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(1.0, 0.0, 0.0, 1.0));
        }
    }

    display.assert_no_error();
}

#[test]
fn get_transform_feedback_varyings() {    
    let display = support::build_display();

    let source = glium::program::ProgramCreationInput::SourceCode {
        tessellation_control_shader: None,
        tessellation_evaluation_shader: None,
        geometry_shader: None,

        vertex_shader: "
            #version 110

            varying vec2 normal;
            varying int color;

            void main() {
                normal = vec2(0.0, 0.0);
                color = 5;

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
        Err(e) => panic!("{:?}")
    };

    assert_eq!(program.get_transform_feedback_varyings()[0],
                glium::program::TransformFeedbackVarying {
                    name: "normal".to_string(),
                    size: 2 * 4,
                    ty: glium::vertex::AttributeType::F32F32,
                });

    assert_eq!(program.get_transform_feedback_varyings()[1],
                glium::program::TransformFeedbackVarying {
                    name: "color".to_string(),
                    size: 4,
                    ty: glium::vertex::AttributeType::U32,
                });

    assert_eq!(program.get_transform_feedback_varyings().len(), 2);

    assert_eq!(program.get_transform_feedback_mode(),
               Some(glium::program::TransformFeedbackMode::Separate));
    
    display.assert_no_error();
}
