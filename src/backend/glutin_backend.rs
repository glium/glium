/*!

Backend implementation for the glutin library

*/
extern crate glutin;

use libc;

use DisplayBuild;
use Frame;
use GliumCreationError;
use texture;

use context;
use backend;
use backend::Context;
use backend::Backend;
use version::Version;

use std::cell::{RefCell, Ref};
use std::rc::Rc;
use std::ops::Deref;

/// Facade implementation for glutin. Wraps both glium and glutin.
#[derive(Clone)]
pub struct GlutinFacade {
    // contains everything related to the current context and its state
    context: Rc<context::Context>,

    // contains the window
    backend: Rc<Option<RefCell<Rc<GlutinWindowBackend>>>>,
}

impl backend::Facade for GlutinFacade {
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

    fn deref(&self) -> &glutin::Window {
        self.0.get_window()
    }
}

impl GlutinFacade {
    /// Reads all events received by the window.
    ///
    /// This iterator polls for events and can be exhausted.
    pub fn poll_events(&self) -> PollEventsIter {
        PollEventsIter {
            window: self.backend.as_ref(),
        }
    }

    /// Reads all events received by the window.
    pub fn wait_events(&self) -> WaitEventsIter {
        WaitEventsIter {
            window: self.backend.as_ref(),
        }
    }

    /// Returns true if the window has been closed.
    pub fn is_closed(&self) -> bool {
        self.backend.as_ref().map(|b| b.borrow().is_closed()).unwrap_or(false)
    }

    /// Returns the underlying window, or `None` if glium uses a headless context.
    pub fn get_window(&self) -> Option<WinRef> {
        self.backend.as_ref().map(|w| WinRef(w.borrow()))
    }

    /// Returns the dimensions of the main framebuffer.
    pub fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        self.context.get_framebuffer_dimensions()
    }

    /// Returns the OpenGL version of the current context.
    pub fn get_opengl_version(&self) -> Version {
        *self.context.get_version()
    }

    /// Returns the supported GLSL version.
    pub fn get_supported_glsl_version(&self) -> Version {
        self.context.get_supported_glsl_version()
    }

    /// Returns true if the given GLSL version is supported.
    pub fn is_glsl_version_supported(&self, version: &Version) -> bool {
        self.context.is_glsl_version_supported(version)
    }

    /// Start drawing on the backbuffer.
    ///
    /// This function returns a `Frame`, which can be used to draw on it. When the `Frame` is
    /// destroyed, the buffers are swapped.
    ///
    /// Note that destroying a `Frame` is immediate, even if vsync is enabled.
    pub fn draw(&self) -> Frame {
        Frame::new(self.context.clone(), self.get_framebuffer_dimensions())
    }

    /// Returns the maximum value that can be used for anisotropic filtering, or `None`
    /// if the hardware doesn't support it.
    pub fn get_max_anisotropy_support(&self) -> Option<u16> {
        self.context.get_max_anisotropy_support()
    }

    /// Returns the maximum dimensions of the viewport.
    ///
    /// Glium will panic if you request a larger viewport than this when drawing.
    pub fn get_max_viewport_dimensions(&self) -> (u32, u32) {
        self.context.get_max_viewport_dimensions()
    }

    /// Releases the shader compiler, indicating that no new programs will be created for a while.
    ///
    /// # Features
    ///
    /// This method is always available, but is a no-op if it's not available in
    /// the implementation.
    pub fn release_shader_compiler(&self) {
        self.context.release_shader_compiler()
    }

    /// Returns an estimate of the amount of video memory available in bytes.
    ///
    /// Returns `None` if no estimate is available.
    pub fn get_free_video_memory(&self) -> Option<usize> {
        self.context.get_free_video_memory()
    }

    /// Reads the content of the front buffer.
    ///
    /// You will only see the data that has finished being drawn.
    ///
    /// This function can return any type that implements `Texture2dData`.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # extern crate glium;
    /// # extern crate glutin;
    /// # fn main() {
    /// # let display: glium::Display = unsafe { ::std::mem::uninitialized() };
    /// let pixels: Vec<Vec<(u8, u8, u8)>> = display.read_front_buffer();
    /// # }
    /// ```
    pub fn read_front_buffer<P, T>(&self) -> T          // TODO: remove Clone for P
                                   where P: texture::PixelValue + Clone + Send,
                                   T: texture::Texture2dDataSink<Data = P>
    {
        self.context.read_front_buffer()
    }

    /// Execute an arbitrary closure with the OpenGL context active. Useful if another
    /// component needs to directly manipulate OpenGL state.
    ///
    /// **If action manipulates any OpenGL state, it must be restored before action
    /// completes.**
    pub unsafe fn exec_in_context<'a, T, F>(&self, action: F) -> T
                                            where T: Send + 'static,
                                            F: FnOnce() -> T + 'a
    {
        self.context.exec_in_context(action)
    }

    /// Asserts that there are no OpenGL errors pending.
    ///
    /// This function should be used in tests.
    pub fn assert_no_error(&self, user_msg: Option<&str>) {
        self.context.assert_no_error(user_msg)
    }

    /// Waits until all the previous commands have finished being executed.
    ///
    /// When you execute OpenGL functions, they are not executed immediately. Instead they are
    /// put in a queue. This function waits until all commands have finished being executed, and
    /// the queue is empty.
    ///
    /// **You don't need to call this function manually, except when running benchmarks.**
    pub fn synchronize(&self) {
        self.context.synchronize()
    }
}

impl DisplayBuild for glutin::WindowBuilder<'static> {
    type Facade = GlutinFacade;
    type Err = GliumCreationError<glutin::CreationError>;

    fn build_glium(self) -> Result<GlutinFacade, GliumCreationError<glutin::CreationError>> {
        let backend = Rc::new(try!(backend::glutin_backend::GlutinWindowBackend::new(self)));
        let context = try!(unsafe { context::Context::new(backend.clone(), true) });

        let display = GlutinFacade {
            context: context,
            backend: Rc::new(Some(RefCell::new(backend))),
        };

        Ok(display)
    }

    unsafe fn build_glium_unchecked(self) -> Result<GlutinFacade, GliumCreationError<glutin::CreationError>> {
        let backend = Rc::new(try!(backend::glutin_backend::GlutinWindowBackend::new(self)));
        let context = try!(context::Context::new(backend.clone(), false));

        let display = GlutinFacade {
            context: context,
            backend: Rc::new(Some(RefCell::new(backend))),
        };

        Ok(display)
    }

    fn rebuild_glium(self, display: &GlutinFacade) -> Result<(), GliumCreationError<glutin::CreationError>> {
        let mut existing_window = display.backend.as_ref()
                                         .expect("can't rebuild a headless display").borrow_mut();
        let new_backend = Rc::new(try!(existing_window.rebuild(self)));
        try!(unsafe { display.context.rebuild(new_backend.clone()) });
        *existing_window = new_backend;
        Ok(())
    }
}

impl DisplayBuild for glutin::HeadlessRendererBuilder {
    type Facade = GlutinFacade;
    type Err = GliumCreationError<glutin::CreationError>;

    fn build_glium(self) -> Result<GlutinFacade, GliumCreationError<glutin::CreationError>> {
        let backend = Rc::new(try!(backend::glutin_backend::GlutinHeadlessBackend::new(self)));
        let context = try!(unsafe { context::Context::new(backend.clone(), true) });

        let display = GlutinFacade {
            context: context,
            backend: Rc::new(None),
        };

        Ok(display)
    }

    unsafe fn build_glium_unchecked(self) -> Result<GlutinFacade, GliumCreationError<glutin::CreationError>> {
        let backend = Rc::new(try!(backend::glutin_backend::GlutinHeadlessBackend::new(self)));
        let context = try!(context::Context::new(backend.clone(), true));

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
    fn swap_buffers(&self) {
        self.window.swap_buffers();
    }

    unsafe fn get_proc_address(&self, symbol: &str) -> *const libc::c_void {
        self.window.get_proc_address(symbol)
    }

    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        let (width, height) = self.window.get_inner_size().unwrap_or((800, 600));      // TODO: 800x600 ?
        let scale = self.window.hidpi_factor();
        (width * scale as u32, height * scale as u32)
    }

    fn is_current(&self) -> bool {
        self.window.is_current()
    }

    unsafe fn make_current(&self) {
        self.window.make_current();
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

    pub fn get_window(&self) -> &glutin::Window {
        &self.window
    }

    pub fn is_closed(&self) -> bool {
        self.window.is_closed()
    }

    pub fn poll_events(&self) -> glutin::PollEventsIterator {
        self.window.poll_events()
    }

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
    fn swap_buffers(&self) {
    }

    unsafe fn get_proc_address(&self, symbol: &str) -> *const libc::c_void {
        self.context.get_proc_address(symbol)
    }

    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        (800, 600)      // FIXME: these are random
    }

    fn is_current(&self) -> bool {
        self.context.is_current()
    }

    unsafe fn make_current(&self) {
        self.context.make_current();
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
