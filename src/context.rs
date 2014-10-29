use gl;
use glutin;
use native::NativeTaskBuilder;
use std::sync::atomic::{AtomicUint, Relaxed};
use std::sync::{Arc, Mutex};
use std::task::TaskBuilder;

enum Message {
    EndFrame,
    Execute(proc(&gl::Gl, &mut GLState):Send),
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

    /// The latest framebuffer bound to `GL_READ_FRAMEBUFFER`.
    pub read_framebuffer: Option<gl::types::GLuint>,

    /// The latest framebuffer bound to `GL_DRAW_FRAMEBUFFER`.
    pub draw_framebuffer: Option<gl::types::GLuint>,

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
}

impl GLState {
    /// Builds the `GLState` corresponding to the default GL values.
    fn new_defaults(viewport: (gl::types::GLint, gl::types::GLint, gl::types::GLsizei, gl::types::GLsizei)) -> GLState {
        GLState {
            enabled_blend: false,
            enabled_cull_face: false,
            enabled_depth_test: false,
            enabled_dither: false,
            enabled_polygon_offset_fill: false,
            enabled_sample_alpha_to_coverage: false,
            enabled_sample_coverage: false,
            enabled_scissor_test: false,
            enabled_stencil_test: false,

            program: 0,
            clear_color: (0.0, 0.0, 0.0, 0.0),
            clear_depth: 1.0,
            clear_stencil: 0,
            array_buffer_binding: None,
            element_array_buffer_binding: None,
            read_framebuffer: None,
            draw_framebuffer: None,
            depth_func: gl::LESS,
            blend_func: (0, 0),     // no default specified
            viewport: viewport,
            line_width: 1.0,
            cull_face: gl::BACK,
        }
    }
}

impl Context {
    pub fn new_from_window(window: glutin::Window) -> Context {
        let (tx_events, rx_events) = channel();
        let (tx_commands, rx_commands) = channel();

        let dimensions = Arc::new((AtomicUint::new(800), AtomicUint::new(600)));

        let context = Context {
            commands: Mutex::new(tx_commands),
            events: Mutex::new(rx_events),
            dimensions: dimensions.clone(),
        };

        TaskBuilder::new().native().spawn(proc() {
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

                GLState::new_defaults(viewport)
            };

            'main: loop {
                // processing commands
                loop {
                    match rx_commands.recv_opt() {
                        Ok(EndFrame) => break,
                        Ok(Execute(cmd)) => cmd(&gl, &mut gl_state),
                        Err(_) => break 'main
                    }
                }

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
                            gl.Viewport(0, 0, width as gl::types::GLsizei,
                                height as gl::types::GLsizei);
                            gl_state.viewport = (0, 0, width as gl::types::GLsizei,
                                height as gl::types::GLsizei);
                        }
                    }

                    // sending the event outside
                    if tx_events.send_opt(event.clone()).is_err() {
                        break 'main;
                    }
                }
            }
        });

        context
    }

    pub fn new_from_headless(window: glutin::HeadlessContext) -> Context {
        let (_, rx_events) = channel();
        let (tx_commands, rx_commands) = channel();

        // TODO: fixme
        let dimensions = Arc::new((AtomicUint::new(800), AtomicUint::new(600)));

        let context = Context {
            commands: Mutex::new(tx_commands),
            events: Mutex::new(rx_events),
            dimensions: dimensions,
        };

        TaskBuilder::new().native().spawn(proc() {
            unsafe { window.make_current(); }

            let gl = gl::Gl::load_with(|symbol| window.get_proc_address(symbol));

            // building the GLState
            let mut gl_state = GLState::new_defaults((0, 0, 0, 0));    // FIXME: 

            loop {
                match rx_commands.recv_opt() {
                    Ok(Execute(cmd)) => cmd(&gl, &mut gl_state),
                    Ok(EndFrame) => (),     // ignoring buffer swapping
                    Err(_) => break
                }
            }
        });

        context
    }

    pub fn get_framebuffer_dimensions(&self) -> (uint, uint) {
        (
            self.dimensions.0.load(Relaxed),
            self.dimensions.1.load(Relaxed),
        )
    }

    pub fn exec(&self, f: proc(&gl::Gl, &mut GLState): Send) {
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
