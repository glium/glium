//! Backend implementation for a glutin headless renderer.

use crate::{Frame, IncompatibleOpenGl, SwapBuffersError};
use crate::debug;
use crate::context;
use crate::backend::{self, Backend};
use std::rc::Rc;
use std::cell::{Ref, RefCell};
use std::ops::Deref;
use std::os::raw::c_void;
use super::glutin;
use super::glutin::{PossiblyCurrent as Pc, ContextCurrentState};

/// A headless glutin context.
pub struct Headless {
    context: Rc<context::Context>,
    glutin: Rc<RefCell<Option<glutin::Context<Pc>>>>,
}

/// An implementation of the `Backend` trait for a glutin headless context.
pub struct GlutinBackend(Rc<RefCell<Option<glutin::Context<Pc>>>>);

impl Deref for Headless {
    type Target = context::Context;
    fn deref(&self) -> &context::Context {
        &self.context
    }
}

impl Deref for GlutinBackend {
    type Target = Rc<RefCell<Option<glutin::Context<Pc>>>>;
    fn deref(&self) -> &Rc<RefCell<Option<glutin::Context<Pc>>>> {
        &self.0
    }
}

unsafe impl Backend for GlutinBackend {
    #[inline]
    fn swap_buffers(&self) -> Result<(), SwapBuffersError> {
        Ok(())
    }

    #[inline]
    unsafe fn get_proc_address(&self, symbol: &str) -> *const c_void {
        self.0.borrow().as_ref().unwrap().get_proc_address(symbol) as *const _
    }

    #[inline]
    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        (800, 600)      // FIXME: these are random
    }

    #[inline]
    fn is_current(&self) -> bool {
        self.0.borrow().as_ref().unwrap().is_current()
    }

    #[inline]
    unsafe fn make_current(&self) {
        let mut gl_window_takeable = self.0.borrow_mut();
        let gl_window = gl_window_takeable.take().unwrap();
        let gl_window_new = gl_window.make_current().unwrap();
        *gl_window_takeable = Some(gl_window_new);
    }
}

impl backend::Facade for Headless {
    #[inline]
    fn get_context(&self) -> &Rc<context::Context> {
        &self.context
    }
}

impl Headless {
    /// Create a new glium `Headless` context.
    ///
    /// Performs a compatibility check to make sure that all core elements of glium are supported
    /// by the implementation.
    pub fn new<T: ContextCurrentState>(context: glutin::Context<T>) -> Result<Self, IncompatibleOpenGl> {
        Self::with_debug(context, Default::default())
    }

    /// Create a new glium `Headless` context.
    ///
    /// This function does the same as `build_glium`, except that the resulting context
    /// will assume that the current OpenGL context will never change.
    pub unsafe fn unchecked<T: ContextCurrentState>(context: glutin::Context<T>) -> Result<Self, IncompatibleOpenGl> {
        Self::unchecked_with_debug(context, Default::default())
    }

    /// The same as the `new` constructor, but allows for specifying debug callback behaviour.
    pub fn with_debug<T: ContextCurrentState>(context: glutin::Context<T>, debug: debug::DebugCallbackBehavior)
        -> Result<Self, IncompatibleOpenGl>
    {
        Self::new_inner(context, debug, true)
    }

    /// The same as the `unchecked` constructor, but allows for specifying debug callback behaviour.
    pub unsafe fn unchecked_with_debug<T: ContextCurrentState>(
        context: glutin::Context<T>,
        debug: debug::DebugCallbackBehavior,
    ) -> Result<Self, IncompatibleOpenGl>
    {
        Self::new_inner(context, debug, false)
    }

    fn new_inner<T: ContextCurrentState>(
        context: glutin::Context<T>,
        debug: debug::DebugCallbackBehavior,
        checked: bool,
    ) -> Result<Self, IncompatibleOpenGl>
    {
        let context = unsafe {
            context.treat_as_current()
        };
        let glutin_context = Rc::new(RefCell::new(Some(context)));
        let glutin_backend = GlutinBackend(glutin_context.clone());
        let context = unsafe { context::Context::new(glutin_backend, checked, debug) }?;
        Ok(Headless { context, glutin: glutin_context })
    }

    /// Borrow the inner glutin context
    pub fn gl_context(&self) -> Ref<'_, Option<glutin::Context<Pc>>> {
        self.glutin.borrow()
    }

    /// Start drawing on the backbuffer.
    ///
    /// This function returns a `Frame`, which can be used to draw on it. When the `Frame` is
    /// destroyed, the buffers are swapped.
    ///
    /// Note that destroying a `Frame` is immediate, even if vsync is enabled.
    ///
    /// If the framebuffer dimensions have changed since the last call to `draw`, the inner glutin
    /// context will be resized accordingly before returning the `Frame`.
    #[inline]
    pub fn draw(&self) -> Frame {
        Frame::new(self.context.clone(), self.get_framebuffer_dimensions())
    }
}
