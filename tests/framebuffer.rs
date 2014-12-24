#![feature(phase)]
#![feature(unboxed_closures)]

#[phase(plugin)]
extern crate glium_macros;

extern crate glutin;
extern crate glium;

use glium::Surface;

mod support;

#[test]
#[should_fail(expected="Requested a depth function but no depth buffer is attached")]
fn no_depth_buffer() {
    let display = support::build_display();
    let (vertex_buffer, index_buffer, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = glium::texture::Texture2d::new_empty(&display,
                            glium::texture::UncompressedFloatFormat::U8U8U8U8, 128, 128);
    let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display, &texture);

    let parameters = glium::DrawParameters {
        depth_function: glium::DepthFunction::IfLess,
        .. std::default::Default::default()
    };

    framebuffer.draw(&vertex_buffer, &index_buffer, &program,
                     &glium::uniforms::EmptyUniforms, &parameters);
}

#[test]
fn simple_dimensions() {
    let display = support::build_display();

    let texture = glium::Texture2d::new_empty(&display,
                                              glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                              128, 128);

    let framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display, &texture);
    assert_eq!(framebuffer.get_dimensions(), (128, 128));

    display.assert_no_error();
}

#[test]
fn simple_render_to_texture() {
    use std::default::Default;

    let display = support::build_display();
    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = glium::Texture2d::new_empty(&display,
                                              glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                              128, 128);

    let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display, &texture);
    framebuffer.draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &Default::default());

    let read_back: Vec<Vec<(f32, f32, f32, f32)>> = texture.read();

    assert_eq!(read_back[0][0], (1.0, 0.0, 0.0, 1.0));
    assert_eq!(read_back[64][64], (1.0, 0.0, 0.0, 1.0));
    assert_eq!(read_back[127][127], (1.0, 0.0, 0.0, 1.0));
    
    display.assert_no_error();
}
