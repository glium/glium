use std::cell::RefCell;
use std::rc::Rc;
use std::ops::Deref;

use libc;

pub mod glutin_backend;

pub trait Backend {
    fn swap_buffers(&self);
    fn get_proc_address(&self, symbol: &str) -> *const libc::c_void;
    fn get_framebuffer_dimensions(&self) -> (u32, u32);
    fn is_current(&self) -> bool;
    fn make_current(&self);
}

impl<T> Backend for Rc<T> where T: Backend {
    fn swap_buffers(&self) {
        self.deref().swap_buffers();
    }

    fn get_proc_address(&self, symbol: &str) -> *const libc::c_void {
        self.deref().get_proc_address(symbol)
    }

    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        self.deref().get_framebuffer_dimensions()
    }

    fn is_current(&self) -> bool {
        self.deref().is_current()
    }

    fn make_current(&self) {
        self.deref().make_current();
    }
}
