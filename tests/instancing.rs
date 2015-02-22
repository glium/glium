extern crate glutin;

#[macro_use]
extern crate glium;

use glium::Surface;

mod support;

#[test]
fn instancing() {
    let display = support::build_display();

    let buffer1 = {
        #[derive(Copy)]
        struct Vertex {
            position: [f32; 2],
        }

        implement_vertex!(Vertex, position);

        glium::VertexBuffer::new(&display, 
            vec![
                Vertex { position: [-1.0,  1.0] },
                Vertex { position: [ 1.0,  1.0] },
                Vertex { position: [-1.0, -1.0] },
                Vertex { position: [ 1.0, -1.0] },
            ]
        )
    };

    let buffer2 = {
        #[derive(Copy)]
        struct Vertex {
            color: [f32; 3],
        }

        implement_vertex!(Vertex, color);

        match glium::vertex::PerInstanceAttributesBuffer::new_if_supported(&display, 
            vec![
                Vertex { color: [0.0, 0.0, 1.0] },
                Vertex { color: [0.0, 0.0, 1.0] },
                Vertex { color: [0.0, 0.0, 1.0] },
                Vertex { color: [1.0, 0.0, 0.0] },
            ]
        ) {
            Some(b) => b,
            None => return
        }
    };

    let index_buffer = glium::IndexBuffer::new(&display,
        glium::index::TriangleStrip(vec![0u16, 1, 2, 3]));

    let program = match glium::Program::from_source(&display,
        "
            #version 330

            in vec2 position;
            in vec3 color;

            out vec3 v_color;
            flat out int instance;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
                v_color = color;
                instance = gl_InstanceID;
            }
        ",
        "
            #version 330
            in vec3 v_color;
            flat in int instance;

            void main() {
                if (instance != 3) {
                    discard;
                }

                gl_FragColor = vec4(v_color, 1.0);
            }
        ",
        None) {
        Ok(p) => p,
        _ => return
    };

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw((&buffer1, &buffer2), &index_buffer, &program, &uniform!{},
                              &std::default::Default::default()).unwrap();

    let data: Vec<Vec<(f32, f32, f32, f32)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(1.0, 0.0, 0.0, 1.0));
        }
    }

    display.assert_no_error();
}

#[test]
fn per_instance_length_mismatch() {
    let display = support::build_display();

    let buffer1 = {
        #[derive(Copy)]
        struct Vertex {
            position: [f32; 2],
        }

        implement_vertex!(Vertex, position);

        glium::VertexBuffer::new(&display, 
            vec![
                Vertex { position: [-1.0,  1.0] },
                Vertex { position: [ 1.0,  1.0] },
                Vertex { position: [-1.0, -1.0] },
                Vertex { position: [ 1.0, -1.0] },
            ]
        )
    };

    let buffer2 = {
        #[derive(Copy)]
        struct Vertex {
            color: [f32; 3],
        }

        implement_vertex!(Vertex, color);

        match glium::vertex::PerInstanceAttributesBuffer::new_if_supported(&display, 
            vec![
                Vertex { color: [0.0, 0.0, 1.0] },
                Vertex { color: [0.0, 0.0, 1.0] },
                Vertex { color: [0.0, 0.0, 1.0] },
                Vertex { color: [1.0, 0.0, 0.0] },
            ]
        ) {
            Some(b) => b,
            None => return
        }
    };

    let buffer3 = {
        #[derive(Copy)]
        struct Vertex {
            color: [f32; 3],
        }

        implement_vertex!(Vertex, color);

        match glium::vertex::PerInstanceAttributesBuffer::new_if_supported(&display, 
            vec![
                Vertex { color: [0.0, 0.0, 1.0] },
                Vertex { color: [0.0, 0.0, 1.0] },
                Vertex { color: [0.0, 0.0, 1.0] },
            ]
        ) {
            Some(b) => b,
            None => return
        }
    };

    let index_buffer = glium::IndexBuffer::new(&display,
        glium::index::TriangleStrip(vec![0u16, 1, 2, 3]));

    let program = glium::Program::from_source(&display,
        "
            #version 110

            void main() {
                gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
            }
        ",
        "
            #version 110

            void main() {
                gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
            }
        ",
        None).unwrap();

    match display.draw().draw((&buffer1, &buffer2, &buffer3), &index_buffer, &program, &uniform!{},
                              &std::default::Default::default())
    {
        Err(glium::DrawError::InstancesCountMismatch) => (),
        a => panic!("{:?}", a)
    }

    display.assert_no_error();
}
