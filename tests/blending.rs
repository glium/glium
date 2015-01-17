#![feature(plugin)]
#![feature(unboxed_closures)]

#[plugin]
extern crate glium_macros;

extern crate glutin;
extern crate glium;

use glium::Surface;

mod support;

#[test]
fn min_blending() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        blending_function: Some(glium::BlendingFunction::Min),
        .. std::default::Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let mut target = display.draw();
    target.clear_color(0.0, 0.2, 0.3, 1.0);
    target.draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params);
    target.finish();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = display.read_front_buffer();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(0, 0, 0, 255));
        }
    }

    display.assert_no_error();
}

#[test]
fn max_blending() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        blending_function: Some(glium::BlendingFunction::Max),
        .. std::default::Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let mut target = display.draw();
    target.clear_color(0.4, 1.0, 1.0, 0.2);
    target.draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params);
    target.finish();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = display.read_front_buffer();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 255, 255, 255));
        }
    }

    display.assert_no_error();
}

#[test]
fn one_plus_one() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        blending_function: Some(glium::BlendingFunction::Addition {
            source: glium::LinearBlendingFactor::One,
            destination: glium::LinearBlendingFactor::One,
        }),
        .. std::default::Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let mut target = display.draw();
    target.clear_color(0.0, 1.0, 1.0, 0.0);
    target.draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params);
    target.finish();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = display.read_front_buffer();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 255, 255, 255));
        }
    }

    display.assert_no_error();
}
