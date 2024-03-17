#![cfg(feature = "glutin")]
/*!

Backend implementation for the glutin library

# Features

Only available if the 'glutin' feature is enabled.

*/
pub use glutin;
use glutin::surface::Surface;

use crate::backend;
use crate::backend::Backend;
use crate::backend::Context;
use crate::context;
use crate::debug;
use crate::glutin::context::PossiblyCurrentContext;
use crate::glutin::display::GetGlDisplay;
use crate::glutin::prelude::*;
use crate::glutin::surface::{ResizeableSurface, SurfaceTypeTrait};
use crate::SwapBuffersError;
use crate::{Frame, IncompatibleOpenGl};
use std::cell::RefCell;
use std::error::Error;
use std::ffi::CString;
use std::fmt;
use std::num::NonZeroU32;
use std::ops::Deref;
use std::os::raw::c_void;
use std::rc::Rc;

/// Wraps a glutin context together with the corresponding Surface.
/// This is necessary so that we can swap buffers and determine the framebuffer size within glium.
pub struct ContextSurfacePair<T: SurfaceTypeTrait + ResizeableSurface> {
    context: PossiblyCurrentContext,
    surface: glutin::surface::Surface<T>,
}

impl<T: SurfaceTypeTrait + ResizeableSurface> ContextSurfacePair<T> {
    fn new(context: PossiblyCurrentContext, surface: glutin::surface::Surface<T>) -> Self {
        Self { context, surface }
    }

    #[inline]
    /// Return the stored framebuffer dimensions
    pub fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        (
            self.surface.width().unwrap(),
            self.surface.height().unwrap(),
        )
    }

    #[inline]
    /// Return the stored framebuffer dimensions
    pub fn swap_buffers(&self) -> Result<(), glutin::error::Error> {
        self.surface.swap_buffers(&self.context)
    }

    #[inline]
    /// Resize the associated surface
    pub fn resize(&self, new_size: (u32, u32)) {
        // Make sure that no dimension is zero, which happens when minimizing on Windows for example.
        let width = NonZeroU32::new(new_size.0).unwrap_or(NonZeroU32::new(1).unwrap());
        let height = NonZeroU32::new(new_size.1).unwrap_or(NonZeroU32::new(1).unwrap());
        self.surface.resize(&self.context, width, height);
    }
}

impl<T: SurfaceTypeTrait + ResizeableSurface> Deref for ContextSurfacePair<T> {
    type Target = PossiblyCurrentContext;
    #[inline]
    fn deref(&self) -> &PossiblyCurrentContext {
        &self.context
    }
}

/// A GL context combined with a facade for drawing upon.
///
/// The `Display` uses **glutin** for the **Window** and its associated GL **Context**.
///
/// These are stored alongside a glium-specific context.
#[derive(Clone)]
pub struct Display<T: SurfaceTypeTrait + ResizeableSurface + 'static> {
    // contains everything related to the current glium context and its state
    context: Rc<context::Context>,
    // The glutin Surface alongside its associated glutin Context.
    gl_context: Rc<RefCell<Option<ContextSurfacePair<T>>>>,
}

/// An implementation of the `Backend` trait for glutin.
#[derive(Clone)]
pub struct GlutinBackend<T: SurfaceTypeTrait + ResizeableSurface>(
    Rc<RefCell<Option<ContextSurfacePair<T>>>>,
);

/// Error that can happen while creating a glium display.
#[derive(Debug)]
pub enum DisplayCreationError {
    /// An error has happened while creating the backend.
    GlutinError(glutin::error::Error),
    /// The OpenGL implementation is too old.
    IncompatibleOpenGl(IncompatibleOpenGl),
}

impl<T: SurfaceTypeTrait + ResizeableSurface> std::fmt::Debug for Display<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[glium::backend::glutin::Display]")
    }
}

impl<T: SurfaceTypeTrait + ResizeableSurface> Display<T> {
    /// Create a new glium `Display` from the given context and surface.
    ///
    /// Performs a compatibility check to make sure that all core elements of glium are supported
    /// by the implementation.
    pub fn new(
        context: PossiblyCurrentContext,
        surface: Surface<T>,
    ) -> Result<Self, DisplayCreationError> {
        Self::from_context_surface(context, surface).map_err(From::from)
    }

    /// Create a new glium `Display` from the given context and surface.
    ///
    /// Performs a compatibility check to make sure that all core elements of glium are supported
    /// by the implementation.
    pub fn from_context_surface(
        context: PossiblyCurrentContext,
        surface: Surface<T>,
    ) -> Result<Self, IncompatibleOpenGl> {
        Self::with_debug(context, surface, Default::default())
    }

    /// Create a new glium `Display` from the given context and surface.
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
    ) -> Result<Self, IncompatibleOpenGl> {
        Self::new_inner(context, surface, debug, true)
    }

    /// The same as the `unchecked` constructor, but allows for specifying debug callback behaviour.
    pub unsafe fn unchecked_with_debug(
        context: PossiblyCurrentContext,
        surface: Surface<T>,
        debug: debug::DebugCallbackBehavior,
    ) -> Result<Self, IncompatibleOpenGl> {
        Self::new_inner(context, surface, debug, false)
    }

    fn new_inner(
        context: PossiblyCurrentContext,
        surface: Surface<T>,
        debug: debug::DebugCallbackBehavior,
        checked: bool,
    ) -> Result<Self, IncompatibleOpenGl> {
        let context_surface_pair = ContextSurfacePair::new(context, surface);
        let gl_window = Rc::new(RefCell::new(Some(context_surface_pair)));
        let glutin_backend = GlutinBackend(gl_window.clone());
        let context = unsafe { context::Context::new(glutin_backend, checked, debug) }?;
        Ok(Display {
            gl_context: gl_window,
            context,
        })
    }

    /// Resize the underlying surface.
    #[inline]
    pub fn resize(&self, new_size: (u32, u32)) {
        self.gl_context.borrow().as_ref().unwrap().resize(new_size)
    }

    /// Start drawing on the backbuffer.
    ///
    /// This function returns a `Frame`, which can be used to draw on it. When the `Frame` is
    /// destroyed, the buffers are swapped.
    ///
    /// Note that destroying a `Frame` is immediate, even if vsync is enabled.
    #[inline]
    pub fn draw(&self) -> Frame {
        let dimensions = self.get_framebuffer_dimensions();
        Frame::new(self.context.clone(), dimensions)
    }
}

impl fmt::Display for DisplayCreationError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            DisplayCreationError::GlutinError(err) => write!(fmt, "{}", err),
            DisplayCreationError::IncompatibleOpenGl(err) => write!(fmt, "{}", err),
        }
    }
}

impl Error for DisplayCreationError {
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            DisplayCreationError::GlutinError(ref err) => Some(err),
            DisplayCreationError::IncompatibleOpenGl(ref err) => Some(err),
        }
    }
}

impl From<glutin::error::Error> for DisplayCreationError {
    #[inline]
    fn from(err: glutin::error::Error) -> DisplayCreationError {
        DisplayCreationError::GlutinError(err)
    }
}

impl From<IncompatibleOpenGl> for DisplayCreationError {
    #[inline]
    fn from(err: IncompatibleOpenGl) -> DisplayCreationError {
        DisplayCreationError::IncompatibleOpenGl(err)
    }
}

impl<T: SurfaceTypeTrait + ResizeableSurface> Deref for Display<T> {
    type Target = Context;
    #[inline]
    fn deref(&self) -> &Context {
        &self.context
    }
}

impl<T: SurfaceTypeTrait + ResizeableSurface> backend::Facade for Display<T> {
    #[inline]
    fn get_context(&self) -> &Rc<Context> {
        &self.context
    }
}

impl<T: SurfaceTypeTrait + ResizeableSurface> Deref for GlutinBackend<T> {
    type Target = Rc<RefCell<Option<ContextSurfacePair<T>>>>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl<T: SurfaceTypeTrait + ResizeableSurface> Backend for GlutinBackend<T> {
    #[inline]
    fn swap_buffers(&self) -> Result<(), SwapBuffersError> {
        match self.borrow().as_ref().unwrap().swap_buffers() {
            Ok(()) => Ok(()),
            _ => Err(SwapBuffersError::ContextLost),
        }
    }

    #[inline]
    unsafe fn get_proc_address(&self, symbol: &str) -> *const c_void {
        let symbol = CString::new(symbol).unwrap();
        self.borrow()
            .as_ref()
            .unwrap()
            .display()
            .get_proc_address(&symbol) as *const _
    }

    #[inline]
    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        self.0
            .borrow()
            .as_ref()
            .unwrap()
            .get_framebuffer_dimensions()
    }

    #[inline]
    fn resize(&self, new_size: (u32, u32)) {
        self.borrow().as_ref().unwrap().resize(new_size)
    }

    #[inline]
    fn is_current(&self) -> bool {
        self.borrow().as_ref().unwrap().is_current()
    }

    #[inline]
    unsafe fn make_current(&self) {
        let pair = self.borrow();
        pair.as_ref()
            .unwrap()
            .context
            .make_current(&pair.as_ref().unwrap().surface)
            .unwrap();
    }
}

#[cfg(feature = "simple_window_builder")]
/// Builder to simplify glium/glutin context creation.
pub struct SimpleWindowBuilder {
    builder: winit::window::WindowBuilder,
}

#[cfg(feature = "simple_window_builder")]
impl SimpleWindowBuilder {
    /// Initializes a new builder with default values.
    pub fn new() -> Self {
        Self {
            builder: winit::window::WindowBuilder::new()
                .with_title("Simple Glium Window")
                .with_inner_size(winit::dpi::PhysicalSize::new(800, 480)),
        }
    }

    /// Requests the window to be of a certain size.
    /// If this is not set, the builder defaults to 800x480.
    pub fn with_inner_size(mut self, width: u32, height: u32) -> Self {
        self.builder = self
            .builder
            .with_inner_size(winit::dpi::PhysicalSize::new(width, height));
        self
    }

    /// Set the initial title for the window.
    pub fn with_title(mut self, title: &str) -> Self {
        self.builder = self.builder.with_title(title);
        self
    }

    /// Replace the used [`WindowBuilder`](winit::window::WindowBuilder),
    /// do this before you set other parameters or you'll overwrite the parameters.
    pub fn set_window_builder(mut self, window_builder: winit::window::WindowBuilder) -> Self {
        self.builder = window_builder;
        self
    }

    /// Returns the inner [`WindowBuilder`](winit::window::WindowBuilder).
    pub fn into_window_builder(self) -> winit::window::WindowBuilder {
        self.builder
    }

    /// Create a new [`Window`](winit::window::Window) and [`Display`]
    /// with the specified parameters.
    pub fn build<T>(
        self,
        event_loop: &winit::event_loop::EventLoop<T>,
    ) -> (
        winit::window::Window,
        Display<glutin::surface::WindowSurface>,
    ) {
        use glutin::prelude::*;
        use raw_window_handle::HasRawWindowHandle;

        // First we start by opening a new Window
        let display_builder =
            glutin_winit::DisplayBuilder::new().with_window_builder(Some(self.builder));
        let config_template_builder = glutin::config::ConfigTemplateBuilder::new();
        let (window, gl_config) = display_builder
            .build(&event_loop, config_template_builder, |mut configs| {
                // Just use the first configuration since we don't have any special preferences here
                configs.next().unwrap()
            })
            .unwrap();
        let window = window.unwrap();

        // Now we get the window size to use as the initial size of the Surface
        let (width, height): (u32, u32) = window.inner_size().into();
        let attrs =
            glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new()
                .build(
                    window.raw_window_handle(),
                    NonZeroU32::new(width).unwrap(),
                    NonZeroU32::new(height).unwrap(),
                );

        // Finally we can create a Surface, use it to make a PossiblyCurrentContext and create the glium Display
        let surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };
        let context_attributes = glutin::context::ContextAttributesBuilder::new()
            .build(Some(window.raw_window_handle()));
        let current_context = Some(unsafe {
            gl_config
                .display()
                .create_context(&gl_config, &context_attributes)
                .expect("failed to create context")
        })
        .unwrap()
        .make_current(&surface)
        .unwrap();
        let display = Display::from_context_surface(current_context, surface).unwrap();

        (window, display)
    }
}
