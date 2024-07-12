#[macro_use]
extern crate glium;
mod support;

use std::thread;
use glium::index::PrimitiveType;
use glium::{Display, Surface};
use glutin::surface::WindowSurface;
use support::{ApplicationContext, State};
use glium::winit::keyboard::{PhysicalKey, KeyCode};

mod screenshot {
    use glium::Surface;
    use std::collections::VecDeque;
    use std::borrow::Cow;

    // Container that holds image data as vector of (u8, u8, u8, u8).
    // This is used to take data from PixelBuffer and move it to another thread
    // with minimum conversions done on main thread.
    pub struct RGBAImageData {
        pub data: Vec<(u8, u8, u8, u8)>,
        pub width: u32,
        pub height: u32,
    }

    impl glium::texture::Texture2dDataSink<(u8, u8, u8, u8)> for RGBAImageData {
        fn from_raw(data: Cow<'_, [(u8, u8, u8, u8)]>, width: u32, height: u32) -> Self {
            RGBAImageData {
                data: data.into_owned(),
                width,
                height,
            }
        }
    }

    struct AsyncScreenshotTask {
        pub target_frame: u64,
        pub pixel_buffer: glium::texture::pixel_buffer::PixelBuffer<(u8, u8, u8, u8)>,
    }

    impl AsyncScreenshotTask {
        fn new(facade: &dyn glium::backend::Facade, target_frame: u64) -> Self {
            // Get information about current framebuffer
            let dimensions = facade.get_context().get_framebuffer_dimensions();
            let rect = glium::Rect {
                left: 0,
                bottom: 0,
                width: dimensions.0,
                height: dimensions.1,
            };
            let blit_target = glium::BlitTarget {
                left: 0,
                bottom: 0,
                width: dimensions.0 as i32,
                height: dimensions.1 as i32,
            };

            // Create temporary texture and blit the front buffer to it
            let texture = glium::texture::Texture2d::empty(facade, dimensions.0, dimensions.1)
                .unwrap();
            let framebuffer = glium::framebuffer::SimpleFrameBuffer::new(facade, &texture).unwrap();
            framebuffer.blit_from_frame(&rect,
                                        &blit_target,
                                        glium::uniforms::MagnifySamplerFilter::Nearest);

            // Read the texture into new pixel buffer
            let pixel_buffer = texture.read_to_pixel_buffer();

            AsyncScreenshotTask {
                target_frame,
                pixel_buffer,
            }
        }

        fn read_image_data(self) -> RGBAImageData {
            self.pixel_buffer.read_as_texture_2d().unwrap()
        }
    }

    pub struct ScreenshotIterator<'a>(&'a mut AsyncScreenshotTaker);

    impl<'a> Iterator for ScreenshotIterator<'a> {
        type Item = RGBAImageData;

        fn next(&mut self) -> Option<RGBAImageData> {
            if self.0.screenshot_tasks.front().map(|task| task.target_frame) == Some(self.0.frame) {
                let task = self.0.screenshot_tasks.pop_front().unwrap();
                Some(task.read_image_data())
            } else {
                None
            }
        }
    }

    pub struct AsyncScreenshotTaker {
        screenshot_delay: u64,
        frame: u64,
        screenshot_tasks: VecDeque<AsyncScreenshotTask>,
    }

    impl AsyncScreenshotTaker {
        pub fn new(screenshot_delay: u64) -> Self {
            AsyncScreenshotTaker {
                screenshot_delay,
                frame: 0,
                screenshot_tasks: VecDeque::new(),
            }
        }

        pub fn next_frame(&mut self) {
            self.frame += 1;
        }

        pub fn frame(&self) -> u64 {
            self.frame
        }

        pub fn pickup_screenshots(&mut self) -> ScreenshotIterator<'_> {
            ScreenshotIterator(self)
        }

        pub fn take_screenshot(&mut self, facade: &dyn glium::backend::Facade) {
            self.screenshot_tasks
                .push_back(AsyncScreenshotTask::new(facade, self.frame + self.screenshot_delay));
        }
    }
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}
implement_vertex!(Vertex, position, color);

struct Application {
    pub vertex_buffer: glium::VertexBuffer<Vertex>,
    pub index_buffer: glium::IndexBuffer<u16>,
    pub program: glium::Program,
    pub screenshot_taker: screenshot::AsyncScreenshotTaker,
    pub take_screenshot: bool,
}

impl ApplicationContext for Application {
    const WINDOW_TITLE:&'static str = "Glium screenshot-asynchronous example - press S to take a screenshot";

    fn new(display: &Display<WindowSurface>) -> Self {
        Self {
            // building the vertex buffer
            vertex_buffer: glium::VertexBuffer::new(
                display,
                &[
                    Vertex {
                        position: [-0.5, -0.5],
                        color: [0.0, 1.0, 0.0],
                    },
                    Vertex {
                        position: [0.0, 0.5],
                        color: [0.0, 0.0, 1.0],
                    },
                    Vertex {
                        position: [0.5, -0.5],
                        color: [1.0, 0.0, 0.0],
                    },
                ],
            )
            .unwrap(),

            // building the index buffer
            index_buffer:
                glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &[0u16, 1, 2]).unwrap(),

            // compiling shaders and linking them together
            program: program!(display,
                100 => {
                    vertex: "
                        #version 100

                        uniform lowp mat4 matrix;

                        attribute lowp vec2 position;
                        attribute lowp vec3 color;

                        varying lowp vec3 vColor;

                        void main() {
                            gl_Position = vec4(position, 0.0, 1.0) * matrix;
                            vColor = color;
                        }
                    ",

                    fragment: "
                        #version 100
                        varying lowp vec3 vColor;

                        void main() {
                            gl_FragColor = vec4(vColor, 1.0);
                        }
                    ",
                },
            )
            .unwrap(),
            screenshot_taker: screenshot::AsyncScreenshotTaker::new(5),
            take_screenshot: false,
        }
    }

    fn draw_frame(&mut self, display: &Display<WindowSurface>) {
        // Tell Screenshot Taker to count next frame
        self.screenshot_taker.next_frame();

        let mut frame = display.draw();
        // For this example a simple identity matrix suffices
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ]
        };

        // Now we can draw the triangle
        frame.clear_color(0.0, 0.0, 0.0, 0.0);
        frame
            .draw(
                &self.vertex_buffer,
                &self.index_buffer,
                &self.program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
        frame.finish().unwrap();

        if self.take_screenshot {
            // Take screenshot and queue it's delivery
            self.screenshot_taker.take_screenshot(display);
            self.take_screenshot = false;
        }

        let frame = self.screenshot_taker.frame();
        // Pick up screenshots that are ready this frame
        for image_data in self.screenshot_taker.pickup_screenshots() {
            // Process and write the image in separate thread to not block the rendering thread.
            thread::spawn(move || {
                // Convert (u8, u8, u8, u8) given by glium's PixelBuffer to flat u8 required by
                // image's ImageBuffer.
                let pixels = {
                    let mut v = Vec::with_capacity(image_data.data.len() * 4);
                    for (a, b, c, d) in image_data.data {
                        v.push(a);
                        v.push(b);
                        v.push(c);
                        v.push(d);
                    }
                    v
                };

                // Create ImageBuffer
                let image_buffer =
                    image::ImageBuffer::from_raw(image_data.width, image_data.height, pixels)
                        .unwrap();

                // Save the screenshot to file
                let image = image::DynamicImage::ImageRgba8(image_buffer).flipv();
                image.save(format!("glium-example-screenshot-{frame}.png")).unwrap();
            });
        }
    }

    fn handle_window_event(&mut self, event: &glium::winit::event::WindowEvent, _window: &glium::winit::window::Window) {
        if let glium::winit::event::WindowEvent::KeyboardInput { event, ..} = event {
            if let glium::winit::event::ElementState::Pressed = event.state {
                if let PhysicalKey::Code(KeyCode::KeyS) = event.physical_key {
                    self.take_screenshot = true;
                }
            }
        }
    }
}

fn main() {
    State::<Application>::run_loop();
}
