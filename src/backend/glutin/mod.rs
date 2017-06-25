#![cfg(feature = "glutin")]
/*!

Backend implementation for the glutin library

# Features

Only available if the 'glutin' feature is enabled.

*/
pub extern crate glutin;

pub mod headless;

use {Frame, GliumCreationError, SwapBuffersError};
use debug;
use context;
use backend;
use backend::Context;
use backend::Backend;
use self::glutin::winit;
use std;
use std::cell::{Cell, RefCell, Ref};
use std::rc::Rc;
use std::ops::Deref;
use std::os::raw::c_void;

/// A GL context combined with a facade for drawing upon.
///
/// The `Display` uses **glutin** for the GL context and **winit** for the window.
///
/// These are stored alongside a glium-specific context.
#[derive(Clone)]
pub struct Display {
    // contains everything related to the current context and its state
    context: Rc<context::Context>,
    // contains the window
    glutin: Rc<Glutin>,
    // Used to check whether the framebuffer dimensions have changed between frames. If they have,
    // the glutin context must be resized accordingly.
    last_framebuffer_dimensions: Cell<(u32, u32)>,
}

/// The winit window and associated glutin context used for the display.
pub struct Glutin {
    window: RefCell<winit::Window>,
    context: Rc<RefCell<glutin::Context>>,
}

/// An implementation of the `Backend` trait for glutin.
#[derive(Clone)]
pub struct GlutinBackend(Rc<Glutin>);

impl Display {
    /// Create a new glium `Display`.
    ///
    /// Performs a compatibility check to make sure that all core elements of glium are supported
    /// by the implementation.
    pub fn new(
        window: winit::Window,
        context: glutin::Context,
    ) -> Result<Self, GliumCreationError<glutin::CreationError>>
    {
        Self::with_debug(window, context, Default::default())
    }

    /// Create a new glium `Display`.
    ///
    /// This function does the same as `build_glium`, except that the resulting context
    /// will assume that the current OpenGL context will never change.
    pub unsafe fn unchecked(
        window: winit::Window,
        context: glutin::Context,
    ) -> Result<Self, GliumCreationError<glutin::CreationError>>
    {
        Self::unchecked_with_debug(window, context, Default::default())
    }

    /// The same as the `new` constructor, but allows for specifying debug callback behaviour.
    pub fn with_debug(
        window: winit::Window,
        context: glutin::Context,
        debug: debug::DebugCallbackBehavior,
    ) -> Result<Self, GliumCreationError<glutin::CreationError>>
    {
        Self::new_inner(window, context, debug, true)
    }

    /// The same as the `unchecked` constructor, but allows for specifying debug callback behaviour.
    pub unsafe fn unchecked_with_debug(
        window: winit::Window,
        context: glutin::Context,
        debug: debug::DebugCallbackBehavior,
    ) -> Result<Self, GliumCreationError<glutin::CreationError>>
    {
        Self::new_inner(window, context, debug, false)
    }

    fn new_inner(
        window: winit::Window,
        context: glutin::Context,
        debug: debug::DebugCallbackBehavior,
        checked: bool,
    ) -> Result<Self, GliumCreationError<glutin::CreationError>>
    {
        let window = RefCell::new(window);
        let glutin_context = Rc::new(RefCell::new(context));
        let glutin = Rc::new(Glutin { window: window, context: glutin_context });
        let glutin_backend = GlutinBackend(glutin.clone());
        let framebuffer_dimensions = glutin_backend.get_framebuffer_dimensions();
        let context = try!(unsafe { context::Context::new(glutin_backend, checked, debug) });
        Ok(Display {
            glutin: glutin,
            context: context,
            last_framebuffer_dimensions: Cell::new(framebuffer_dimensions),
        })
    }

    /// Rebuilds the Display's glutin GL context with the given ContextBuilder.
    ///
    /// This method ensures that the created `glutin::Context` will share the display lists of the
    /// original context.
    pub fn rebuild(&self, builder: glutin::ContextBuilder)
        -> Result<(), GliumCreationError<glutin::CreationError>>
    {
        // Share the display lists of the existing context.
        let new_context = {
            let context = self.glutin.context.borrow();
            try!(builder.with_shared_lists(&context).build(&self.glutin.window.borrow()))
        };

        // Replace the stored context with the newly built one.
        let mut context = self.glutin.context.borrow_mut();
        std::mem::replace(&mut (*context), new_context);

        Ok(())
    }

    /// Rebuilds the Display's window and glutin GL context.
    ///
    /// This method ensures that the created `glutin::Context` will share the display lists of the
    /// original context.
    pub fn rebuild_window(&self, new_window: winit::Window, builder: glutin::ContextBuilder)
        -> Result<(), GliumCreationError<glutin::CreationError>>
    {
        // Share the display lists of the existing context.
        let new_context = {
            let context = self.glutin.context.borrow();
            try!(builder.with_shared_lists(&context).build(&new_window))
        };

        // Replace the stored window and context with the new ones.
        let mut window = self.glutin.window.borrow_mut();
        std::mem::replace(&mut (*window), new_window);
        let mut context = self.glutin.context.borrow_mut();
        std::mem::replace(&mut (*context), new_context);

        Ok(())
    }

    /// Borrow the inner winit window.
    #[inline]
    pub fn get_window(&self) -> Ref<winit::Window> {
        self.glutin.window.borrow()
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
        let (w, h) = self.get_framebuffer_dimensions();

        // If the size of the framebuffer has changed, resize the context.
        if self.last_framebuffer_dimensions.get() != (w, h) {
            self.last_framebuffer_dimensions.set((w, h));
            self.glutin.context.borrow().resize(w, h);
        }

        Frame::new(self.context.clone(), (w, h))
    }
}

impl Deref for Display {
    type Target = Context;
    #[inline]
    fn deref(&self) -> &Context {
        &self.context
    }
}

impl backend::Facade for Display {
    #[inline]
    fn get_context(&self) -> &Rc<Context> {
        &self.context
    }
}

impl Deref for GlutinBackend {
    type Target = Glutin;
    #[inline]
    fn deref(&self) -> &Glutin {
        &self.0
    }
}

unsafe impl Backend for GlutinBackend {
    #[inline]
    fn swap_buffers(&self) -> Result<(), SwapBuffersError> {
        match self.context.borrow().swap_buffers() {
            Ok(()) => Ok(()),
            Err(glutin::ContextError::IoError(e)) => panic!("Error while swapping buffers: {:?}", e),
            Err(glutin::ContextError::ContextLost) => Err(SwapBuffersError::ContextLost),
        }
    }

    #[inline]
    unsafe fn get_proc_address(&self, symbol: &str) -> *const c_void {
        self.context.borrow().get_proc_address(symbol) as *const _
    }

    #[inline]
    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        // TODO: 800x600 ?
        let window = self.window.borrow();
        let (width, height) = window.get_inner_size().unwrap_or((800, 600));
        let scale = window.hidpi_factor();
        ((width as f32 * scale) as u32, (height as f32 * scale) as u32)
    }

    #[inline]
    fn is_current(&self) -> bool {
        self.context.borrow().is_current()
    }

    #[inline]
    unsafe fn make_current(&self) {
        self.context.borrow().make_current().unwrap();
    }
}

impl Glutin {
    #[allow(missing_docs)]
    #[inline]
    pub fn get_window(&self) -> Ref<winit::Window> {
        self.window.borrow()
    }
}
