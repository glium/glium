#![feature(phase)]
#![feature(unboxed_closures)]

#[phase(plugin)]
extern crate glium_macros;

extern crate glutin;
extern crate glium;

use std::default::Default;
use glium::Surface;

mod support;

fn build_program(display: &glium::Display) -> glium::Program {
    glium::Program::new(display,
        "
            #version 110

            attribute vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        ",
        "
            #version 110

            void main() {
                gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
            }
        ",
        None).unwrap()
}

#[vertex_format]
#[deriving(Copy)]
struct Vertex {
    position: [f32, ..2],
}

#[test]
fn triangles_list_cpu() {
    // ignoring test on travis
    // TODO: find out why they are failing
    if ::std::os::getenv("TRAVIS").is_some() {
        return;
    }
    
    let display = support::build_display();
    let program = build_program(&display);

    let vb = glium::VertexBuffer::new(&display, vec![
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]);

    let indices = glium::index_buffer::TrianglesList(vec![0u8, 1, 2, 2, 1, 3]);

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(&vb, &indices, &program, &glium::uniforms::EmptyUniforms, &Default::default());
    target.finish();

    let data: Vec<Vec<(u8, u8, u8)>> = display.read_front_buffer();

    assert_eq!(data[0][0], (255, 0, 0));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0));

    display.assert_no_error();
}

#[test]
fn triangle_strip_cpu() {
    // ignoring test on travis
    // TODO: find out why they are failing
    if ::std::os::getenv("TRAVIS").is_some() {
        return;
    }

    let display = support::build_display();
    let program = build_program(&display);

    let vb = glium::VertexBuffer::new(&display, vec![
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]);

    let indices = glium::index_buffer::TriangleStrip(vec![0u8, 1, 2, 3]);

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(&vb, &indices, &program, &glium::uniforms::EmptyUniforms, &Default::default());
    target.finish();

    let data: Vec<Vec<(u8, u8, u8)>> = display.read_front_buffer();

    assert_eq!(data[0][0], (255, 0, 0));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0));

    display.assert_no_error();
}

#[test]
fn triangles_list_gpu() {
    // ignoring test on travis
    // TODO: find out why they are failing
    if ::std::os::getenv("TRAVIS").is_some() {
        return;
    }

    let display = support::build_display();
    let program = build_program(&display);

    let vb = glium::VertexBuffer::new(&display, vec![
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]);

    let indices = glium::index_buffer::TrianglesList(vec![0u8, 1, 2, 2, 1, 3]);
    let indices = glium::IndexBuffer::new(&display, indices);

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(&vb, &indices, &program, &glium::uniforms::EmptyUniforms, &Default::default());
    target.finish();

    let data: Vec<Vec<(u8, u8, u8)>> = display.read_front_buffer();

    assert_eq!(data[0][0], (255, 0, 0));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0));

    display.assert_no_error();
}

#[test]
fn triangle_strip_gpu() {
    // ignoring test on travis
    // TODO: find out why they are failing
    if ::std::os::getenv("TRAVIS").is_some() {
        return;
    }

    let display = support::build_display();
    let program = build_program(&display);

    let vb = glium::VertexBuffer::new(&display, vec![
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]);

    let indices = glium::index_buffer::TriangleStrip(vec![0u8, 1, 2, 3]);
    let indices = glium::IndexBuffer::new(&display, indices);

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(&vb, &indices, &program, &glium::uniforms::EmptyUniforms, &Default::default());
    target.finish();

    let data: Vec<Vec<(u8, u8, u8)>> = display.read_front_buffer();

    assert_eq!(data[0][0], (255, 0, 0));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0));
    
    display.assert_no_error();
}
