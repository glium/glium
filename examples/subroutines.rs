#[macro_use]
extern crate glium;

mod support;

use glium::Surface;
use glium::glutin;
use glium::index::PrimitiveType;
use glium::program::ShaderStage;

fn main() {
    // building the display, ie. the main object
    let events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new().build(&events_loop).unwrap();
    let display = glium::build(window).unwrap();

    // building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
        }

        implement_vertex!(Vertex, position);

        glium::VertexBuffer::new(&display,
            &[
                Vertex { position: [-0.5, -0.5] },
                Vertex { position: [ 0.0,  0.5] },
                Vertex { position: [ 0.5, -0.5] },
            ]
        ).unwrap()
    };

    // building the index buffer
    let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                               &[0u16, 1, 2]).unwrap();

    // compiling shaders and linking them together
    let program = program!(&display,
        150 => {
            vertex: "
                #version 150

                uniform mat4 matrix;

                in vec2 position;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0) * matrix;
                }
            ",

            fragment: "
                #version 150
                #extension GL_ARB_shader_subroutine : require

                out vec4 fragColor;
                subroutine vec4 color_t();

                subroutine uniform color_t Color;

                subroutine(color_t)
                vec4 ColorRed()
                {
                  return vec4(1, 0, 0, 1);
                }

                subroutine(color_t)
                vec4 ColorBlue()
                {
                  return vec4(0, 0.4, 1, 1);
                }

                subroutine(color_t)
                vec4 ColorYellow()
                {
                  return vec4(1, 1, 0, 1);
                }

                void main()
                {
                    fragColor = Color();
                }
            "
        },
    ).unwrap();

    let mut i = 0;
    // the main loop
    support::start_loop(|| {
        if i == 120 { i = 0; }
        let subroutine = if i % 120 < 40 {
            "ColorYellow"
        } else if i % 120 < 80{
            "ColorBlue"
        } else {
            "ColorRed"
        };

        // building the uniforms
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ],
            Color: (subroutine, ShaderStage::Fragment)
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();

        let mut action = support::Action::Continue;

        events_loop.poll_events(|event| {
            match event {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::Closed => action = support::Action::Stop,
                    _ => ()
                },
            }
        });
        i += 1;

        action
    });
}
