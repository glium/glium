#[macro_use]
extern crate glium;
extern crate rand;

#[allow(unused_imports)]
use glium::{glutin, Surface};

mod support;

fn main() {
    println!("This example draws 10,000 instanced teapots. Each teapot gets a random position and \
              direction at initialization. Then the CPU updates and uploads the positions of each \
              teapot at each frame.");

    // building the display, ie. the main object
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    // building the vertex and index buffers
    let vertex_buffer = support::load_wavefront(&display, include_bytes!("support/teapot.obj"));
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    // list of teapots with position and direction
    let mut teapots = (0 .. 10000)
        .map(|_| {
            let pos: (f32, f32, f32) = (rand::random(), rand::random(), rand::random());
            let dir: (f32, f32, f32) = (rand::random(), rand::random(), rand::random());
            let pos = (pos.0 * 1.5 - 0.75, pos.1 * 1.5 - 0.75, pos.2 * 1.5 - 0.75);
            let dir = (dir.0 * 1.5 - 0.75, dir.1 * 1.5 - 0.75, dir.2 * 1.5 - 0.75);
            (pos, dir)
        })
        .collect::<Vec<_>>();

    // building the vertex buffer with the attributes per instance
    let mut per_instance = {
        #[derive(Copy, Clone)]
        struct Attr {
            world_position: (f32, f32, f32),
        }

        implement_vertex!(Attr, world_position);

        let data = teapots.iter().map(|_| {
            Attr {
                world_position: (0.0, 0.0, 0.0),
            }
        }).collect::<Vec<_>>();

        glium::vertex::VertexBuffer::dynamic(&display, &data).unwrap()
    };

    let program = glium::Program::from_source(&display,
        "
            #version 140

            in vec3 position;
            in vec3 normal;
            in vec3 world_position;
            out vec3 v_position;
            out vec3 v_normal;
            out vec3 v_color;

            void main() {
                v_position = position;
                v_normal = normal;
                v_color = vec3(float(gl_InstanceID) / 10000.0, 1.0, 1.0);
                gl_Position = vec4(position * 0.0005 + world_position, 1.0);
            }
        ",
        "
            #version 140

            in vec3 v_normal;
            in vec3 v_color;
            out vec4 f_color;

            const vec3 LIGHT = vec3(-0.2, 0.8, 0.1);

            void main() {
                float lum = max(dot(normalize(v_normal), normalize(LIGHT)), 0.0);
                vec3 color = (0.3 + 0.7 * lum) * v_color;
                f_color = vec4(color, 1.0);
            }
        ",
        None)
        .unwrap();

    let camera = support::camera::CameraState::new();
    
    // the main loop
    support::start_loop(event_loop, move |events| {
        // updating the teapots
        {
            let mut mapping = per_instance.map();
            for (src, dest) in teapots.iter_mut().zip(mapping.iter_mut()) {
                (src.0).0 += (src.1).0 * 0.001;
                (src.0).1 += (src.1).1 * 0.001;
                (src.0).2 += (src.1).2 * 0.001;

                dest.world_position = src.0;
            }
        }

        // drawing a frame
        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            .. Default::default()
        };

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        target.draw((&vertex_buffer, per_instance.per_instance().unwrap()),
                    &indices, &program, &uniform! { matrix: camera.get_perspective() },
                    &params).unwrap();
        target.finish().unwrap();

        let mut action = support::Action::Continue;

        // polling and handling the events received by the window
        for event in events {
            match event {
                glutin::event::Event::WindowEvent { event, .. } => match event {
                    glutin::event::WindowEvent::CloseRequested => action = support::Action::Stop,
                    _ => (),
                },
                _ => (),
            }
        };

        action
    });
}
