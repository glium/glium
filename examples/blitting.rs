extern crate glutin;
extern crate glium;

#[cfg(feature = "image")]
extern crate image;

use std::io::BufReader;
use std::rand;

use glium::{DisplayBuild, Texture, Surface, Rect};

#[cfg(not(feature = "image"))]
fn main() {
    println!("This example requires the `image` feature to be enabled");
}

#[cfg(feature = "image")]
fn main() {
    // building the display, ie. the main object
    let display = glutin::WindowBuilder::new()
        .with_vsync()
        .build_glium()
        .unwrap();

    // building a texture with "OpenGL" drawn on it
    let image = image::load(BufReader::new(include_bytes!("../tests/fixture/opengl.png")),
        image::PNG).unwrap();
    let opengl_texture = glium::Texture2d::new(&display, image);

    // building a 1024x1024 empty texture
    let dest_texture = glium::Texture2d::new_empty(&display, glium::texture::
                                                             UncompressedFloatFormat::U8U8U8U8,
                                                   1024, 1024);

    // the main loop
    // each cycle will draw once
    'main: loop {
        // we have one out of 60 chances to blit one `opengl_texture` over `dest_texture`
        if rand::random::<f64>() <= 0.016666 {
            let (left, bottom, dimensions): (f32, f32, f32) = rand::random();
            let dest_rect = glium::Rect {
                left: (left * dest_texture.get_width() as f32) as u32,
                bottom: (bottom * dest_texture.get_height().unwrap() as f32) as u32,
                width: (dimensions * dest_texture.get_width() as f32) as u32,
                height: (dimensions * dest_texture.get_height().unwrap() as f32) as u32,
            };

            opengl_texture.as_surface().blit_whole_color_to(&dest_texture.as_surface(), &dest_rect,
                                                            glium::uniforms::MagnifySamplerFilter::Linear);
        }

        // drawing a frame
        let target = display.draw();
        dest_texture.as_surface().fill(&target, glium::uniforms::MagnifySamplerFilter::Linear);
        target.finish();

        // polling and handling the events received by the window
        for event in display.poll_events().into_iter() {
            match event {
                glutin::Event::Closed => break 'main,
                _ => ()
            }
        }
    }
}
