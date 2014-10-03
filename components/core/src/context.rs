use gl;
use glutin;
use native::NativeTaskBuilder;
use std::sync::atomic::{AtomicUint, Relaxed};
use std::sync::{Arc, Mutex};
use std::task::TaskBuilder;
use time;

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

    /// The latest value passed to `glDepthFunc`.
    pub depth_func: gl::types::GLenum,

    /// The latest values passed to `glViewport`.
    pub viewport: (gl::types::GLint, gl::types::GLint, gl::types::GLsizei, gl::types::GLsizei),

}

impl Context {
    pub fn new(window: glutin::Window) -> Context {
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
                gl.DepthFunc(gl::ALWAYS);
                gl.Enable(gl::DEPTH_TEST);

                let viewport = {
                    let dim = window.get_inner_size().unwrap();
                    dimensions.0.store(dim.0, Relaxed);
                    dimensions.1.store(dim.1, Relaxed);
                    (0, 0, dim.0 as gl::types::GLsizei, dim.1 as gl::types::GLsizei)
                };

                GLState {
                    program: 0,
                    clear_color: (0.0, 0.0, 0.0, 0.0),
                    clear_depth: 1.0,
                    clear_stencil: 0,
                    array_buffer_binding: None,
                    element_array_buffer_binding: None,
                    depth_func: gl::ALWAYS,
                    viewport: viewport,
                }
            };

            let mut next_loop = time::precise_time_ns();
            'main: loop {
                // sleeping until next frame must be drawn
                use std::io::timer;
                timer::sleep({ 
                    use std::time::Duration;

                    let now = time::precise_time_ns();
                    if next_loop < now {
                        Duration::nanoseconds(0)
                    } else {
                        Duration::nanoseconds((next_loop - now) as i64)
                    }
                });

                // calling glViewport
                {
                    match window.get_inner_size() {
                        Some(dim) => {
                            dimensions.0.store(dim.0, Relaxed);
                            dimensions.1.store(dim.1, Relaxed);

                            // TODO: this should not be here
                            if gl_state.viewport != (0, 0, dim.0 as gl::types::GLsizei,
                                                     dim.0 as gl::types::GLsizei)
                            {
                                gl.Viewport(0, 0, *dim.ref0() as gl::types::GLsizei,
                                    *dim.ref1() as gl::types::GLsizei);
                                gl_state.viewport = (0, 0, dim.0 as gl::types::GLsizei,
                                    dim.1 as gl::types::GLsizei);
                            }
                        },
                        None => ()
                    };
                }

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
                    if tx_events.send_opt(event.clone()).is_err() {
                        break 'main;
                    }
                }

                // finding time to next loop
                next_loop += 16666667;
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
        let mut events = self.events.lock();

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
