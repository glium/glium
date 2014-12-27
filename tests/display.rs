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

#[test]
fn release_shader_compiler() {
    let display = support::build_display();
    display.release_shader_compiler();
    display.assert_no_error();
}

#[test]
#[should_fail(expected = "Viewport dimensions are too large")]
fn viewport_too_large() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        viewport: Some(glium::Rect {
            left: 0,
            bottom: 0,
            width: 4294967295,
            height: 4294967295,
        }),
        .. std::default::Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);
    display.draw().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params);
}

#[test]
fn timestamp_query() {
    let display = support::build_display();

    let query1 = glium::debug::TimestampQuery::new(&display);
    let query1 = query1.map(|q| q.get());

    let query2 = glium::debug::TimestampQuery::new(&display);
    let query2 = query2.map(|q| q.get());

    match (query1, query2) {
        (Some(q1), Some(q2)) => assert!(q2 >= q1 && q1 != 0),
        _ => ()
    };

    display.assert_no_error();
}
