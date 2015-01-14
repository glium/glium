#![feature(plugin)]
#![feature(unboxed_closures)]

#[plugin]
extern crate glium_macros;

extern crate glutin;
extern crate glium;

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
