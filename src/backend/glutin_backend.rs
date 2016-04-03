#![cfg(feature = "glutin")]
/*!

Backend implementation for the glutin library

# Features

Only available if the 'glutin' feature is enabled.

*/
pub extern crate glutin;

use DisplayBuild;
use Frame;
use GliumCreationError;
use SwapBuffersError;

use debug;
use context;
use backend;
use backend::Context;
use backend::Backend;

use std::cell::{RefCell, Ref};
use std::rc::Rc;
use std::ops::Deref;
use std::os::raw::c_void;

/// Facade implementation for glutin. Wraps both glium and glutin.
#[derive(Clone)]
pub struct GlutinFacade {
    // contains everything related to the current context and its state
    context: Rc<context::Context>,

    // contains the window
    backend: Rc<Option<RefCell<Rc<GlutinWindowBackend>>>>,
}

impl backend::Facade for GlutinFacade {
    #[inline]
    fn get_context(&self) -> &Rc<Context> {
        &self.context
    }
}

/// Iterator for all the events received by the window.
pub struct PollEventsIter<'a> {
    window: Option<&'a RefCell<Rc<GlutinWindowBackend>>>,
}

impl<'a> Iterator for PollEventsIter<'a> {
    type Item = glutin::Event;

    #[inline]
    fn next(&mut self) -> Option<glutin::Event> {
        if let Some(window) = self.window.as_ref() {
            window.borrow().poll_events().next()
        } else {
            None
        }
    }
}

/// Blocking iterator over all the events received by the window.
///
/// This iterator polls for events, until the window associated with its context
/// is closed.
pub struct WaitEventsIter<'a> {
    window: Option<&'a RefCell<Rc<GlutinWindowBackend>>>,
}

impl<'a> Iterator for WaitEventsIter<'a> {
    type Item = glutin::Event;

    #[inline]
    fn next(&mut self) -> Option<glutin::Event> {
        if let Some(window) = self.window.as_ref() {
            window.borrow().wait_events().next()
        } else {
            None
        }
    }
}

/// Borrow of the glutin window.
pub struct WinRef<'a>(Ref<'a, Rc<GlutinWindowBackend>>);

impl<'a> Deref for WinRef<'a> {
    type Target = glutin::Window;

    #[inline]
    fn deref(&self) -> &glutin::Window {
        self.0.get_window()
    }
}

impl GlutinFacade {
    /// Reads all events received by the window.
    ///
    /// This iterator polls for events and can be exhausted.
    #[inline]
    pub fn poll_events(&self) -> PollEventsIter {
        PollEventsIter {
            window: Option::as_ref(&self.backend),
        }
    }

    /// Reads all events received by the window.
    #[inline]
    pub fn wait_events(&self) -> WaitEventsIter {
        WaitEventsIter {
            window: Option::as_ref(&self.backend),
        }
    }

    /// Returns the underlying window, or `None` if glium uses a headless context.
    #[inline]
    pub fn get_window(&self) -> Option<WinRef> {
        Option::as_ref(&self.backend).map(|w| WinRef(w.borrow()))
    }

    /// Start drawing on the backbuffer.
    ///
    /// This function returns a `Frame`, which can be used to draw on it. When the `Frame` is
    /// destroyed, the buffers are swapped.
    ///
    /// Note that destroying a `Frame` is immediate, even if vsync is enabled.
    #[inline]
    pub fn draw(&self) -> Frame {
        Frame::new(self.context.clone(), self.get_framebuffer_dimensions())
    }
}

impl Deref for GlutinFacade {
    type Target = Context;

    #[inline]
    fn deref(&self) -> &Context {
        &self.context
    }
}

impl DisplayBuild for glutin::WindowBuilder<'static> {
    type Facade = GlutinFacade;
    type Err = GliumCreationError<glutin::CreationError>;

    fn build_glium_debug(self, debug: debug::DebugCallbackBehavior)
                         -> Result<GlutinFacade, GliumCreationError<glutin::CreationError>>
    {
        let backend = Rc::new(try!(backend::glutin_backend::GlutinWindowBackend::new(self)));
        let context = try!(unsafe { context::Context::new(backend.clone(), true, debug) });

        let display = GlutinFacade {
            context: context,
            backend: Rc::new(Some(RefCell::new(backend))),
        };

        Ok(display)
    }

    unsafe fn build_glium_unchecked_debug(self, debug: debug::DebugCallbackBehavior)
                                          -> Result<GlutinFacade, GliumCreationError<glutin::CreationError>>
    {
        let backend = Rc::new(try!(backend::glutin_backend::GlutinWindowBackend::new(self)));
        let context = try!(context::Context::new(backend.clone(), false, debug));

        let display = GlutinFacade {
            context: context,
            backend: Rc::new(Some(RefCell::new(backend))),
        };

        Ok(display)
    }

    fn rebuild_glium(self, display: &GlutinFacade) -> Result<(), GliumCreationError<glutin::CreationError>> {
        let mut existing_window = Option::as_ref(&display.backend)
                                         .expect("can't rebuild a headless display").borrow_mut();
        let new_backend = Rc::new(try!(existing_window.rebuild(self)));
        try!(unsafe { display.context.rebuild(new_backend.clone()) });
        *existing_window = new_backend;
        Ok(())
    }
}

impl<'a> DisplayBuild for glutin::HeadlessRendererBuilder<'a> {
    type Facade = GlutinFacade;
    type Err = GliumCreationError<glutin::CreationError>;

    fn build_glium_debug(self, debug: debug::DebugCallbackBehavior) -> Result<GlutinFacade, GliumCreationError<glutin::CreationError>> {
        let backend = Rc::new(try!(backend::glutin_backend::GlutinHeadlessBackend::new(self)));
        let context = try!(unsafe { context::Context::new(backend.clone(), true, Default::default()) });

        let display = GlutinFacade {
            context: context,
            backend: Rc::new(None),
        };

        Ok(display)
    }

    unsafe fn build_glium_unchecked_debug(self, debug: debug::DebugCallbackBehavior) -> Result<GlutinFacade, GliumCreationError<glutin::CreationError>> {
        let backend = Rc::new(try!(backend::glutin_backend::GlutinHeadlessBackend::new(self)));
        let context = try!(context::Context::new(backend.clone(), true, Default::default()));

        let display = GlutinFacade {
            context: context,
            backend: Rc::new(None),
        };

        Ok(display)
    }

    fn rebuild_glium(self, _: &GlutinFacade) -> Result<(), GliumCreationError<glutin::CreationError>> {
        unimplemented!()
    }
}

/// An implementation of the `Backend` trait for a glutin window.
pub struct GlutinWindowBackend {
    window: glutin::Window,
}

unsafe impl Backend for GlutinWindowBackend {
    #[inline]
    fn swap_buffers(&self) -> Result<(), SwapBuffersError> {
        match self.window.swap_buffers() {
            Ok(()) => Ok(()),
            Err(glutin::ContextError::IoError(e)) => panic!("Error while swapping buffers: {:?}", e),
            Err(glutin::ContextError::ContextLost) => Err(SwapBuffersError::ContextLost),
        }
    }

    #[inline]
    unsafe fn get_proc_address(&self, symbol: &str) -> *const c_void {
        self.window.get_proc_address(symbol) as *const _
    }

    #[inline]
    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        let (width, height) = self.window.get_inner_size().unwrap_or((800, 600));      // TODO: 800x600 ?
        let scale = self.window.hidpi_factor();
        ((width as f32 * scale) as u32, (height as f32 * scale) as u32)
    }

    #[inline]
    fn is_current(&self) -> bool {
        self.window.is_current()
    }

    #[inline]
    unsafe fn make_current(&self) {
        self.window.make_current().unwrap();
    }
}

#[allow(missing_docs)]
impl GlutinWindowBackend {
    /// Builds a new backend from the builder.
    pub fn new(builder: glutin::WindowBuilder)
               -> Result<GlutinWindowBackend, GliumCreationError<glutin::CreationError>>
    {
        let window = try!(builder.build());

        Ok(GlutinWindowBackend {
            window: window,
        })
    }

    #[inline]
    pub fn get_window(&self) -> &glutin::Window {
        &self.window
    }

    #[inline]
    pub fn poll_events(&self) -> glutin::PollEventsIterator {
        self.window.poll_events()
    }

    #[inline]
    pub fn wait_events(&self) -> glutin::WaitEventsIterator {
        self.window.wait_events()
    }

    pub fn rebuild(&self, builder: glutin::WindowBuilder)
                   -> Result<GlutinWindowBackend, GliumCreationError<glutin::CreationError>>
    {
        let window = try!(builder.with_shared_lists(&self.window).build());

        Ok(GlutinWindowBackend {
            window: window,
        })
    }
}

/// An implementation of the `Backend` trait for a glutin headless context.
pub struct GlutinHeadlessBackend {
    context: glutin::HeadlessContext,
}

unsafe impl Backend for GlutinHeadlessBackend {
    #[inline]
    fn swap_buffers(&self) -> Result<(), SwapBuffersError> {
        Ok(())
    }

    #[inline]
    unsafe fn get_proc_address(&self, symbol: &str) -> *const c_void {
        self.context.get_proc_address(symbol) as *const _
    }

    #[inline]
    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        (800, 600)      // FIXME: these are random
    }

    #[inline]
    fn is_current(&self) -> bool {
        self.context.is_current()
    }

    #[inline]
    unsafe fn make_current(&self) {
        self.context.make_current().unwrap();
    }
}

impl GlutinHeadlessBackend {
    /// Builds a new backend from the builder.
    pub fn new(builder: glutin::HeadlessRendererBuilder)
               -> Result<GlutinHeadlessBackend, GliumCreationError<glutin::CreationError>>
    {
        let context = try!(builder.build());

        Ok(GlutinHeadlessBackend {
            context: context,
        })
    }
}
