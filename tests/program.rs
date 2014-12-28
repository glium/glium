#![feature(phase)]
#![feature(unboxed_closures)]

#[phase(plugin)]
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
