#![feature(plugin)]
#![feature(unboxed_closures)]

#[plugin]
extern crate glium_macros;

extern crate glutin;
extern crate glium;

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
