extern crate glutin;
extern crate glium;

use std::io::BufReader;
use std::rand;

use glium::{DisplayBuild, Texture};
use glium::blit::BlitSurface;

mod support;

fn main() {
    // building the display, ie. the main object
    let display = glutin::WindowBuilder::new()
        .with_vsync()
        .build_glium()
        .unwrap();

    // building a texture with "OpenGL" drawn on it
    let image = support::Image::load(BufReader::new(include_bin!("../tests/fixture/opengl.png")))
        .unwrap();
    let opengl_texture = glium::Texture2D::new(&display, image);

    // building a 1024x1024 black texture
    // TODO: use a nicer way
    let dest_texture = glium::Texture2D::new(&display,
        Vec::from_elem(1024, Vec::from_elem(1024, (0u8, 0u8, 0u8)))
    );

    // the main loop
    // each cycle will draw once
    'main: loop {
        // we have one out of 60 chances to blit one `opengl_texture` over `dest_texture`
        if rand::random::<f64>() <= 0.016666 {
            let (left, bottom, dimensions): (f32, f32, f32) = rand::random();
            let dest_rect = glium::blit::Rect {
                left: (left * dest_texture.get_width() as f32) as u32,
                bottom: (bottom * dest_texture.get_height().unwrap() as f32) as u32,
                width: (dimensions * dest_texture.get_width() as f32) as u32,
                height: (dimensions * dest_texture.get_height().unwrap() as f32) as u32,
            };

            opengl_texture.blit_all_to(&dest_texture, &dest_rect, glium::uniforms::Linear);
        }

        // drawing a frame
        let target = display.draw();
        dest_texture.fill(&target, glium::uniforms::Linear);
        target.finish();

        // polling and handling the events received by the window
        for event in display.poll_events().into_iter() {
            match event {
                glutin::Closed => break 'main,
                _ => ()
            }
        }
    }
}
