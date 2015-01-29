use gl;
use glutin;
use std::sync::atomic::{self, AtomicUint};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use GliumCreationError;

pub use self::capabilities::Capabilities;
pub use self::extensions::ExtensionsList;
pub use self::glutin_context::new_from_window;
#[cfg(feature = "headless")]
pub use self::glutin_context::new_from_headless;
pub use self::state::GLState;
pub use self::version::GlVersion;

mod capabilities;
mod extensions;
mod glutin_context;
mod state;
mod version;

enum Message {
    EndFrame,
    Execute(Box<for<'a, 'b> ::std::thunk::Invoke<CommandContext<'a, 'b>, ()> + Send>),
    NextEvent(Sender<glutin::Event>),
}

pub struct Context {
    commands: Mutex<Sender<Message>>,
    events: Mutex<Receiver<glutin::Event>>,

    /// Dimensions of the frame buffer.
    dimensions: Arc<(AtomicUint, AtomicUint)>,

    capabilities: Arc<Capabilities>,

    version: GlVersion,

    extensions: ExtensionsList,
}

pub struct CommandContext<'a, 'b> {
    pub gl: &'a gl::Gl,
    pub state: &'b mut GLState,
    pub version: &'a GlVersion,
    pub extensions: &'a ExtensionsList,
    pub opengl_es: bool,
    pub capabilities: &'a Capabilities,
}

/// Iterator for all the events received by the window.
pub struct PollEventsIter<'a> {
    context: &'a Context,
    current_rx: Option<Receiver<glutin::Event>>,
}

impl<'a> Iterator for PollEventsIter<'a> {
    type Item = glutin::Event;

    fn next(&mut self) -> Option<glutin::Event> {
        if let Some(ref mut current_rx) = self.current_rx {
            match current_rx.recv() {
                Ok(ev) => return Some(ev),
                Err(_) => ()
            };
        }

        let (tx, rx) = channel();
        self.context.commands.lock().unwrap().send(Message::NextEvent(tx));

        match rx.recv() {
            Ok(ev) => {
                self.current_rx = Some(rx);
                return Some(ev);
            },
            Err(_) => {
                return None;
            }
        };
    }
}

/// Blocking iterator over all the events received by the window.
///
/// This iterator polls for events, until the window associated with its context
/// is closed.
pub struct WaitEventsIter<'a> {
    context: &'a Context,
    current_rx: Option<Receiver<glutin::Event>>,
    closing: bool,
}

impl<'a> WaitEventsIter<'a> {
    fn new(context: &'a Context) -> WaitEventsIter<'a> {
        WaitEventsIter {
            context: context,
            current_rx: None,
            closing: false,
        }
    }
}

impl<'a> Iterator for WaitEventsIter<'a> {
    type Item = glutin::Event;

    fn next(&mut self) -> Option<glutin::Event> {
        if self.closing {
            return None;
        }
        if let Some(ref mut current_rx) = self.current_rx {
            match current_rx.recv() {
                Ok(ev) => return Some(ev),
                Err(_) => ()
            };
        }

        loop {
            let (tx, rx) = channel();
            self.context.commands.lock().unwrap().send(Message::NextEvent(tx));

            match rx.recv() {
                Ok(ev @ glutin::Event::Closed) => {
                    self.closing = true;
                    return Some(ev);
                },
                Ok(ev) => {
                    self.current_rx = Some(rx);
                    return Some(ev);
                },
                Err(_) => (),
            };
        }
    }
}

impl Context {
    pub fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        (
            self.dimensions.0.load(atomic::Ordering::Relaxed) as u32,
            self.dimensions.1.load(atomic::Ordering::Relaxed) as u32,
        )
    }

    pub fn exec<F>(&self, f: F) where F: FnOnce(CommandContext) + Send {
        self.commands.lock().unwrap().send(Message::Execute(Box::new(f))).unwrap();
    }

    pub fn swap_buffers(&self) {
        self.commands.lock().unwrap().send(Message::EndFrame).unwrap();
    }

    pub fn poll_events(&self) -> PollEventsIter {
        PollEventsIter {
            context: self,
            current_rx: None,
        }
    }

    pub fn wait_events(&self) -> WaitEventsIter {
        WaitEventsIter::new(self)
    }

    pub fn capabilities(&self) -> &Capabilities {
        &*self.capabilities
    }

    pub fn get_version(&self) -> &GlVersion {
        &self.version
    }

    pub fn get_extensions(&self) -> &ExtensionsList {
        &self.extensions
    }
}

fn check_gl_compatibility(ctxt: CommandContext) -> Result<(), GliumCreationError> {
    let mut result = Vec::new();

    if ctxt.opengl_es {
        if ctxt.version < &GlVersion(3, 0) {
            result.push("OpenGL ES version inferior to 3.0");
        }

        if cfg!(feature = "gl_read_buffer") {
            result.push("OpenGL ES doesn't support gl_read_buffer");
        }

    } else {
        if ctxt.version < &GlVersion(1, 5) && (!ctxt.extensions.gl_arb_vertex_buffer_object ||
            !ctxt.extensions.gl_arb_map_buffer_range)
        {
            result.push("OpenGL implementation doesn't support buffer objects");
        }

        if ctxt.version < &GlVersion(2, 0) && (!ctxt.extensions.gl_arb_shader_objects ||
            !ctxt.extensions.gl_arb_vertex_shader || !ctxt.extensions.gl_arb_fragment_shader)
        {
            result.push("OpenGL implementation doesn't support vertex/fragment shaders");
        }

        if !ctxt.extensions.gl_ext_framebuffer_object && ctxt.version < &GlVersion(3, 0) {
            result.push("OpenGL implementation doesn't support framebuffers");
        }

        if !ctxt.extensions.gl_ext_framebuffer_blit && ctxt.version < &GlVersion(3, 0) {
            result.push("OpenGL implementation doesn't support blitting framebuffers");
        }

        if !ctxt.extensions.gl_arb_vertex_array_object &&
            !ctxt.extensions.gl_apple_vertex_array_object &&
            ctxt.version < &GlVersion(3, 0)
        {
            result.push("OpenGL implementation doesn't support vertex array objects");
        }

        if cfg!(feature = "gl_uniform_blocks") && ctxt.version < &GlVersion(3, 1) &&
            !ctxt.extensions.gl_arb_uniform_buffer_object
        {
            result.push("OpenGL implementation doesn't support uniform blocks");
        }

        if cfg!(feature = "gl_sync") && ctxt.version < &GlVersion(3, 2) &&
            !ctxt.extensions.gl_arb_sync
        {
            result.push("OpenGL implementation doesn't support synchronization objects");
        }

        if cfg!(feature = "gl_persistent_mapping") && ctxt.version < &GlVersion(4, 4) &&
            !ctxt.extensions.gl_arb_buffer_storage
        {
            result.push("OpenGL implementation doesn't support persistent mapping");
        }

        if cfg!(feature = "gl_program_binary") && ctxt.version < &GlVersion(4, 1) &&
            !ctxt.extensions.gl_arb_get_programy_binary
        {
            result.push("OpenGL implementation doesn't support program binary");
        }

        if cfg!(feature = "gl_tessellation") && ctxt.version < &GlVersion(4, 0) &&
            !ctxt.extensions.gl_arb_tessellation_shader
        {
            result.push("OpenGL implementation doesn't support tessellation");
        }

        if cfg!(feature = "gl_instancing") && ctxt.version < &GlVersion(3, 3) &&
            !ctxt.extensions.gl_arb_instanced_arrays
        {
            result.push("OpenGL implementation doesn't support instancing");
        }
    }

    if result.len() == 0 {
        Ok(())
    } else {
        Err(GliumCreationError::IncompatibleOpenGl(result.connect("\n")))
    }
}
