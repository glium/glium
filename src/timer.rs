//! Utilities to measure GPU execution times.
//!
//! GPUs work in an asynchronous way. This means that for example when an application
//! asks to render a Vertex Array, the CPU sends the command to the GPU, and immediately
//! afterwards the CPU starts executing the next instruction. Meanwhile, the GPU will
//! start drawing the Vertex Array, and will continue this task without forcing the CPU
//! to wait for its completion.
//!
//! The utilities in this module allow you to query actual execution times of the GPU.
//! During application development, timing information can help identify application or
//! driver bottlenecks. At run time, applications can use timing information to
//! dynamically adjust the amount of detail in a scene to achieve constant frame rates.
//!
//! ## Example
//! ```ignore
//! # extern crate glium; fn main() {
//! # use std::mem;
//! # use glium::Surface;
//! # let display: glium::Display = unsafe { mem::uninitialized() };
//! # let vertex_buffer: glium::VertexBuffer = unsafe {mem::uninitialized() };
//! # let index_buffer: glium::IndexBuffer = unsafe {mem::uninitialized() };
//! # let program: glium::Program = unsafe {mem::uninitialized() };
//! # let uniforms: glium::uniforms::UniformsStorage = unsafe {mem::uninitialized() };
//! // drawing a frame
//! use glium::timer;
//! let mut target = display.draw();
//! let handle = timer::measure(&display, || {
//!     target.clear_color(0.0, 0.0, 0.0, 0.0);
//!     target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &Default::default()).unwrap();
//!     target.finish().unwrap();
//! }).unwrap();
//! println!("GPU needed {:?} to draw", handle.get_measurement().interval());
//! # }
//! ```

use std::mem;
use std::fmt;
use std::rc::Rc;
use std::error::Error;
use std::time::Duration;
use std::cell::Cell;

use gl;

use backend::Facade;
use context::{Context, CommandContext};
use {Api, Version};
use {CapabilitiesSource, ContextExt};

/// Errors that can occur when using timers.
#[derive(Clone, Debug)]
pub enum TimerCreationError {
    /// The backend does not support timer queries.
    TimerQueryNotSupported,
}

impl fmt::Display for TimerCreationError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            _ => write!(fmt, "{}", self.description()),
        }
    }
}

impl Error for TimerCreationError {
    fn description(&self) -> &str {
        use self::TimerCreationError::*;
        match *self {
            TimerQueryNotSupported =>
                "The backend does not support timer queries."
        }
    }
}

/// A time measurement of GPU commands.
#[derive(Copy, Clone, Debug)]
pub struct TimeMeasurement {
    /// The start timestamp
    pub start_time: Duration,
    /// The end timestamp
    pub end_time: Duration,
}



impl TimeMeasurement {
    /// Returns the difference between `end_time` and `start_time`.
    #[inline]
    pub fn interval(&self) -> Duration {
        self.end_time - self.start_time
    }
}

/// Safe wrapper around a OpenGL timer query.
struct TimerQuery {
    ctxt: Rc<Context>,
    id: gl::types::GLuint,
}

impl TimerQuery {
    /// Creates a timer query and sends it to the GPU.
    #[inline]
    pub fn new<F: Facade>(facade: &F) -> Self {
        let mut ctxt = facade.get_context().make_current();
        let timer_query_id = unsafe {
            let mut timer_query_id = mem::uninitialized();
            ctxt.gl.GenQueries(1, &mut timer_query_id);
            ctxt.gl.QueryCounter(timer_query_id, gl::TIMESTAMP);
            timer_query_id
        };

        TimerQuery {
            ctxt: facade.get_context().clone(),
            id: timer_query_id
        }
    }

    #[inline]
    pub fn is_result_available(&self) -> bool {
        let mut ctxt = self.ctxt.make_current();
        let result_available = unsafe {
            let mut result_available = mem::uninitialized();
            ctxt.gl.GetQueryObjectiv(self.id, gl::QUERY_RESULT_AVAILABLE, &mut result_available);
            result_available
        };
        if result_available == 0 { false } else { true }
    }

    // Unsafety: Result must be available.
    #[inline]
    pub unsafe fn query_result(&self) -> gl::types::GLuint64 {
        let mut ctxt = self.ctxt.make_current();
        let mut timestamp = mem::uninitialized();
        ctxt.gl.GetQueryObjectui64v(self.id, gl::QUERY_RESULT, &mut timestamp);
        timestamp
    }
}

impl Drop for TimerQuery {
    fn drop(&mut self) {
        let mut ctxt = self.ctxt.make_current();
        unsafe {
            ctxt.gl.DeleteQueries(1, &self.id);
        }
    }
}

/// A handle to an asynchronous time measurement on the GPU.
pub struct AsyncTimerHandle {
    start_query: TimerQuery,
    end_query: TimerQuery,
    result_available: Cell<bool>,
}

impl AsyncTimerHandle {
    /// Returns true if the timer result is available.
    #[inline]
    pub fn is_result_available(&self) -> bool {
        let result_available = { self.result_available.get() };
        if result_available { return true }
        if self.end_query.is_result_available() {
            self.result_available.set(true);
            true
        } else {
            false
        }
    }

    /// Blocks until the timer result is available,
    /// then returns the `TimeMeasurement` measured by the timer.
    ///
    /// If `is_result_available` returned `true`, this method will not block.
    pub fn get_measurement(self) -> TimeMeasurement {
        let result_available = self.result_available.get();
        let (start_time, end_time) = {
            if result_available {
                unsafe { ( self.start_query.query_result(), self.end_query.query_result() ) }
            } else {
                // block until results are available
                loop {
                    if self.end_query.is_result_available() { break }
                }
                unsafe { ( self.start_query.query_result(), self.end_query.query_result() ) }
            }
        };

        TimeMeasurement {
            start_time: nanoseconds_to_duration(start_time),
            end_time: nanoseconds_to_duration(end_time),
        }
    }
}

fn nanoseconds_to_duration(nanos: u64) -> Duration {
    const NANOS_PER_SECOND: u64 = 1_000_000_000;
    let seconds = nanos / NANOS_PER_SECOND;
    let nanos = (nanos % NANOS_PER_SECOND) as u32;
    Duration::new(seconds, nanos)
}

/// Executes the closure and measures the time needed by the GPU.
/// Any pending GPU operations will not be included.
///
/// If the GPU is idle during execution, the clock keeps ticking, so it doesn't make a lot
/// of sense to use this function for CPU-heavy tasks.
pub fn measure<F, M>(facade: &F, measure: M) -> Result<AsyncTimerHandle, TimerCreationError>
                     where F: Facade, M: FnOnce() -> ()
{
    if !is_timer_query_supported(facade.get_context()) {
        return Err(TimerCreationError::TimerQueryNotSupported)
    }

    let start_query = TimerQuery::new(facade);
    measure();
    let end_query = TimerQuery::new(facade);

    Ok(AsyncTimerHandle {
        start_query: start_query,
        end_query: end_query,
        result_available: Cell::new(false),
    })
}

/// Queries the current timestamp from the GPU.
///
/// In combination with `timer::measure`, this method is useful for measuring GPU latency of the application.
pub fn timestamp<F: Facade>(facade: &F) -> Result<Duration, TimerCreationError> {
    if !is_timer_query_supported(facade.get_context()) {
        return Err(TimerCreationError::TimerQueryNotSupported)
    }
    let mut ctxt = facade.get_context().make_current();
    let timestamp = unsafe {
        let mut t = mem::uninitialized();
        ctxt.gl.GetInteger64v(gl::TIMESTAMP, &mut t);
        t
    };
    Ok(nanoseconds_to_duration(timestamp as u64))
}

/// Returns true if the backend supports timer queries.
#[inline]
pub fn is_timer_query_supported<C>(ctxt: &C) -> bool where C: CapabilitiesSource {
    ctxt.get_version() >= &Version(Api::Gl, 3, 3) || ctxt.get_extensions().gl_arb_timer_query ||
    ctxt.get_extensions().gl_ext_disjoint_timer_query
}
