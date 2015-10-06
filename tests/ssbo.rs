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
            buffer layout(std140);

            out vec4 f_color;

            buffer MyBlock {
                vec3 color;
            };

            void main() {
                color = vec3(1.0, 1.0, 0.5);
                f_color = vec4(0.0, 0.0, 0.0, 1.0);
            }
        ",
        None);

    // ignoring test in case of compilation error (version may not be supported)
    let program = match program {
        Ok(p) => p,
        Err(_) => return
    };

    #[derive(Copy, Clone)]
    struct Data {
        color: (f32, f32, f32),
    }

    implement_uniform_block!(Data, color);

    let buffer = match glium::uniforms::UniformBuffer::new(&display, Data { color: (0.0f32, 0.0f32, 0.0f32) }) {
        Err(_) => return,
        Ok(b) => b
    };

    let uniforms = uniform!{
        MyBlock: &buffer
    };

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &uniforms, &Default::default()).unwrap();

    let data = buffer.read().unwrap();
    assert_eq!(data.color, (1.0, 1.0, 0.5));

    display.assert_no_error(None);
}

#[test]
fn custom_bind_point() {
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
            buffer layout(std140);

            out vec4 f_color;

            layout(binding = 2) buffer MyBlock {
                vec3 color;
            };

            void main() {
                color = vec3(1.0, 1.0, 0.5);
                f_color = vec4(0.0, 0.0, 0.0, 1.0);
            }
        ",
        None);

    // ignoring test in case of compilation error (version may not be supported)
    let program = match program {
        Ok(p) => p,
        Err(_) => return
    };

    #[derive(Copy, Clone)]
    struct Data {
        color: (f32, f32, f32),
    }

    implement_uniform_block!(Data, color);

    let buffer = match glium::uniforms::UniformBuffer::new(&display, Data { color: (0.0f32, 0.0f32, 0.0f32) }) {
        Err(_) => return,
        Ok(b) => b
    };

    let uniforms = uniform!{
        MyBlock: &buffer
    };

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &uniforms, &Default::default()).unwrap();

    let data = buffer.read().unwrap();
    assert_eq!(data.color, (1.0, 1.0, 0.5));

    display.assert_no_error(None);
}
