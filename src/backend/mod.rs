/*!

The `backend` module allows one to link between glium and the OpenGL context..

There are three concepts in play:

 - The `Backend` trait describes the glue between glium and the OpenGL context provider like
   glutin, SDL, GLFW, etc.
 - The `Context` struct is the main brick of glium. It manages everything that glium needs to
   execute OpenGL commands. Creating a `Context` requires a `Backend`.
 - The `Facade` trait. Calling functions like `VertexBuffer::new` requires passing an object
   that implements this trait. It is implemented on `Rc<Context>`.

*/
use std::ops::Deref;
use std::os::raw::c_void;
use std::rc::Rc;

use crate::CapabilitiesSource;
use crate::SwapBuffersError;

use crate::context::Capabilities;
use crate::context::ExtensionsList;
use crate::version::Version;

pub use crate::context::Context;
pub use crate::context::ReleaseBehavior;

#[cfg(feature = "glutin")]
pub mod glutin;

use ::glutin::surface::SwapInterval;
#[cfg(feature = "simple_window_builder")]
pub use winit;

/// Trait for types that can be used as a backend for a glium context.
///
/// This trait is unsafe, as you can get undefined behaviors or crashes if you don't implement
/// the methods correctly.
pub unsafe trait Backend {
    /// Swaps buffers at the end of a frame.
    fn swap_buffers(&self) -> Result<(), SwapBuffersError>;

    /// Returns the address of an OpenGL function.
    ///
    /// Supposes that the context has been made current before this function is called.
    unsafe fn get_proc_address(&self, symbol: &str) -> *const c_void;

    /// Returns the dimensions of the window, or screen, etc.
    fn get_framebuffer_dimensions(&self) -> (u32, u32);

    /// Resizes the underlying surface, should be called when the window's size has changed for example.
    fn resize(&self, new_size: (u32, u32));

    /// Set swap interval for the surface.
    fn set_swap_interval(&self, interval: SwapInterval);

    /// Returns true if the OpenGL context is the current one in the thread.
    fn is_current(&self) -> bool;

    /// Makes the OpenGL context the current context in the current thread.
    unsafe fn make_current(&self);
}

unsafe impl<T> Backend for Rc<T>
where
    T: Backend,
{
    fn swap_buffers(&self) -> Result<(), SwapBuffersError> {
        self.deref().swap_buffers()
    }

    unsafe fn get_proc_address(&self, symbol: &str) -> *const c_void {
        self.deref().get_proc_address(symbol)
    }

    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        self.deref().get_framebuffer_dimensions()
    }

    fn resize(&self, new_size: (u32, u32)) {
        self.deref().resize(new_size);
    }

    fn set_swap_interval(&self, interval: SwapInterval) {
        self.deref().set_swap_interval(interval);
    }

    fn is_current(&self) -> bool {
        self.deref().is_current()
    }

    unsafe fn make_current(&self) {
        self.deref().make_current();
    }
}

/// Trait for types that provide a safe access for glium functions.
pub trait Facade {
    /// Returns an opaque type that contains the OpenGL state, extensions, version, etc.
    fn get_context(&self) -> &Rc<Context>;
}

impl<T: ?Sized> CapabilitiesSource for T
where
    T: Facade,
{
    fn get_version(&self) -> &Version {
        self.get_context().deref().get_opengl_version()
    }

    fn get_extensions(&self) -> &ExtensionsList {
        self.get_context().deref().get_extensions()
    }

    fn get_capabilities(&self) -> &Capabilities {
        self.get_context().deref().get_capabilities()
    }
}

impl Facade for Rc<Context> {
    #[inline]
    fn get_context(&self) -> &Rc<Context> {
        self
    }
}
