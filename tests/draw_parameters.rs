#[macro_use]
extern crate glium;

use glium::Surface;

mod support;

#[test]
fn color_mask() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        color_mask: (false, true, true, true),
        .. Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(0, 0, 0, 255));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn viewport_too_large() {
    let display = support::build_display();

    let params = glium::DrawParameters::new(&display)
                    .with_viewport(glium::Rect {
                        left: 0,
                        bottom: 0,
                        width: 4294967295,
                        height: 4294967295,
                    });

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let mut frame = display.draw();
    match frame.draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params) {
        Err(glium::DrawError::ViewportTooLarge) => (),
        a => panic!("{:?}", a)
    };
    frame.finish().unwrap();

    display.assert_no_error(None);
}

#[test]
fn wrong_depth_range() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        depth_range: (-0.1, 1.0),
        .. Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let mut frame = display.draw();
    match frame.draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params) {
        Err(glium::DrawError::InvalidDepthRange) => (),
        a => panic!("{:?}", a)
    };
    frame.finish().unwrap();

    display.assert_no_error(None);
}

#[test]
fn scissor() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        scissor: Some(glium::Rect {
            left: 0,
            bottom: 0,
            width: 1,
            height: 1,
        }),
        .. Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    assert_eq!(data[0][0], (255, 0, 0, 255));
    assert_eq!(data[1][0], (0, 0, 0, 0));
    assert_eq!(data[0][1], (0, 0, 0, 0));
    assert_eq!(data[1][1], (0, 0, 0, 0));

    for row in data.iter().skip(1) {
        for pixel in row.iter().skip(1) {
            assert_eq!(pixel, &(0, 0, 0, 0));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn scissor_followed_by_clear() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        scissor: Some(glium::Rect {
            left: 2,
            bottom: 2,
            width: 2,
            height: 2,
        }),
        .. Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms,
                              &params).unwrap();
    texture.as_surface().clear_color(1.0, 0.0, 1.0, 1.0);

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 0, 255, 255));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn viewport_followed_by_clear() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        viewport: Some(glium::Rect {
            left: 2,
            bottom: 2,
            width: 2,
            height: 2,
        }),
        .. Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms,
                              &params).unwrap();
    texture.as_surface().clear_color(1.0, 0.0, 1.0, 1.0);

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 0, 255, 255));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn viewport() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        viewport: Some(glium::Rect {
            left: 0,
            bottom: 0,
            width: 1,
            height: 1,
        }),
        .. Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    assert_eq!(data[0][0], (255, 0, 0, 255));
    assert_eq!(data[1][0], (0, 0, 0, 0));
    assert_eq!(data[0][1], (0, 0, 0, 0));
    assert_eq!(data[1][1], (0, 0, 0, 0));

    for row in data.iter().skip(1) {
        for pixel in row.iter().skip(1) {
            assert_eq!(pixel, &(0, 0, 0, 0));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn dont_draw_primitives() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        draw_primitives: false,
        .. Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 1.0, 0.0, 0.0);
    match texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params) {
        Ok(_) => (),
        Err(glium::DrawError::TransformFeedbackNotSupported) => return,
        e => e.unwrap()
    }

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(0, 255, 0, 0));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn dont_draw_primitives_then_draw() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        draw_primitives: false,
        .. Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 1.0, 0.0, 0.0);
    match texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params) {
        Ok(_) => (),
        Err(glium::DrawError::TransformFeedbackNotSupported) => return,
        e => e.unwrap()
    }
    texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 0, 0, 255));
        }
    }

    display.assert_no_error(None);
}
