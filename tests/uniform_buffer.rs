#![feature(plugin)]
#![feature(unboxed_closures)]

#[plugin]
extern crate glium_macros;

extern crate glutin;
#[macro_use]
extern crate glium;

use glium::Surface;
use std::default::Default;

mod support;

#[test]
fn uniform_buffer_creation() {
    let display = support::build_display();

    glium::uniforms::UniformBuffer::new_if_supported(&display, 12);

    display.assert_no_error();
}

#[test]
fn uniform_buffer_mapping_read() {
    let display = support::build_display();

    let mut vb = match glium::uniforms::UniformBuffer::new_if_supported(&display, 12) {
        None => return,
        Some(b) => b
    };

    let mapping = vb.map();
    assert_eq!(*mapping, 12);

    display.assert_no_error();
}

#[test]
fn uniform_buffer_mapping_write() {
    let display = support::build_display();

    let mut vb = match glium::uniforms::UniformBuffer::new_if_supported(&display, 6) {
        None => return,
        Some(b) => b
    };

    {
        let mut mapping = vb.map();
        *mapping = 15;
    }

    let mapping = vb.map();
    assert_eq!(*mapping, 15);

    display.assert_no_error();
}

#[test]
fn uniform_buffer_read() {
    let display = support::build_display();

    let vb = match glium::uniforms::UniformBuffer::new_if_supported(&display, 12) {
        None => return,
        Some(b) => b
    };

    let data = match vb.read_if_supported() {
        Some(d) => d,
        None => return
    };

    assert_eq!(data, 12);

    display.assert_no_error();
}

#[test]
fn uniform_buffer_write() {
    let display = support::build_display();

    let mut vb = match glium::uniforms::UniformBuffer::new_if_supported(&display, 5) {
        None => return,
        Some(b) => b
    };

    vb.upload(24);

    let data = match vb.read_if_supported() {
        Some(d) => d,
        None => return
    };

    assert_eq!(data, 24);

    display.assert_no_error();
}

#[test]
fn block() {
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
            #version 330
            uniform layout(std140);

            uniform MyBlock {
                vec3 color;
            };

            void main() {
                gl_FragColor = vec4(color, 1.0);
            }
        ",
        None);

    // ignoring test in case of compilation error (version may not be supported)
    let program = match program {
        Ok(p) => p,
        Err(_) => return
    };

    let buffer = match glium::uniforms::UniformBuffer::new_if_supported(&display, (1.0f32, 1.0f32, 0.0f32)) {
        None => return,
        Some(b) => b
    };

    let uniforms = uniform!{
        MyBlock: &buffer
    };

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(&vb, &ib, &program, &uniforms, &Default::default());
    target.finish();

    let data: Vec<Vec<(f32, f32, f32)>> = display.read_front_buffer();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(1.0, 1.0, 0.0));
        }
    }

    display.assert_no_error();
}

#[test]
#[should_fail(expected = "The content of the uniform buffer does not match the layout of the block")]
fn block_wrong_type() {
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
            #version 330
            uniform layout(std140);

            uniform MyBlock {
                vec3 color;
            };

            void main() {
                gl_FragColor = vec4(color, 1.0);
            }
        ",
        None);

    // ignoring test in case of compilation error (version may not be supported)
    let program = match program {
        Ok(p) => p,
        // yeah, that's hacky
        Err(_) => panic!("The content of the uniform buffer does not match the layout of the block")
    };

    let buffer = match glium::uniforms::UniformBuffer::new_if_supported(&display, 2) {
        // hacky too
        None => panic!("The content of the uniform buffer does not match the layout of the block"),
        Some(b) => b
    };

    let uniforms = uniform!{
        MyBlock: &buffer
    };

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(&vb, &ib, &program, &uniforms, &Default::default());
}
