use gl;
use libc;

use std::env;
use std::mem;
use std::ptr;
use std::collections::HashMap;
use std::default::Default;
use std::cell::{Cell, RefCell, RefMut};
use std::ffi::CStr;
use std::rc::Rc;

use GliumCreationError;
use ContextExt;
use backend::Backend;
use version;
use version::Api;
use version::Version;

use fbo;
use ops;
use sampler_object;
use texture;
use uniforms;
use vertex_array_object;

pub use self::capabilities::Capabilities;
pub use self::extensions::ExtensionsList;
pub use self::state::GLState;

mod capabilities;
mod extensions;
mod state;

/// Stores the state and information required for glium to execute commands. Most public glium
/// functions require passing a `Context`.
pub struct Context {
    gl: gl::Gl,
    state: RefCell<GLState>,
    version: Version,
    extensions: ExtensionsList,
    capabilities: Capabilities,

    backend: RefCell<Box<Backend>>,
    check_current_context: bool,

    report_debug_output_errors: Cell<bool>,

    // we maintain a list of FBOs
    // the option is here to destroy the container
    pub framebuffer_objects: Option<fbo::FramebuffersContainer>,

    pub vertex_array_objects: vertex_array_object::VertexAttributesSystem,

    // we maintain a list of samplers for each possible behavior
    pub samplers: RefCell<HashMap<uniforms::SamplerBehavior, sampler_object::SamplerObject>>,
}

pub struct CommandContext<'a, 'b> {
    pub gl: &'a gl::Gl,
    pub state: RefMut<'b, GLState>,
    pub version: &'a Version,
    pub extensions: &'a ExtensionsList,
    pub capabilities: &'a Capabilities,
    pub report_debug_output_errors: &'a Cell<bool>,
}

impl Context {
    /// Builds a new context.
    ///
    /// The `check_current_context` parameter tells the context whether it should check
    /// if the backend's OpenGL context is the current one before each OpenGL operation.
    ///
    /// If you pass `false`, you must ensure that no other OpenGL context is going to be made
    /// current in the same thread as this context. Passing `true` makes things safe but
    /// is slightly slower.
    pub unsafe fn new<B, E>(backend: B, check_current_context: bool)
                            -> Result<Rc<Context>, GliumCreationError<E>>
                            where B: Backend + 'static
    {
        backend.make_current();
        let gl = gl::Gl::load_with(|symbol| backend.get_proc_address(symbol));

        let gl_state = RefCell::new(Default::default());
        let version = version::get_gl_version(&gl);
        let extensions = extensions::get_extensions(&gl);
        let capabilities = capabilities::get_capabilities(&gl, &version, &extensions);
        let report_debug_output_errors = Cell::new(true);

        {
            let mut ctxt = CommandContext {
                gl: &gl,
                state: gl_state.borrow_mut(),
                version: &version,
                extensions: &extensions,
                capabilities: &capabilities,
                report_debug_output_errors: &report_debug_output_errors,
            };

            try!(check_gl_compatibility(&mut ctxt));
        }

        let context = Rc::new(Context {
            gl: gl,
            state: gl_state,
            version: version,
            extensions: extensions,
            capabilities: capabilities,
            report_debug_output_errors: report_debug_output_errors,
            backend: RefCell::new(Box::new(backend)),
            check_current_context: check_current_context,
            framebuffer_objects: Some(fbo::FramebuffersContainer::new()),
            vertex_array_objects: vertex_array_object::VertexAttributesSystem::new(),
            samplers: RefCell::new(HashMap::new()),
        });

        init_debug_callback(&context);

        Ok(context)
    }

    /// Calls `get_framebuffer_dimensions` on the backend object stored by this context.
    pub fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        self.backend.borrow().get_framebuffer_dimensions()
    }

    /// Changes the OpenGL context associated with this context.
    ///
    /// The new context must have lists shared with the old one.
    pub unsafe fn rebuild<B, E>(&self, new_backend: B)
                                -> Result<(), GliumCreationError<E>>
                                where B: Backend + 'static
    {
        {
            let mut ctxt = self.make_current();

            // framebuffer objects and vertex array objects aren't shared, so we have to destroy them
            if let Some(ref fbos) = self.framebuffer_objects {
                fbos.purge_all(&mut ctxt);
            }

            self.vertex_array_objects.purge_all(&mut ctxt);
        }

        new_backend.make_current();

        *self.state.borrow_mut() = Default::default();
        // FIXME: verify version, capabilities and extensions
        *self.backend.borrow_mut() = Box::new(new_backend);

        Ok(())
    }

    /// Swaps the buffers in the backend.
    pub fn swap_buffers(&self) {
        let backend = self.backend.borrow();

        if self.check_current_context {
            if !backend.is_current() {
                unsafe { backend.make_current() };
            }
        }

        // this is necessary on Windows 8, or nothing is being displayed
        unsafe { self.gl.Flush(); }

        // swapping
        backend.swap_buffers();
    }

    // TODO: make me private
    pub fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }

    /// Returns the OpenGL version detected by this context.
    pub fn get_version(&self) -> &Version {
        &self.version
    }

    // TODO: make me private
    pub fn get_extensions(&self) -> &ExtensionsList {
        &self.extensions
    }

    /// Returns the GLSL version guaranteed to be supported.
    pub fn get_supported_glsl_version(&self) -> Version {
        version::get_supported_glsl_version(self.get_version())
    }

    /// Returns true if the given GLSL version is supported.
    pub fn is_glsl_version_supported(&self, version: &Version) -> bool {
        self.capabilities().supported_glsl_versions.iter().find(|&v| v == version).is_some()
    }

    /// Returns the maximum value that can be used for anisotropic filtering, or `None`
    /// if the hardware doesn't support it.
    pub fn get_max_anisotropy_support(&self) -> Option<u16> {
        self.capabilities().max_texture_max_anisotropy.map(|v| v as u16)
    }

    /// Returns the maximum dimensions of the viewport.
    ///
    /// Glium will panic if you request a larger viewport than this when drawing.
    pub fn get_max_viewport_dimensions(&self) -> (u32, u32) {
        let d = self.capabilities().max_viewport_dims;
        (d.0 as u32, d.1 as u32)
    }

    /// Releases the shader compiler, indicating that no new programs will be created for a while.
    ///
    /// This method is a no-op if it's not available in the implementation.
    pub fn release_shader_compiler(&self) {
        unsafe {
            let ctxt = self.make_current();

            if ctxt.version >= &Version(Api::GlEs, 2, 0) ||
                ctxt.version >= &Version(Api::Gl, 4, 1)
            {
                if !ctxt.capabilities.supported_glsl_versions.is_empty() {
                    ctxt.gl.ReleaseShaderCompiler();
                }
            }
        }
    }

    /// Returns an estimate of the amount of video memory available in bytes.
    ///
    /// Returns `None` if no estimate is available.
    pub fn get_free_video_memory(&self) -> Option<usize> {
        unsafe {
            let ctxt = self.make_current();

            let mut value: [gl::types::GLint; 4] = mem::uninitialized();

            if ctxt.extensions.gl_nvx_gpu_memory_info {
                ctxt.gl.GetIntegerv(gl::GPU_MEMORY_INFO_CURRENT_AVAILABLE_VIDMEM_NVX,
                               &mut value[0]);
                Some(value[0] as usize * 1024)

            } else if ctxt.extensions.gl_ati_meminfo {
                ctxt.gl.GetIntegerv(gl::TEXTURE_FREE_MEMORY_ATI, &mut value[0]);
                Some(value[0] as usize * 1024)

            } else {
                return None;
            }
        }
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
        ops::read_from_default_fb(gl::FRONT_LEFT, &self)
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
        let _ctxt = self.make_current();
        action()
    }

    /// Asserts that there are no OpenGL errors pending.
    ///
    /// This function should be used in tests.
    pub fn assert_no_error(&self) {
        let mut ctxt = self.make_current();

        match ::get_gl_error(&mut ctxt) {
            Some(msg) => panic!("{}", msg),
            None => ()
        };
    }

    /// Waits until all the previous commands have finished being executed.
    ///
    /// When you execute OpenGL functions, they are not executed immediately. Instead they are
    /// put in a queue. This function waits until all commands have finished being executed, and
    /// the queue is empty.
    ///
    /// **You don't need to call this function manually, except when running benchmarks.**
    pub fn synchronize(&self) {
        let ctxt = self.make_current();
        unsafe { ctxt.gl.Finish(); }
    }
}

impl ContextExt for Context {
    fn set_report_debug_output_errors(&self, value: bool) {
        self.report_debug_output_errors.set(value);
    }

    fn make_current<'a>(&'a self) -> CommandContext<'a, 'a> {
        if self.check_current_context {
            let backend = self.backend.borrow();
            if !backend.is_current() {
                unsafe { backend.make_current() };
            }
        }

        CommandContext {
            gl: &self.gl,
            state: self.state.borrow_mut(),
            version: &self.version,
            extensions: &self.extensions,
            capabilities: &self.capabilities,
            report_debug_output_errors: &self.report_debug_output_errors,
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            // this is the code of make_current duplicated here because we can't borrow
            // `self` twice
            if self.check_current_context {
                let backend = self.backend.borrow();
                if !backend.is_current() {
                    backend.make_current();
                }
            }

            let mut ctxt = CommandContext {
                gl: &self.gl,
                state: self.state.borrow_mut(),
                version: &self.version,
                extensions: &self.extensions,
                capabilities: &self.capabilities,
                report_debug_output_errors: &self.report_debug_output_errors,
            };

            let fbos = self.framebuffer_objects.take();
            fbos.unwrap().cleanup(&mut ctxt);

            self.vertex_array_objects.cleanup(&mut ctxt);

            let mut samplers = self.samplers.borrow_mut();
            for (_, s) in mem::replace(&mut *samplers, HashMap::with_capacity(0)) {
                s.destroy(&mut ctxt);
            }

            // disabling callback
            if ctxt.state.enabled_debug_output != Some(false) {
                if ctxt.version >= &Version(Api::Gl, 4,5) || ctxt.extensions.gl_khr_debug {
                    ctxt.gl.Disable(gl::DEBUG_OUTPUT);
                } else if ctxt.extensions.gl_arb_debug_output {
                    ctxt.gl.DebugMessageCallbackARB(mem::transmute(0usize),
                                                    ptr::null());
                }

                ctxt.state.enabled_debug_output = Some(false);
                ctxt.gl.Finish();
            }
        }
    }
}

fn check_gl_compatibility<T>(ctxt: &mut CommandContext) -> Result<(), GliumCreationError<T>> {
    let mut result = Vec::new();

    if !(ctxt.version >= &Version(Api::Gl, 1, 5)) &&
        !(ctxt.version >= &Version(Api::GlEs, 2, 0)) &&
        (!ctxt.extensions.gl_arb_vertex_buffer_object || !ctxt.extensions.gl_arb_map_buffer_range)
    {
        result.push("OpenGL implementation doesn't support buffer objects");
    }

    if !(ctxt.version >= &Version(Api::Gl, 2, 0)) &&
        !(ctxt.version >= &Version(Api::GlEs, 2, 0)) &&
        (!ctxt.extensions.gl_arb_shader_objects ||
            !ctxt.extensions.gl_arb_vertex_shader || !ctxt.extensions.gl_arb_fragment_shader)
    {
        result.push("OpenGL implementation doesn't support vertex/fragment shaders");
    }

    if !ctxt.extensions.gl_ext_framebuffer_object && !(ctxt.version >= &Version(Api::Gl, 3, 0)) &&
        !(ctxt.version >= &Version(Api::GlEs, 2, 0))
    {
        result.push("OpenGL implementation doesn't support framebuffers");
    }

    if !ctxt.extensions.gl_ext_framebuffer_blit && !(ctxt.version >= &Version(Api::Gl, 3, 0)) &&
        !(ctxt.version >= &Version(Api::GlEs, 2, 0))
    {
        result.push("OpenGL implementation doesn't support blitting framebuffers");
    }

    if cfg!(feature = "gl_uniform_blocks") && !(ctxt.version >= &Version(Api::Gl, 3, 1)) &&
        !ctxt.extensions.gl_arb_uniform_buffer_object
    {
        result.push("OpenGL implementation doesn't support uniform blocks");
    }

    if cfg!(feature = "gl_sync") && !(ctxt.version >= &Version(Api::Gl, 3, 2)) &&
        !(ctxt.version >= &Version(Api::GlEs, 3, 0)) && !ctxt.extensions.gl_arb_sync
    {
        result.push("OpenGL implementation doesn't support synchronization objects");
    }

    if cfg!(feature = "gl_program_binary") && !(ctxt.version >= &Version(Api::Gl, 4, 1)) &&
        !ctxt.extensions.gl_arb_get_programy_binary
    {
        result.push("OpenGL implementation doesn't support program binary");
    }

    if cfg!(feature = "gl_tessellation") && !(ctxt.version >= &Version(Api::Gl, 4, 0)) &&
        !ctxt.extensions.gl_arb_tessellation_shader
    {
        result.push("OpenGL implementation doesn't support tessellation");
    }

    if cfg!(feature = "gl_instancing") && !(ctxt.version >= &Version(Api::Gl, 3, 3)) &&
        !ctxt.extensions.gl_arb_instanced_arrays
    {
        result.push("OpenGL implementation doesn't support instancing");
    }

    if cfg!(feature = "gl_integral_textures") && !(ctxt.version >= &Version(Api::Gl, 3, 0)) &&
        !ctxt.extensions.gl_ext_texture_integer
    {
        result.push("OpenGL implementation doesn't support integral textures");
    }

    if cfg!(feature = "gl_depth_textures") && !(ctxt.version >= &Version(Api::Gl, 3, 0)) &&
        (!ctxt.extensions.gl_arb_depth_texture || !ctxt.extensions.gl_ext_packed_depth_stencil)
    {
        result.push("OpenGL implementation doesn't support depth or depth-stencil textures");
    }

    if cfg!(feature = "gl_stencil_textures") && !(ctxt.version >= &Version(Api::Gl, 3, 0))
    {
        result.push("OpenGL implementation doesn't support stencil textures");
    }

    if cfg!(feature = "gl_texture_multisample") && !(ctxt.version >= &Version(Api::Gl, 3, 2))
    {
        result.push("OpenGL implementation doesn't support multisample textures");
    }

    if cfg!(feature = "gl_texture_multisample_array") &&
        !(ctxt.version >= &Version(Api::Gl, 3, 2))
    {
        result.push("OpenGL implementation doesn't support arrays of multisample textures");
    }

    if result.len() == 0 {
        Ok(())
    } else {
        Err(GliumCreationError::IncompatibleOpenGl(result.connect("\n")))
    }
}

fn init_debug_callback(context: &Rc<Context>) {
    if !cfg!(debug_assertions) {
        return;
    }

    if env::var("GLIUM_DISABLE_DEBUG_OUTPUT").is_ok() {
        return;
    }

    // this is the C callback
    extern "system" fn callback_wrapper(source: gl::types::GLenum, ty: gl::types::GLenum,
                                        id: gl::types::GLuint, severity: gl::types::GLenum,
                                        _length: gl::types::GLsizei,
                                        message: *const gl::types::GLchar,
                                        user_param: *mut libc::c_void)
    {
        let user_param = user_param as *const Context;
        let user_param: &Context = unsafe { mem::transmute(user_param) };

        if (severity == gl::DEBUG_SEVERITY_HIGH || severity == gl::DEBUG_SEVERITY_MEDIUM) && 
           (ty == gl::DEBUG_TYPE_ERROR || ty == gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR ||
            ty == gl::DEBUG_TYPE_PORTABILITY || ty == gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR)
        {
            if user_param.report_debug_output_errors.get() {
                let message = unsafe {
                    String::from_utf8(CStr::from_ptr(message).to_bytes().to_vec()).unwrap()
                };

                panic!("Debug message with high or medium severity: `{}`.\n\
                        Please report this error: https://github.com/tomaka/glium/issues",
                        message);
            }
        }
    }

    struct ContextRawPtr(*const Context);
    unsafe impl Send for ContextRawPtr {}
    let context_raw_ptr = ContextRawPtr(&**context);

    unsafe {
        let mut ctxt = context.make_current();

        if ctxt.version >= &Version(Api::Gl, 4,5) || ctxt.extensions.gl_khr_debug ||
            ctxt.extensions.gl_arb_debug_output
        {
            if ctxt.state.enabled_debug_output_synchronous != true {
                ctxt.gl.Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
                ctxt.state.enabled_debug_output_synchronous = true;
            }

            if ctxt.version >= &Version(Api::Gl, 4, 5) ||
                (ctxt.version >= &Version(Api::Gl, 1, 0) && ctxt.extensions.gl_khr_debug)
            {
                ctxt.gl.DebugMessageCallback(callback_wrapper, context_raw_ptr.0
                                                                 as *const libc::c_void);
                ctxt.gl.DebugMessageControl(gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE, 0,
                                            ptr::null(), gl::TRUE);

                if ctxt.state.enabled_debug_output != Some(true) {
                    ctxt.gl.Enable(gl::DEBUG_OUTPUT);
                    ctxt.state.enabled_debug_output = Some(true);
                }

            } else if ctxt.version >= &Version(Api::GlEs, 2, 0) &&
                ctxt.extensions.gl_khr_debug
            {
                ctxt.gl.DebugMessageCallbackKHR(callback_wrapper, context_raw_ptr.0
                                                                 as *const libc::c_void);
                ctxt.gl.DebugMessageControlKHR(gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE, 0,
                                               ptr::null(), gl::TRUE);

                if ctxt.state.enabled_debug_output != Some(true) {
                    ctxt.gl.Enable(gl::DEBUG_OUTPUT);
                    ctxt.state.enabled_debug_output = Some(true);
                }

            } else {
                ctxt.gl.DebugMessageCallbackARB(callback_wrapper, context_raw_ptr.0
                                                                    as *const libc::c_void);
                ctxt.gl.DebugMessageControlARB(gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE,
                                               0, ptr::null(), gl::TRUE);

                ctxt.state.enabled_debug_output = Some(true);
            }
        }
    }
}
