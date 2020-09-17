#[macro_use]
extern crate glium;

mod support;

// Tests for support of the cube map array type in programs.
fn program_support_for_cube_map_array(type_prefix: &str) {
    let display = support::build_display();
    let _ = glium::Program::from_source(
        &display,
        "
            #version 400

            attribute vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        ",
        &format!(
            "
                #version 400

                uniform {}samplerCubeArray cube_textures;

                void main() {{
                    gl_FragColor = texture(cube_textures, vec4(0, 0, 0, 0));
                }}
            ",
            type_prefix
        ),
        None
    ).unwrap();
}

#[test]
fn program_support_for_cube_map_array_float() {
    program_support_for_cube_map_array("");
}

#[test]
fn program_support_for_cube_map_array_unsigned() {
    program_support_for_cube_map_array("u");
}

#[test]
fn program_support_for_cube_map_array_integral() {
    program_support_for_cube_map_array("i");
}
