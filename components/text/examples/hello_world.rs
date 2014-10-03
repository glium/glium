#![feature(phase)]

extern crate glutin;
extern crate glium_core;
extern crate glium_text;

fn main() {
    use glium_core::DisplayBuild;

    let display = glutin::WindowBuilder::new().with_dimensions(1024, 768).build_glium_core().unwrap();
    let system = glium_text::TextSystem::new(&display);

    let font = glium_text::FontTexture::new(&display, std::io::BufReader::new(include_bin!("font.ttf")), 24).unwrap();

    let text = glium_text::TextDisplay::new(&system, &font, "Hello world!");

    'main: loop {
        use std::io::timer;
        use std::time::Duration;

        let (w, h) = (1024.0f32, 768.0f32);

        let matrix = [
            [0.5, 0.0, 0.0, 0.0],
            [0.0, 0.5 * h / w, 0.0, 0.0],
            [0.0, 0.0, 0.5, 0.0],
            [0.0, 0.0, 0.0, 1.0f32],
        ];

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.draw(glium_text::DrawCommand(&text, matrix, [1.0, 1.0, 0.0, 1.0]));
        target.finish();

        timer::sleep(Duration::milliseconds(17));

        for event in display.poll_events().into_iter() {
            match event {
                glutin::Closed => break 'main,
                _ => ()
            }
        }
    }
}
