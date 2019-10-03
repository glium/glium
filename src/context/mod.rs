//! Contains everything related to the interface between glium and the OpenGL implementation.

use gl;
use backtrace;

use std::collections::HashMap;
use std::mem;
use std::ptr;
use std::str;
use std::borrow::Cow;
use std::cell::{Cell, RefCell, RefMut};
use std::marker::PhantomData;
use std::ffi::CStr;
use std::rc::Rc;
use std::os::raw;
use std::hash::BuildHasherDefault;

use fnv::FnvHasher;

use IncompatibleOpenGl;
use SwapBuffersError;
use CapabilitiesSource;
use ContextExt;
use backend::Backend;
use version;
use version::Api;
use version::Version;

use debug;
use fbo;
use ops;
use sampler_object;
use texture;
use uniforms;
use vertex_array_object;

pub use self::capabilities::{ReleaseBehavior, Capabilities, Profile};
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

    /// The current state of the OpenGL state machine. Contains for example which buffer is bound
    /// to which bind point, whether depth testing is activated, etc.
    state: RefCell<GlState>,

    /// Version of OpenGL of the backend.
    version: Version,

    /// Tells whether or not the backend supports each extension.
    extensions: ExtensionsList,

    /// Constants defined by the backend and retrieved at initialization. For example, number
    /// of texture units, maximum size of the viewport, etc.
    capabilities: Capabilities,

    /// Glue between glium and the code that handles windowing. Contains functions that allows
    /// you to swap buffers, retrieve the size of the framebuffer, etc.
    backend: RefCell<Box<dyn Backend>>,

    /// Whether or not glium must check that the OpenGL context is the current one before each
    /// call.
    check_current_context: bool,

    /// The callback that is used by the debug output feature.
    debug_callback: Option<debug::DebugCallback>,

    /// Whether or not errors triggered by ARB_debug_output (and similar extensions) should be
    /// reported to the user when `DebugCallbackBehavior::DebugMessageOnError` is used. This must
    /// be set to `false` in some situations, like compiling/linking shaders.
    report_debug_output_errors: Cell<bool>,

    /// We maintain a cache of FBOs.
    /// The `Option` is here in order to destroy the container. It must be filled at all time
    /// is a normal situation.
    framebuffer_objects: Option<fbo::FramebuffersContainer>,

    /// We maintain a list of vertex array objects.
    vertex_array_objects: vertex_array_object::VertexAttributesSystem,

    /// We maintain a list of samplers for each possible behavior.
    samplers: RefCell<HashMap<uniforms::SamplerBehavior, sampler_object::SamplerObject, BuildHasherDefault<FnvHasher>>>,

    /// List of texture handles that are resident. We need to call `MakeTextureHandleResidentARB`
    /// when rebuilding the context.
    resident_texture_handles: RefCell<Vec<gl::types::GLuint64>>,

    /// List of images handles that are resident. We need to call `MakeImageHandleResidentARB`
    /// when rebuilding the context.
    resident_image_handles: RefCell<Vec<(gl::types::GLuint64, gl::types::GLenum)>>,
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

    /// The list of framebuffer objects.
    pub framebuffer_objects: &'a fbo::FramebuffersContainer,

    /// The list of samplers.
    pub samplers: RefMut<'a, HashMap<uniforms::SamplerBehavior, sampler_object::SamplerObject, BuildHasherDefault<FnvHasher>>>,

    /// List of texture handles that need to be made resident.
    pub resident_texture_handles: RefMut<'a, Vec<gl::types::GLuint64>>,

    /// List of image handles and their access that need to be made resident.
    pub resident_image_handles: RefMut<'a, Vec<(gl::types::GLuint64, gl::types::GLenum)>>,

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
    pub unsafe fn new<B>(
        backend: B,
        check_current_context: bool,
        callback_behavior: DebugCallbackBehavior,
    ) -> Result<Rc<Context>, IncompatibleOpenGl>
        where B: Backend + 'static
    {
        backend.make_current();

        let gl = gl::Gl::load_with(|symbol| backend.get_proc_address(symbol) as *const _);
        let gl_state: RefCell<GlState> = RefCell::new(Default::default());

        let version = version::get_gl_version(&gl);
        let extensions = extensions::get_extensions(&gl, &version);
        check_gl_compatibility(&version, &extensions)?;

        let capabilities = capabilities::get_capabilities(&gl, &version, &extensions);
        let report_debug_output_errors = Cell::new(true);

        let vertex_array_objects = vertex_array_object::VertexAttributesSystem::new();
        let framebuffer_objects = fbo::FramebuffersContainer::new();
        let samplers = RefCell::new({
            let mut map = HashMap::with_hasher(Default::default());
            map.reserve(16);
            map
        });
        let resident_texture_handles = RefCell::new(Vec::new());
        let resident_image_handles = RefCell::new(Vec::new());

        let (debug_callback, synchronous) = match callback_behavior {
            DebugCallbackBehavior::Ignore => (None, false),
            DebugCallbackBehavior::DebugMessageOnError => {
                (Some(Box::new(default_debug_callback) as debug::DebugCallback), true)
            },
            DebugCallbackBehavior::PrintAll => {
                (Some(Box::new(printall_debug_callback) as debug::DebugCallback), false)
            },
            DebugCallbackBehavior::Custom { callback, synchronous } => {
                (Some(callback), synchronous)
            },
        };

        let context = Rc::new(Context {
            gl: gl,
            state: gl_state,
            version: version,
            extensions: extensions,
            capabilities: capabilities,
            debug_callback: debug_callback,
            report_debug_output_errors: report_debug_output_errors,
            backend: RefCell::new(Box::new(backend)),
            check_current_context: check_current_context,
            framebuffer_objects: Some(framebuffer_objects),
            vertex_array_objects: vertex_array_objects,
            samplers: samplers,
            resident_texture_handles: resident_texture_handles,
            resident_image_handles: resident_image_handles,
        });

        if context.debug_callback.is_some() {
            init_debug_callback(&context, synchronous);
        }

        // making sure that an error wasn't triggered during initialization
        {
            let mut ctxt = context.make_current();
            if ::get_gl_error(&mut ctxt).is_some() {
                println!("glium has triggered an OpenGL error during initialization. Please report \
                          this error: https://github.com/tomaka/glium/issues");
            }
            /*assert!(::get_gl_error(&mut ctxt).is_none(),
                    "glium has triggered an OpenGL error during initialization. Please report \
                     this error: https://github.com/tomaka/glium/issues");*/
        }

        Ok(context)
    }

    /// Calls `get_framebuffer_dimensions` on the backend object stored by this context.
    #[inline]
    pub fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        self.backend.borrow().get_framebuffer_dimensions()
    }

    /// Changes the OpenGL context associated with this context.
    ///
    /// The new context **must** have lists shared with the old one.
    pub unsafe fn rebuild<B>(&self, new_backend: B) -> Result<(), IncompatibleOpenGl>
        where B: Backend + 'static
    {
        // framebuffer objects and vertex array objects aren't shared,
        // so we have to destroy them
        {
            let mut ctxt = self.make_current();
            fbo::FramebuffersContainer::purge_all(&mut ctxt);
            vertex_array_object::VertexAttributesSystem::purge_all(&mut ctxt);
        }

        new_backend.make_current();

        *self.state.borrow_mut() = Default::default();
        // FIXME: verify version, capabilities and extensions
        *self.backend.borrow_mut() = Box::new(new_backend);

        // making textures resident
        let textures = self.resident_texture_handles.borrow();
        for &texture in textures.iter() {
            self.gl.MakeTextureHandleResidentARB(texture);
        }

        // making images resident
        let images = self.resident_image_handles.borrow();
        for &(image, access) in images.iter() {
            self.gl.MakeImageHandleResidentARB(image, access);
        }

        Ok(())
    }

    /// Swaps the buffers in the backend.
    pub fn swap_buffers(&self) -> Result<(), SwapBuffersError> {
        if self.state.borrow().lost_context {
            return Err(SwapBuffersError::ContextLost);
        }

        // Note: This is a work-around for the FRAPS software.
        //       The Fraps software calls `glClear` with scissoring and reads the image of the
        //       current framebuffer.
        //       Therefore we need to bind the default framebuffer before swapping.
        if self.state.borrow().draw_framebuffer != 0 || self.state.borrow().read_framebuffer != 0 {
            let mut ctxt = self.make_current();

            if ctxt.version >= &Version(Api::Gl, 3, 0) ||
               ctxt.extensions.gl_arb_framebuffer_object
            {
                unsafe { ctxt.gl.BindFramebuffer(gl::FRAMEBUFFER, 0); }
                ctxt.state.draw_framebuffer = 0;
                ctxt.state.read_framebuffer = 0;
            } else if ctxt.version >= &Version(Api::GlEs, 2, 0) {
                unsafe { ctxt.gl.BindFramebuffer(gl::FRAMEBUFFER, 0); }
                ctxt.state.draw_framebuffer = 0;
                ctxt.state.read_framebuffer = 0;
            } else if ctxt.extensions.gl_ext_framebuffer_object {
                unsafe { ctxt.gl.BindFramebufferEXT(gl::FRAMEBUFFER_EXT, 0); }
                ctxt.state.draw_framebuffer = 0;
                ctxt.state.read_framebuffer = 0;
            } else {
                unreachable!();
            }
        }

        let backend = self.backend.borrow();
        if self.check_current_context {
            if !backend.is_current() {
                unsafe { backend.make_current() };
            }
        }

        // swapping
        let err = backend.swap_buffers();
        if let Err(SwapBuffersError::ContextLost) = err {
            self.state.borrow_mut().lost_context = true;
        }
        err
    }

    /// DEPRECATED. Use `get_opengl_version` instead.
    #[inline]
    pub fn get_version(&self) -> &Version {
        &self.version
    }

    /// Returns the OpenGL version detected by this context.
    #[inline]
    pub fn get_opengl_version(&self) -> &Version {
        &self.version
    }

    /// Returns the GLSL version guaranteed to be supported.
    #[inline]
    pub fn get_supported_glsl_version(&self) -> Version {
        version::get_supported_glsl_version(self.get_version())
    }

    /// Returns true if the given GLSL version is supported.
    #[inline]
    pub fn is_glsl_version_supported(&self, version: &Version) -> bool {
        self.capabilities().supported_glsl_versions.iter().find(|&v| v == version).is_some()
    }

    /// Returns a string containing this GL version or release number used by this context.
    ///
    /// Vendor-specific information may follow the version number.
    #[inline]
    pub fn get_opengl_version_string(&self) -> &str {
        &self.capabilities().version
    }

    /// Returns a string containing the company responsible for this GL implementation.
    #[inline]
    pub fn get_opengl_vendor_string(&self) -> &str {
        &self.capabilities().vendor
    }

    /// Returns a string containing the name of the GL renderer used by this context.
    ///
    /// This name is typically specific to a particular configuration of a hardware platform.
    #[inline]
    pub fn get_opengl_renderer_string(&self) -> &str {
        &self.capabilities().renderer
    }

    /// Returns true if the context is in debug mode.
    ///
    /// Debug mode may provide additional error and performance issue reporting functionality.
    #[inline]
    pub fn is_debug(&self) -> bool {
        self.capabilities().debug
    }

    /// Returns true if the context is in "forward-compatible" mode.
    ///
    /// Forward-compatible mode means that no deprecated functionality will be supported.
    #[inline]
    pub fn is_forward_compatible(&self) -> bool {
        self.capabilities().forward_compatible
    }

    /// Returns this context's OpenGL profile if available.
    ///
    /// The context profile is available from OpenGL 3.2 onwards. Returns `None` if not supported.
    pub fn get_opengl_profile(&self) -> Option<Profile> {
        self.capabilities().profile
    }

    /// Returns true if out-of-bound buffer access from the GPU side (inside a program) cannot
    /// result in a crash.
    ///
    /// You should take extra care if `is_robust` returns false.
    #[inline]
    pub fn is_robust(&self) -> bool {
        self.capabilities().robustness
    }

    /// Returns true if a context loss is possible.
    #[inline]
    pub fn is_context_loss_possible(&self) -> bool {
        self.capabilities().can_lose_context
    }

    /// Returns true if the context has been lost and needs to be recreated.
    ///
    /// # Implementation
    ///
    /// If it has been determined that the context has been lost before, then the function
    /// immediately returns true. Otherwise, calls `glGetGraphicsResetStatus`. If this function
    /// is not available, returns false.
    pub fn is_context_lost(&self) -> bool {
        if self.state.borrow().lost_context {
            return true;
        }

        let mut ctxt = self.make_current();

        let lost = if ctxt.version >= &Version(Api::Gl, 4, 5) ||
                      ctxt.version >= &Version(Api::GlEs, 3, 2) ||
                      ctxt.extensions.gl_khr_robustness
        {
            unsafe { ctxt.gl.GetGraphicsResetStatus() != gl::NO_ERROR }
        } else if ctxt.extensions.gl_ext_robustness {
            unsafe { ctxt.gl.GetGraphicsResetStatusEXT() != gl::NO_ERROR }
        } else if ctxt.extensions.gl_arb_robustness {
            unsafe { ctxt.gl.GetGraphicsResetStatusARB() != gl::NO_ERROR }
        } else {
            false
        };

        if lost { ctxt.state.lost_context = true; }
        lost
    }

    /// Returns the behavior when the current OpenGL context is changed.
    ///
    /// The most common value is `Flush`. In order to get `None` you must explicitly request it
    /// during creation.
    #[inline]
    pub fn get_release_behavior(&self) -> ReleaseBehavior {
        self.capabilities().release_behavior
    }

    /// Returns the maximum value that can be used for anisotropic filtering, or `None`
    /// if the hardware doesn't support it.
    #[inline]
    pub fn get_max_anisotropy_support(&self) -> Option<u16> {
        self.capabilities().max_texture_max_anisotropy.map(|v| v as u16)
    }

    /// Returns the maximum dimensions of the viewport.
    ///
    /// Glium will panic if you request a larger viewport than this when drawing.
    #[inline]
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

            let mut value: [gl::types::GLint; 4] = [0; 4];

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
    /// This function can return any type that implements `Texture2dDataSink<(u8, u8, u8, u8)>`.
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
    pub fn read_front_buffer<T>(&self) -> Result<T, ops::ReadError>
                                where T: texture::Texture2dDataSink<(u8, u8, u8, u8)>
    {
        let mut ctxt = self.make_current();
        let dimensions = self.get_framebuffer_dimensions();
        let rect = ::Rect { left: 0, bottom: 0, width: dimensions.0, height: dimensions.1 };

        let mut data = Vec::with_capacity(0);
        ops::read(&mut ctxt, ops::Source::DefaultFramebuffer(gl::FRONT_LEFT), &rect,
                          &mut data, false)?;
        Ok(T::from_raw(Cow::Owned(data), dimensions.0, dimensions.1))
    }

    /// Execute an arbitrary closure with the OpenGL context active. Useful if another
    /// component needs to directly manipulate OpenGL state.
    ///
    /// **If `action` manipulates any OpenGL state, it must be restored before `action`
    /// completes.**
    #[inline]
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

    /// DEPRECATED. Renamed `finish`.
    #[inline]
    pub fn synchronize(&self) {
        self.finish();
    }

    /// Calls `glFinish()`. This waits until all the previously issued commands have finished
    /// being executed.
    ///
    /// When you execute OpenGL functions, they are not executed immediately. Instead they are
    /// put in a queue. This function flushes this queue, then waits until all commands
    /// have finished being executed.
    ///
    /// You normally don't need to call this function manually, except for debugging purposes.
    #[inline]
    pub fn finish(&self) {
        let ctxt = self.make_current();
        unsafe { ctxt.gl.Finish(); }
    }

    /// Calls `glFlush()`. This starts executing the commands that you have issued if it is not
    /// yet the case.
    ///
    /// When you execute OpenGL functions, they are not executed immediately. Instead they are
    /// put in a queue. This function flushes this queue so that commands start being executed.
    ///
    /// You normally don't need to call this function manually. Swapping buffers automatically
    /// flushes the queue. This function can be useful if you want to benchmark the time it
    /// takes from your OpenGL driver to process commands.
    #[inline]
    pub fn flush(&self) {
        let ctxt = self.make_current();
        unsafe { ctxt.gl.Flush(); }
    }

    /// Inserts a debugging string in the commands queue. If you use an OpenGL debugger, you will
    /// be able to see that string.
    ///
    /// This is helpful to understand where you are when you have big applications.
    ///
    /// Returns `Err` if the backend doesn't support this functionality. You can choose whether
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
    #[inline]
    pub fn debug_insert_debug_marker(&self, marker: &str) -> Result<(), ()> {
        if cfg!(debug_assertions) {
            self.insert_debug_marker(marker)
        } else {
            Ok(())
        }
    }
}

impl ContextExt for Context {
    #[inline]
    fn set_report_debug_output_errors(&self, value: bool) {
        self.report_debug_output_errors.set(value);
    }

    fn make_current(&self) -> CommandContext {
        if self.check_current_context {
            let backend = self.backend.borrow();
            if !backend.is_current() {
                unsafe { backend.make_current() };
                debug_assert!(backend.is_current());
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
            framebuffer_objects: self.framebuffer_objects.as_ref().unwrap(),
            samplers: self.samplers.borrow_mut(),
            resident_texture_handles: self.resident_texture_handles.borrow_mut(),
            resident_image_handles: self.resident_image_handles.borrow_mut(),
            marker: PhantomData,
        }
    }

    #[inline]
    fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }
}

impl CapabilitiesSource for Context {
    #[inline]
    fn get_version(&self) -> &Version {
        &self.version
    }

    #[inline]
    fn get_extensions(&self) -> &ExtensionsList {
        &self.extensions
    }

    #[inline]
    fn get_capabilities(&self) -> &Capabilities {
        &self.capabilities
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
                framebuffer_objects: self.framebuffer_objects.as_ref().unwrap(),
                samplers: self.samplers.borrow_mut(),
                resident_texture_handles: self.resident_texture_handles.borrow_mut(),
                resident_image_handles: self.resident_image_handles.borrow_mut(),
                marker: PhantomData,
            };

            fbo::FramebuffersContainer::cleanup(&mut ctxt);
            vertex_array_object::VertexAttributesSystem::cleanup(&mut ctxt);

            for (_, s) in mem::replace(&mut *ctxt.samplers, HashMap::with_hasher(Default::default())) {
                s.destroy(&mut ctxt);
            }

            // disabling callback
            if ctxt.state.enabled_debug_output != Some(false) {
                if ctxt.version >= &Version(Api::Gl, 4,5) || ctxt.extensions.gl_khr_debug {
                    ctxt.gl.Disable(gl::DEBUG_OUTPUT);
                } else if ctxt.extensions.gl_arb_debug_output {
                    ctxt.gl.DebugMessageCallbackARB(None,
                                                    ptr::null());
                }

                ctxt.state.enabled_debug_output = Some(false);
                ctxt.gl.Finish();
            }
        }
    }
}

impl<'a> CapabilitiesSource for CommandContext<'a> {
    #[inline]
    fn get_version(&self) -> &Version {
        self.version
    }

    #[inline]
    fn get_extensions(&self) -> &ExtensionsList {
        self.extensions
    }

    #[inline]
    fn get_capabilities(&self) -> &Capabilities {
        self.capabilities
    }
}

/// Checks whether the backend supports glium. Returns an `Err` if it doesn't.
fn check_gl_compatibility(version: &Version, extensions: &ExtensionsList)
    -> Result<(), IncompatibleOpenGl>
{
    let mut result = Vec::with_capacity(0);

    if !(version >= &Version(Api::Gl, 1, 5)) &&
        !(version >= &Version(Api::GlEs, 2, 0)) &&
        (!extensions.gl_arb_vertex_buffer_object || !extensions.gl_arb_map_buffer_range)
    {
        result.push("OpenGL implementation doesn't support buffer objects");
    }

    if !(version >= &Version(Api::Gl, 2, 0)) &&
        !(version >= &Version(Api::GlEs, 2, 0)) &&
        (!extensions.gl_arb_shader_objects ||
            !extensions.gl_arb_vertex_shader || !extensions.gl_arb_fragment_shader)
    {
        result.push("OpenGL implementation doesn't support vertex/fragment shaders");
    }

    if !extensions.gl_ext_framebuffer_object && !(version >= &Version(Api::Gl, 3, 0)) &&
        !(version >= &Version(Api::GlEs, 2, 0)) && !extensions.gl_arb_framebuffer_object
    {
        result.push("OpenGL implementation doesn't support framebuffers");
    }

    if !extensions.gl_ext_framebuffer_blit && !(version >= &Version(Api::Gl, 3, 0)) &&
        !(version >= &Version(Api::GlEs, 2, 0))
    {
        result.push("OpenGL implementation doesn't support blitting framebuffers");
    }

    if result.len() == 0 {
        Ok(())
    } else {
        Err(IncompatibleOpenGl(result.join("\n")))
    }
}

/// Describes the behavior that the debug output should have.
pub enum DebugCallbackBehavior {
    /// Don't do anything. This is the default behavior in release.
    Ignore,

    /// Print a message on stdout on error, except in some circumstances like when compiling
    /// shaders. This is the default behavior in debug mode.
    DebugMessageOnError,

    /// Print every single output received by the driver.
    PrintAll,

    /// Use a custom callback.
    Custom {
        /// The function to be called.
        callback: debug::DebugCallback,
        /// Whether or not it should be called immediately (true) or asynchronously (false).
        synchronous: bool,
    },
}

impl Default for DebugCallbackBehavior {
    #[inline]
    fn default() -> DebugCallbackBehavior {
        if cfg!(debug_assertions) {
            DebugCallbackBehavior::DebugMessageOnError
        } else {
            DebugCallbackBehavior::Ignore
        }
    }
}

/// The callback corresponding to `DebugMessageOnError`.
fn default_debug_callback(_: debug::Source, ty: debug::MessageType, severity: debug::Severity,
                          _: u32, report_debug_output_errors: bool, message: &str)
{
    match severity {
        debug::Severity::Medium => (),
        debug::Severity::High => (),
        _ => return
    };

    match ty {
        debug::MessageType::Error => (),
        debug::MessageType::DeprecatedBehavior => (),
        debug::MessageType::UndefinedBehavior => (),
        debug::MessageType::Portability => (),
        _ => return,
    };

    if report_debug_output_errors {
        print!("Debug message with high or medium severity: `{}`.\n\
                Please report this error: https://github.com/tomaka/glium/issues\n\
                Backtrace:",
                message);

        let mut frame_id = 1;
        backtrace::trace(|frame| {
            let ip = frame.ip();
            print!("\n{:>#4} - {:p}", frame_id, ip);

            backtrace::resolve(ip, |symbol| {
                let name = symbol.name()
                                 .map(|n| n.as_str().unwrap_or("<not-utf8>"))
                                 .unwrap_or("<unknown>");
                let filename = symbol.filename()
                                     .map(|p| p.to_str().unwrap_or("<not-utf8>"))
                                     .unwrap_or("<unknown>");
                let line = symbol.lineno().map(|l| l.to_string())
                                          .unwrap_or_else(|| "??".to_owned());

                print!("\n         {} at {}:{}", name, filename, line);
            });

            frame_id += 1;
            true
        });

        println!("\n");
    }
}

/// The callback corresponding to `DebugMessageOnError`.
fn printall_debug_callback(source: debug::Source, ty: debug::MessageType, severity: debug::Severity,
                           id: u32, _: bool, message: &str)
{
    println!("Source: {src:?}\t\tSeverity: {sev:?}\t\tType: {ty:?}\t\tId: {id}\n{msg}",
              src = source, sev = severity, ty = ty, id = id, msg = message);
}

/// Initializes `GL_KHR_debug`, `GL_ARB_debug`, or a similar extension so that the debug output
/// is reported.
fn init_debug_callback(context: &Rc<Context>, synchronous: bool) {
    // this is the C callback
    extern "system" fn callback_wrapper(source: gl::types::GLenum, ty: gl::types::GLenum,
                                        id: gl::types::GLuint, severity: gl::types::GLenum,
                                        _length: gl::types::GLsizei,
                                        message: *const gl::types::GLchar,
                                        user_param: *mut raw::c_void)
    {
        // note that we transmute the user param into a proper context
        // in order to enforce safety here, the context disables debug output and flushes in its
        // destructor

        let user_param = user_param as *const Context;
        let user_param: &mut Context = unsafe { mem::transmute(user_param) };

        let message = unsafe {
            String::from_utf8(CStr::from_ptr(message).to_bytes().to_vec()).unwrap()
        };

        let severity = match severity {
            gl::DEBUG_SEVERITY_NOTIFICATION => debug::Severity::Notification,
            gl::DEBUG_SEVERITY_LOW => debug::Severity::Low,
            gl::DEBUG_SEVERITY_MEDIUM => debug::Severity::Medium,
            gl::DEBUG_SEVERITY_HIGH => debug::Severity::High,
            _ => return,        // TODO: what to do in this situation?
        };

        let source = match source {
            gl::DEBUG_SOURCE_API => debug::Source::Api,
            gl::DEBUG_SOURCE_WINDOW_SYSTEM => debug::Source::WindowSystem,
            gl::DEBUG_SOURCE_SHADER_COMPILER => debug::Source::ShaderCompiler,
            gl::DEBUG_SOURCE_THIRD_PARTY => debug::Source::ThirdParty,
            gl::DEBUG_SOURCE_APPLICATION => debug::Source::Application,
            gl::DEBUG_SOURCE_OTHER => debug::Source::OtherSource,
            _ => return,        // TODO: what to do in this situation?
        };

        let ty = match ty {
            gl::DEBUG_TYPE_ERROR => debug::MessageType::Error,
            gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => debug::MessageType::DeprecatedBehavior,
            gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => debug::MessageType::UndefinedBehavior,
            gl::DEBUG_TYPE_PORTABILITY => debug::MessageType::Portability,
            gl::DEBUG_TYPE_PERFORMANCE => debug::MessageType::Performance,
            gl::DEBUG_TYPE_MARKER => debug::MessageType::Marker,
            gl::DEBUG_TYPE_PUSH_GROUP => debug::MessageType::PushGroup,
            gl::DEBUG_TYPE_POP_GROUP => debug::MessageType::PopGroup,
            gl::DEBUG_TYPE_OTHER => debug::MessageType::Other,
            _ => return,        // TODO: what to do in this situation?
        };

        if let Some(callback) = user_param.debug_callback.as_mut() {
            // FIXME: catch_panic here once it's stable
            callback(source, ty, severity, id, user_param.report_debug_output_errors.get(),
                     &message);
        }
    }

    struct ContextRawPtr(*const Context);
    unsafe impl Send for ContextRawPtr {}
    let context_raw_ptr = ContextRawPtr(&**context);

    unsafe {
        let mut ctxt = context.make_current();

        if ctxt.version >= &Version(Api::Gl, 4,5) || ctxt.version >= &Version(Api::GlEs, 3, 2) ||
           ctxt.extensions.gl_khr_debug || ctxt.extensions.gl_arb_debug_output
        {
            if synchronous {
                if ctxt.state.enabled_debug_output_synchronous != true {
                    ctxt.gl.Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
                    ctxt.state.enabled_debug_output_synchronous = true;
                }
            }

            if ctxt.version >= &Version(Api::Gl, 4, 5) ||
               ctxt.version >= &Version(Api::GlEs, 3, 2) ||
               (ctxt.version >= &Version(Api::Gl, 1, 0) && ctxt.extensions.gl_khr_debug)
            {
                ctxt.gl.DebugMessageCallback(Some(callback_wrapper), context_raw_ptr.0
                                                                     as *const _);
                ctxt.gl.DebugMessageControl(gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE, 0,
                                            ptr::null(), gl::TRUE);

                if ctxt.state.enabled_debug_output != Some(true) {
                    ctxt.gl.Enable(gl::DEBUG_OUTPUT);
                    ctxt.state.enabled_debug_output = Some(true);
                }

            } else if ctxt.version >= &Version(Api::GlEs, 2, 0) &&
                      ctxt.extensions.gl_khr_debug
            {
                ctxt.gl.DebugMessageCallbackKHR(Some(callback_wrapper), context_raw_ptr.0
                                                                        as *const _);
                ctxt.gl.DebugMessageControlKHR(gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE, 0,
                                               ptr::null(), gl::TRUE);

                if ctxt.state.enabled_debug_output != Some(true) {
                    ctxt.gl.Enable(gl::DEBUG_OUTPUT);
                    ctxt.state.enabled_debug_output = Some(true);
                }

            } else {
                ctxt.gl.DebugMessageCallbackARB(Some(callback_wrapper), context_raw_ptr.0
                                                                        as *const _);
                ctxt.gl.DebugMessageControlARB(gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE,
                                               0, ptr::null(), gl::TRUE);

                ctxt.state.enabled_debug_output = Some(true);
            }
        }
    }
}
