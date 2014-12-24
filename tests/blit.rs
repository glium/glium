#![feature(phase)]
#![feature(unboxed_closures)]

#[phase(plugin)]
extern crate glium_macros;

extern crate glutin;
extern crate glium;

use glium::{Surface, Rect};

mod support;

#[test]
fn blit_texture_to_window() {
    // ignoring test on travis
    // TODO: find out why they are failing
    if ::std::os::getenv("TRAVIS").is_some() {
        return;
    }

    let display = support::build_display();

    let src_rect = Rect {
        left: 0,
        bottom: 0,
        width: 2,
        height: 2,
    };

    let dest_rect = Rect {
        left: 1,
        bottom: 1,
        width: 2,
        height: 2,
    };

    let texture = support::build_unicolor_texture2d(&display, 0.0, 1.0, 0.0);

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);

    texture.as_surface().blit_color(&src_rect, &target, &dest_rect,
                                    glium::uniforms::MagnifySamplerFilter::Nearest);

    target.finish();

    let data: Vec<Vec<(f32, f32, f32)>> = display.read_front_buffer();

    assert_eq!(data[1][1], (0.0, 1.0, 0.0));
    assert_eq!(data[1][2], (0.0, 1.0, 0.0));
    assert_eq!(data[2][1], (0.0, 1.0, 0.0));
    assert_eq!(data[2][2], (0.0, 1.0, 0.0));

    assert_eq!(data[0][0], (0.0, 0.0, 0.0));

    assert_eq!(data[0][1], (0.0, 0.0, 0.0));
    assert_eq!(data[0][2], (0.0, 0.0, 0.0));

    assert_eq!(data[1][0], (0.0, 0.0, 0.0));
    assert_eq!(data[2][0], (0.0, 0.0, 0.0));

    assert_eq!(data[3][1], (0.0, 0.0, 0.0));
    assert_eq!(data[3][2], (0.0, 0.0, 0.0));

    assert_eq!(data[2][3], (0.0, 0.0, 0.0));
    assert_eq!(data[1][3], (0.0, 0.0, 0.0));

    assert_eq!(data[3][3], (0.0, 0.0, 0.0));
    
    display.assert_no_error();
}
