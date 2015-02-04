use gl;
use glutin;
use context::{Context, Message, CommandContext, GLState, check_gl_compatibility};
use context::{capabilities, extensions, version};
use GliumCreationError;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, sync_channel};

pub fn new_from_window(window: glutin::WindowBuilder, previous: Option<Context>)
    -> Result<Context, GliumCreationError>
{
    use std::thread::Builder;

    let (tx_commands, rx_commands) = channel();

    let (tx_end_frame, rx_end_frame) = sync_channel(0);

    let org_window = Arc::new(try!(window.build()));
    let window = org_window.clone();
    let (tx_success, rx_success) = channel();

    Builder::new().name("glium rendering thread".to_string()).spawn(move || {
        unsafe { window.make_current(); }

        let gl = gl::Gl::load_with(|symbol| window.get_proc_address(symbol));

        // building the GLState and modifying to GL state to match it
        let mut gl_state = {
            let viewport = {
                let dim = window.get_inner_size().unwrap();
                (0, 0, dim.0 as gl::types::GLsizei, dim.1 as gl::types::GLsizei)
            };

            unsafe { gl.Viewport(viewport.0, viewport.1, viewport.2, viewport.3) };
            GLState::new_defaults(viewport)
        };

        // getting the GL version and extensions
        let opengl_es = match window.get_api() { glutin::Api::OpenGlEs => true, _ => false };       // TODO: fix glutin::Api not implementing Eq
        let version = version::get_gl_version(&gl);
        let extensions = extensions::get_extensions(&gl);
        let capabilities = Arc::new(capabilities::get_capabilities(&gl, &version,
                                                                   &extensions, opengl_es));

        // checking compatibility with glium
        match check_gl_compatibility(CommandContext {
            gl: &gl,
            state: &mut gl_state,
            version: &version,
            extensions: &extensions,
            opengl_es: opengl_es,
            capabilities: &*capabilities
        }) {
            Err(e) => {
                tx_success.send(Err(e)).unwrap();
                return;
            },
            Ok(_) => {
                let ret = (capabilities.clone(), version.clone(), extensions.clone());
                tx_success.send(Ok(ret)).unwrap();
            }
        };

        // main loop
        loop {
            match rx_commands.recv() {
                Ok(Message::EndFrame) => {
                    // this is necessary on Windows 8, or nothing is being displayed
                    unsafe { gl.Flush(); }

                    // swapping
                    window.swap_buffers();
                    tx_end_frame.send(());
                },
                Ok(Message::Execute(cmd)) => cmd.invoke(CommandContext {
                    gl: &gl,
                    state: &mut gl_state,
                    version: &version,
                    extensions: &extensions,
                    opengl_es: opengl_es,
                    capabilities: &*capabilities
                }),
                Err(_) => break
            }
        }
    });

    let (capabilities, version, extensions) = try!(rx_success.recv().unwrap());
    Ok(Context {
        commands: Mutex::new(tx_commands),
        frame_ended: Mutex::new(rx_end_frame),
        window: Some(org_window),
        capabilities: capabilities,
        version: version,
        extensions: extensions,
    })
}

#[cfg(feature = "headless")]
pub fn new_from_headless(window: glutin::HeadlessRendererBuilder)
    -> Result<Context, GliumCreationError>
{
    use std::thread::Builder;

    let (tx_commands, rx_commands) = channel();

    let (tx_success, rx_success) = channel();

    Builder::new().name("glium rendering thread".to_string()).spawn(move || {
        let window = match window.build() {
            Ok(w) => w,
            Err(e) => {
                tx_success.send(Err(::std::error::FromError::from_error(e))).unwrap();
                return;
            }
        };
        unsafe { window.make_current(); }

        let gl = gl::Gl::load_with(|symbol| window.get_proc_address(symbol));
        // TODO: call glViewport

        // building the GLState, version, and extensions
        let mut gl_state = GLState::new_defaults((0, 0, 0, 0));    // FIXME:
        let opengl_es = match window.get_api() { glutin::Api::OpenGlEs => true, _ => false };       // TODO: fix glutin::Api not implementing Eq
        let version = version::get_gl_version(&gl);
        let extensions = extensions::get_extensions(&gl);
        let capabilities = Arc::new(capabilities::get_capabilities(&gl, &version,
                                                                   &extensions, opengl_es));

        // checking compatibility with glium
        match check_gl_compatibility(CommandContext {
            gl: &gl,
            state: &mut gl_state,
            version: &version,
            extensions: &extensions,
            opengl_es: opengl_es,
            capabilities: &*capabilities,
        }) {
            Err(e) => {
                tx_success.send(Err(e)).unwrap();
                return;
            },
            Ok(_) => {
                let ret = (capabilities.clone(), version.clone(), extensions.clone());
                tx_success.send(Ok(ret)).unwrap();
            }
        };

        loop {
            match rx_commands.recv() {
                Ok(Message::Execute(cmd)) => cmd.invoke(CommandContext {
                    gl: &gl,
                    state: &mut gl_state,
                    version: &version,
                    extensions: &extensions,
                    opengl_es: opengl_es,
                    capabilities: &*capabilities,
                }),
                Ok(Message::EndFrame) => (),        // ignoring buffer swapping
                Err(_) => break
            }
        }
    });

    let (capabilities, version, extensions) = try!(rx_success.recv().unwrap());
    Ok(Context {
        commands: Mutex::new(tx_commands),
        window: None,
        capabilities: capabilities,
        version: version,
        extensions: extensions,
    })
}
