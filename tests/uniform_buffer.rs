#[macro_use]
extern crate glium;
extern crate rand;

use glium::Surface;

mod support;

#[test]
fn uniform_buffer_creation() {
    let display = support::build_display();

    let _ = glium::uniforms::UniformBuffer::new(&display, 12);

    display.assert_no_error(None);
}

#[test]
fn uniform_buffer_mapping_read() {
    let display = support::build_display();

    let mut vb = match glium::uniforms::UniformBuffer::new(&display, 12) {
        Err(_) => return,
        Ok(b) => b
    };

    let mapping = vb.map();
    assert_eq!(*mapping, 12);

    display.assert_no_error(None);
}

#[test]
fn uniform_buffer_mapping_write() {
    let display = support::build_display();

    let mut vb = match glium::uniforms::UniformBuffer::new(&display, 6) {
        Err(_) => return,
        Ok(b) => b
    };

    {
        let mut mapping = vb.map();
        *mapping = 15;
    }

    let mapping = vb.map();
    assert_eq!(*mapping, 15);

    display.assert_no_error(None);
}

#[test]
fn uniform_buffer_read() {
    let display = support::build_display();

    let vb = match glium::uniforms::UniformBuffer::new(&display, 12) {
        Err(_) => return,
        Ok(b) => b
    };

    let data = match vb.read() {
        Ok(r) => r,
        Err(glium::buffer::ReadError::NotSupported) => return,
        e => e.unwrap()
    };

    assert_eq!(data, 12);

    display.assert_no_error(None);
}

#[test]
fn uniform_buffer_write() {
    let display = support::build_display();

    let vb = match glium::uniforms::UniformBuffer::new(&display, 5) {
        Err(_) => return,
        Ok(b) => b
    };

    vb.write(&24);

    let data = match vb.read() {
        Ok(r) => r,
        Err(glium::buffer::ReadError::NotSupported) => return,
        e => e.unwrap()
    };

    assert_eq!(data, 24);

    display.assert_no_error(None);
}

#[test]
fn block() {
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
            #version 330
            uniform layout(std140);

            uniform MyBlock {
                vec3 color;
            };

            void main() {
                gl_FragColor = vec4(color, 1.0);
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

    let buffer = match glium::uniforms::UniformBuffer::new(&display, Data { color: (1.0f32, 1.0f32, 0.0f32) }) {
        Err(_) => return,
        Ok(b) => b
    };

    let uniforms = uniform!{
        MyBlock: &buffer
    };

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &ib, &program, &uniforms, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 255, 0, 255));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn block_wrong_type() {
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
            #version 330
            uniform layout(std140);

            uniform MyBlock {
                vec3 color;
            };

            void main() {
                gl_FragColor = vec4(color, 1.0);
            }
        ",
        None);

    // ignoring test in case of compilation error (version may not be supported)
    let program = match program {
        Ok(p) => p,
        Err(_) => return
    };

    let buffer = match glium::uniforms::UniformBuffer::new(&display, 2) {
        Err(_) => return,
        Ok(b) => b
    };

    let uniforms = uniform!{
        MyBlock: &buffer
    };

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);

    match target.draw(&vb, &ib, &program, &uniforms, &Default::default()) {
        Err(glium::DrawError::UniformBlockLayoutMismatch { ref name, .. })
            if name == &"MyBlock" => (),
        a => panic!("{:?}", a)
    }

    target.finish().unwrap();

    display.assert_no_error(None);
}

#[test]
fn buffer_write() {
    let display = support::build_display();

    let mut buf = match glium::uniforms::UniformBuffer::new(&display, (5, 3)) {
        Err(_) => return,
        Ok(b) => b
    };

    {
        let mut mapping = buf.map();
        mapping.1 = 8;
    }

    let mapping = buf.map();
    assert_eq!(*mapping, (5, 8));

    display.assert_no_error(None);
}

#[test]
fn persistent_block_race_condition() {
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
            #version 330
            uniform layout(std140);

            uniform MyBlock {
                vec3 color;
            };

            void main() {
                gl_FragColor = vec4(color, 1.0);
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

    let mut buffer = match glium::uniforms::UniformBuffer::new(&display, Data { color: (0.5f32, 0.5f32, 0.5f32) }) {
        Err(_) => return,
        Ok(b) => b
    };

    // checking for synchronization issues by quickly drawing and modifying the buffer
    let texture = support::build_renderable_texture(&display);
    let mut target = texture.as_surface();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    for _ in 0 .. 1000 {
        {
            let mut mapping = buffer.map();
            mapping.color.0 = rand::random();
            mapping.color.1 = rand::random();
            mapping.color.2 = rand::random();
        }

        target.draw(&vb, &ib, &program, &uniform!{
            MyBlock: &buffer
        }, &Default::default()).unwrap();
    }
    {
        let mut mapping = buffer.map();
        mapping.color.0 = 1.0;
        mapping.color.1 = 1.0;
        mapping.color.2 = 1.0;
    }
    target.draw(&vb, &ib, &program, &uniform!{
        MyBlock: &buffer
    }, &Default::default()).unwrap();
    {
        let mut mapping = buffer.map();
        mapping.color.0 = 0.0;
        mapping.color.1 = 0.0;
        mapping.color.2 = 0.0;
    }

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 255, 255, 255));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn empty_uniform_buffer() {
    let display = support::build_display();

    let _ = match glium::uniforms::UniformBuffer::new(&display, ()) {
        Err(_) => return,
        Ok(b) => b
    };

    display.assert_no_error(None);
}
