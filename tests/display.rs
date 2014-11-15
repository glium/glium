#![feature(phase)]
#![feature(unboxed_closures)]

#[phase(plugin)]
extern crate glium_macros;

extern crate glutin;
extern crate glium;

use glium::Surface;

mod support;

#[test]
fn display_clear_color() {
    let display = support::build_display();

    let mut target = display.draw();
    target.clear_color(1.0, 0.0, 0.0, 1.0);
    target.finish();

    let data: Vec<Vec<(f32, f32, f32)>> = display.read_front_buffer();

    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(1.0, 0.0, 0.0));
        }
    }

    display.assert_no_error();
}
