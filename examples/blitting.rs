extern crate glutin;
extern crate glium;
extern crate image;

use std::io::BufReader;
use std::rand;

use glium::{DisplayBuild, Texture, Surface, Rect};

fn main() {
    // building the display, ie. the main object
    let display = glutin::WindowBuilder::new()
        .with_vsync()
        .build_glium()
        .unwrap();

    // building a texture with "OpenGL" drawn on it
    let image = image::load(BufReader::new(include_bin!("../tests/fixture/opengl.png")),
        image::PNG).unwrap();
    let opengl_texture = glium::Texture2D::new(&display, image);

    // building a 1024x1024 black texture
    let dest_texture = glium::Texture2D::new_empty::<(u8,u8,u8,u8)>(&display, 1024, 1024);

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
                glium::uniforms::Linear);
        }

        // drawing a frame
        let target = display.draw();
        dest_texture.as_surface().fill(&target, glium::uniforms::Linear);
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
