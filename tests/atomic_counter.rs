#[macro_use]
extern crate glium;

use glium::Surface;

mod support;

#[test]
fn basic() {
    let display = support::build_display();

    let (vb, ib) = support::build_rectangle_vb_ib(&display);

    let program = glium::Program::from_source(&display,
        "
            #version 110

            attribute vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        ",
        "
            #version 430

            out vec4 f_color;

            layout(binding = 1) uniform atomic_uint counter;

            void main() {
                f_color = vec4(0.0, 0.0, 0.0, 1.0);
                atomicCounterIncrement(counter);
            }
        ",
        None);

    // ignoring test in case of compilation error (version may not be supported)
    let program = match program {
        Ok(p) => p,
        Err(_) => return
    };

    let buffer = match glium::buffer::Buffer::new(&display, &10u32, glium::buffer::BufferType::AtomicCounterBuffer,
                                                  glium::buffer::BufferMode::Default) {
        Err(_) => return,
        Ok(b) => b
    };

    let uniforms = uniform!{
        counter: &buffer,
    };

    // Texture size 1024x1024
    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &uniforms, &Default::default()).unwrap();

    let data = buffer.read().unwrap();
    assert_eq!(data, 1024 * 1024 + 10);

    display.assert_no_error(None);
}
