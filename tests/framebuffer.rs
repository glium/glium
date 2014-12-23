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
    let mut framebuffer = glium::FrameBuffer::new(&display).with_color_texture(&texture);

    let parameters = glium::DrawParameters {
        depth_function: glium::DepthFunction::IfLess,
        .. std::default::Default::default()
    };

    framebuffer.draw(&vertex_buffer, &index_buffer, &program,
                     &glium::uniforms::EmptyUniforms, &parameters);
}
