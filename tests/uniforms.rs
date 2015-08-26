#[macro_use]
extern crate glium;

use glium::Surface;

mod support;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);

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
    assert_eq!(data[0][0], (255, 0, 0, 128));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0, 128));

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
    assert_eq!(data[0][0], (255, 0, 0, 128));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0, 128));

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

macro_rules! uniform_test(
    ($name:ident, $glsl_ty:expr, $value:expr) => (
        #[test]
        fn $name() {
            let display = support::build_display();
            let (vb, ib) = support::build_rectangle_vb_ib(&display);
            // TODO Supply #version 400 for doubles and #version 100 for fallback (but uint needs 130 I believe)
            let program = glium::Program::from_source(&display,
                &format!("
                    #version 130
                    attribute vec2 position;
                    uniform {} my_uniform;
                    void main() {{
                        gl_Position = vec4(position, 0.0, 1.0);
                    }}
                ", $glsl_ty),
                &format!("
                    #version 130
                    uniform {} my_uniform;
                    void main() {{
                        gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
                    }}
                ", $glsl_ty),
                None).unwrap();

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
// TODO these need at least version 400 shaders to compile.
// These get ignored until double precision uniforms are implemented.
// uniform_test!(uniform_type_f64_double, "double", 12.567f64);
// uniform_test!(uniform_type_f64arr_doublevec2, "dvec2", [1.0f64, 2.4]);
// uniform_test!(uniform_type_f64tup_doublevec2, "dvec2", (1.0f64, 2.4));
// uniform_test!(uniform_type_f64arr_doublevec3, "dvec3", [1.0f64, 2.4, 0.0003]);
// uniform_test!(uniform_type_f64tup_doublevec3, "dvec3", (1.0f64, 2.4, 0.0003));
// uniform_test!(uniform_type_f64arr_doublevec4, "dvec4", [1.0f64, 2.4, 0.0003, 123456]);
// uniform_test!(uniform_type_f64tup_doublevec4, "dvec4", (1.0f64, 2.4, 0.0003, 123456));
// uniform_test!(uniform_type_f64arr_doublemat2, "dmat2", [[1.0f64, 2.4], [-2.0f64, -7.8867]]);
// uniform_test!(uniform_type_f64arr_doublemat3, "dmat3", [[ 1.0f64,     2.4, -1000000.0],
//                                                         [-2.0f64, -7.8867,     6.6666],
//                                                         [ 1.1f64,     7.7,       -6.1]]);
// uniform_test!(uniform_type_f64arr_doublemat4, "dmat4", [[ 1.0f64,     2.4, -1000000.0,  0.0],
//                                                         [-2.0f64, -7.8867,     6.6666, -0.0],
//                                                         [ 1.1f64,     7.7,       -6.1,  0.0],
//                                                         [12.0f64, 12345.0,    0.11111,  0.0]]);

// Integer
uniform_test!(uniform_type_i32_int, "int", 12i32);
uniform_test!(uniform_type_i32arr_intvec2, "ivec2", [1i32, 24]);
uniform_test!(uniform_type_i32tup_intvec2, "ivec2", (1i32, 24));
uniform_test!(uniform_type_i32arr_intvec3, "ivec3", [1i32, 24, -7]);
uniform_test!(uniform_type_i32tup_intvec3, "ivec3", (1i32, 24, -7));
uniform_test!(uniform_type_i32arr_intvec4, "ivec4", [1i32, 24, -7, -123456]);
uniform_test!(uniform_type_i32tup_intvec4, "ivec4", (1i32, 24, -7, -123456));

// Unsigned integer
uniform_test!(uniform_type_u32_uint, "uint", 12u32);
uniform_test!(uniform_type_u32arr_uintvec2, "uvec2", [1u32, 24]);
uniform_test!(uniform_type_u32tup_uintvec2, "uvec2", (1u32, 24));
uniform_test!(uniform_type_u32arr_uintvec3, "uvec3", [1u32, 24, 7]);
uniform_test!(uniform_type_u32tup_uintvec3, "uvec3", (1u32, 24, 7));
uniform_test!(uniform_type_u32arr_uintvec4, "uvec4", [1u32, 24, 7, 123456]);
uniform_test!(uniform_type_u32tup_uintvec4, "uvec4", (1u32, 24, 7, 123456));

// Booleans
uniform_test!(uniform_type_bool_bool, "bool", true);
uniform_test!(uniform_type_boolarr_boolvec2, "bvec2", [true, false]);
uniform_test!(uniform_type_booltup_boolvec2, "bvec2", (false, true));
uniform_test!(uniform_type_boolarr_boolvec3, "bvec3", [true, true, true]);
uniform_test!(uniform_type_booltup_boolvec3, "bvec3", (false, false, false));
uniform_test!(uniform_type_boolarr_boolvec4, "bvec4", [true, false, false, true]);
uniform_test!(uniform_type_booltup_boolvec4, "bvec4", (false, true, true, false));
