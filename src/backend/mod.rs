use std::rc::Rc;
use std::ops::Deref;

use libc;

pub use context::Context;

pub mod glutin_backend;

/// Trait for types that can be used as a backend for a glium context.
///
/// This trait is unsafe, as you can get undefined behaviors or crashes if you don't implement
/// the methods correctly.
pub unsafe trait Backend {
    /// Swaps buffers at the end of a frame.
    fn swap_buffers(&self);

    /// Returns the address of an OpenGL function.
    ///
    /// Supposes that the context has been made current before this function is called.
    unsafe fn get_proc_address(&self, symbol: &str) -> *const libc::c_void;

    /// Returns the dimensions of the window, or screen, etc.
    fn get_framebuffer_dimensions(&self) -> (u32, u32);

    /// Returns true if the OpenGL context is the current one in the thread.
    fn is_current(&self) -> bool;

    /// Makes the OpenGL context the current context in the current thread.
    unsafe fn make_current(&self);
}

/// Trait for types that provide a safe access for glium functions.
pub trait Facade {
    /// Returns an opaque type that contains the OpenGL state, extensions, version, etc.
    fn get_context(&self) -> &Rc<Context>;
}

unsafe impl<T> Backend for Rc<T> where T: Backend {
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

impl Facade for Rc<Context> {
    fn get_context(&self) -> &Rc<Context> {
        self
    }
}
