#[macro_use]
extern crate glium;

#[path = "../book/tuto-07-teapot.rs"]
mod teapot;

fn main() {
    use glium::Surface;
    use glium::glutin::{self, winit};

    let mut events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new().build(&events_loop).unwrap();
    let context = glutin::ContextBuilder::new().build(&window).unwrap();
    let display = glium::Display::new(window, context).unwrap();

    let positions = glium::VertexBuffer::new(&display, &teapot::VERTICES).unwrap();
    let normals = glium::VertexBuffer::new(&display, &teapot::NORMALS).unwrap();
    let indices = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList,
                                          &teapot::INDICES).unwrap();

    let vertex_shader_src = r#"
        #version 140

        in vec3 position;
        in vec3 normal;

        uniform mat4 matrix;

        void main() {
            gl_Position = matrix * vec4(position, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        out vec4 color;

        void main() {
            color = vec4(1.0, 0.0, 0.0, 1.0);
        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src,
                                              None).unwrap();

    loop {
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        let matrix = [
            [0.01, 0.0, 0.0, 0.0],
            [0.0, 0.01, 0.0, 0.0],
            [0.0, 0.0, 0.01, 0.0],
            [0.0, 0.0, 0.0, 1.0f32]
        ];

        target.draw((&positions, &normals), &indices, &program, &uniform! { matrix: matrix },
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
