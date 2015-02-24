use gl;
use glutin;
use version;
use context::{Context, CommandContext, GLState, SharedDebugOutput, check_gl_compatibility};
use context::{capabilities, extensions, commands};
use GliumCreationError;

use std::default::Default;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::channel;

pub fn new_from_window(window: glutin::WindowBuilder)
                       -> Result<(Context, Arc<SharedDebugOutput>), GliumCreationError>
{
    use std::thread::Builder;

    let (commands_frontend, commands_backend) = commands::channel();

    let shared_debug_frontend = SharedDebugOutput::new();
    let shared_debug_backend = shared_debug_frontend.clone();

    let org_window = Arc::new(RwLock::new(try!(window.build())));
    let window = org_window.clone();
    let (tx_success, rx_success) = channel();

    Builder::new().name("glium rendering thread".to_string()).spawn(move || {
        let locked_win = window.read().unwrap();

        unsafe { locked_win.make_current(); }

        let gl = gl::Gl::load_with(|symbol| locked_win.get_proc_address(symbol));

        // building the GLState
        let mut gl_state = Default::default();

        // getting the GL version and extensions
        let version = version::get_gl_version(&gl);
        let extensions = extensions::get_extensions(&gl);
        let capabilities = Arc::new(capabilities::get_capabilities(&gl, &version, &extensions));

        drop(locked_win);

        // checking compatibility with glium
        match check_gl_compatibility(CommandContext {
            gl: &gl,
            state: &mut gl_state,
            version: &version,
            extensions: &extensions,
            capabilities: &*capabilities,
            shared_debug_output: &shared_debug_backend,
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
            match commands_backend.pop() {
                commands::Message::EndFrame => {
                    // this is necessary on Windows 8, or nothing is being displayed
                    unsafe { gl.Flush(); }

                    // swapping
                    window.read().unwrap().swap_buffers();
                },
                commands::Message::Rebuild(builder, notification) => {
                    let win_tmp = {
                        let window = window.read().unwrap();
                        let builder = builder.with_shared_lists(&*window);
                        match builder.build() {
                            Ok(win) => {
                                notification.send(Ok(())).ok();
                                win
                            },
                            Err(e) => {
                                let e = ::std::error::FromError::from_error(e);
                                notification.send(Err(e)).ok();
                                continue;
                            }
                        }
                    };

                    let mut window = window.write().unwrap();
                    unsafe { win_tmp.make_current(); };
                    gl_state = Default::default();
                    *window = win_tmp;
                },
                commands::Message::Execute(cmd) => cmd.execute(CommandContext {
                    gl: &gl,
                    state: &mut gl_state,
                    version: &version,
                    extensions: &extensions,
                    capabilities: &*capabilities,
                    shared_debug_output: &shared_debug_backend,
                }),
                commands::Message::Stop => break
            }
        }
    });

    let (capabilities, version, extensions) = try!(rx_success.recv().unwrap());
    Ok((Context {
        commands: commands_frontend,
        window: Some(org_window),
        capabilities: capabilities,
        version: version,
        extensions: extensions,
    }, shared_debug_frontend))
}

#[cfg(feature = "headless")]
pub fn new_from_headless(window: glutin::HeadlessRendererBuilder)
    -> Result<(Context, Arc<SharedDebugOutput>), GliumCreationError>
{
    use std::thread::Builder;

    let (commands_frontend, commands_backend) = commands::channel();

    let shared_debug_frontend = SharedDebugOutput::new();
    let shared_debug_backend = shared_debug_frontend.clone();

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
        let mut gl_state = Default::default();
        let version = version::get_gl_version(&gl);
        let extensions = extensions::get_extensions(&gl);
        let capabilities = Arc::new(capabilities::get_capabilities(&gl, &version, &extensions));

        // checking compatibility with glium
        match check_gl_compatibility(CommandContext {
            gl: &gl,
            state: &mut gl_state,
            version: &version,
            extensions: &extensions,
            capabilities: &*capabilities,
            shared_debug_output: &shared_debug_backend,
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
            match commands_backend.pop() {
                commands::Message::Execute(cmd) => cmd.execute(CommandContext {
                    gl: &gl,
                    state: &mut gl_state,
                    version: &version,
                    extensions: &extensions,
                    capabilities: &*capabilities,
                    shared_debug_output: &shared_debug_backend,
                }),
                commands::Message::EndFrame => (),        // ignoring buffer swapping
                commands::Message::Rebuild(_, _) => unimplemented!(),
                commands::Message::Stop => break
            }
        }
    });

    let (capabilities, version, extensions) = try!(rx_success.recv().unwrap());
    Ok((Context {
        commands: commands_frontend,
        window: None,
        capabilities: capabilities,
        version: version,
        extensions: extensions,
    }, shared_debug_frontend))
}
