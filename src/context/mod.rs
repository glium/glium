use gl;
use glutin;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Sender, channel};
use version::Api;
use GliumCreationError;

pub use self::capabilities::Capabilities;
pub use self::extensions::ExtensionsList;
pub use self::glutin_context::new_from_window;
#[cfg(feature = "headless")]
pub use self::glutin_context::new_from_headless;
pub use self::state::GLState;
pub use version::Version as GlVersion;

mod capabilities;
mod commands;
mod extensions;
mod glutin_context;
mod state;

pub struct Context {
    commands: commands::Sender,

    window: Option<Arc<RwLock<glutin::Window>>>,

    capabilities: Arc<Capabilities>,

    version: GlVersion,

    extensions: ExtensionsList,
}

pub struct CommandContext<'a, 'b> {
    pub gl: &'a gl::Gl,
    pub state: &'b mut GLState,
    pub version: &'a GlVersion,
    pub extensions: &'a ExtensionsList,
    pub capabilities: &'a Capabilities,
    pub shared_debug_output: &'a SharedDebugOutput,
}

/// Struct shared with the debug output callback.
pub struct SharedDebugOutput {
    /// Whether debug output should report errors
    pub report_errors: AtomicBool,
}

impl SharedDebugOutput {
    pub fn new() -> Arc<SharedDebugOutput> {
        Arc::new(SharedDebugOutput {
            report_errors: AtomicBool::new(true),
        })
    }
}

/// Iterator for all the events received by the window.
pub struct PollEventsIter<'a> {
    window: Option<RwLockReadGuard<'a, glutin::Window>>,
}

impl<'a> Iterator for PollEventsIter<'a> {
    type Item = glutin::Event;

    fn next(&mut self) -> Option<glutin::Event> {
        if let Some(window) = self.window.as_ref() {
            window.poll_events().next()
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
    window: Option<RwLockReadGuard<'a, glutin::Window>>,
}

impl<'a> Iterator for WaitEventsIter<'a> {
    type Item = glutin::Event;

    fn next(&mut self) -> Option<glutin::Event> {
        if let Some(window) = self.window.as_ref() {
            window.wait_events().next()
        } else {
            None
        }
    }
}

impl Context {
    pub fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        // FIXME: 800x600?
        self.window.as_ref().and_then(|w| w.read().unwrap().get_inner_size()).unwrap_or((800, 600))
    }

    pub fn exec<F>(&self, f: F) where F: FnOnce(CommandContext) + Send + 'static {
        self.commands.push(f);
    }

    pub fn exec_maybe_sync<'a, F>(&self, sync: bool, f: F) where F: FnOnce(CommandContext) + 'a {
        let (tx, rx) = if sync {
            let (tx, rx) = channel();
            (Some(tx), Some(rx))
        } else {
            (None, None)
        };

        self.commands.push(move |c: CommandContext| {
            f(c);
            if sync {
                tx.unwrap().send(()).unwrap();
            };
        });

        if sync {
            rx.unwrap().recv().unwrap();
        }
    }

    pub fn rebuild(&self, builder: glutin::WindowBuilder<'static>)
                   -> Result<(), GliumCreationError>
    {
        let (tx, rx) = channel();
        self.commands.push_rebuild(builder, tx);
        rx.recv().unwrap()
    }

    pub fn swap_buffers(&self) {
        self.commands.push_endframe();
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

    pub fn poll_events(&self) -> PollEventsIter {
        PollEventsIter {
            window: self.get_window(),
        }
    }

    pub fn wait_events(&self) -> WaitEventsIter {
        WaitEventsIter {
            window: self.get_window(),
        }
    }

    /// Returns the underlying window, or `None` if a headless context is used.
    pub fn get_window(&self) -> Option<RwLockReadGuard<glutin::Window>> {
        self.window.as_ref().map(|w| w.read().unwrap())
    }
}

fn check_gl_compatibility(ctxt: CommandContext) -> Result<(), GliumCreationError> {
    let mut result = Vec::new();

    if !(ctxt.version >= &GlVersion(Api::Gl, 1, 5)) &&
        (!ctxt.extensions.gl_arb_vertex_buffer_object ||
            !ctxt.extensions.gl_arb_map_buffer_range)
    {
        result.push("OpenGL implementation doesn't support buffer objects");
    }

    if !(ctxt.version >= &GlVersion(Api::Gl, 2, 0)) && (!ctxt.extensions.gl_arb_shader_objects ||
        !ctxt.extensions.gl_arb_vertex_shader || !ctxt.extensions.gl_arb_fragment_shader)
    {
        result.push("OpenGL implementation doesn't support vertex/fragment shaders");
    }

    if !ctxt.extensions.gl_ext_framebuffer_object && ctxt.version < &GlVersion(Api::Gl, 3, 0) {
        result.push("OpenGL implementation doesn't support framebuffers");
    }

    if !ctxt.extensions.gl_ext_framebuffer_blit && ctxt.version < &GlVersion(Api::Gl, 3, 0) {
        result.push("OpenGL implementation doesn't support blitting framebuffers");
    }

    if !ctxt.extensions.gl_arb_vertex_array_object &&
        !ctxt.extensions.gl_apple_vertex_array_object &&
        !(ctxt.version >= &GlVersion(Api::Gl, 3, 0))
    {
        result.push("OpenGL implementation doesn't support vertex array objects");
    }

    if cfg!(feature = "gl_uniform_blocks") && ctxt.version < &GlVersion(Api::Gl, 3, 1) &&
        !ctxt.extensions.gl_arb_uniform_buffer_object
    {
        result.push("OpenGL implementation doesn't support uniform blocks");
    }

    if cfg!(feature = "gl_sync") && ctxt.version < &GlVersion(Api::Gl, 3, 2) &&
        !ctxt.extensions.gl_arb_sync
    {
        result.push("OpenGL implementation doesn't support synchronization objects");
    }

    if cfg!(feature = "gl_persistent_mapping") && ctxt.version < &GlVersion(Api::Gl, 4, 4) &&
        !ctxt.extensions.gl_arb_buffer_storage
    {
        result.push("OpenGL implementation doesn't support persistent mapping");
    }

    if cfg!(feature = "gl_program_binary") && ctxt.version < &GlVersion(Api::Gl, 4, 1) &&
        !ctxt.extensions.gl_arb_get_programy_binary
    {
        result.push("OpenGL implementation doesn't support program binary");
    }

    if cfg!(feature = "gl_tessellation") && ctxt.version < &GlVersion(Api::Gl, 4, 0) &&
        !ctxt.extensions.gl_arb_tessellation_shader
    {
        result.push("OpenGL implementation doesn't support tessellation");
    }

    if cfg!(feature = "gl_instancing") && ctxt.version < &GlVersion(Api::Gl, 3, 3) &&
        !ctxt.extensions.gl_arb_instanced_arrays
    {
        result.push("OpenGL implementation doesn't support instancing");
    }

    if cfg!(feature = "gl_integral_textures") && ctxt.version < &GlVersion(Api::Gl, 3, 0) &&
        !ctxt.extensions.gl_ext_texture_integer
    {
        result.push("OpenGL implementation doesn't support integral textures");
    }

    if cfg!(feature = "gl_depth_textures") && ctxt.version < &GlVersion(Api::Gl, 3, 0) &&
        (!ctxt.extensions.gl_arb_depth_texture || !ctxt.extensions.gl_ext_packed_depth_stencil)
    {
        result.push("OpenGL implementation doesn't support depth or depth-stencil textures");
    }

    if cfg!(feature = "gl_stencil_textures") && ctxt.version < &GlVersion(Api::Gl, 3, 0)
    {
        result.push("OpenGL implementation doesn't support stencil textures");
    }

    if result.len() == 0 {
        Ok(())
    } else {
        Err(GliumCreationError::IncompatibleOpenGl(result.connect("\n")))
    }
}
