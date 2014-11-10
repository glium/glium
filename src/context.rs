use gl;
use glutin;
use native::NativeTaskBuilder;
use std::sync::atomic::{AtomicUint, Relaxed};
use std::sync::{Arc, Mutex};
use std::task::TaskBuilder;
use GliumCreationError;

enum Message {
    EndFrame,
    Execute(proc(&gl::Gl, &mut GLState, &GlVersion, &ExtensionsList):Send),
}

pub struct Context {
    commands: Mutex<Sender<Message>>,
    events: Mutex<Receiver<glutin::Event>>,

    /// Dimensions of the frame buffer.
    dimensions: Arc<(AtomicUint, AtomicUint)>,
}

/// Represents the current OpenGL state.
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
    pub array_buffer_binding: Option<gl::types::GLuint>,

    /// The latest buffer bound to `GL_ELEMENT_ARRAY_BUFFER`.
    pub element_array_buffer_binding: Option<gl::types::GLuint>,

    /// The latest buffer bound to `GL_PIXEL_PACK_BUFFER`.
    pub pixel_pack_buffer_binding: Option<gl::types::GLuint>,

    /// The latest buffer bound to `GL_PIXEL_UNPACK_BUFFER`.
    pub pixel_unpack_buffer_binding: Option<gl::types::GLuint>,

    /// The latest buffer bound to `GL_READ_FRAMEBUFFER`.
    pub read_framebuffer: Option<gl::types::GLuint>,

    /// The latest buffer bound to `GL_DRAW_FRAMEBUFFER`.
    pub draw_framebuffer: Option<gl::types::GLuint>,

    /// The latest render buffer bound with `glBindRenderbuffer`.
    pub renderbuffer: Option<gl::types::GLuint>,

    /// The latest values passed to `glBlendFunc`.
    pub blend_func: (gl::types::GLenum, gl::types::GLenum),

    /// The latest value passed to `glDepthFunc`.
    pub depth_func: gl::types::GLenum,

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
            array_buffer_binding: None,
            element_array_buffer_binding: None,
            pixel_pack_buffer_binding: None,
            pixel_unpack_buffer_binding: None,
            read_framebuffer: None,
            draw_framebuffer: None,
            renderbuffer: None,
            depth_func: gl::LESS,
            blend_func: (0, 0),     // no default specified
            viewport: viewport,
            line_width: 1.0,
            cull_face: gl::BACK,
            polygon_mode: gl::FILL,
        }
    }
}

/// Describes an OpenGL version.
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
}

impl Context {
    pub fn new_from_window(window: glutin::WindowBuilder) -> Result<Context, GliumCreationError> {
        let (tx_events, rx_events) = channel();
        let (tx_commands, rx_commands) = channel();

        let dimensions = Arc::new((AtomicUint::new(800), AtomicUint::new(600)));

        let context = Context {
            commands: Mutex::new(tx_commands),
            events: Mutex::new(rx_events),
            dimensions: dimensions.clone(),
        };

        let (tx_success, rx_success) = channel();

        TaskBuilder::new().native().spawn(proc() {
            let window = match window.build() {
                Ok(w) => w,
                Err(e) => {
                    tx_success.send(Err(e));
                    return;
                }
            };

            unsafe { window.make_current(); }
            tx_success.send(Ok(()));

            let gl = gl::Gl::load_with(|symbol| window.get_proc_address(symbol));

            // building the GLState and modifying to GL state to match it
            let mut gl_state = {
                let viewport = {
                    let dim = window.get_inner_size().unwrap();
                    dimensions.0.store(dim.0, Relaxed);
                    dimensions.1.store(dim.1, Relaxed);
                    (0, 0, dim.0 as gl::types::GLsizei, dim.1 as gl::types::GLsizei)
                };

                GLState::new_defaults(viewport)
            };

            // getting the GL version and extensions
            let version = get_gl_version(&gl);
            let extensions = get_extensions(&gl);

            // main loop
            'main: loop {
                // processing commands
                loop {
                    match rx_commands.recv_opt() {
                        Ok(EndFrame) => break,
                        Ok(Execute(cmd)) => cmd(&gl, &mut gl_state, &version, &extensions),
                        Err(_) => break 'main
                    }
                }

                // this is necessary on Windows 8, or nothing is being displayed
                unsafe { gl.Flush(); }

                // swapping
                window.swap_buffers();

                // getting events
                for event in window.poll_events() {
                    // calling `glViewport` if the window has been resized
                    if let &glutin::Resized(width, height) = &event {
                        dimensions.0.store(width, Relaxed);
                        dimensions.1.store(height, Relaxed);

                        if gl_state.viewport != (0, 0, width as gl::types::GLsizei,
                                                 height as gl::types::GLsizei)
                        {
                            unsafe {
                                gl.Viewport(0, 0, width as gl::types::GLsizei,
                                    height as gl::types::GLsizei);
                                gl_state.viewport = (0, 0, width as gl::types::GLsizei,
                                    height as gl::types::GLsizei);
                            }
                        }
                    }

                    // sending the event outside
                    if tx_events.send_opt(event.clone()).is_err() {
                        break 'main;
                    }
                }
            }
        });

        try!(rx_success.recv());
        Ok(context)
    }

    pub fn new_from_headless(window: glutin::HeadlessRendererBuilder)
        -> Result<Context, GliumCreationError>
    {
        let (_, rx_events) = channel();
        let (tx_commands, rx_commands) = channel();

        // TODO: fixme
        let dimensions = Arc::new((AtomicUint::new(800), AtomicUint::new(600)));

        let context = Context {
            commands: Mutex::new(tx_commands),
            events: Mutex::new(rx_events),
            dimensions: dimensions,
        };

        let (tx_success, rx_success) = channel();

        TaskBuilder::new().native().spawn(proc() {
            let window = match window.build() {
                Ok(w) => w,
                Err(e) => {
                    tx_success.send(Err(e));
                    return;
                }
            };
            unsafe { window.make_current(); }
            tx_success.send(Ok(()));

            let gl = gl::Gl::load_with(|symbol| window.get_proc_address(symbol));

            // building the GLState, version, and extensions
            let mut gl_state = GLState::new_defaults((0, 0, 0, 0));    // FIXME: 
            let version = get_gl_version(&gl);
            let extensions = get_extensions(&gl);

            loop {
                match rx_commands.recv_opt() {
                    Ok(Execute(cmd)) => cmd(&gl, &mut gl_state, &version, &extensions),
                    Ok(EndFrame) => (),     // ignoring buffer swapping
                    Err(_) => break
                }
            }
        });

        try!(rx_success.recv());
        Ok(context)
    }

    pub fn get_framebuffer_dimensions(&self) -> (uint, uint) {
        (
            self.dimensions.0.load(Relaxed),
            self.dimensions.1.load(Relaxed),
        )
    }

    pub fn exec(&self, f: proc(&gl::Gl, &mut GLState, &GlVersion, &ExtensionsList): Send) {
        self.commands.lock().send(Execute(f));
    }

    pub fn swap_buffers(&self) {
        self.commands.lock().send(EndFrame);
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
}

fn get_gl_version(gl: &gl::Gl) -> GlVersion {
    use std::c_str::CString;

    unsafe {
        let version = gl.GetString(gl::VERSION);
        let version = CString::new(version as *const i8, false);
        let version = version.as_str().expect("OpenGL version contains non-utf8 characters");

        let version = version.words().next().expect("glGetString(GL_VERSION) returned an empty \
                                                     string");

        let mut iter = version.split(|c: char| c == '.');
        let major = iter.next().unwrap();
        let minor = iter.next().expect("glGetString(GL_VERSION) did not return a correct version");

        GlVersion(
            from_str(major).expect("failed to parse GL major version"),
            from_str(minor).expect("failed to parse GL minor version"),
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
    };

    for extension in strings.into_iter() {
        match extension.as_slice() {
            "GL_EXT_direct_state_access" => extensions.gl_ext_direct_state_access = true,
            "GL_EXT_framebuffer_object" => extensions.gl_ext_framebuffer_object = true,
            "GL_EXT_geometry_shader4" => extensions.gl_ext_geometry_shader4 = true,
            "GL_EXT_framebuffer_blit" => extensions.gl_ext_framebuffer_blit = true,
            "GL_KHR_debug" => extensions.gl_khr_debug = true,
            _ => ()
        }
    }

    extensions
}
