#[macro_use]
extern crate glium;
extern crate image;

use std::thread;

#[allow(unused_imports)]
use glium::{glutin, Surface};
use glium::index::PrimitiveType;


mod screenshot {
    use glium::Surface;
    use std::collections::VecDeque;
    use std::vec::Vec;
    use std::borrow::Cow;

    use glium;

    // Container that holds image data as vector of (u8, u8, u8, u8).
    // This is used to take data from PixelBuffer and move it to another thread
    // with minimum conversions done on main thread.
    pub struct RGBAImageData {
        pub data: Vec<(u8, u8, u8, u8)>,
        pub width: u32,
        pub height: u32,
    }

    impl glium::texture::Texture2dDataSink<(u8, u8, u8, u8)> for RGBAImageData {
        fn from_raw(data: Cow<[(u8, u8, u8, u8)]>, width: u32, height: u32) -> Self {
            RGBAImageData {
                data: data.into_owned(),
                width: width,
                height: height,
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
                target_frame: target_frame,
                pixel_buffer: pixel_buffer,
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
                screenshot_delay: screenshot_delay,
                frame: 0,
                screenshot_tasks: VecDeque::new(),
            }
        }

        pub fn next_frame(&mut self) {
            self.frame += 1;
        }

        pub fn pickup_screenshots(&mut self) -> ScreenshotIterator {
            ScreenshotIterator(self)
        }

        pub fn take_screenshot(&mut self, facade: &dyn glium::backend::Facade) {
            self.screenshot_tasks
                .push_back(AsyncScreenshotTask::new(facade, self.frame + self.screenshot_delay));
        }
    }
}

fn main() {
    // building the display, ie. the main object
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_title("Press S to take screenshot");
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    // building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
            color: [f32; 3],
        }

        implement_vertex!(Vertex, position, color);

        glium::VertexBuffer::new(&display,
                                 &[Vertex {
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
                                   }])
            .unwrap()
    };

    // building the index buffer
    let index_buffer =
        glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList, &[0u16, 1, 2]).unwrap();

    // compiling shaders and linking them together
    let program = program!(&display,
        140 => {
            vertex: "
                #version 140

                uniform mat4 matrix;

                in vec2 position;
                in vec3 color;

                out vec3 vColor;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0) * matrix;
                    vColor = color;
                }
            ",

            fragment: "
                #version 140
                in vec3 vColor;
                out vec4 f_color;

                void main() {
                    f_color = vec4(vColor, 1.0);
                }
            "
        },

        110 => {
            vertex: "
                #version 110

                uniform mat4 matrix;

                attribute vec2 position;
                attribute vec3 color;

                varying vec3 vColor;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0) * matrix;
                    vColor = color;
                }
            ",

            fragment: "
                #version 110
                varying vec3 vColor;

                void main() {
                    gl_FragColor = vec4(vColor, 1.0);
                }
            ",
        },
    )
        .unwrap();

    // drawing once

    // building the uniforms
    let uniforms = uniform! {
        matrix: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0f32]
        ]
    };

    // The parameter sets the amount of frames between requesting the image
    // transfer and picking it up. If the value is too small, the main thread
    // will block waiting for the image to finish transferring. Tune it based on
    // your requirements.
    let mut screenshot_taker = screenshot::AsyncScreenshotTaker::new(5);

    event_loop.run(move |event, _, control_flow| {

        // React to events
        use glium::glutin::event::{Event, WindowEvent, ElementState, VirtualKeyCode};

        let mut take_screenshot = false;

        let next_frame_time = std::time::Instant::now() +
            std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                WindowEvent::KeyboardInput { input, .. } => {
                    if let ElementState::Pressed = input.state {
                        if let Some(VirtualKeyCode::S) = input.virtual_keycode {
                            take_screenshot = true;
                        }
                    }
                },
                _ => return,
            },
            Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }

        // Tell Screenshot Taker to count next frame
        screenshot_taker.next_frame();

        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.draw(&vertex_buffer,
                  &index_buffer,
                  &program,
                  &uniforms,
                  &Default::default())
            .unwrap();
        target.finish().unwrap();

        if take_screenshot {
            // Take screenshot and queue it's delivery
            screenshot_taker.take_screenshot(&display);
        }

        // Pick up screenshots that are ready this frame
        for image_data in screenshot_taker.pickup_screenshots() {
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
                image.save("glium-example-screenshot.png").unwrap();
            });
        }
    });
}
