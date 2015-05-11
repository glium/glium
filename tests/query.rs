#[macro_use]
extern crate glium;

use std::default::Default;
use glium::Surface;

mod support;

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
        let params = glium::DrawParameters::new(&display)
                        .with_samples_passed_query(&query);

        texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params)
               .unwrap();
    }

    let result = query.get();
    assert!(result == 3 * 1024 * 1024); // 3 * texture dimensions

    display.assert_no_error(None);
}

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
        let params = glium::DrawParameters::new(&display)
                        .with_samples_passed_query(&query);

        texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params)
               .unwrap();
    }

    let result = query.get();
    assert!(result == 1024 * 1024); // texture dimensions

    display.assert_no_error(None);
}

#[test]
fn any_samples_passed() {
    let display = support::build_display();

    let query = match glium::draw_parameters::AnySamplesPassedQuery::new_if_supported(&display, false) {
        Some(q) => q,
        None => return
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);

    {
        let params = glium::DrawParameters::new(&display)
                        .with_samples_passed_query(&query);

        texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params)
               .unwrap();
    }

    assert!(query.get());

    let query2 = match glium::draw_parameters::AnySamplesPassedQuery::new_if_supported(&display, false) {
        Some(q) => q,
        None => return
    };

    assert!(!query2.get());

    display.assert_no_error(None);
}

#[test]
fn any_samples_passed_conservative() {
    let display = support::build_display();

    let query = match glium::draw_parameters::AnySamplesPassedQuery::new_if_supported(&display, true) {
        Some(q) => q,
        None => return
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);

    {
        let params = glium::DrawParameters::new(&display)
                        .with_samples_passed_query(&query);

        texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params)
               .unwrap();
    }

    assert!(query.get());

    let query2 = match glium::draw_parameters::AnySamplesPassedQuery::new_if_supported(&display, true) {
        Some(q) => q,
        None => return
    };

    assert!(!query2.get());

    display.assert_no_error(None);
}

#[test]
fn time_elapsed() {
    let display = support::build_display();

    let query = match glium::draw_parameters::TimeElapsedQuery::new_if_supported(&display) {
        Some(q) => q,
        None => return
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);

    {
        let params = glium::DrawParameters::new(&display)
                        .with_time_elapsed_query(&query);

        texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params)
               .unwrap();
    }

    let result = query.get();
    assert!(result >= 1);

    display.assert_no_error(None);
}

#[test]
#[ignore]       // not sure about the interaction between pritmives_generated and no geometry shader
fn primitives_generated() {
    let display = support::build_display();

    let query = match glium::draw_parameters::PrimitivesGeneratedQuery::new_if_supported(&display) {
        Some(q) => q,
        None => return
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);

    {
        let params = glium::DrawParameters::new(&display)
                        .with_primitives_generated_query(&query);

        texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params)
               .unwrap();
    }

    let result = query.get();
    assert_eq!(result, 2);

    display.assert_no_error(None);
}
