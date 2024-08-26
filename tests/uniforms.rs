#[macro_use]
extern crate glium;

use glium::Surface;

mod support;

// Checks the float to byte conversion conforms to the OpenGL specification
#[inline]
fn f2b_check_rha(first: (u8, u8, u8, u8), last: (u8, u8, u8, u8)) {
    assert!(
        first == (255, 0, 0, 127) || first == (255, 0, 0, 128),
        "Unexpected conversion of (1.0, 0, 0, 0.5) to {:?}",
        first,
    );
    assert_eq!(first, last);
}

#[test]
fn uniforms_storage_single_value() {
    let display = support::build_display();
    let (vb, ib) = support::build_rectangle_vb_ib(&display);

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

            uniform vec4 color;

            void main() {
                gl_FragColor = color;
            }
        ",
        None).unwrap();

    let uniforms = glium::uniforms::UniformsStorage::new("color", [1.0, 0.0, 0.0, 0.5f32]);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &uniforms, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    f2b_check_rha(data[0][0], *data.last().unwrap().last().unwrap());

    display.assert_no_error(None);
}

#[test]
fn uniforms_storage_multiple_values() {
    let display = support::build_display();
    let (vb, ib) = support::build_rectangle_vb_ib(&display);

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

            uniform vec4 color1;
            uniform vec4 color2;

            void main() {
                gl_FragColor = color1 + color2;
            }
        ",
        None).unwrap();

    let uniforms = glium::uniforms::UniformsStorage::new("color1", [0.7, 0.0, 0.0, 0.5f32]);
    let uniforms = uniforms.add("color2", [0.3, 0.0, 0.0, 0.5f32]);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &uniforms, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    assert_eq!(data[0][0], (255, 0, 0, 255));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0, 255));

    display.assert_no_error(None);
}

#[test]
fn uniforms_storage_ignore_inactive_uniforms() {
    let display = support::build_display();
    let (vb, ib) = support::build_rectangle_vb_ib(&display);

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

            uniform vec4 color;

            void main() {
                gl_FragColor = color;
            }
        ",
        None).unwrap();

    let uniforms = glium::uniforms::UniformsStorage::new("color", [1.0, 0.0, 0.0, 0.5f32]);
    let uniforms = uniforms.add("color2", 0.8f32);
    let uniforms = uniforms.add("color3", [0.1, 1.2f32]);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &uniforms, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    f2b_check_rha(data[0][0], *data.last().unwrap().last().unwrap());

    display.assert_no_error(None);
}

#[test]
fn uniform_wrong_type() {
    let display = support::build_display();
    let (vb, ib) = support::build_rectangle_vb_ib(&display);

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

            uniform vec4 color;

            void main() {
                gl_FragColor = color;
            }
        ",
        None).unwrap();

    let uniforms = glium::uniforms::UniformsStorage::new("color", 1.0f32);

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    match target.draw(&vb, &ib, &program, &uniforms, &Default::default()) {
        Err(glium::DrawError::UniformTypeMismatch { .. }) => (),
        a => panic!("{:?}", a)
    };
    target.finish().unwrap();

    display.assert_no_error(None);
}

#[test]
fn uniforms_dynamic_single_value() {
    let display = support::build_display();
    let (vb, ib) = support::build_rectangle_vb_ib(&display);

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

            uniform vec4 color;

            void main() {
                gl_FragColor = color;
            }
        ",
        None).unwrap();

    let mut uniforms = glium::uniforms::DynamicUniforms::new();
    uniforms.add("color", &[1.0, 0.0, 0.0, 0.5f32]);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &uniforms, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    f2b_check_rha(data[0][0], *data.last().unwrap().last().unwrap());

    display.assert_no_error(None);
}

#[test]
fn uniforms_dynamic_multiple_values() {
    let display = support::build_display();
    let (vb, ib) = support::build_rectangle_vb_ib(&display);

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

            uniform vec4 color1;
            uniform vec4 color2;

            void main() {
                gl_FragColor = color1 + color2;
            }
        ",
        None).unwrap();

    let mut uniforms = glium::uniforms::DynamicUniforms::new();
    uniforms.add("color1", &[0.7, 0.0, 0.0, 0.5f32]);
    uniforms.add("color2", &[0.3, 0.0, 0.0, 0.5f32]);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &uniforms, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    assert_eq!(data[0][0], (255, 0, 0, 255));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0, 255));

    display.assert_no_error(None);
}

#[test]
fn uniforms_dynamic_ignore_inactive_uniforms() {
    let display = support::build_display();
    let (vb, ib) = support::build_rectangle_vb_ib(&display);

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

            uniform vec4 color;

            void main() {
                gl_FragColor = color;
            }
        ",
        None).unwrap();

    let mut uniforms = glium::uniforms::DynamicUniforms::new();
    uniforms.add("color", &[1.0, 0.0, 0.0, 0.5f32]);
    uniforms.add("color2", &0.8f32);
    uniforms.add("color3", &[0.1, 1.2f32]);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &uniforms, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    f2b_check_rha(data[0][0], *data.last().unwrap().last().unwrap());

    display.assert_no_error(None);
}

macro_rules! uniform_test(
    ($name:ident, $glsl_ty:expr, $value:expr) => (
        #[test]
        fn $name() {
            let display = support::build_display();
            let (vb, ib) = support::build_rectangle_vb_ib(&display);

            // Definition of multiple shader versions
            let v400 = (
                &format!("
                    #version 400
                    in vec2 position;
                    uniform {} my_uniform;
                    void main() {{
                        gl_Position = vec4(position, 0.0, 1.0);
                    }}
                ", $glsl_ty),
                &format!("
                    #version 400
                    uniform {} my_uniform;
                    out vec4 color;
                    void main() {{
                        color = vec4(1.0, 1.0, 1.0, 1.0);
                    }}
                ", $glsl_ty)
            );

            let v140 = (
                &format!("
                    #version 140
                    in vec2 position;
                    uniform {} my_uniform;
                    void main() {{
                        gl_Position = vec4(position, 0.0, 1.0);
                    }}
                ", $glsl_ty),
                &format!("
                    #version 140
                    uniform {} my_uniform;
                    out vec4 color;
                    void main() {{
                        color = vec4(1.0, 1.0, 1.0, 1.0);
                    }}
                ", $glsl_ty)
            );

            let v110 = (
                &format!("
                    #version 110
                    attribute vec2 position;
                    uniform {} my_uniform;
                    void main() {{
                        gl_Position = vec4(position, 0.0, 1.0);
                    }}
                ", $glsl_ty),
                &format!("
                    #version 110
                    uniform {} my_uniform;
                    void main() {{
                        gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
                    }}
                ", $glsl_ty)
            );

            let v100 = (
                &format!("
                    #version 100
                    attribute vec2 position;
                    uniform {} my_uniform;
                    void main() {{
                        gl_Position = vec4(position, 0.0, 1.0);
                    }}
                ", $glsl_ty),
                &format!("
                    #version 100
                    uniform {} my_uniform;
                    void main() {{
                        gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
                    }}
                ", $glsl_ty)
            );

            let program = if let Ok(program) = glium::Program::from_source(&display, v400.0, v400.1, None) {
                program
            } else if let Ok(program) = glium::Program::from_source(&display, v140.0, v140.1, None) {
                program
            } else if let Ok(program) = glium::Program::from_source(&display, v110.0, v110.1, None) {
                program
            } else if let Ok(program) = glium::Program::from_source(&display, v100.0, v100.1, None) {
                program
            } else {
                return
            };

            let uniforms = glium::uniforms::UniformsStorage::new("my_uniform", $value);
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 0.0);
            target.draw(&vb, &ib, &program, &uniforms, &Default::default()).unwrap();
            target.finish().unwrap();
            display.assert_no_error(None);
        }
    )
);

// Floats
uniform_test!(uniform_type_f32_float, "float", 12.567f32);
uniform_test!(uniform_type_f32arr_floatvec2, "vec2", [1.0f32, 2.4]);
uniform_test!(uniform_type_f32tup_floatvec2, "vec2", (1.0f32, 2.4));
uniform_test!(uniform_type_f32arr_floatvec3, "vec3", [1.0f32, 2.4, 0.0003]);
uniform_test!(uniform_type_f32tup_floatvec3, "vec3", (1.0f32, 2.4, 0.0003));
uniform_test!(uniform_type_f32arr_floatvec4, "vec4", [1.0f32, 2.4, 0.0003, 123456.55]);
uniform_test!(uniform_type_f32tup_floatvec4, "vec4", (1.0f32, 2.4, 0.0003, 123456.55));
uniform_test!(uniform_type_f32arr_floatmat2, "mat2", [[1.0f32, 2.4], [-2.0f32, -7.8867]]);
uniform_test!(uniform_type_f32arr_floatmat3, "mat3", [[ 1.0f32,     2.4, -1000000.0],
                                                      [-2.0f32, -7.8867,     6.6666],
                                                      [ 1.1f32,     7.7,       -6.1]]);
uniform_test!(uniform_type_f32arr_floatmat4, "mat4", [[ 1.0f32,     2.4, -1000000.0,  0.0],
                                                      [-2.0f32, -7.8867,     6.6666, -0.0],
                                                      [ 1.1f32,     7.7,       -6.1,  0.0],
                                                      [12.0f32, 12345.0,    0.11111,  0.0]]);

// Doubles
uniform_test!(uniform_type_f64_double, "double", 12.567f64);
uniform_test!(uniform_type_f64arr_doublevec2, "dvec2", [1.0f64, 2.4]);
uniform_test!(uniform_type_f64tup_doublevec2, "dvec2", (1.0f64, 2.4));
uniform_test!(uniform_type_f64arr_doublevec3, "dvec3", [1.0f64, 2.4, 0.0003]);
uniform_test!(uniform_type_f64tup_doublevec3, "dvec3", (1.0f64, 2.4, 0.0003));
uniform_test!(uniform_type_f64arr_doublevec4, "dvec4", [1.0f64, 2.4, 0.0003, 123456.0]);
uniform_test!(uniform_type_f64tup_doublevec4, "dvec4", (1.0f64, 2.4, 0.0003, 123456.0));
uniform_test!(uniform_type_f64arr_doublemat2, "dmat2", [[1.0f64, 2.4], [-2.0f64, -7.8867]]);
uniform_test!(uniform_type_f64arr_doublemat3, "dmat3", [[ 1.0f64,     2.4, -1000000.0],
                                                        [-2.0f64, -7.8867,     6.6666],
                                                        [ 1.1f64,     7.7,       -6.1]]);
uniform_test!(uniform_type_f64arr_doublemat4, "dmat4", [[ 1.0f64,     2.4, -1000000.0,  0.0],
                                                        [-2.0f64, -7.8867,     6.6666, -0.0],
                                                        [ 1.1f64,     7.7,       -6.1,  0.0],
                                                        [12.0f64, 12345.0,    0.11111,  0.0]]);

// Integer
uniform_test!(uniform_type_i8_int, "int", 5i8);
uniform_test!(uniform_type_i16_int, "int", -1000i16);
uniform_test!(uniform_type_i32_int, "int", 12i32);
uniform_test!(uniform_type_i32arr_intvec2, "ivec2", [1i32, 24]);
uniform_test!(uniform_type_i32tup_intvec2, "ivec2", (1i32, 24));
uniform_test!(uniform_type_i32arr_intvec3, "ivec3", [1i32, 24, -7]);
uniform_test!(uniform_type_i32tup_intvec3, "ivec3", (1i32, 24, -7));
uniform_test!(uniform_type_i32arr_intvec4, "ivec4", [1i32, 24, -7, -123456]);
uniform_test!(uniform_type_i32tup_intvec4, "ivec4", (1i32, 24, -7, -123456));

// Unsigned integer
uniform_test!(uniform_type_u8_uint, "uint", 5u8);
uniform_test!(uniform_type_u16_uint, "uint", 1000u16);
uniform_test!(uniform_type_u32_uint, "uint", 12u32);
uniform_test!(uniform_type_u32arr_uintvec2, "uvec2", [1u32, 24]);
uniform_test!(uniform_type_u32tup_uintvec2, "uvec2", (1u32, 24));
uniform_test!(uniform_type_u32arr_uintvec3, "uvec3", [1u32, 24, 7]);
uniform_test!(uniform_type_u32tup_uintvec3, "uvec3", (1u32, 24, 7));
uniform_test!(uniform_type_u32arr_uintvec4, "uvec4", [1u32, 24, 7, 123456]);
uniform_test!(uniform_type_u32tup_uintvec4, "uvec4", (1u32, 24, 7, 123456));

// Integer 64 bit
uniform_test!(uniform_type_i64_int64_t, "int64_t", 9_223_372_036_854_775_807i64);
uniform_test!(uniform_type_i64arr_i64vec2, "i64vec2", [1i64, 24]);
uniform_test!(uniform_type_i64tup_i64vec2, "i64vec2", (1i64, 24));
uniform_test!(uniform_type_i64arr_i64vec3, "i64vec3", [1i64, 24, -7]);
uniform_test!(uniform_type_i64tup_i64vec3, "i64vec3", (1i64, 24, -7));
uniform_test!(uniform_type_i64arr_i64vec4, "i64vec4", [1i64, 24, -7, -123456]);
uniform_test!(uniform_type_i64tup_i64vec4, "i64vec4", (1i64, 24, -7, -123456));

// Unsigned integer 64 bit
uniform_test!(uniform_type_u64_uint64_t, "uint64_t", 9_223_372_036_854_775_807u64);
uniform_test!(uniform_type_u64arr_u64vec2, "u64vec2", [1u64, 24]);
uniform_test!(uniform_type_u64tup_u64vec2, "u64vec2", (1u64, 24));
uniform_test!(uniform_type_u64arr_u64vec3, "u64vec3", [1u64, 24, 7]);
uniform_test!(uniform_type_u64tup_u64vec3, "u64vec3", (1u64, 24, 7));
uniform_test!(uniform_type_u64arr_u64vec4, "u64vec4", [1u64, 24, 7, 123456]);
uniform_test!(uniform_type_u64tup_u64vec4, "u64vec4", (1u64, 24, 7, 123456));

// Booleans
uniform_test!(uniform_type_bool_bool, "bool", true);
uniform_test!(uniform_type_boolarr_boolvec2, "bvec2", [true, false]);
uniform_test!(uniform_type_booltup_boolvec2, "bvec2", (false, true));
uniform_test!(uniform_type_boolarr_boolvec3, "bvec3", [true, true, true]);
uniform_test!(uniform_type_booltup_boolvec3, "bvec3", (false, false, false));
uniform_test!(uniform_type_boolarr_boolvec4, "bvec4", [true, false, false, true]);
uniform_test!(uniform_type_booltup_boolvec4, "bvec4", (false, true, true, false));
