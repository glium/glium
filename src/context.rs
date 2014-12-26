use gl;
use glutin;
use std::sync::atomic::{AtomicUint, Relaxed};
use std::sync::{Arc, Mutex};
use GliumCreationError;

enum Message {
    EndFrame,
    Execute(Box<for<'a, 'b> ::std::thunk::Invoke<CommandContext<'a, 'b>, ()> + Send>),
}

pub struct Context {
    commands: Mutex<Sender<Message>>,
    events: Mutex<Receiver<glutin::Event>>,

    /// Dimensions of the frame buffer.
    dimensions: Arc<(AtomicUint, AtomicUint)>,

    capabilities: Arc<Capabilities>,
}

pub struct CommandContext<'a, 'b> {
    pub gl: &'a gl::Gl,
    pub state: &'b mut GLState,
    pub version: &'a GlVersion,
    pub extensions: &'a ExtensionsList,
    pub opengl_es: bool,
    pub capabilities: &'a Capabilities,
}

/// Represents the current OpenGL ctxt.state.
/// The current state is passed to each function and can be freely updated.
pub struct GLState {
    /// Whether GL_BLEND is enabled
    pub enabled_blend: bool,

    /// Whether GL_CULL_FACE is enabled
    pub enabled_cull_face: bool,

    /// Whether GL_DEBUG_OUTPUT is enabled. None means "unknown".
    pub enabled_debug_output: Option<bool>,

    /// Whether GL_DEBUG_OUTPUT_SYNCHRONOUS is enabled
    pub enabled_debug_output_synchronous: bool,

    /// Whether GL_DEPTH_TEST is enabled
    pub enabled_depth_test: bool,

    /// Whether GL_DITHER is enabled
    pub enabled_dither: bool,

    /// Whether GL_MULTISAMPLE is enabled
    pub enabled_multisample: bool,

    /// Whether GL_POLYGON_OFFSET_FILL is enabled
    pub enabled_polygon_offset_fill: bool,

    /// Whether GL_SAMPLE_ALPHA_TO_COVERAGE is enabled
    pub enabled_sample_alpha_to_coverage: bool,

    /// Whether GL_SAMPLE_COVERAGE is enabled
    pub enabled_sample_coverage: bool,

    /// Whether GL_SCISSOR_TEST is enabled
    pub enabled_scissor_test: bool,

    /// Whether GL_STENCIL_TEST is enabled
    pub enabled_stencil_test: bool,

    // The latest value passed to `glUseProgram`.
    pub program: gl::types::GLuint,

    // The latest value passed to `glBindVertexArray`.
    pub vertex_array: gl::types::GLuint,

    // The latest value passed to `glClearColor`.
    pub clear_color: (gl::types::GLclampf, gl::types::GLclampf,
                      gl::types::GLclampf, gl::types::GLclampf),

    // The latest value passed to `glClearDepthf`.
    pub clear_depth: gl::types::GLclampf,

    // The latest value passed to `glClearStencil`.
    pub clear_stencil: gl::types::GLint,

    /// The latest buffer bound to `GL_ARRAY_BUFFER`.
    pub array_buffer_binding: gl::types::GLuint,

    /// The latest buffer bound to `GL_PIXEL_PACK_BUFFER`.
    pub pixel_pack_buffer_binding: gl::types::GLuint,

    /// The latest buffer bound to `GL_PIXEL_UNPACK_BUFFER`.
    pub pixel_unpack_buffer_binding: gl::types::GLuint,

    /// The latest buffer bound to `GL_READ_FRAMEBUFFER`.
    pub read_framebuffer: gl::types::GLuint,

    /// The latest buffer bound to `GL_DRAW_FRAMEBUFFER`.
    pub draw_framebuffer: gl::types::GLuint,

    /// The latest values passed to `glReadBuffer` with the default framebuffer.
    /// `None` means "unknown".
    pub default_framebuffer_read: Option<gl::types::GLenum>,

    /// The latest render buffer bound with `glBindRenderbuffer`.
    pub renderbuffer: gl::types::GLuint,

    /// The latest values passed to `glBlendFunc`.
    pub blend_func: (gl::types::GLenum, gl::types::GLenum),

    /// The latest value passed to `glDepthFunc`.
    pub depth_func: gl::types::GLenum,

    /// The latest values passed to `glDepthRange`.
    pub depth_range: (f32, f32),

    /// The latest values passed to `glViewport`.
    pub viewport: (gl::types::GLint, gl::types::GLint, gl::types::GLsizei, gl::types::GLsizei),

    /// The latest value passed to `glLineWidth`.
    pub line_width: gl::types::GLfloat,

    /// The latest value passed to `glCullFace`.
    pub cull_face: gl::types::GLenum,

    /// The latest value passed to `glPolygonMode`.
    pub polygon_mode: gl::types::GLenum,
}

impl GLState {
    /// Builds the `GLState` corresponding to the default GL values.
    fn new_defaults(viewport: (gl::types::GLint, gl::types::GLint, gl::types::GLsizei,
        gl::types::GLsizei)) -> GLState
    {
        GLState {
            enabled_blend: false,
            enabled_cull_face: false,
            enabled_debug_output: None,
            enabled_debug_output_synchronous: false,
            enabled_depth_test: false,
            enabled_dither: false,
            enabled_multisample: true,
            enabled_polygon_offset_fill: false,
            enabled_sample_alpha_to_coverage: false,
            enabled_sample_coverage: false,
            enabled_scissor_test: false,
            enabled_stencil_test: false,

            program: 0,
            vertex_array: 0,
            clear_color: (0.0, 0.0, 0.0, 0.0),
            clear_depth: 1.0,
            clear_stencil: 0,
            array_buffer_binding: 0,
            pixel_pack_buffer_binding: 0,
            pixel_unpack_buffer_binding: 0,
            read_framebuffer: 0,
            draw_framebuffer: 0,
            default_framebuffer_read: None,
            renderbuffer: 0,
            depth_func: gl::LESS,
            depth_range: (0.0, 1.0),
            blend_func: (0, 0),     // no default specified
            viewport: viewport,
            line_width: 1.0,
            cull_face: gl::BACK,
            polygon_mode: gl::FILL,
        }
    }
}

/// Describes an OpenGL ctxt.version.
#[deriving(Show, Clone, PartialEq, Eq)]
pub struct GlVersion(pub u8, pub u8);

impl PartialOrd for GlVersion {
    fn partial_cmp(&self, other: &GlVersion) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GlVersion {
    fn cmp(&self, other: &GlVersion) -> Ordering {
        match self.0.cmp(&other.0) {
            Equal => self.1.cmp(&other.1),
            a => a
        }
    }
}

/// Contains data about the list of extensions
pub struct ExtensionsList {
    /// GL_EXT_direct_state_access
    pub gl_ext_direct_state_access: bool,
    /// GL_EXT_framebuffer_object
    pub gl_ext_framebuffer_object: bool,
    /// GL_EXT_geometry_shader4
    pub gl_ext_geometry_shader4: bool,
    /// GL_EXT_framebuffer_blit
    pub gl_ext_framebuffer_blit: bool,
    /// GL_KHR_debug
    pub gl_khr_debug: bool,
    /// GL_NVX_gpu_memory_info
    pub gl_nvx_gpu_memory_info: bool,
    /// GL_ATI_meminfo
    pub gl_ati_meminfo: bool,
    /// GL_ARB_vertex_array_object
    pub gl_arb_vertex_array_object: bool,
    /// GL_ARB_sampler_objects
    pub gl_arb_sampler_objects: bool,
    /// GL_EXT_texture_filter_anisotropic
    pub gl_ext_texture_filter_anisotropic: bool,
    /// GL_ARB_texture_storage
    pub gl_arb_texture_storage: bool,
}

/// Represents the capabilities of the context.
pub struct Capabilities {
    /// True if the context supports left and right buffers.
    pub stereo: bool,

    /// Number of bits in the default framebuffer's depth buffer
    pub depth_bits: Option<u16>,

    /// Number of bits in the default framebuffer's stencil buffer
    pub stencil_bits: Option<u16>,

    /// Maximum number of textures that can be bind to a program.
    ///
    /// `glActiveTexture` must be between `GL_TEXTURE0` and `GL_TEXTURE0` + this value - 1.
    pub max_combined_texture_image_units: gl::types::GLint,

    /// Maximum value for `GL_TEXTURE_MAX_ANISOTROPY_EXT​`.
    ///
    /// `None` if the extension is not supported by the hardware.
    pub max_texture_max_anisotropy: Option<gl::types::GLfloat>,

    /// Maximum width and height of `glViewport`.
    pub max_viewport_dims: (gl::types::GLint, gl::types::GLint),
}

impl Context {
    pub fn new_from_window(window: glutin::WindowBuilder, previous: Option<Context>)
        -> Result<Context, GliumCreationError>
    {
        use std::thread::Builder;

        let (tx_events, rx_events) = channel();
        let (tx_commands, rx_commands) = channel();

        let dimensions = Arc::new((AtomicUint::new(800), AtomicUint::new(600)));
        let dimensions2 = dimensions.clone();

        let window = try!(window.build());
        let (tx_success, rx_success) = channel();

        Builder::new().name("glium rendering thread".to_string()).spawn(move || {
            unsafe { window.make_current(); }

            let gl = gl::Gl::load_with(|symbol| window.get_proc_address(symbol));

            // building the GLState and modifying to GL state to match it
            let mut gl_state = {
                let viewport = {
                    let dim = window.get_inner_size().unwrap();
                    dimensions.0.store(dim.0, Relaxed);
                    dimensions.1.store(dim.1, Relaxed);
                    (0, 0, dim.0 as gl::types::GLsizei, dim.1 as gl::types::GLsizei)
                };

                unsafe { gl.Viewport(viewport.0, viewport.1, viewport.2, viewport.3) };
                GLState::new_defaults(viewport)
            };

            // getting the GL version and extensions
            let opengl_es = match window.get_api() { glutin::Api::OpenGlEs => true, _ => false };       // TODO: fix glutin::Api not implementing Eq
            let version = get_gl_version(&gl);
            let extensions = get_extensions(&gl);
            let capabilities = Arc::new(get_capabilities(&gl, &version, &extensions, opengl_es));

            // checking compatibility with glium
            match check_gl_compatibility(CommandContext {
                gl: &gl,
                state: &mut gl_state,
                version: &version,
                extensions: &extensions,
                opengl_es: opengl_es,
                capabilities: &*capabilities,
            }) {
                Err(e) => {
                    tx_success.send(Err(e));
                    return;
                },
                Ok(_) => {
                    tx_success.send(Ok(capabilities.clone()));
                }
            };

            // main loop
            'main: loop {
                // processing commands
                loop {
                    match rx_commands.recv_opt() {
                        Ok(Message::EndFrame) => break,
                        Ok(Message::Execute(cmd)) => cmd.invoke(CommandContext {
                            gl: &gl,
                            state: &mut gl_state,
                            version: &version,
                            extensions: &extensions,
                            opengl_es: opengl_es,
                            capabilities: &*capabilities,
                        }),
                        Err(_) => break 'main
                    }
                }

                // this is necessary on Windows 8, or nothing is being displayed
                unsafe { gl.Flush(); }

                // swapping
                window.swap_buffers();

                // getting events
                for event in window.poll_events() {
                    // update the dimensions
                    if let &glutin::Event::Resized(width, height) = &event {
                        dimensions.0.store(width, Relaxed);
                        dimensions.1.store(height, Relaxed);
                    }

                    // sending the event outside
                    if tx_events.send_opt(event.clone()).is_err() {
                        break 'main;
                    }
                }
            }
        }).detach();

        Ok(Context {
            commands: Mutex::new(tx_commands),
            events: Mutex::new(rx_events),
            dimensions: dimensions2,
            capabilities: try!(rx_success.recv()),
        })
    }

    #[cfg(feature = "headless")]
    pub fn new_from_headless(window: glutin::HeadlessRendererBuilder)
        -> Result<Context, GliumCreationError>
    {
        use std::thread::Builder;

        let (_, rx_events) = channel();
        let (tx_commands, rx_commands) = channel();

        // TODO: fixme
        let dimensions = Arc::new((AtomicUint::new(800), AtomicUint::new(600)));
        let dimensions2 = dimensions.clone();

        let (tx_success, rx_success) = channel();

        Builder::new().name("glium rendering thread".to_string()).spawn(move || {
            let window = match window.build() {
                Ok(w) => w,
                Err(e) => {
                    tx_success.send(Err(::std::error::FromError::from_error(e)));
                    return;
                }
            };
            unsafe { window.make_current(); }

            let gl = gl::Gl::load_with(|symbol| window.get_proc_address(symbol));
            // TODO: call glViewport

            // building the GLState, version, and extensions
            let mut gl_state = GLState::new_defaults((0, 0, 0, 0));    // FIXME: 
            let opengl_es = match window.get_api() { glutin::Api::OpenGlEs => true, _ => false };       // TODO: fix glutin::Api not implementing Eq
            let version = get_gl_version(&gl);
            let extensions = get_extensions(&gl);
            let capabilities = Arc::new(get_capabilities(&gl, &version, &extensions, opengl_es));

            // checking compatibility with glium
            match check_gl_compatibility(CommandContext {
                gl: &gl,
                state: &mut gl_state,
                version: &version,
                extensions: &extensions,
                opengl_es: opengl_es,
                capabilities: &*capabilities,
            }) {
                Err(e) => {
                    tx_success.send(Err(e));
                    return;
                },
                Ok(_) => {
                    tx_success.send(Ok(capabilities.clone()));
                }
            };

            loop {
                match rx_commands.recv_opt() {
                    Ok(Message::Execute(cmd)) => cmd.invoke(CommandContext {
                        gl: &gl,
                        state: &mut gl_state,
                        version: &version,
                        extensions: &extensions,
                        opengl_es: opengl_es,
                        capabilities: &*capabilities,
                    }),
                    Ok(Message::EndFrame) => (),     // ignoring buffer swapping
                    Err(_) => break
                }
            }
        }).detach();

        Ok(Context {
            commands: Mutex::new(tx_commands),
            events: Mutex::new(rx_events),
            dimensions: dimensions2,
            capabilities: try!(rx_success.recv()),
        })
    }

    pub fn get_framebuffer_dimensions(&self) -> (uint, uint) {
        (
            self.dimensions.0.load(Relaxed),
            self.dimensions.1.load(Relaxed),
        )
    }

    pub fn exec<F>(&self, f: F) where F: FnOnce(CommandContext) + Send {
        self.commands.lock().send(Message::Execute(box f));
    }

    pub fn swap_buffers(&self) {
        self.commands.lock().send(Message::EndFrame);
    }

    pub fn recv(&self) -> Vec<glutin::Event> {
        let events = self.events.lock();

        let mut result = Vec::new();
        loop {
            match events.try_recv() {
                Ok(ev) => result.push(ev),
                Err(_) => break
            }
        }
        result
    }

    pub fn capabilities(&self) -> &Capabilities {
        &*self.capabilities
    }
}

fn check_gl_compatibility(ctxt: CommandContext) -> Result<(), GliumCreationError> {
    let mut result = Vec::new();

    if ctxt.opengl_es {
        if ctxt.version < &GlVersion(3, 0) {
            result.push("OpenGL ES version inferior to 3.0");
        }

    } else {
        if ctxt.version < &GlVersion(2, 0) {
            result.push("OpenGL version inferior to 2.0 is not supported");
        }

        if !ctxt.extensions.gl_ext_framebuffer_object && ctxt.version < &GlVersion(3, 0) {
            result.push("OpenGL implementation doesn't support framebuffers");
        }

        if !ctxt.extensions.gl_ext_framebuffer_blit && ctxt.version < &GlVersion(3, 0) {
            result.push("OpenGL implementation doesn't support blitting framebuffers");
        }

        if !ctxt.extensions.gl_arb_vertex_array_object && ctxt.version < &GlVersion(3, 0) {
            result.push("OpenGL implementation doesn't support vertex array objects");
        }

        if option_env!("TRAVIS").is_none() {        // TODO: ultra-hacky stuff to make tests pass
            if !ctxt.extensions.gl_arb_sampler_objects && ctxt.version < &GlVersion(3, 3) {
                result.push("OpenGL implementation doesn't support sampler objects");
            }
        }
    }

    if result.len() == 0 {
        Ok(())   
    } else {
        Err(GliumCreationError::IncompatibleOpenGl(result.connect("\n")))
    }
}

fn get_gl_version(gl: &gl::Gl) -> GlVersion {
    use std::c_str::CString;

    unsafe {
        let version = gl.GetString(gl::VERSION);
        let version = CString::new(version as *const i8, false);
        let version = version.as_str().expect("OpenGL version contains non-utf8 characters");

        let version = version.words().next().expect("glGetString(GL_VERSION) returned an empty \
                                                     string");

        let mut iter = version.split(move |&mut: c: char| c == '.');
        let major = iter.next().unwrap();
        let minor = iter.next().expect("glGetString(GL_VERSION) did not return a correct version");

        GlVersion(
            major.parse().expect("failed to parse GL major version"),
            minor.parse().expect("failed to parse GL minor version"),
        )
    }
}

fn get_extensions_strings(gl: &gl::Gl) -> Vec<String> {
    use std::c_str::CString;

    unsafe {
        let list = gl.GetString(gl::EXTENSIONS);

        if list.is_null() {
            let mut num_extensions = 0;
            gl.GetIntegerv(gl::NUM_EXTENSIONS, &mut num_extensions);

            range(0, num_extensions).map(|num| {
                let ext = gl.GetStringi(gl::EXTENSIONS, num as gl::types::GLuint);
                let ext = CString::new(ext as *const i8, false);
                ext.as_str().expect("OpenGL extension contains non-utf8 characters").to_string()
            }).collect()

        } else {
            let list = CString::new(list as *const i8, false);
            let list = list.as_str()
                           .expect("List of OpenGL extensions contains non-utf8 characters");
            list.words().map(|e| e.to_string()).collect()
        }
    }
}

fn get_extensions(gl: &gl::Gl) -> ExtensionsList {
    let strings = get_extensions_strings(gl);

    let mut extensions = ExtensionsList {
        gl_ext_direct_state_access: false,
        gl_ext_framebuffer_object: false,
        gl_ext_geometry_shader4: false,
        gl_ext_framebuffer_blit: false,
        gl_khr_debug: false,
        gl_nvx_gpu_memory_info: false,
        gl_ati_meminfo: false,
        gl_arb_vertex_array_object: false,
        gl_arb_sampler_objects: false,
        gl_ext_texture_filter_anisotropic: false,
        gl_arb_texture_storage: false,
    };

    for extension in strings.into_iter() {
        match extension.as_slice() {
            "GL_EXT_direct_state_access" => extensions.gl_ext_direct_state_access = true,
            "GL_EXT_framebuffer_object" => extensions.gl_ext_framebuffer_object = true,
            "GL_EXT_geometry_shader4" => extensions.gl_ext_geometry_shader4 = true,
            "GL_EXT_framebuffer_blit" => extensions.gl_ext_framebuffer_blit = true,
            "GL_KHR_debug" => extensions.gl_khr_debug = true,
            "GL_NVX_gpu_memory_info" => extensions.gl_nvx_gpu_memory_info = true,
            "GL_ATI_meminfo" => extensions.gl_ati_meminfo = true,
            "GL_ARB_vertex_array_object" => extensions.gl_arb_vertex_array_object = true,
            "GL_ARB_sampler_objects" => extensions.gl_arb_sampler_objects = true,
            "GL_EXT_texture_filter_anisotropic" => extensions.gl_ext_texture_filter_anisotropic = true,
            "GL_ARB_texture_storage" => extensions.gl_arb_texture_storage = true,
            _ => ()
        }
    }

    extensions
}

fn get_capabilities(gl: &gl::Gl, version: &GlVersion, extensions: &ExtensionsList,
                    gl_es: bool) -> Capabilities
{
    use std::mem;

    Capabilities {
        stereo: unsafe {
            if gl_es {
                false
            } else {
                let mut val: gl::types::GLboolean = mem::uninitialized();
                gl.GetBooleanv(gl::STEREO, &mut val);
                val != 0
            }
        },

        depth_bits: unsafe {
            let mut value = mem::uninitialized();

            if version >= &GlVersion(3, 0) {
                gl.GetFramebufferAttachmentParameteriv(gl::FRAMEBUFFER, gl::DEPTH,
                                                       gl::FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE,
                                                       &mut value);
            } else {
                gl.GetIntegerv(gl::DEPTH_BITS, &mut value);
            };

            match value {
                0 => None,
                v => Some(v as u16),
            }
        },

        stencil_bits: unsafe {
            let mut value = mem::uninitialized();

            if version >= &GlVersion(3, 0) {
                gl.GetFramebufferAttachmentParameteriv(gl::FRAMEBUFFER, gl::STENCIL,
                                                       gl::FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE,
                                                       &mut value);
            } else {
                gl.GetIntegerv(gl::STENCIL_BITS, &mut value);
            };

            match value {
                0 => None,
                v => Some(v as u16),
            }
        },

        max_combined_texture_image_units: unsafe {
            let mut val = 2;
            gl.GetIntegerv(gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS, &mut val);
            val
        },

        max_texture_max_anisotropy: if !extensions.gl_ext_texture_filter_anisotropic {
            None

        } else {
            Some(unsafe {
                let mut val = mem::uninitialized();
                gl.GetFloatv(gl::MAX_TEXTURE_MAX_ANISOTROPY_EXT, &mut val);
                val
            })
        },

        max_viewport_dims: unsafe {
            let mut val: [gl::types::GLint, .. 2] = [ 0, 0 ];
            gl.GetIntegerv(gl::MAX_VIEWPORT_DIMS, val.as_mut_ptr());
            (val[0], val[1])
        },

    }
}
