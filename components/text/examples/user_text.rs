#![feature(phase)]

extern crate glutin;
extern crate glium_core;
extern crate glium_text;

fn main() {
    use glium_core::DisplayBuild;
    use std::io::File;

    let display = glutin::WindowBuilder::new().with_dimensions(1024, 768).build_glium_core().unwrap();
    let system = glium_text::TextSystem::new(&display);

    let font = match std::os::args().into_iter().nth(1) {
        Some(file) => glium_text::FontTexture::new(&display, File::open(&Path::new(file)), 48),
        None => {
            match File::open(&Path::new("C:\\Windows\\Fonts\\Arial.ttf")) {
                Ok(f) => glium_text::FontTexture::new(&display, f, 48),
                Err(_) => glium_text::FontTexture::new(&display, std::io::BufReader::new(include_bin!("font.ttf")), 48),
            }
        }
    }.unwrap();

    let mut buffer = String::new();

    println!("Type with your keyboard");

    'main: loop {
        use std::io::timer;
        use std::time::Duration;

        let text = glium_text::TextDisplay::new(&system, &font, buffer.as_slice());

        let (w, h) = (1024.0f32, 768.0f32);

        let matrix = [
            [2.0, 0.0, 0.0, -0.5],
            [0.0, 2.0 * h / w, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0f32],
        ];

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.draw(glium_text::DrawCommand(&text, matrix, [1.0, 1.0, 0.0, 1.0]));
        target.finish();

        timer::sleep(Duration::milliseconds(17));

        for event in display.poll_events().into_iter() {
            match event {
                glutin::ReceivedCharacter('\r') => buffer.clear(),
                glutin::ReceivedCharacter(c) if c as u32 == 8 => { buffer.pop(); },
                glutin::ReceivedCharacter(chr) => buffer.push(chr),
                glutin::Closed => break 'main,
                _ => ()
            }
        }
    }
}
