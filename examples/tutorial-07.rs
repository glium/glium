#[macro_use]
extern crate glium;
use glium::Surface;

#[path = "../book/tuto-07-teapot.rs"]
mod teapot;

fn main() {
    let event_loop = winit::event_loop::EventLoopBuilder::new().build();
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("Glium tutorial #7")
        .build(&event_loop);

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

    event_loop.run(move |ev, _, control_flow| {
        match ev {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow =  winit::event_loop::ControlFlow::Exit;
                },
                // Because glium doesn't know about windows we need to resize the display
                // when the window's size has changed.
                winit::event::WindowEvent::Resized(window_size) => {
                    display.resize(window_size.into());
                },
                _ => (),
            },
            // We now need to render everyting in response to a RedrawRequested event due to the animation
            winit::event::Event::RedrawRequested(_) => {
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
            },
            // By requesting a redraw in response to a RedrawEventsCleared event we get continuous rendering.
            // For applications that only change due to user input you could remove this handler.
            winit::event::Event::RedrawEventsCleared => {
                window.request_redraw();
            },
            _ => (),
        }
    });
}
