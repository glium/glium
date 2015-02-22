extern crate glutin;
#[macro_use]
extern crate glium;

use std::default::Default;
use glium::Surface;

mod support;

#[derive(Copy)]
struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);

#[test]
fn uniforms_storage_single_value() {    
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

            uniform vec4 color;

            void main() {
                gl_FragColor = color;
            }
        ",
        None).unwrap();

    let uniforms = glium::uniforms::UniformsStorage::new("color", [1.0, 0.0, 0.0, 0.5f32]);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &uniforms, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8)>> = texture.read();
    assert_eq!(data[0][0], (255, 0, 0));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0));

    display.assert_no_error();
}

#[test]
fn uniforms_storage_multiple_values() {    
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

            uniform vec4 color1;
            uniform vec4 color2;

            void main() {
                gl_FragColor = color1 + color2;
            }
        ",
        None).unwrap();

    let uniforms = glium::uniforms::UniformsStorage::new("color1", [0.7, 0.0, 0.0, 0.5f32]);
    let uniforms = uniforms.add("color2", [0.3, 0.0, 0.0, 0.5f32]);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &uniforms, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8)>> = texture.read();
    assert_eq!(data[0][0], (255, 0, 0));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0));

    display.assert_no_error();
}

#[test]
fn uniforms_storage_ignore_inactive_uniforms() {    
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

            uniform vec4 color;

            void main() {
                gl_FragColor = color;
            }
        ",
        None).unwrap();

    let uniforms = glium::uniforms::UniformsStorage::new("color", [1.0, 0.0, 0.0, 0.5f32]);
    let uniforms = uniforms.add("color2", 0.8f32);
    let uniforms = uniforms.add("color3", [0.1, 1.2f32]);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &uniforms, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8)>> = texture.read();
    assert_eq!(data[0][0], (255, 0, 0));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0));
    
    display.assert_no_error();
}

#[test]
fn uniform_wrong_type() {    
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

            uniform vec4 color;

            void main() {
                gl_FragColor = color;
            }
        ",
        None).unwrap();

    let uniforms = glium::uniforms::UniformsStorage::new("color", 1.0f32);

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    match target.draw(&vb, &ib, &program, &uniforms, &Default::default()) {
        Err(glium::DrawError::UniformTypeMismatch { .. }) => (),
        a => panic!("{:?}", a)
    };
    target.finish();

    display.assert_no_error();
}
