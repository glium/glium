#![cfg(feature = "glutin")]
/*!

Backend implementation for the glutin library

# Features

Only available if the 'glutin' feature is enabled.

*/
pub extern crate glutin;

pub mod headless;

use takeable_option::Takeable;
use {Frame, IncompatibleOpenGl, SwapBuffersError};
use debug;
use context;
use backend;
use backend::Context;
use backend::Backend;
use std::cell::{Cell, RefCell, Ref};
use std::error::Error;
use std::fmt;
use std::rc::Rc;
use std::ops::Deref;
use std::os::raw::c_void;
use glutin::{PossiblyCurrent as Pc, ContextCurrentState};

/// A GL context combined with a facade for drawing upon.
///
/// The `Display` uses **glutin** for the **Window** and its associated GL **Context**.
///
/// These are stored alongside a glium-specific context.
#[derive(Clone)]
pub struct Display {
    // contains everything related to the current context and its state
    context: Rc<context::Context>,
    // The glutin Window alongside its associated GL Context.
    gl_window: Rc<RefCell<Takeable<glutin::WindowedContext<Pc>>>>,
    // Used to check whether the framebuffer dimensions have changed between frames. If they have,
    // the glutin context must be resized accordingly.
    last_framebuffer_dimensions: Cell<(u32, u32)>,
}

/// An implementation of the `Backend` trait for glutin.
#[derive(Clone)]
pub struct GlutinBackend(Rc<RefCell<Takeable<glutin::WindowedContext<Pc>>>>);

/// Error that can happen while creating a glium display.
#[derive(Debug)]
pub enum DisplayCreationError {
    /// An error has happened while creating the backend.
    GlutinCreationError(glutin::CreationError),
    /// The OpenGL implementation is too old.
    IncompatibleOpenGl(IncompatibleOpenGl),
}

impl Display {
    /// Create a new glium `Display` from the given context and window builders.
    ///
    /// Performs a compatibility check to make sure that all core elements of glium are supported
    /// by the implementation.
    pub fn new<T: ContextCurrentState, E>(
        wb: glutin::window::WindowBuilder,
        cb: glutin::ContextBuilder<T>,
        events_loop: &glutin::event_loop::EventLoop<E>,
    ) -> Result<Self, DisplayCreationError>
    {
        let gl_window = cb.build_windowed(wb, events_loop)?;
        Self::from_gl_window(gl_window).map_err(From::from)
    }

    /// Create a new glium `Display`.
    ///
    /// Performs a compatibility check to make sure that all core elements of glium are supported
    /// by the implementation.
    pub fn from_gl_window<T: ContextCurrentState>(gl_window: glutin::WindowedContext<T>) -> Result<Self, IncompatibleOpenGl> {
        Self::with_debug(gl_window, Default::default())
    }

    /// Create a new glium `Display`.
    ///
    /// This function does the same as `build_glium`, except that the resulting context
    /// will assume that the current OpenGL context will never change.
    pub unsafe fn unchecked<T: ContextCurrentState>(gl_window: glutin::WindowedContext<T>) -> Result<Self, IncompatibleOpenGl> {
        Self::unchecked_with_debug(gl_window, Default::default())
    }

    /// The same as the `new` constructor, but allows for specifying debug callback behaviour.
    pub fn with_debug<T: ContextCurrentState>(gl_window: glutin::WindowedContext<T>, debug: debug::DebugCallbackBehavior)
        -> Result<Self, IncompatibleOpenGl>
    {
        Self::new_inner(gl_window, debug, true)
    }

    /// The same as the `unchecked` constructor, but allows for specifying debug callback behaviour.
    pub unsafe fn unchecked_with_debug<T: ContextCurrentState>(
        gl_window: glutin::WindowedContext<T>,
        debug: debug::DebugCallbackBehavior,
    ) -> Result<Self, IncompatibleOpenGl>
    {
        Self::new_inner(gl_window, debug, false)
    }

    fn new_inner<T: ContextCurrentState>(
        gl_window: glutin::WindowedContext<T>,
        debug: debug::DebugCallbackBehavior,
        checked: bool,
    ) -> Result<Self, IncompatibleOpenGl>
    {
        let gl_window = unsafe {
            gl_window.treat_as_current()
        };
        let gl_window = Rc::new(RefCell::new(Takeable::new(gl_window)));
        let glutin_backend = GlutinBackend(gl_window.clone());
        let framebuffer_dimensions = glutin_backend.get_framebuffer_dimensions();
        let context = unsafe { context::Context::new(glutin_backend, checked, debug) }?;
        Ok(Display {
            gl_window: gl_window,
            context: context,
            last_framebuffer_dimensions: Cell::new(framebuffer_dimensions),
        })
    }

    /// Rebuilds the Display's `WindowedContext` with the given window and context builders.
    ///
    /// This method ensures that the new `WindowedContext`'s `Context` will share the display lists of the
    /// original `WindowedContext`'s `Context`.
    pub fn rebuild<T: ContextCurrentState>(
        &self,
        wb: glutin::window::WindowBuilder,
        cb: glutin::ContextBuilder<T>,
        events_loop: &glutin::event_loop::EventLoop<()>,
    ) -> Result<(), DisplayCreationError>
    {
        // Share the display lists of the existing context.
        let new_gl_window = {
            let gl_window = self.gl_window.borrow();
            let cb = cb.with_shared_lists(gl_window.context());
            cb.build_windowed(wb, events_loop)?
        };
        let new_gl_window = unsafe {
            new_gl_window.treat_as_current()
        };

        // Replace the stored WindowedContext with the new one.
        {
            let mut gl_window = self.gl_window.borrow_mut();
            Takeable::insert(&mut gl_window, new_gl_window);
        }

        // Rebuild the Context.
        let backend = GlutinBackend(self.gl_window.clone());
        unsafe { self.context.rebuild(backend) }?;

        Ok(())
    }

    /// Borrow the inner glutin WindowedContext.
    #[inline]
    pub fn gl_window(&self) -> Ref<Takeable<glutin::WindowedContext<Pc>>> {
        self.gl_window.borrow()
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
            self.gl_window.borrow().resize((w, h).into());
        }

        Frame::new(self.context.clone(), (w, h))
    }
}

impl fmt::Display for DisplayCreationError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", self.description())
    }
}

impl Error for DisplayCreationError {
    #[inline]
    fn description(&self) -> &str {
        match *self {
            DisplayCreationError::GlutinCreationError(ref err) => err.description(),
            DisplayCreationError::IncompatibleOpenGl(ref err) => err.description(),
        }
    }

    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            DisplayCreationError::GlutinCreationError(ref err) => Some(err),
            DisplayCreationError::IncompatibleOpenGl(ref err) => Some(err),
        }
    }
}

impl From<glutin::CreationError> for DisplayCreationError {
    #[inline]
    fn from(err: glutin::CreationError) -> DisplayCreationError {
        DisplayCreationError::GlutinCreationError(err)
    }
}

impl From<IncompatibleOpenGl> for DisplayCreationError {
    #[inline]
    fn from(err: IncompatibleOpenGl) -> DisplayCreationError {
        DisplayCreationError::IncompatibleOpenGl(err)
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
    type Target = Rc<RefCell<Takeable<glutin::WindowedContext<Pc>>>>;
    #[inline]
    fn deref(&self) -> &Rc<RefCell<Takeable<glutin::WindowedContext<Pc>>>> {
        &self.0
    }
}

unsafe impl Backend for GlutinBackend {
    #[inline]
    fn swap_buffers(&self) -> Result<(), SwapBuffersError> {
        match self.borrow().swap_buffers() {
            Ok(()) => Ok(()),
            Err(glutin::ContextError::IoError(e)) => panic!("I/O Error while swapping buffers: {:?}", e),
            Err(glutin::ContextError::OsError(e)) => panic!("OS Error while swapping buffers: {:?}", e),
            // As of writing the FunctionUnavailable error is only thrown if
            // you are swapping buffers with damage rectangles specified.
            // Currently we don't support this so we just panic as this
            // case should be unreachable.
            Err(glutin::ContextError::FunctionUnavailable) => panic!("function unavailable error while swapping buffers"),
            Err(glutin::ContextError::ContextLost) => Err(SwapBuffersError::ContextLost),
        }
    }

    #[inline]
    unsafe fn get_proc_address(&self, symbol: &str) -> *const c_void {
        self.borrow().get_proc_address(symbol) as *const _
    }

    #[inline]
    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        let gl_window_takeable = self.borrow();
        let gl_window = gl_window_takeable.window();
        let (width, height) = gl_window.inner_size()
            .into();
        (width, height)
    }

    #[inline]
    fn is_current(&self) -> bool {
        self.borrow().is_current()
    }

    #[inline]
    unsafe fn make_current(&self) {
        let mut gl_window_takeable = self.borrow_mut();
        let gl_window = Takeable::take(&mut gl_window_takeable);
        let new_gl_window = gl_window.make_current().unwrap();
        Takeable::insert(&mut gl_window_takeable, new_gl_window);
    }
}
