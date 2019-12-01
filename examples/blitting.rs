extern crate rand;

#[macro_use]
extern crate glium;
extern crate image;

use std::io::Cursor;
#[allow(unused_imports)]
use glium::{glutin, Surface};

mod support;

fn main() {
    // Building the display, ie. the main object
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    // building a texture with "OpenGL" drawn on it
    let image = image::load(Cursor::new(&include_bytes!("../tests/fixture/opengl.png")[..]),
                            image::PNG).unwrap().to_rgba();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let opengl_texture = glium::Texture2d::new(&display, image).unwrap();

    // building a 1024x1024 empty texture
    let dest_texture = glium::Texture2d::empty_with_format(&display,
                                               glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                               glium::texture::MipmapsOption::NoMipmap,
                                               1024, 1024).unwrap();
    dest_texture.as_surface().clear_color(0.0, 0.0, 0.0, 1.0);

    // the main loop
    support::start_loop(event_loop, move |events| {
        // we have one out of 60 chances to blit one `opengl_texture` over `dest_texture`
        if rand::random::<f64>() <= 0.016666 {
            let (left, bottom, dimensions): (f32, f32, f32) = rand::random();
            let dest_rect = glium::BlitTarget {
                left: (left * dest_texture.get_width() as f32) as u32,
                bottom: (bottom * dest_texture.get_height().unwrap() as f32) as u32,
                width: (dimensions * dest_texture.get_width() as f32) as i32,
                height: (dimensions * dest_texture.get_height().unwrap() as f32) as i32,
            };

            opengl_texture.as_surface().blit_whole_color_to(&dest_texture.as_surface(), &dest_rect,
                                                            glium::uniforms::MagnifySamplerFilter::Linear);
        }

        // drawing a frame
        let target = display.draw();
        dest_texture.as_surface().fill(&target, glium::uniforms::MagnifySamplerFilter::Linear);
        target.finish().unwrap();

        let mut action = support::Action::Continue;

            // handling the events received by the window since the last frame
        for event in events {
            match event {
                glutin::event::Event::WindowEvent { event, .. } => match event {
                    glutin::event::WindowEvent::CloseRequested => action = support::Action::Stop,
                    _ => (),
                },
                _ => (),
            }
        }

        action
    });
}
