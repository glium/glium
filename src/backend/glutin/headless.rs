//! Backend implementation for a glutin headless renderer.

use crate::{Frame, IncompatibleOpenGl, SwapBuffersError};
use crate::debug;
use crate::context;
use crate::backend::{self, Backend};
use std::ffi::CString;
use std::rc::Rc;
use std::cell::{RefCell, Cell};
use std::ops::Deref;
use std::os::raw::c_void;
use super::ContextSurfacePair;
use super::glutin::display::GetGlDisplay;
use super::glutin::prelude::*;
use super::glutin::context::PossiblyCurrentContext;
use glutin::surface::{SurfaceTypeTrait, ResizeableSurface, Surface};
use takeable_option::Takeable;

/// A headless glutin context.
pub struct Headless<T: SurfaceTypeTrait + ResizeableSurface + 'static> {
    context: Rc<context::Context>,
    context_surface_pair: Rc<RefCell<Takeable<ContextSurfacePair<T>>>>,
    framebuffer_dimensions: Cell<(u32, u32)>,
}

/// An implementation of the `Backend` trait for a glutin headless context.
pub struct GlutinBackend<T: SurfaceTypeTrait + ResizeableSurface>(Rc<RefCell<Takeable<ContextSurfacePair<T>>>>);

impl<T: SurfaceTypeTrait + ResizeableSurface> Deref for Headless<T> {
    type Target = context::Context;
    fn deref(&self) -> &context::Context {
        &self.context
    }
}

impl<T: SurfaceTypeTrait + ResizeableSurface> Deref for GlutinBackend<T> {
    type Target = Rc<RefCell<Takeable<ContextSurfacePair<T>>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl<T: SurfaceTypeTrait + ResizeableSurface> Backend for GlutinBackend<T> {
    #[inline]
    fn swap_buffers(&self) -> Result<(), SwapBuffersError> {
        Ok(())
    }

    #[inline]
    unsafe fn get_proc_address(&self, symbol: &str) -> *const c_void {
        let symbol = CString::new(symbol).unwrap();
        self.0.borrow().display().get_proc_address(&symbol) as *const _
    }

    #[inline]
    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        (800, 600)      // FIXME: these are random
    }

    #[inline]
    fn resize(&self, new_size:(u32, u32)) {}

    #[inline]
    fn is_current(&self) -> bool {
        self.0.borrow().is_current()
    }

    #[inline]
    unsafe fn make_current(&self) {
        let pair = self.borrow();
        pair.context.make_current(&pair.surface).unwrap();
    }
}

impl<T: SurfaceTypeTrait + ResizeableSurface> backend::Facade for Headless<T> {
    #[inline]
    fn get_context(&self) -> &Rc<context::Context> {
        &self.context
    }
}

impl<T: SurfaceTypeTrait + ResizeableSurface> Headless<T> {
    /// Create a new glium `Headless` context.
    ///
    /// Performs a compatibility check to make sure that all core elements of glium are supported
    /// by the implementation.
    pub fn new(
        context: PossiblyCurrentContext,
        surface: Surface<T>,
    ) -> Result<Self, IncompatibleOpenGl> {
        Self::with_debug(context, surface, Default::default())
    }

    /// Create a new glium `Headless` context.
    ///
    /// This function does the same as `build_glium`, except that the resulting context
    /// will assume that the current OpenGL context will never change.
    pub unsafe fn unchecked(
        context: PossiblyCurrentContext,
        surface: Surface<T>,
    ) -> Result<Self, IncompatibleOpenGl> {
        Self::unchecked_with_debug(context, surface, Default::default())
    }

    /// The same as the `new` constructor, but allows for specifying debug callback behaviour.
    pub fn with_debug(
        context: PossiblyCurrentContext,
        surface: Surface<T>,
        debug: debug::DebugCallbackBehavior,
    )
        -> Result<Self, IncompatibleOpenGl>
    {
        Self::new_inner(context, surface, debug, true)
    }

    /// The same as the `unchecked` constructor, but allows for specifying debug callback behaviour.
    pub unsafe fn unchecked_with_debug(
        context: PossiblyCurrentContext,
        surface: Surface<T>,
        debug: debug::DebugCallbackBehavior,
    ) -> Result<Self, IncompatibleOpenGl>
    {
        Self::new_inner(context, surface, debug, false)
    }

    fn new_inner(
        context: PossiblyCurrentContext,
        surface: Surface<T>,
        debug: debug::DebugCallbackBehavior,
        checked: bool,
    ) -> Result<Self, IncompatibleOpenGl>
    {
        let context_surface_pair = ContextSurfacePair::new(context, surface);
        let glutin_context = Rc::new(RefCell::new(Takeable::new(context_surface_pair)));
        let glutin_backend = GlutinBackend(glutin_context.clone());
        let context = unsafe { context::Context::new(glutin_backend, checked, debug) }?;
        let framebuffer_dimensions = Cell::new((800, 600));
        Ok(Headless { context, context_surface_pair: glutin_context, framebuffer_dimensions })
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
