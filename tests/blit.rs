#[macro_use]
extern crate glium;

use glium::{Surface, BlitTarget, Rect, BlitMask};
use glium::framebuffer::SimpleFrameBuffer;
use glium::uniforms::MagnifySamplerFilter;

mod support;

#[test]
fn blit_texture_to_window() {
    let display = support::build_display();

    let src_rect = Rect {
        left: 0,
        bottom: 0,
        width: 2,
        height: 2,
    };

    let dest_rect = BlitTarget {
        left: 1,
        bottom: 1,
        width: 2,
        height: 2,
    };

    let texture = support::build_unicolor_texture2d(&display, 0.0, 1.0, 0.0);

    let target = support::build_renderable_texture(&display);
    target.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);

    texture.as_surface().blit_color(&src_rect, &target.as_surface(), &dest_rect,
                                    glium::uniforms::MagnifySamplerFilter::Nearest);

    let data: Vec<Vec<(u8, u8, u8, u8)>> = target.read();

    assert_eq!(data[1][1], (0, 255, 0, 255));
    assert_eq!(data[1][2], (0, 255, 0, 255));
    assert_eq!(data[2][1], (0, 255, 0, 255));
    assert_eq!(data[2][2], (0, 255, 0, 255));

    assert_eq!(data[0][0], (0, 0, 0, 0));

    assert_eq!(data[0][1], (0, 0, 0, 0));
    assert_eq!(data[0][2], (0, 0, 0, 0));

    assert_eq!(data[1][0], (0, 0, 0, 0));
    assert_eq!(data[2][0], (0, 0, 0, 0));

    assert_eq!(data[3][1], (0, 0, 0, 0));
    assert_eq!(data[3][2], (0, 0, 0, 0));

    assert_eq!(data[2][3], (0, 0, 0, 0));
    assert_eq!(data[1][3], (0, 0, 0, 0));

    assert_eq!(data[3][3], (0, 0, 0, 0));

    display.assert_no_error(None);
}

#[test]
fn blit_color_and_depth_buffer() {
    let display = support::build_display();

    // source frame buffer
    let src_tex_color = support::build_unicolor_texture2d(&display, 0.0, 0.5, 1.0);
    let src_tex_depth = support::build_constant_depth_texture(&display, 0.5);
    let src_frame_buffer = SimpleFrameBuffer::with_depth_buffer(&display, &src_tex_color, &src_tex_depth).unwrap();

    // destination frame buffer
    let dst_tex_color = support::build_unicolor_texture2d(&display, 0.0, 0.0, 0.0);
    let dst_tex_depth = support::build_constant_depth_texture(&display, 0.0);
    let dst_frame_buffer = SimpleFrameBuffer::with_depth_buffer(&display, &dst_tex_color, &dst_tex_depth).unwrap();

    // blit
    let src_rect = Rect {left: 0, bottom: 0, width: 2, height: 2, };
    let dst_rect = BlitTarget { left: 0, bottom: 0, width: 2, height: 2, };
    dst_frame_buffer.blit_buffers_from_simple_framebuffer(
        &src_frame_buffer,
        &src_rect,
        &dst_rect,
        MagnifySamplerFilter::Nearest,
        BlitMask::color_and_depth()
    );

    // check result
    let color_data: Vec<Vec<(u8, u8, u8, u8)>> = dst_tex_color.read();
    assert_eq!(color_data, vec![
        vec![(0, 127, 255, 255), (0, 127, 255, 255),],
        vec![(0, 127, 255, 255), (0, 127, 255, 255),],
    ]);
    // todo: how to check dst_tex_depth? There is no .read() on a DepthTexture2d...
    display.assert_no_error(None);
}
