extern crate glutin;

#[macro_use]
extern crate glium;

use glium::Surface;

mod support;

macro_rules! blending_test {
    ($name:ident, $func:expr, $source:expr, $dest:expr, $result:expr) => (
        #[test]
        fn $name() {
            let display = support::build_display();

            let params = glium::DrawParameters {
                blending_function: Some($func),
                .. std::default::Default::default()
            };

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

            let texture = support::build_renderable_texture(&display);
            texture.as_surface().clear(Some($source), None, None);
            texture.as_surface().draw(&vb, &ib, &program, &uniform!{ color: $dest },
                                      &params).unwrap();

            let data: Vec<Vec<(f32, f32, f32, f32)>> = texture.read();
            for row in data.iter() {
                for pixel in row.iter() {
                    assert_eq!(pixel, &$result);
                }
            }

            display.assert_no_error();
        }
    )
}


blending_test!(min_blending, glium::BlendingFunction::Min,
               (0.0, 0.2, 0.3, 1.0), (1.0, 0.0, 0.0, 1.0), (0.0, 0.0, 0.0, 1.0));

blending_test!(max_blending, glium::BlendingFunction::Max,
               (0.4, 1.0, 1.0, 0.2), (1.0, 0.0, 0.0, 1.0), (1.0, 1.0, 1.0, 1.0));

blending_test!(one_plus_one, glium::BlendingFunction::Addition {
                   source: glium::LinearBlendingFactor::One,
                   destination: glium::LinearBlendingFactor::One,
               },
               (0.0, 1.0, 1.0, 0.0), (1.0, 0.0, 0.0, 1.0), (1.0, 1.0, 1.0, 1.0));
