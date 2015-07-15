#[macro_use]
extern crate glium;

use glium::Surface;

mod support;

macro_rules! texture_sample_test {
    ($test_name:ident, $tex_ty:ident, $sampler_ty:expr, $fun:expr, $coords:expr, $data:expr) => (
        #[test]
        fn $test_name() {
            let display = support::build_display();
            let (vb, ib) = support::build_rectangle_vb_ib(&display);

            let texture = glium::texture::$tex_ty::new(&display, vec![
                vec![(255, 0, 0, 255), (255, 0, 0, 255)],
                vec![(255, 0, 0, 255), (255, 0, 0, 255u8)],
            ]).unwrap();

            let program = glium::Program::from_source(&display,
                "
                    #version 110

                    attribute vec2 position;

                    void main() {
                        gl_Position = vec4(position, 0.0, 1.0);
                    }
                ",
                &format!("
                    #version 110

                    uniform {} texture;

                    void main() {{
                        gl_FragColor = {}(texture, {});
                    }}
                ", $sampler_ty, $fun, $coords),
                None).unwrap();

            let output = support::build_renderable_texture(&display);
            output.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
            output.as_surface().draw(&vb, &ib, &program, &uniform!{ texture: &texture },
                                     &Default::default()).unwrap();

            let data: Vec<Vec<(u8, u8, u8, u8)>> = output.read();
            for row in data.iter() {
                for pixel in row.iter() {
                    assert_eq!(pixel, &(255, 0, 0, 255));
                }
            }

            display.assert_no_error(None);
        }
    );
}

texture_sample_test!(texture_2d_draw, Texture2d, "sampler2D", "texture2D", "vec2(0.5, 0.5)",
    vec![
        vec![(255, 0, 0, 255), (255, 0, 0, 255)],
        vec![(255, 0, 0, 255), (255, 0, 0, 255u8)],
    ]);

texture_sample_test!(compressed_texture_2d_draw, CompressedTexture2d, "sampler2D", "texture2D",
    "vec2(0.5, 0.5)",
    vec![
        vec![(255, 0, 0, 255), (255, 0, 0, 255)],
        vec![(255, 0, 0, 255), (255, 0, 0, 255u8)],
    ]);


#[test]
fn bindless_texture() {
    let display = support::build_display();
    let (vb, ib) = support::build_rectangle_vb_ib(&display);

    let texture = glium::texture::Texture2d::new(&display, vec![
        vec![(255, 0, 0, 255), (255, 0, 0, 255)],
        vec![(255, 0, 0, 255), (255, 0, 0, 255u8)],
    ]).unwrap();

    let texture = match texture.resident() {
        Ok(t) => t,
        Err(_) => return
    };

    // if bindless textures are supported, we can call .unwrap() and expect that everything
    // else is supported here as well

    let program = glium::Program::from_source(&display,
        "
            #version 100

            attribute lowp vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        ",
        "
            #version 400
            #extension GL_ARB_bindless_texture : require

            uniform Samplers {
                sampler2D tex;
            };

            out vec4 f_color;

            void main() {
                f_color = texture(tex, vec2(0.0, 0.0));
            }
        ",
        None).unwrap();

    let buffer = glium::uniforms::UniformBuffer::new(&display,
                                            glium::texture::TextureHandle::new(&texture, &Default::default())).unwrap();

    let output = support::build_renderable_texture(&display);
    output.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    output.as_surface().draw(&vb, &ib, &program, &uniform!{ Samplers: &buffer },
                             &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = output.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 0, 0, 255));
        }
    }

    display.assert_no_error(None);
}
