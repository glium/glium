extern crate glutin;

#[macro_use]
extern crate glium;

use std::default::Default;
use glium::Surface;

mod support;

macro_rules! texture_draw_test {
    ($test_name:ident, $tex_ty:ident, $sampler_ty:expr, $fun:expr, $coords:expr, $data:expr) => (
        #[test]
        fn $test_name() {
            let display = support::build_display();
            let (vb, ib) = support::build_rectangle_vb_ib(&display);

            let texture = glium::texture::$tex_ty::new(&display, vec![
                vec![(255, 0, 0, 255), (255, 0, 0, 255)],
                vec![(255, 0, 0, 255), (255, 0, 0, 255u8)],
            ]);

            let program = glium::Program::from_source(&display,
                "
                    #version 110

                    attribute vec2 position;

                    void main() {
                        gl_Position = vec4(position, 0.0, 1.0);
                    }
                ",
                format!("
                    #version 110

                    uniform {} texture;

                    void main() {{
                        gl_FragColor = {}(texture, {});
                    }}
                ", $sampler_ty, $fun, $coords).as_slice(),
                None).unwrap();

            let output = support::build_renderable_texture(&display);
            output.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
            output.as_surface().draw(&vb, &ib, &program, &uniform!{ texture: &texture },
                                     &Default::default()).unwrap();

            let data: Vec<Vec<(f32, f32, f32, f32)>> = output.read();
            for row in data.iter() {
                for pixel in row.iter() {
                    assert_eq!(pixel, &(1.0, 0.0, 0.0, 1.0));
                }
            }

            display.assert_no_error();
        }
    );
}

texture_draw_test!(texture_2d_draw, Texture2d, "sampler2D", "texture2D", "vec2(0.5, 0.5)",
    vec![
        vec![(255, 0, 0, 255), (255, 0, 0, 255)],
        vec![(255, 0, 0, 255), (255, 0, 0, 255u8)],
    ]);

texture_draw_test!(compressed_texture_2d_draw, CompressedTexture2d, "sampler2D", "texture2D",
    "vec2(0.5, 0.5)",
    vec![
        vec![(255, 0, 0, 255), (255, 0, 0, 255)],
        vec![(255, 0, 0, 255), (255, 0, 0, 255u8)],
    ]);
