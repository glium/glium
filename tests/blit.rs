#[macro_use]
extern crate glium;

use glium::{Surface, BlitTarget, Rect};

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
