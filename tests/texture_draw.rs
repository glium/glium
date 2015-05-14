#[macro_use]
extern crate glium;

use glium::Surface;

mod support;

macro_rules! texture_draw_test {
    ($test_name:ident, $tex_ty:ident, [$($dims:expr),+], $glsl_ty:expr, $glsl_value:expr,
     $rust_value:expr) => (
        #[test]
        fn $test_name() {
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
                &format!("
                    #version 130

                    out {} color;

                    void main() {{
                        color = {};
                    }}
                ", $glsl_ty, $glsl_value),
                None);

            let program = match program {
                Ok(p) => p,
                Err(_) => return
            };

            let texture = glium::texture::$tex_ty::empty(&display, $($dims),+);

            texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
            texture.as_surface().draw(&vb, &ib, &program, &uniform!{ texture: &texture },
                                     &Default::default()).unwrap();

            display.assert_no_error(None);

            let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
            for row in data.iter() {
                for pixel in row.iter() {
                    assert_eq!(pixel, &$rust_value);
                }
            }

            display.assert_no_error(None);
        }
    );
}

texture_draw_test!(texture_2d_draw, Texture2d, [1024, 1024], "vec4",
                   "vec4(1.0, 0.0, 1.0, 0.0)", (255, 0, 255, 0));
