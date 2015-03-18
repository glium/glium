use std::rc::Rc;
use std::ops::Deref;

use libc;

pub mod glutin_backend;

/// Trait for types that can be used as a backend for a glium context.
pub trait Backend {
    /// Swaps buffers at the end of a frame.
    fn swap_buffers(&self);

    /// Returns the address of an OpenGL function.
    ///
    /// Must be called in the same thread and after the backend has been made current
    /// with `make_current`.
    unsafe fn get_proc_address(&self, symbol: &str) -> *const libc::c_void;
    
    /// Returns the dimensions of the window, or screen, etc.
    fn get_framebuffer_dimensions(&self) -> (u32, u32);

    /// Returns true if the OpenGL context is the current one in the thread.
    fn is_current(&self) -> bool;

    /// Makes the OpenGL context the current context in the current thread.
    unsafe fn make_current(&self);
}

impl<T> Backend for Rc<T> where T: Backend {
    fn swap_buffers(&self) {
        self.deref().swap_buffers();
    }

    unsafe fn get_proc_address(&self, symbol: &str) -> *const libc::c_void {
        self.deref().get_proc_address(symbol)
    }

    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        self.deref().get_framebuffer_dimensions()
    }

    fn is_current(&self) -> bool {
        self.deref().is_current()
    }

    unsafe fn make_current(&self) {
        self.deref().make_current();
    }
}
