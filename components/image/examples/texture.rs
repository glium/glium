#![feature(phase)]

extern crate glutin;
extern crate image;
extern crate glium_core;
extern crate glium_image;
extern crate glium_sprite2d;

fn main() {
    use glium_core::DisplayBuild;

    let display = glutin::WindowBuilder::new().build_glium_core().unwrap();
    let sprite2d_sys = glium_sprite2d::Sprite2DSystem::new(&display);

    let texture: glium_core::Texture = glium_image::ImageLoad::from_image(&display, {
        use std::io::BufReader;
        static TEXTURE_DATA: &'static [u8] = include_bin!("texture.png");
        BufReader::new(TEXTURE_DATA)
    });

    'main: loop {
        use std::io::timer;
        use std::time::Duration;

        display.draw().draw(glium_sprite2d::SpriteDisplay {
            sprite: &sprite2d_sys,
            texture: &texture,
            matrix: &[
                [ 1.0, 0.0, 0.0, 0.0 ],
                [ 0.0, 1.0, 0.0, 0.0 ],
                [ 0.0, 0.0, 1.0, 0.0 ],
                [ 0.0, 0.0, 0.0, 1.0 ]
            ]
        });

        timer::sleep(Duration::milliseconds(17));

        for event in display.poll_events().into_iter() {
            match event {
                glutin::Closed => break 'main,
                _ => ()
            }
        }
    }
}
