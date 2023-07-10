#[macro_use]
extern crate glium;

#[allow(unused_imports)]
use glium::{Display, Frame, Surface};
use glutin::surface::WindowSurface;
use support::{camera::CameraState, ApplicationContext, State};

mod support;

#[derive(Copy, Clone)]
struct Attr {
    world_position: (f32, f32, f32),
}
implement_vertex!(Attr, world_position);

struct Application {
    pub vertex_buffer: glium::vertex::VertexBufferAny,
    pub teapots: Vec<((f32, f32, f32), (f32, f32, f32))>,
    pub per_instance: glium::VertexBuffer<Attr>,
    pub camera: CameraState,
    pub program: glium::Program,
}

impl ApplicationContext for Application {
    const WINDOW_TITLE:&'static str = "Glium instancing example";

    fn new(display: &Display<WindowSurface>) -> Self {
        // building the vertex and index buffers
        let vertex_buffer = support::load_wavefront(display, include_bytes!("support/teapot.obj"));

        // list of teapots with position and direction
        let teapots = (0..10000)
            .map(|_| {
                let pos: (f32, f32, f32) = (rand::random(), rand::random(), rand::random());
                let dir: (f32, f32, f32) = (rand::random(), rand::random(), rand::random());
                let pos = (pos.0 * 1.5 - 0.75, pos.1 * 1.5 - 0.75, pos.2 * 1.5 - 0.75);
                let dir = (dir.0 * 1.5 - 0.75, dir.1 * 1.5 - 0.75, dir.2 * 1.5 - 0.75);
                (pos, dir)
            })
            .collect::<Vec<_>>();

        // building the vertex buffer with the attributes per instance
        let per_instance = {
            let data = teapots
                .iter()
                .map(|_| Attr {
                    world_position: (0.0, 0.0, 0.0),
                })
                .collect::<Vec<_>>();

            glium::vertex::VertexBuffer::dynamic(display, &data).unwrap()
        };

        let program = glium::Program::from_source(
            display,
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
            None,
        )
        .unwrap();

        let camera = support::camera::CameraState::new();

        Self {
            vertex_buffer,
            teapots,
            per_instance,
            program,
            camera,
        }
    }

    fn draw_frame(&mut self, display: &Display<WindowSurface>) {
        let mut frame = display.draw();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        // drawing a frame
        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };

        frame.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        frame
            .draw(
                (
                    &self.vertex_buffer,
                    self.per_instance.per_instance().unwrap(),
                ),
                &indices,
                &self.program,
                &uniform! { matrix: self.camera.get_perspective() },
                &params,
            )
            .unwrap();
        frame.finish().unwrap();
    }

    fn handle_window_event(&mut self, event: &winit::event::WindowEvent, _window: &winit::window::Window) {
        self.camera.process_input(&event);
    }

    fn update(&mut self) {
        self.camera.update();
        let mut mapping = self.per_instance.map();
        for (src, dest) in self.teapots.iter_mut().zip(mapping.iter_mut()) {
            (src.0).0 += (src.1).0 * 0.001;
            (src.0).1 += (src.1).1 * 0.001;
            (src.0).2 += (src.1).2 * 0.001;

            dest.world_position = src.0;
        }
    }
}

fn main() {
    println!(
        "This example draws 10,000 instanced teapots. Each teapot gets a random position and \
              direction at initialization. Then the CPU updates and uploads the positions of each \
              teapot at each frame."
    );
    State::<Application>::run_loop();
}
