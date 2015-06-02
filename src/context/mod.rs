//! Contains everything related to the interface between glium and the OpenGL implementation.

use gl;
use libc;

use std::env;
use std::mem;
use std::ptr;
use std::borrow::Cow;
use std::collections::HashMap;
use std::cell::{Cell, RefCell, RefMut};
use std::marker::PhantomData;
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
pub use self::state::GlState;

mod capabilities;
mod extensions;
mod state;

/// Stores the state and information required for glium to execute commands. Most public glium
/// functions require passing a `Rc<Context>`.
pub struct Context {
    /// Contains the pointers to OpenGL functions.
    gl: gl::Gl,

    /// The current state of the OpenGL state machine. Contains for example which buffer is binded
    /// to which bind point, whether depth testing is activated, etc.
    state: RefCell<GlState>,

    /// Version of OpenGL of the backend.
    version: Version,

    /// Tells whether or not the backend supports each extension.
    extensions: ExtensionsList,

    /// Constants defined by the backend and retreived at initialization. For example, number
    /// of texture units, maximum size of the viewport, etc.
    capabilities: Capabilities,

    /// Glue between glium and the code that handles windowing. Contains functions that allows
    /// you to swap buffers, retreive the size of the framebuffer, etc.
    backend: RefCell<Box<Backend>>,

    /// Whether or not glium must check that the OpenGL context is the current one before each
    /// call.
    check_current_context: bool,

    /// Whether or not errors triggered by ARB_debug_output (and similar extensions) should be
    /// reported to the user (by panicking). This must be set to `false` in some situations,
    /// like compiling/linking shaders.
    report_debug_output_errors: Cell<bool>,

    /// We maintain a cache of FBOs.
    /// The `Option` is here in order to destroy the container. It must be filled at all time
    /// is a normal situation.
    framebuffer_objects: Option<fbo::FramebuffersContainer>,

    /// We maintain a list of vertex array objecs.
    vertex_array_objects: vertex_array_object::VertexAttributesSystem,

    /// We maintain a list of samplers for each possible behavior.
    samplers: RefCell<HashMap<uniforms::SamplerBehavior, sampler_object::SamplerObject>>,
}

/// This struct is a guard that is returned when you want to access the OpenGL backend.
pub struct CommandContext<'a> {
    /// Source of OpenGL function pointers.
    pub gl: &'a gl::Gl,

    /// Refers to the state of the OpenGL backend. Maintained between multiple calls.
    /// **Must** be synchronized with the real state of the backend.
    pub state: RefMut<'a, GlState>,

    /// Version of the backend.
    pub version: &'a Version,

    /// Extensions supported by the backend.
    pub extensions: &'a ExtensionsList,

    /// Capabilities of the backend.
    pub capabilities: &'a Capabilities,

    /// Whether or not errors triggered by ARB_debug_output (and similar extensions) should be
    /// reported to the user (by panicking).
    pub report_debug_output_errors: &'a Cell<bool>,

    /// The list of vertex array objects.
    pub vertex_array_objects: &'a vertex_array_object::VertexAttributesSystem,

    /// This marker is here to prevent `CommandContext` from implementing `Send`
    // TODO: use this when possible
    //impl<'a, 'b> !Send for CommandContext<'a, 'b> {}
    marker: PhantomData<*mut u8>,
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
    ///
    /// The OpenGL context must be newly-created. If you make modifications to the context before
    /// passing it to this function, glium's state cache may mismatch the actual one.
    ///
    pub unsafe fn new<B, E>(backend: B, check_current_context: bool)
                            -> Result<Rc<Context>, GliumCreationError<E>>
                            where B: Backend + 'static
    {
        backend.make_current();

        let gl = gl::Gl::load_with(|symbol| backend.get_proc_address(symbol));
        let gl_state: RefCell<GlState> = RefCell::new(Default::default());
        let version = version::get_gl_version(&gl);
        let extensions = extensions::get_extensions(&gl, &version);
        let capabilities = capabilities::get_capabilities(&gl, &version, &extensions);
        let report_debug_output_errors = Cell::new(true);

        {
            let mut state = gl_state.borrow_mut();
            state.texture_units.reserve(capabilities.max_combined_texture_image_units as usize);
            state.indexed_atomic_counter_buffer_bindings.reserve(capabilities.max_indexed_atomic_counter_buffer as usize);
            state.indexed_shader_storage_buffer_bindings.reserve(capabilities.max_indexed_shader_storage_buffer as usize);
            state.indexed_transform_feedback_buffer_bindings.reserve(capabilities.max_indexed_transform_feedback_buffer as usize);
            state.indexed_uniform_buffer_bindings.reserve(capabilities.max_indexed_uniform_buffer as usize);
        }

        let vertex_array_objects = vertex_array_object::VertexAttributesSystem::new();

        // checking whether the backend supports glium
        // TODO: do this more properly
        {
            let mut ctxt = CommandContext {
                gl: &gl,
                state: gl_state.borrow_mut(),
                version: &version,
                extensions: &extensions,
                capabilities: &capabilities,
                report_debug_output_errors: &report_debug_output_errors,
                vertex_array_objects: &vertex_array_objects,
                marker: PhantomData,
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
            vertex_array_objects: vertex_array_objects,
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
    /// The new context **must** have lists shared with the old one.
    pub unsafe fn rebuild<B, E>(&self, new_backend: B)
                                -> Result<(), GliumCreationError<E>>
                                where B: Backend + 'static
    {
        // framebuffer objects and vertex array objects aren't shared,
        // so we have to destroy them
        {
            let mut ctxt = self.make_current();

            if let Some(ref fbos) = self.framebuffer_objects {
                fbos.purge_all(&mut ctxt);
            }

            vertex_array_object::VertexAttributesSystem::purge_all(&mut ctxt);
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

        // swapping
        backend.swap_buffers();
    }

    /// Returns the OpenGL version detected by this context.
    pub fn get_version(&self) -> &Version {
        &self.version
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
    /// let pixels: Vec<Vec<(u8, u8, u8, u8)>> = display.read_front_buffer();
    /// # }
    /// ```
    pub fn read_front_buffer<T>(&self) -> T
                                where T: texture::Texture2dDataSink<(u8, u8, u8, u8)>
    {
        let mut ctxt = self.make_current();
        let dimensions = self.get_framebuffer_dimensions();
        let rect = ::Rect { left: 0, bottom: 0, width: dimensions.0, height: dimensions.1 };

        let mut data = Vec::with_capacity(0);
        ops::read(&mut ctxt, ops::Source::DefaultFramebuffer(gl::FRONT_LEFT), &rect, &mut data);
        T::from_raw(Cow::Owned(data), dimensions.0, dimensions.1)
    }

    /// Execute an arbitrary closure with the OpenGL context active. Useful if another
    /// component needs to directly manipulate OpenGL state.
    ///
    /// **If `action` manipulates any OpenGL state, it must be restored before `action`
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
    pub fn assert_no_error(&self, user_msg: Option<&str>) {
        let mut ctxt = self.make_current();

        match (::get_gl_error(&mut ctxt), user_msg) {
            (Some(msg), None) => panic!("{}", msg),
            (Some(msg), Some(user_msg)) => panic!("{} : {}", user_msg, msg),
            (None, _) => ()
        };
    }

    /// Waits until all the previous commands have finished being executed.
    ///
    /// When you execute OpenGL functions, they are not executed immediately. Instead they are
    /// put in a queue. This function waits until all commands have finished being executed, and
    /// the queue is empty.
    ///
    /// You normally don't need to call this function manually, except for debugging purposes.
    pub fn synchronize(&self) {
        let ctxt = self.make_current();
        unsafe { ctxt.gl.Finish(); }
    }

    /// Inserts a debugging string in the commands queue. If you use an OpenGL debugger, you will
    /// be able to see that string.
    ///
    /// This is helpful to understand where you are when you have big applications.
    ///
    /// Returns `Err` if the backend doesn't support this functionnality. You can choose whether
    /// to call `.unwrap()` if you want to make sure that it works, or `.ok()` if you don't care.
    pub fn insert_debug_marker(&self, marker: &str) -> Result<(), ()> {
        let ctxt = self.make_current();

        if ctxt.extensions.gl_gremedy_string_marker {
            let marker = marker.as_bytes();
            unsafe { ctxt.gl.StringMarkerGREMEDY(marker.len() as gl::types::GLsizei,
                                                 marker.as_ptr() as *const _) };
            Ok(())

        } else if ctxt.extensions.gl_ext_debug_marker {
            let marker = marker.as_bytes();
            unsafe { ctxt.gl.InsertEventMarkerEXT(marker.len() as gl::types::GLsizei,
                                                  marker.as_ptr() as *const _) };
            Ok(())

        } else {
            Err(())
        }
    }

    /// Same as `insert_debug_marker`, except that if you don't compile with `debug_assertions`
    /// it is a no-op and returns `Ok`.
    pub fn debug_insert_debug_marker(&self, marker: &str) -> Result<(), ()> {
        if cfg!(debug_assertions) {
            self.insert_debug_marker(marker)
        } else {
            Ok(())
        }
    }
}

impl ContextExt for Context {
    fn set_report_debug_output_errors(&self, value: bool) {
        self.report_debug_output_errors.set(value);
    }

    fn make_current(&self) -> CommandContext {
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
            vertex_array_objects: &self.vertex_array_objects,
            marker: PhantomData,
        }
    }

    fn get_framebuffer_objects(&self) -> &fbo::FramebuffersContainer {
        self.framebuffer_objects.as_ref().unwrap()
    }

    fn get_samplers(&self) -> &RefCell<HashMap<uniforms::SamplerBehavior,
                                               sampler_object::SamplerObject>>
    {
        &self.samplers
    }

    fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }

    fn get_extensions(&self) -> &ExtensionsList {
        &self.extensions
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
                vertex_array_objects: &self.vertex_array_objects,
                marker: PhantomData,
            };

            let fbos = self.framebuffer_objects.take();
            fbos.unwrap().cleanup(&mut ctxt);

            vertex_array_object::VertexAttributesSystem::cleanup(&mut ctxt);

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

/// Checks whether the backend supports glium. Returns an `Err` if it doesn't.
fn check_gl_compatibility<T>(ctxt: &mut CommandContext) -> Result<(), GliumCreationError<T>> {
    let mut result = Vec::with_capacity(0);

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
        (!ctxt.extensions.gl_arb_depth_texture || !ctxt.extensions.gl_ext_packed_depth_stencil) &&
        (!ctxt.extensions.gl_oes_depth_texture || !ctxt.extensions.gl_oes_packed_depth_stencil)
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

/// Initializes `GL_KHR_debug`, `GL_ARB_debug`, or a similar extension so that the debug output
/// is reported.
fn init_debug_callback(context: &Rc<Context>) {
    if !cfg!(debug_assertions) {
        return;
    }

    // TODO: remove this
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
                // reporting
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
