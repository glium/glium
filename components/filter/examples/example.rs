#![feature(phase)]

extern crate glutin;
extern crate glium_core;
extern crate glium_filter;
extern crate glium_image;
extern crate glium_sprite2d;

fn main() {
    use glium_core::DisplayBuild;

    let display = glutin::WindowBuilder::new().build_glium_core().unwrap();
    let sprite2d_sys = glium_sprite2d::Sprite2DSystem::new(&display);

    let filter = glium_filter::Filter::new(&display, "
        #version 110
        uniform sampler2D uTexture;
        varying vec2 vTexCoords;

        const float blurSize = 4.0 / 512.0;

        void main() {
            vec4 sum = vec4(0.0);

            sum += texture2D(uTexture, vec2(vTexCoords.x - 4.0 * blurSize, vTexCoords.y)) * 0.05;
            sum += texture2D(uTexture, vec2(vTexCoords.x - 3.0 * blurSize, vTexCoords.y)) * 0.09;
            sum += texture2D(uTexture, vec2(vTexCoords.x - 2.0 * blurSize, vTexCoords.y)) * 0.12;
            sum += texture2D(uTexture, vec2(vTexCoords.x - blurSize, vTexCoords.y)) * 0.15;
            sum += texture2D(uTexture, vec2(vTexCoords.x, vTexCoords.y)) * 0.16;
            sum += texture2D(uTexture, vec2(vTexCoords.x + blurSize, vTexCoords.y)) * 0.15;
            sum += texture2D(uTexture, vec2(vTexCoords.x + 2.0 * blurSize, vTexCoords.y)) * 0.12;
            sum += texture2D(uTexture, vec2(vTexCoords.x + 3.0 * blurSize, vTexCoords.y)) * 0.09;
            sum += texture2D(uTexture, vec2(vTexCoords.x + 4.0 * blurSize, vTexCoords.y)) * 0.05;

            gl_FragColor = sum;
        }
    ");

    let texture: glium_core::Texture = glium_image::ImageLoad::from_image(&display, {
        use std::io::BufReader;
        static TEXTURE_DATA: &'static [u8] = include_bin!("texture.png");
        BufReader::new(TEXTURE_DATA)
    });

    'main: loop {
        use std::io::timer;
        use std::time::Duration;

        display.draw().draw(glium_filter::WithFilter(&filter, |target| {
            target.draw(glium_sprite2d::SpriteDisplay {
                sprite: &sprite2d_sys,
                texture: &texture,
                matrix: &[
                    [ 1.0, 0.0, 0.0, 0.0 ],
                    [ 0.0, 1.0, 0.0, 0.0 ],
                    [ 0.0, 0.0, 1.0, 0.0 ],
                    [ 0.0, 0.0, 0.0, 1.0 ]
                ]
            });
        }));

        timer::sleep(Duration::milliseconds(17));

        for event in display.poll_events().into_iter() {
            match event {
                glutin::Closed => break 'main,
                _ => ()
            }
        }
    }
}
