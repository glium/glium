/*!
Test supports module.

*/

#![allow(dead_code)]

use glium::{self, Display};
use glium::backend::Facade;
use glium::index::PrimitiveType;

use std::num::NonZeroU32;
use winit::event::Event;
use winit::event_loop::{EventLoopBuilder, EventLoopProxy};
use winit::window::{Window, WindowBuilder};
use glutin::config::{Config, ConfigTemplateBuilder};
use glutin::context::{ContextAttributesBuilder};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;

// There is a Wayland version of this extension trait but the X11 version also works on Wayland
#[cfg(unix)]
use winit::platform::x11::EventLoopBuilderExtX11;
#[cfg(windows)]
use winit::platform::windows::EventLoopBuilderExtWindows;

use std::env;
use std::thread;
use std::sync::{mpsc::Receiver, Mutex, Once, RwLock};

/// Builds a display for tests.
pub fn build_display() -> WindowedDisplay {

    // Thread communication
    static mut EVENT_LOOP_PROXY: RwLock<Option<EventLoopProxy<()>>> = RwLock::new(None);
    static mut WINDOW_RECEIVER: Mutex<Option<Receiver<(Window, Config)>>> = Mutex::new(None);
    // Initialization
    static mut INIT_EVENT_LOOP: Once = Once::new();
    static mut SEND_PROXY: Once = Once::new();

    // SAFETY
    // This is the first code to run when any test thread calls build_display.
    // It spawns the event loop in a new thread and sets up the communication channels
    //   so that the static mut variables are only ever read after initialization.
    unsafe {
        INIT_EVENT_LOOP.call_once(|| {

            // One-time-use channel to get the event loop proxy
            let (ots, otr) = std::sync::mpsc::sync_channel(0);
            // Transfers window and config for creating display
            let (sender, receiver) = std::sync::mpsc::channel();

            let builder = thread::Builder::new().name("event_loop".into());
            builder.spawn(|| {

                let event_loop = if cfg!(unix) || cfg!(windows) {
                    EventLoopBuilder::new().with_any_thread(true).build()
                } else {
                    EventLoopBuilder::new().build()
                };
                let proxy = event_loop.create_proxy();

                event_loop.run(move |event, event_loop, _| {
                    match event {
                        Event::UserEvent(_) => {
                            let window_builder = WindowBuilder::new().with_visible(false);

                            let config_template_builder = ConfigTemplateBuilder::new();
                            let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));
                            let (window, gl_config) = display_builder
                                .build(&event_loop, config_template_builder, |mut configs| {
                                    // Just use the first configuration since we don't have any special preferences here
                                    configs.next().unwrap()
                                })
                                .unwrap();

                            sender.send((window.unwrap(), gl_config)).unwrap();
                        }
                        _ => {
                            // Send event loop proxy ASAP
                            SEND_PROXY.call_once(|| {
                                ots.send(proxy.clone()).unwrap();
                            });
                        }
                    }
                });
            }).unwrap();

            // `recv` will block until any non-user event is sent
            let event_loop_proxy = otr.recv().unwrap();

            // Write to the static mut variables while still in `call_once`'s closure
            *EVENT_LOOP_PROXY.write().unwrap() = Some(event_loop_proxy);

            *WINDOW_RECEIVER.lock().unwrap() = Some(receiver);
        });
    }

    // Tell event loop to create a window and config for creating a display
    let guard = unsafe {
        EVENT_LOOP_PROXY.read().unwrap()
    };
    guard.as_ref().unwrap().send_event(()).unwrap();

    // Receive said window and config one thread at a time
    let (window, gl_config) = {
        let guard = unsafe {
            WINDOW_RECEIVER.lock().unwrap()
        };
        guard.as_ref().unwrap().recv().unwrap()
    };

    // Then the configuration which decides which OpenGL version we'll end up using, here we just use the default which is currently 3.3 core
    // When this fails we'll try and create an ES context, this is mainly used on mobile devices or various ARM SBC's
    // If you depend on features available in modern OpenGL Versions you need to request a specific, modern, version. Otherwise things will very likely fail.
    let version = parse_version();
    let raw_window_handle = window.raw_window_handle();
    let context_attributes = ContextAttributesBuilder::new()
        .with_context_api(version)
        .build(Some(raw_window_handle));

    let not_current_gl_context = unsafe {
        gl_config.display().create_context(&gl_config, &context_attributes).unwrap()
    };

    let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        raw_window_handle,
        NonZeroU32::new(800).unwrap(),
        NonZeroU32::new(600).unwrap(),
    );

    // Now we can create our surface, use it to make our context current and finally create our display
    let surface = unsafe { gl_config.display().create_window_surface(&gl_config, &attrs).unwrap() };
    let current_context = not_current_gl_context.make_current(&surface).unwrap();

    let display = Display::from_context_surface(current_context, surface).unwrap();

    WindowedDisplay { window, display }
}

// Keep the Window alive via ownership until the test completes
pub struct WindowedDisplay {
    window: Window,
    display: Display<WindowSurface>,
}

// `Deref` abuse but private and localised to the tests
impl std::ops::Deref for WindowedDisplay {
    type Target = Display<WindowSurface>;

    fn deref(&self) -> &Self::Target {
        &self.display
    }
}

impl Facade for WindowedDisplay {
    fn get_context(&self) -> &std::rc::Rc<glium::backend::Context> {
        self.display.get_context()
    }
}


/// Rebuilds an existing display.
///
/// In real applications this is used for things such as switching to fullscreen. Some things are
/// invalidated during a rebuild, and this has to be handled by glium.
pub fn rebuild_display(_display: &glium::Display<WindowSurface>) {
    todo!();
    /*
    let version = parse_version();
    let event_loop = winit::event_loop::EventLoop::new();
    let wb = winit::window::WindowBuilder::new().with_visible(false);
    let cb = glutin::ContextBuilder::new()
        .with_gl_debug_flag(true)
        .with_gl(version);
    display.rebuild(wb, cb, &event_loop).unwrap();
     */
}

fn parse_version() -> glutin::context::ContextApi {
    match env::var("GLIUM_GL_VERSION") {
        Ok(version) => {
            // expects "OpenGL 3.3" for example

            let mut iter = version.rsplitn(2, ' ');

            let version = iter.next().unwrap();
            let ty = iter.next().unwrap();

            let mut iter = version.split('.');
            let major = iter.next().unwrap().parse().unwrap();
            let minor = iter.next().unwrap().parse().unwrap();

            if ty == "OpenGL" {
                glutin::context::ContextApi::OpenGl(Some(glutin::context::Version::new(major, minor)))
            } else if ty == "OpenGL ES" {
                glutin::context::ContextApi::Gles(Some(glutin::context::Version::new(major, minor)))
            } else if ty == "WebGL" {
                glutin::context::ContextApi::Gles(Some(glutin::context::Version::new(major, minor)))
            } else {
                panic!();
            }
        },
        Err(_) => glutin::context::ContextApi::OpenGl(None),
    }
}

/// Builds a 2x2 unicolor texture.
pub fn build_unicolor_texture2d<F: ?Sized>(facade: &F, red: f32, green: f32, blue: f32)
    -> glium::Texture2d where F: Facade
{
    let color = ((red * 255.0) as u8, (green * 255.0) as u8, (blue * 255.0) as u8);

    glium::texture::Texture2d::new(facade, vec![
        vec![color, color],
        vec![color, color],
    ]).unwrap()
}

/// Builds a 2x2 depth texture.
pub fn build_constant_depth_texture<F>(facade: &F, depth: f32) -> glium::texture::DepthTexture2d
where F: Facade + ?Sized
{
    glium::texture::DepthTexture2d::new(facade, vec![
        vec![depth, depth],
        vec![depth, depth],
    ]).unwrap()
}

/// Builds a vertex buffer, index buffer, and program, to draw red `(1.0, 0.0, 0.0, 1.0)` to the whole screen.
pub fn build_fullscreen_red_pipeline<F: ?Sized>(facade: &F) -> (glium::vertex::VertexBufferAny,
    glium::index::IndexBufferAny, glium::Program) where F: Facade
{
    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }

    implement_vertex!(Vertex, position);

    (
        glium::VertexBuffer::new(facade, &[
            Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
            Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
        ]).unwrap().into(),

        glium::IndexBuffer::new(facade, PrimitiveType::TriangleStrip, &[0u8, 1, 2, 3]).unwrap().into(),

        program!(facade,
            110 => {
                vertex: "
                    #version 110

                    attribute vec2 position;

                    void main() {
                        gl_Position = vec4(position, 0.0, 1.0);
                    }
                ",
                fragment: "
                    #version 110

                    void main() {
                        gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
                    }
                ",
            },
            100 => {
                vertex: "
                    #version 100

                    attribute lowp vec2 position;

                    void main() {
                        gl_Position = vec4(position, 0.0, 1.0);
                    }
                ",
                fragment: "
                    #version 100

                    void main() {
                        gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
                    }
                ",
            },
        ).unwrap()
    )
}

/// Builds a vertex buffer and an index buffer corresponding to a rectangle.
///
/// The vertex buffer has the "position" attribute of type "vec2".
pub fn build_rectangle_vb_ib<F: ?Sized>(facade: &F)
    -> (glium::vertex::VertexBufferAny, glium::index::IndexBufferAny) where F: Facade
{
    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }

    implement_vertex!(Vertex, position);

    (
        glium::VertexBuffer::new(facade, &[
            Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
            Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
        ]).unwrap().into(),

        glium::IndexBuffer::new(facade, PrimitiveType::TriangleStrip, &[0u8, 1, 2, 3]).unwrap().into(),
    )
}

/// Builds a texture suitable for rendering.
pub fn build_renderable_texture<F: ?Sized>(facade: &F) -> glium::Texture2d where F: Facade {
    glium::Texture2d::empty(facade, 1024, 1024).unwrap()
}
