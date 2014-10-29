#![feature(phase)]

#[phase(plugin)]
extern crate glium_macros;

extern crate glutin;
extern crate glium;

use glium::{DisplayBuild, Texture};

#[test]
fn texture_1d_creation() {
    let display = glutin::HeadlessRendererBuilder::new(1024, 768)
        .build_glium().unwrap();

    let texture = glium::texture::Texture1D::new(&display, vec![
        (0.0, 0.0, 0.0, 0.0),
        (0.0, 0.0, 0.0, 0.0),
        (0.0, 0.0, 0.0, 0.0),
    ]);

    assert_eq!(texture.get_width(), 3);
    assert_eq!(texture.get_height(), None);
    assert_eq!(texture.get_depth(), None);
    assert_eq!(texture.get_array_size(), None);
}

#[test]
fn texture_2d_creation() {
    let display = glutin::HeadlessRendererBuilder::new(1024, 768)
        .build_glium().unwrap();

    let texture = glium::texture::Texture2D::new(&display, vec![
        vec![(0.0, 0.0, 0.0, 0.0), (0.0, 0.0, 0.0, 0.0)],
        vec![(0.0, 0.0, 0.0, 0.0), (0.0, 0.0, 0.0, 0.0)],
        vec![(0.0, 0.0, 0.0, 0.0), (0.0, 0.0, 0.0, 0.0f32)],
    ]);

    assert_eq!(texture.get_width(), 2);
    assert_eq!(texture.get_height(), Some(3));
    assert_eq!(texture.get_depth(), None);
    assert_eq!(texture.get_array_size(), None);
}

#[test]
fn texture_3d_creation() {
    let display = glutin::HeadlessRendererBuilder::new(1024, 768)
        .build_glium().unwrap();

    let texture = glium::texture::Texture3D::new(&display, vec![
        vec![
            vec![(0.0, 0.0, 0.0, 0.0)],
            vec![(0.0, 0.0, 0.0, 0.0)],
        ],
        vec![
            vec![(0.0, 0.0, 0.0, 0.0)],
            vec![(0.0, 0.0, 0.0, 0.0)],
        ],
        vec![
            vec![(0.0, 0.0, 0.0, 0.0)],
            vec![(0.0, 0.0, 0.0, 0.0f32)],
        ],
    ]);

    assert_eq!(texture.get_width(), 1);
    assert_eq!(texture.get_height(), Some(2));
    assert_eq!(texture.get_depth(), Some(3));
    assert_eq!(texture.get_array_size(), None);
}
