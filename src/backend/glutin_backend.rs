use libc;

use glutin;

use version;
use GliumCreationError;

use super::Backend;

use std::default::Default;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::channel;

pub struct GlutinWindowBackend {
    window: glutin::Window,
}

impl Backend for GlutinWindowBackend {
    fn swap_buffers(&self) {
        self.window.swap_buffers();
    }

    fn get_proc_address(&self, symbol: &str) -> *const libc::c_void {
        self.window.get_proc_address(symbol)
    }

    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        self.window.get_inner_size().unwrap_or((800, 600))      // TODO: 800x600 ?
    }

    fn is_current(&self) -> bool {
        self.window.is_current()
    }

    fn make_current(&self) {
        unsafe {
            self.window.make_current();
        }
    }
}

impl GlutinWindowBackend {
    pub fn new(builder: glutin::WindowBuilder)
               -> Result<GlutinWindowBackend, GliumCreationError>
    {
        let window = try!(builder.build());

        Ok(GlutinWindowBackend {
            window: window,
        })
    }

    pub fn get_window(&self) -> &glutin::Window {
        &self.window
    }

    pub fn is_closed(&self) -> bool {
        self.window.is_closed()
    }

    pub fn poll_events(&self) -> glutin::PollEventsIterator {
        self.window.poll_events()
    }

    pub fn wait_events(&self) -> glutin::WaitEventsIterator {
        self.window.wait_events()
    }
}

#[cfg(feature = "headless")]
pub struct GlutinHeadlessBackend {
    context: glutin::HeadlessContext,
}

#[cfg(feature = "headless")]
impl Backend for GlutinHeadlessBackend {
    fn swap_buffers(&self) {
    }

    fn get_proc_address(&self, symbol: &str) -> *const libc::c_void {
        self.context.get_proc_address(symbol)
    }

    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        (800, 600)      // FIXME: these are random
    }

    fn is_current(&self) -> bool {
        self.context.is_current()
    }

    fn make_current(&self) {
        unsafe {
            self.context.make_current();
        }
    }
}

#[cfg(feature = "headless")]
impl GlutinHeadlessBackend {
    pub fn new(builder: glutin::HeadlessRendererBuilder)
               -> Result<GlutinHeadlessBackend, GliumCreationError>
    {
        let context = try!(builder.build());

        Ok(GlutinHeadlessBackend {
            context: context,
        })
    }
}
