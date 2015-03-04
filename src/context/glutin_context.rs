use gl;
use glutin;
use version;
use context::{Context, CommandContext, GLState, SharedDebugOutput, check_gl_compatibility};
use context::{capabilities, extensions, Backend};
use GliumCreationError;

use std::default::Default;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::channel;

impl Backend for Rc<RefCell<glutin::Window>> {
    fn is_closed(&self) -> bool {
        self.borrow().is_closed()
    }

    fn is_current(&self) -> bool {
        self.borrow().is_current()
    }

    unsafe fn make_current(&self) {
        self.borrow().make_current();
    }
}

pub fn new_from_window(window: glutin::WindowBuilder, check_context: bool)
                       -> Result<(Context, Rc<SharedDebugOutput>), GliumCreationError>
{
    let shared_debug_frontend = SharedDebugOutput::new();
    let shared_debug_backend = shared_debug_frontend.clone();

    let window = Rc::new(RefCell::new(try!(window.build())));

    let gl = {
        let locked_win = window.borrow();
        unsafe { locked_win.make_current(); }
        gl::Gl::load_with(|symbol| locked_win.get_proc_address(symbol))
    };

    // building the GLState
    let mut gl_state = Default::default();

    // getting the GL version and extensions
    let version = version::get_gl_version(&gl);
    let extensions = extensions::get_extensions(&gl);
    let capabilities = capabilities::get_capabilities(&gl, &version, &extensions);

    // checking compatibility with glium
    try!(check_gl_compatibility(CommandContext {
        gl: &gl,
        state: &mut gl_state,
        version: &version,
        extensions: &extensions,
        capabilities: &capabilities,
        shared_debug_output: &shared_debug_backend,
    }));

    let backend = Box::new(window.clone());

    Ok((Context {
        gl: gl,
        state: RefCell::new(gl_state),
        version: version,
        extensions: extensions,
        capabilities: capabilities,
        shared_debug_output: shared_debug_backend,
        window: Some(window),
        backend: backend,
        check_current_context: check_context,
    }, shared_debug_frontend))
}

#[cfg(feature = "headless")]
impl Backend for glutin::HeadlessContext {
    fn is_closed(&self) -> bool {
        false
    }

    fn is_current(&self) -> bool {
        self.is_current()
    }

    unsafe fn make_current(&self) {
        self.make_current();
    }
}

#[cfg(feature = "headless")]
pub fn new_from_headless(window: glutin::HeadlessRendererBuilder, check_context: bool)
                         -> Result<(Context, Rc<SharedDebugOutput>), GliumCreationError>
{
    let shared_debug_frontend = SharedDebugOutput::new();
    let shared_debug_backend = shared_debug_frontend.clone();

    let window = try!(window.build());
    unsafe { window.make_current(); }

    let gl = gl::Gl::load_with(|symbol| window.get_proc_address(symbol));

    // building the GLState, version, and extensions
    let mut gl_state = Default::default();
    let version = version::get_gl_version(&gl);
    let extensions = extensions::get_extensions(&gl);
    let capabilities = capabilities::get_capabilities(&gl, &version, &extensions);

    // checking compatibility with glium
    try!(check_gl_compatibility(CommandContext {
        gl: &gl,
        state: &mut gl_state,
        version: &version,
        extensions: &extensions,
        capabilities: &capabilities,
        shared_debug_output: &shared_debug_backend,
    }));

    let backend = Box::new(window);

    Ok((Context {
        gl: gl,
        state: RefCell::new(gl_state),
        version: version,
        extensions: extensions,
        capabilities: capabilities,
        shared_debug_output: shared_debug_backend,
        window: None,
        backend: backend,
        check_current_context: check_context,
    }, shared_debug_frontend))
}
