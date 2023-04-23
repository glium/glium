/*!
Test supports module.

*/

#![allow(dead_code)]

use glium::{self, glutin};
use glium::backend::Facade;
use glium::index::PrimitiveType;
use glutin::{event::Event, event_loop::{EventLoop, EventLoopBuilder, EventLoopProxy}, NotCurrent, WindowedContext};

use std::env;
use std::thread;
use std::sync::{mpsc::Receiver, Once, RwLock};

/// Builds a display for tests.
#[cfg(not(feature = "test_headless"))]
pub fn build_display() -> glium::Display {
    // TODO: assess sync usage and prove this approach is safe

    static mut EVENT_LOOP_PROXY: RwLock<Option<EventLoopProxy<()>>> = RwLock::new(None);
    static mut CONTEXT_RECEIVER: RwLock<Option<Receiver<WindowedContext<NotCurrent>>>> = RwLock::new(None);
    static mut INIT_EVENT_LOOP: Once = Once::new();
    static mut SEND_PROXY: Once = Once::new();

    unsafe {
        INIT_EVENT_LOOP.call_once(|| {

            // this channel is used one time to get the event loop proxy
            let (ots, otr) = std::sync::mpsc::sync_channel(0);
            // this channel transfers windowed contexts to create displays
            let (sender, receiver) = std::sync::mpsc::channel();

            // create event loop in a separate thread so we can process events
            let builder = thread::Builder::new().name("event_loop".into());
            builder.spawn(|| {
                let event_loop = if cfg!(windows) {
                    use glutin::platform::windows::EventLoopBuilderExtWindows;

                    EventLoopBuilder::new().with_any_thread(true).build()
                } else {
                    EventLoop::new()
                };

                let proxy = event_loop.create_proxy();

                event_loop.run(move |event, event_loop, _| {
                    match event {
                        Event::UserEvent(_) => {
                            let version = parse_version();
                            let wb = glutin::window::WindowBuilder::new().with_visible(false);
                            let cb = glutin::ContextBuilder::new()
                                .with_gl_debug_flag(true)
                                .with_gl(version);
                            sender.send(cb.build_windowed(wb, event_loop).unwrap()).unwrap();
                        }
                        _ => {
                            // cloning the proxy here for convenience
                            // TODO: move the proxy instead of cloning, if possible
                            SEND_PROXY.call_once(|| {
                                ots.send(proxy.clone()).unwrap();
                            });
                        }
                    }
                });
            }).unwrap();

            let event_loop_proxy = otr.recv().unwrap();

            let mut guard = EVENT_LOOP_PROXY.write().unwrap();

            *guard = Some(event_loop_proxy);

            let mut guard = CONTEXT_RECEIVER.write().unwrap();

            *guard = Some(receiver);
        });
    }

    // tell event loop to create a windowed context
    let guard = unsafe {
        EVENT_LOOP_PROXY.read().unwrap()
    };
    guard.as_ref().unwrap().send_event(()).unwrap();

    // receive context and create display
    let guard = unsafe {
        CONTEXT_RECEIVER.read().unwrap()
    };
    let windowed_context = guard.as_ref().unwrap().recv().unwrap();

    glium::Display::from_gl_window(windowed_context).unwrap()
}

/// Builds a headless display for tests.
#[cfg(feature = "test_headless")]
pub fn build_display() -> glium::HeadlessRenderer {
    let version = parse_version();
    let hrb = glutin::HeadlessRendererBuilder::new(1024, 768)
        .with_gl_debug_flag(true)
        .with_gl(version)
        .build()
        .unwrap();
    glium::HeadlessRenderer::new(hrb).unwrap()
}

/// Rebuilds an existing display.
///
/// In real applications this is used for things such as switching to fullscreen. Some things are
/// invalidated during a rebuild, and this has to be handled by glium.
pub fn rebuild_display(display: &glium::Display) {
    /*let version = parse_version();
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_visible(false);
    let cb = glutin::ContextBuilder::new()
        .with_gl_debug_flag(true)
        .with_gl(version);
    display.rebuild(wb, cb, get_cached_event_loop().as_ref().unwrap()).unwrap();*/
}

fn parse_version() -> glutin::GlRequest {
    match env::var("GLIUM_GL_VERSION") {
        Ok(version) => {
            // expects "OpenGL 3.3" for example

            let mut iter = version.rsplitn(2, ' ');

            let version = iter.next().unwrap();
            let ty = iter.next().unwrap();

            let mut iter = version.split('.');
            let major = iter.next().unwrap().parse().unwrap();
            let minor = iter.next().unwrap().parse().unwrap();

            let ty = if ty == "OpenGL" {
                glutin::Api::OpenGl
            } else if ty == "OpenGL ES" {
                glutin::Api::OpenGlEs
            } else if ty == "WebGL" {
                glutin::Api::WebGl
            } else {
                panic!();
            };

            glutin::GlRequest::Specific(ty, (major, minor))
        },
        Err(_) => glutin::GlRequest::Latest,
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
