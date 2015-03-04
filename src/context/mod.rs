use gl;

use std::default::Default;
use std::cell::{RefCell, Ref};
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Sender, channel};

use GliumCreationError;
use backend::Backend;
use version;
use version::Api;

pub use self::capabilities::Capabilities;
pub use self::extensions::ExtensionsList;
pub use self::state::GLState;
pub use version::Version as GlVersion;      // TODO: remove

mod capabilities;
mod extensions;
mod state;

pub struct Context {
    gl: gl::Gl,
    state: RefCell<GLState>,
    version: GlVersion,
    extensions: ExtensionsList,
    capabilities: Capabilities,
    shared_debug_output: Rc<SharedDebugOutput>,

    backend: Box<Backend>,
    check_current_context: bool,
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
    pub fn new() -> Rc<SharedDebugOutput> {
        Rc::new(SharedDebugOutput {
            report_errors: AtomicBool::new(true),
        })
    }
}

impl Context {
    pub fn new<B>(backend: B, check_current_context: bool)
                  -> Result<(Context, Rc<SharedDebugOutput>), GliumCreationError>
                  where B: Backend + 'static
    {
        backend.make_current();
        let gl = gl::Gl::load_with(|symbol| backend.get_proc_address(symbol));

        let mut gl_state = Default::default();
        let version = version::get_gl_version(&gl);
        let extensions = extensions::get_extensions(&gl);
        let capabilities = capabilities::get_capabilities(&gl, &version, &extensions);

        let shared_debug_frontend = SharedDebugOutput::new();
        let shared_debug_backend = shared_debug_frontend.clone();

        try!(check_gl_compatibility(CommandContext {
            gl: &gl,
            state: &mut gl_state,
            version: &version,
            extensions: &extensions,
            capabilities: &capabilities,
            shared_debug_output: &shared_debug_backend,
        }));

        let backend = Box::new(backend);

        Ok((Context {
            gl: gl,
            state: RefCell::new(gl_state),
            version: version,
            extensions: extensions,
            capabilities: capabilities,
            shared_debug_output: shared_debug_backend,
            backend: backend,
            check_current_context: check_current_context,
        }, shared_debug_frontend))
    }

    pub fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        self.backend.get_framebuffer_dimensions()
    }

    pub fn exec<F>(&self, f: F) where F: FnOnce(CommandContext) {
        unsafe { self.make_current() };

        f(CommandContext {
            gl: &self.gl,
            state: &mut *self.state.borrow_mut(),
            version: &self.version,
            extensions: &self.extensions,
            capabilities: &self.capabilities,
            shared_debug_output: &*self.shared_debug_output,
        });
    }

    pub fn rebuild<B>(&self, new_backend: B)
                      -> Result<(), GliumCreationError>
                      where B: Backend + 'static
    {
        unsafe { new_backend.make_current(); };

        // FIXME: remove this hack
        let me: &mut Context = unsafe { ::std::mem::transmute(self) };
        me.state = Default::default();
        // FIXME: verify version, capabilities and extensions

        me.backend = Box::new(new_backend);

        Ok(())
    }

    pub fn swap_buffers(&self) {
        self.make_current();

        // this is necessary on Windows 8, or nothing is being displayed
        unsafe { self.gl.Flush(); }

        // swapping
        self.backend.swap_buffers();
    }

    pub fn make_current(&self) {
        if !self.backend.is_current() {
            unsafe {
                self.backend.make_current();
            }
        }
    }

    pub fn capabilities(&self) -> &Capabilities {
        &self.capabilities
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

    if cfg!(feature = "gl_texture_multisample") && ctxt.version < &GlVersion(Api::Gl, 3, 2)
    {
        result.push("OpenGL implementation doesn't support multisample textures");
    }

    if cfg!(feature = "gl_texture_multisample_array") && ctxt.version < &GlVersion(Api::Gl, 3, 2)
    {
        result.push("OpenGL implementation doesn't support arrays of multisample textures");
    }

    if result.len() == 0 {
        Ok(())
    } else {
        Err(GliumCreationError::IncompatibleOpenGl(result.connect("\n")))
    }
}
