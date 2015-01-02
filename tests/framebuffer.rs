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
#[cfg(feature = "gl_extensions")]       // TODO: remove
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

#[test]
#[cfg(feature = "gl_extensions")]       // TODO: remove
fn depth_texture2d() {
    use std::iter;

    let display = support::build_display();
    let (vb, ib) = support::build_rectangle_vb_ib(&display);

    // the program returns a Z coordinate between 0 (left of screen) and 1 (right of screen)
    let program = glium::Program::from_source(&display,
        "
            #version 110

            attribute vec2 position;

            void main() {
                gl_Position = vec4(position, position.x, 1.0);
            }
        ",
        "
            #version 110

            void main() {
                gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
            }
        ",
        None).unwrap();

    // empty color attachment to put the data
    let color = glium::Texture2d::new_empty(&display,
                                            glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                            128, 128);

    // depth texture with a value of 0.5 everywhere
    let depth_data = iter::repeat(iter::repeat(0.5f32).take(128).collect::<Vec<_>>())
                                  .take(128).collect::<Vec<_>>();
    let depth = glium::texture::DepthTexture2d::new(&display, depth_data);

    // drawing with the `IfLess` depth test
    let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(&display,
                                                                                   &color, &depth);
    let params = glium::DrawParameters {
        depth_function: glium::DepthFunction::IfLess,
        .. std::default::Default::default()
    };

    framebuffer.clear_color(0.0, 0.0, 0.0, 1.0);
    framebuffer.draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params);

    // reading back the color
    let read_back: Vec<Vec<(f32, f32, f32, f32)>> = color.read();

    assert_eq!(read_back[0][0], (1.0, 1.0, 1.0, 1.0));
    assert_eq!(read_back[127][127], (0.0, 0.0, 0.0, 1.0));

    display.assert_no_error();
}
