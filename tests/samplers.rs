#![feature(phase)]
#![feature(unboxed_closures)]

#[phase(plugin)]
extern crate glium_macros;

extern crate glutin;
extern crate glium;

use std::default::Default;
use glium::Surface;

mod support;

#[test]
fn magnify_nearest_filtering() {
    // ignoring test on travis
    // TODO: find out why they are failing
    if ::std::os::getenv("TRAVIS").is_some() {
        return;
    }
    
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

            uniform sampler2D texture;

            void main() {
                gl_FragColor = texture2D(texture, vec2(0.51, 0.0));
            }
        ",
        None).unwrap();

    let texture_data = vec![vec![(0u8, 0, 0), (255, 255, 255)]];
    let texture = glium::texture::Texture2d::new(&display, texture_data);

    let uniforms = glium::uniforms::UniformsStorage::new("texture",
        glium::uniforms::Sampler(&texture, glium::uniforms::SamplerBehavior {
            magnify_filter: glium::uniforms::MagnifySamplerFilter::Nearest,
            .. Default::default()
        }));

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(&vb, &ib, &program, &uniforms, &Default::default());
    target.finish();

    let data: Vec<Vec<(u8, u8, u8)>> = display.read_front_buffer();
    assert_eq!(data[0][0], (255, 255, 255));

    display.assert_no_error();
}
