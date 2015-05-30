use context::CommandContext;
use version::Api;
use version::Version;
use gl;

use backend::Facade;
use context::Context;
use ContextExt;
use std::rc::Rc;

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
    context: Rc<Context>,
    id: Option<gl::types::GLsync>,
}

impl SyncFence {
    /// Builds a new `SyncFence` that is injected in the server.
    ///
    /// # Features
    ///
    /// Only available if the `gl_sync` feature is enabled.
    #[cfg(feature = "gl_sync")]
    pub fn new<F>(facade: &F) -> SyncFence where F: Facade {
        SyncFence::new_if_supported(facade).unwrap()
    }

    /// Builds a new `SyncFence` that is injected in the server.
    ///
    /// Returns `None` is this is not supported by the backend.
    pub fn new_if_supported<F>(facade: &F) -> Option<SyncFence> where F: Facade {
        let mut ctxt = facade.get_context().make_current();

        unsafe { new_linear_sync_fence_if_supported(&mut ctxt) }
            .map(|f| f.into_sync_fence(facade))
    }

    /// Blocks until the operation has finished on the server.
    pub fn wait(mut self) {
        let sync = self.id.take().unwrap();

        let ctxt = self.context.make_current();

        let result = unsafe {
            // waiting with a deadline of one year
            // the reason why the deadline is so long is because if you attach a GL debugger,
            // the wait can be blocked during a breaking point of the debugger
            let result = ctxt.gl.ClientWaitSync(sync, gl::SYNC_FLUSH_COMMANDS_BIT,
                                                365 * 24 * 3600 * 1000 * 1000 * 1000);
            ctxt.gl.DeleteSync(sync);
            result
        };

        match result {
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

        let ctxt = self.context.make_current();
        unsafe { ctxt.gl.DeleteSync(sync); }
    }
}

/// Prototype for a `SyncFence`.
///
/// The fence must be consumed with either `into_sync_fence`, otherwise
/// the destructor will panic.
#[must_use]
pub struct LinearSyncFence {
    id: Option<gl::types::GLsync>,
}

unsafe impl Send for LinearSyncFence {}

impl LinearSyncFence {
    /// Turns the prototype into a real fence.
    pub fn into_sync_fence<F>(mut self, facade: &F) -> SyncFence where F: Facade {
        SyncFence {
            context: facade.get_context().clone(),
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
pub unsafe fn new_linear_sync_fence(ctxt: &mut CommandContext) -> LinearSyncFence {
    LinearSyncFence {
        id: Some(ctxt.gl.FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0)),
    }
}

pub unsafe fn new_linear_sync_fence_if_supported(ctxt: &mut CommandContext) -> Option<LinearSyncFence> {
    if !(ctxt.version >= &Version(Api::Gl, 3, 2)) && !ctxt.extensions.gl_arb_sync
        && !(ctxt.version >= &Version(Api::GlEs, 3, 0))
    {
        return None;
    }

    Some(LinearSyncFence {
        id: Some(ctxt.gl.FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0)),
    })
}

/// Waits for this fence and destroys it, from within the commands context.
pub unsafe fn wait_linear_sync_fence_and_drop(mut fence: LinearSyncFence,
                                              ctxt: &mut CommandContext)
{
    let fence = fence.id.take().unwrap();

    // we try waiting without flushing first
    match ctxt.gl.ClientWaitSync(fence, 0, 0) {
        gl::ALREADY_SIGNALED | gl::CONDITION_SATISFIED => {
            ctxt.gl.DeleteSync(fence);
            return;
        },
        gl::TIMEOUT_EXPIRED => (),
        gl::WAIT_FAILED => (),
        _ => unreachable!()
    };

    // waiting *with* flushing this time
    ctxt.gl.ClientWaitSync(fence, gl::SYNC_FLUSH_COMMANDS_BIT,
                           365 * 24 * 3600 * 1000 * 1000 * 1000);
    ctxt.gl.DeleteSync(fence);
}

/// Destroys a fence, from within the commands context.
pub unsafe fn destroy_linear_sync_fence(ctxt: &mut CommandContext, mut fence: LinearSyncFence) {
    let fence = fence.id.take().unwrap();
    ctxt.gl.DeleteSync(fence);
}
