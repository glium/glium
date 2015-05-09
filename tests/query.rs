#[macro_use]
extern crate glium;

use std::default::Default;
use glium::Surface;

mod support;

#[test]
fn samples_passed() {
    let display = support::build_display();

    let query = match glium::draw_parameters::SamplesPassedQuery::new_if_supported(&display) {
        Some(q) => q,
        None => return
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);

    {
        let params = glium::DrawParameters {
            samples_passed_query: Some(&query),
            .. Default::default()
        };

        texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params)
               .unwrap();
    }

    let result = query.get();
    assert!(result == 1024 * 1024); // texture dimensions

    display.assert_no_error(None);
}

#[test]
fn query_sequence() {
    let display = support::build_display();

    let query = match glium::draw_parameters::SamplesPassedQuery::new_if_supported(&display) {
        Some(q) => q,
        None => return
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);

    for _ in (0 .. 3) {
        let params = glium::DrawParameters {
            samples_passed_query: Some(&query),
            .. Default::default()
        };

        texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params)
               .unwrap();
    }

    let result = query.get();
    assert!(result == 3 * 1024 * 1024); // 3 * texture dimensions

    display.assert_no_error(None);
}
