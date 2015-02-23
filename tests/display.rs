extern crate glutin;
#[macro_use]
extern crate glium;

use glium::Surface;

mod support;

#[test]
fn get_opengl_version() {
    let display = support::build_display();
    let version = display.get_opengl_version();
    display.assert_no_error();

    assert!(version.1 >= 1);
}

#[test]
fn clear_color() {
    let display = support::build_display();

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(1.0, 0.0, 0.0, 1.0);

    let data: Vec<Vec<(f32, f32, f32)>> = texture.read();

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
    
    match display.draw().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params) {
        Err(glium::DrawError::ViewportTooLarge) => (),
        a => panic!("{:?}", a)
    };

    display.assert_no_error();
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

#[test]
fn wrong_depth_range() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        depth_range: (-0.1, 1.0),
        .. std::default::Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);
    
    match display.draw().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params) {
        Err(glium::DrawError::InvalidDepthRange) => (),
        a => panic!("{:?}", a)
    };

    display.assert_no_error();
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
        .. std::default::Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params).unwrap();

    let data: Vec<Vec<(f32, f32, f32)>> = texture.read();

    assert_eq!(data[0][0], (1.0, 0.0, 0.0));
    assert_eq!(data[1][0], (0.0, 0.0, 0.0));
    assert_eq!(data[0][1], (0.0, 0.0, 0.0));
    assert_eq!(data[1][1], (0.0, 0.0, 0.0));

    for row in data.iter().skip(1) {
        for pixel in row.iter().skip(1) {
            assert_eq!(pixel, &(0.0, 0.0, 0.0));
        }
    }

    display.assert_no_error();
}

#[test]
fn sync() {    
    let display = support::build_display();

    let fence = glium::SyncFence::new_if_supported(&display);
    fence.map(|f| f.wait());

    display.assert_no_error();
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
        .. std::default::Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms,
                              &params).unwrap();
    texture.as_surface().clear_color(1.0, 0.0, 1.0, 1.0);

    let data: Vec<Vec<(f32, f32, f32, f32)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(1.0, 0.0, 1.0, 1.0));
        }
    }

    display.assert_no_error();
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
        .. std::default::Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms,
                              &params).unwrap();
    texture.as_surface().clear_color(1.0, 0.0, 1.0, 1.0);

    let data: Vec<Vec<(f32, f32, f32, f32)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(1.0, 0.0, 1.0, 1.0));
        }
    }

    display.assert_no_error();
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
        .. std::default::Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params).unwrap();

    let data: Vec<Vec<(f32, f32, f32)>> = texture.read();

    assert_eq!(data[0][0], (1.0, 0.0, 0.0));
    assert_eq!(data[1][0], (0.0, 0.0, 0.0));
    assert_eq!(data[0][1], (0.0, 0.0, 0.0));
    assert_eq!(data[1][1], (0.0, 0.0, 0.0));

    for row in data.iter().skip(1) {
        for pixel in row.iter().skip(1) {
            assert_eq!(pixel, &(0.0, 0.0, 0.0));
        }
    }

    display.assert_no_error();
}

#[test]
fn dont_draw_primitives() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        draw_primitives: false,
        .. std::default::Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 1.0, 0.0, 0.0);
    match texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params) {
        Ok(_) => (),
        Err(glium::DrawError::TransformFeedbackNotSupported) => return,
        e => e.unwrap()
    }

    let data: Vec<Vec<(f32, f32, f32)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(0.0, 1.0, 0.0));
        }
    }

    display.assert_no_error();
}

#[test]
fn dont_draw_primitives_then_draw() {
    let display = support::build_display();

    let params = glium::DrawParameters {
        draw_primitives: false,
        .. std::default::Default::default()
    };

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 1.0, 0.0, 0.0);
    match texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params) {
        Ok(_) => (),
        Err(glium::DrawError::TransformFeedbackNotSupported) => return,
        e => e.unwrap()
    }
    texture.as_surface().draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &std::default::Default::default()).unwrap();

    let data: Vec<Vec<(f32, f32, f32)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(1.0, 0.0, 0.0));
        }
    }

    display.assert_no_error();
}
