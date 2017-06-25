#[macro_use]
extern crate glium;

fn main() {
    use glium::Surface;
    use glium::glutin::{self, winit};

    let mut events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new().build(&events_loop).unwrap();
    let context = glutin::ContextBuilder::new().build(&window).unwrap();
    let display = glium::Display::new(window, context).unwrap();

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }

    implement_vertex!(Vertex, position);

    let vertex1 = Vertex { position: [-0.5, -0.5] };
    let vertex2 = Vertex { position: [ 0.0,  0.5] };
    let vertex3 = Vertex { position: [ 0.5, -0.25] };
    let shape = vec![vertex1, vertex2, vertex3];

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let vertex_shader_src = r#"
        #version 140

        in vec2 position;

        void main() {
            gl_Position = vec4(position, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        out vec4 color;

        void main() {
            color = vec4(1.0, 0.0, 0.0, 1.0);
        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    loop {
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        target.draw(&vertex_buffer, &indices, &program, &glium::uniforms::EmptyUniforms,
                    &Default::default()).unwrap();
        target.finish().unwrap();

        let mut closed = false;
        events_loop.poll_events(|event| {
            match event {
                winit::Event::WindowEvent { event, .. } => match event {
                    winit::WindowEvent::Closed => closed = true,
                    _ => ()
                },
                _ => (),
            }
        });

        if closed { break; }
    }
}
