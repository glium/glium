use std::sync::mpsc;
use std::ptr;

use Display;

use libc;
use context;
use version::Api;
use gl;

/// Provides a way to wait for a server-side operation to be finished.
///
/// Creating a `SyncFence` injects an element in the commands queue of the backend.
/// When this element is reached, the fence becomes signaled.
///
/// ## Example
///
/// ```no_run
/// # let display: glium::Display = unsafe { std::mem::uninitialized() };
/// # fn do_something<T>(_: &T) {}
/// let fence = glium::SyncFence::new_if_supported(&display).unwrap();
/// do_something(&display);
/// fence.wait();   // blocks until the previous operations have finished
/// ```
pub struct SyncFence {
    display: Display,
    id: Option<SyncObjectWrapper>,
}

impl SyncFence {
    /// Builds a new `SyncFence` that is injected in the server.
    ///
    /// ## Features
    ///
    /// Only available if the `gl_sync` feature is enabled.
    #[cfg(feature = "gl_sync")]
    pub fn new(display: &Display) -> SyncFence {
        SyncFence::new_if_supported(display).unwrap()
    }

    /// Builds a new `SyncFence` that is injected in the server.
    ///
    /// Returns `None` is this is not supported by the backend.
    pub fn new_if_supported(display: &Display) -> Option<SyncFence> {
        let (tx, rx) = mpsc::channel();

        display.context.context.exec(move |mut ctxt| {
            tx.send(unsafe { new_linear_sync_fence_if_supported(&mut ctxt) }).unwrap();
        });

        rx.recv().unwrap().map(|f| f.into_sync_fence(display))
    }

    /// Blocks until the operation has finished on the server.
    pub fn wait(mut self) {
        let sync = self.id.take().unwrap();
        let (tx, rx) = mpsc::channel();

        self.display.context.context.exec(move |ctxt| {
            unsafe {
                // waiting with a deadline of one year
                // the reason why the deadline is so long is because if you attach a GL debugger,
                // the wait can be blocked during a breaking point of the debugger
                let result = ctxt.gl.ClientWaitSync(sync.0, gl::SYNC_FLUSH_COMMANDS_BIT,
                                                    365 * 24 * 3600 * 1000 * 1000 * 1000);
                tx.send(result).unwrap();
                ctxt.gl.DeleteSync(sync.0);
            }
        });

        match rx.recv().unwrap() {
            gl::ALREADY_SIGNALED | gl::CONDITION_SATISFIED => (),
            _ => panic!("Could not wait for the fence")
        };
    }
}

impl Drop for SyncFence {
    fn drop(&mut self) {
        let sync = match self.id {
            None => return,     // fence has already been deleted
            Some(s) => s
        };

        self.display.context.context.exec(move |ctxt| {
            unsafe {
                ctxt.gl.DeleteSync(sync.0);
            }
        });
    }
}

/// Prototype for a `SyncFence`.
///
/// The fence must be consumed with either `into_sync_fence`, otherwise
/// the destructor will panic.
#[must_use]
pub struct LinearSyncFence {
    id: Option<SyncObjectWrapper>,
}

unsafe impl Send for LinearSyncFence {}

impl LinearSyncFence {
    /// Turns the prototype into a real fence.
    pub fn into_sync_fence(mut self, display: &Display) -> SyncFence {
        SyncFence {
            display: display.clone(),
            id: self.id.take()
        }
    }
}

impl Drop for LinearSyncFence {
    fn drop(&mut self) {
        assert!(self.id.is_none());
    }
}

#[cfg(feature = "gl_sync")]
pub unsafe fn new_linear_sync_fence(ctxt: &mut context::CommandContext) -> LinearSyncFence {
    LinearSyncFence {
        id: Some(SyncObjectWrapper(ctxt.gl.FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0))),
    }
}

pub unsafe fn new_linear_sync_fence_if_supported(ctxt: &mut context::CommandContext) -> Option<LinearSyncFence> {
    if ctxt.version < &context::GlVersion(Api::Gl, 3, 2) && !ctxt.extensions.gl_arb_sync {
        return None;
    }

    Some(LinearSyncFence {
        id: Some(SyncObjectWrapper(ctxt.gl.FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0))),
    })
}

/// Waits for this fence and destroys it, from within the commands context.
pub unsafe fn wait_linear_sync_fence_and_drop(mut fence: LinearSyncFence, ctxt: &mut context::CommandContext) {
    let fence = fence.id.take().unwrap();

    // we try waiting without flushing first
    let success = match ctxt.gl.ClientWaitSync(fence.0, 0, 0) {
        gl::ALREADY_SIGNALED => true,
        gl::TIMEOUT_EXPIRED => false,
        gl::WAIT_FAILED => false,
        gl::CONDITION_SATISFIED => true,
        _ => unreachable!()
    };

    // waiting *with* flushing this time
    if !success {
        ctxt.gl.ClientWaitSync(fence.0, gl::SYNC_FLUSH_COMMANDS_BIT,
                               365 * 24 * 3600 * 1000 * 1000 * 1000);
    }

    ctxt.gl.DeleteSync(fence.0);
}

#[derive(Copy)]
struct SyncObjectWrapper(gl::types::GLsync);
unsafe impl Send for SyncObjectWrapper {}
unsafe impl Sync for SyncObjectWrapper {}
