#[macro_use]
extern crate glium;
extern crate glutin;

use glium::Surface;

mod support;

fn main() {
    use glium::DisplayBuild;

    // building the display, ie. the main object
    let display = glutin::WindowBuilder::new()
        .build_glium()
        .unwrap();

    // building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2]
        }

        implement_vertex!(Vertex, position);

        glium::VertexBuffer::new(&display, 
            vec![
                Vertex { position: [-0.005, -0.005] },
                Vertex { position: [  0.0 , 0.005] },
                Vertex { position: [ 0.005, -0.005] },
            ]
        )
    };

    // building the vertex buffer with the attributes per instance
    let per_instance = {
        #[derive(Copy, Clone)]
        struct Attr {
            world_position: [f32; 2],
        }

        implement_vertex!(Attr, world_position);

        let mut data = Vec::new();
        for x in (0u32 .. 104) {
            for y in (0u32 .. 82) {
                data.push(Attr {
                    world_position: [((x as f32) / 50.0) - 1.0, ((y as f32) / 40.0) - 1.0],
                });
            }
        }

        glium::vertex::VertexBuffer::new(&display, data)
    };

    let index_buffer = glium::IndexBuffer::new(&display,
        glium::index::TrianglesList(vec![0u16, 1, 2]));

    let program = glium::Program::from_source(&display,
        "
            #version 140

            in vec2 position;
            in vec2 world_position;

            void main() {
                gl_Position = vec4(position + world_position, 0.0, 1.0);
            }
        ",
        "
            #version 140

            out vec4 color;

            void main() {
                color = vec4(1.0, 0.0, 0.0, 1.0);
            }
        ",
        None)
        .unwrap();
    
    // the main loop
    support::start_loop(|| {
        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.draw((&vertex_buffer, per_instance.per_instance_if_supported().unwrap()),
                    &index_buffer, &program, &uniform!{},
                    &std::default::Default::default()).unwrap();
        target.finish();

        // polling and handling the events received by the window
        for event in display.poll_events() {
            match event {
                glutin::Event::Closed => return support::Action::Stop,
                _ => ()
            }
        }

        support::Action::Continue
    });
}
