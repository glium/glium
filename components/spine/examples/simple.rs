#![feature(phase)]

#[phase(plugin)]
extern crate resources_package;

extern crate cgmath;
extern crate glutin;
extern crate resources_package_package;
extern crate glium_core;
extern crate glium_image;
extern crate glium_spine;
extern crate glium_sprite2d;
extern crate time;

use std::io::BufReader;

static SPINE: resources_package_package::Package = resources_package!("document");

fn main() {
    use glium_core::DisplayBuild;

    let display = glutin::WindowBuilder::new().with_dimensions(900, 900).build_glium_core().unwrap();
    let system = glium_sprite2d::Sprite2DSystem::new(&display);

    let spine_doc = glium_spine::Spine::new(
        BufReader::new(SPINE.find(&Path::new("speedy.json")).unwrap()),
        |name| glium_image::ImageLoad::from_image(&display,
        BufReader::new(SPINE.find(&Path::new(format!("{}.png", name))).unwrap())));

    let animations_list = spine_doc.get_animations_list().into_iter().map(|e| Some(e))
        .chain((vec![None]).into_iter()).collect::<Vec<_>>();

    let mut current_animation = 0;

    println!("Press space to change the current animation");
    println!("Now playing {}", animations_list[current_animation]);

    'main: loop {
        use std::io::timer;
        use std::time::Duration;

        let matrix = cgmath::Matrix4::new(0.0015f32, 0.0, 0.0, 0.0, 0.0, 0.0015, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0);

        let time = (time::precise_time_ns() / 1000000) as f32 / 1000.0;

        let mut target = display.draw();
        target.clear_color(0.5, 0.5, 0.5, 1.0);
        target.draw(glium_spine::SpineDraw(&spine_doc, &system, "default", animations_list[current_animation], time, &matrix));
        target.finish();

        timer::sleep(Duration::milliseconds(17));

        for event in display.poll_events().into_iter() {
            match event {
                glutin::KeyboardInput(glutin::Pressed, _, Some(glutin::Space), _) => {
                    current_animation += 1;
                    current_animation %= animations_list.len();
                    println!("Now playing {}", animations_list[current_animation]);
                },
                glutin::Closed => break 'main,
                _ => ()
            }
        }
    }
}
