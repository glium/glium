use crate::context::CommandContext;
use crate::version::Api;
use crate::version::Version;
use crate::gl;

use crate::backend::Facade;
use crate::context::Context;
use crate::ContextExt;
use std::rc::Rc;

use std::thread;

/// Error that happens when sync functionalities are not supported.
#[derive(Copy, Clone, Debug)]
pub struct SyncNotSupportedError;

/// Provides a way to wait for a server-side operation to be finished.
///
/// Creating a `SyncFence` injects an element in the commands queue of the backend.
/// When this element is reached, the fence becomes signaled.
///
/// ## Example
///
/// ```no_run
/// # use glutin::surface::{ResizeableSurface, SurfaceTypeTrait};
/// # fn example<T>(display: glium::Display<T>) where T: SurfaceTypeTrait + ResizeableSurface {
/// # fn do_something<T>(_: &T) {}
/// let fence = glium::SyncFence::new(&display).unwrap();
/// do_something(&display);
/// fence.wait();   // blocks until the previous operations have finished
/// # }
/// ```
pub struct SyncFence {
    context: Rc<Context>,
    id: Option<gl::types::GLsync>,
}

impl SyncFence {
    /// Builds a new `SyncFence` that is injected in the server.
    #[inline]
    pub fn new<F: ?Sized>(facade: &F) -> Result<SyncFence, SyncNotSupportedError> where F: Facade {
        let mut ctxt = facade.get_context().make_current();
        unsafe { new_linear_sync_fence(&mut ctxt) }.map(|f| f.into_sync_fence(facade))
    }

    /// Blocks until the operation has finished on the server.
    pub fn wait(mut self) {
        let sync = self.id.take().unwrap();

        let mut ctxt = self.context.make_current();
        let result = unsafe { client_wait(&mut ctxt, sync) };
        unsafe { delete_fence(&mut ctxt, sync) };

        match result {
            gl::ALREADY_SIGNALED | gl::CONDITION_SATISFIED => (),
            _ => panic!("Could not wait for the fence")
        };
    }
}

impl Drop for SyncFence {
    #[inline]
    fn drop(&mut self) {
        let sync = match self.id {
            None => return,     // fence has already been deleted
            Some(s) => s
        };

        let mut ctxt = self.context.make_current();
        unsafe { delete_fence(&mut ctxt, sync) };
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
    #[inline]
    pub fn into_sync_fence<F: ?Sized>(mut self, facade: &F) -> SyncFence where F: Facade {
        SyncFence {
            context: facade.get_context().clone(),
            id: self.id.take()
        }
    }
}

impl Drop for LinearSyncFence {
    #[inline]
    fn drop(&mut self) {
        if !thread::panicking() {
            assert!(self.id.is_none());
        }
    }
}

pub unsafe fn new_linear_sync_fence(ctxt: &mut CommandContext<'_>)
                                    -> Result<LinearSyncFence, SyncNotSupportedError>
{
    if ctxt.version >= &Version(Api::Gl, 3, 2) ||
       ctxt.version >= &Version(Api::GlEs, 3, 0) || ctxt.extensions.gl_arb_sync
    {
        Ok(LinearSyncFence {
            id: Some(ctxt.gl.FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0)),
        })

    } else if ctxt.extensions.gl_apple_sync {
        Ok(LinearSyncFence {
            id: Some(ctxt.gl.FenceSyncAPPLE(gl::SYNC_GPU_COMMANDS_COMPLETE_APPLE, 0)),
        })

    } else {
        Err(SyncNotSupportedError)
    }
}

/// Waits for this fence and destroys it, from within the commands context.
#[inline]
pub unsafe fn wait_linear_sync_fence_and_drop(mut fence: LinearSyncFence,
                                              ctxt: &mut CommandContext<'_>)
{
    let fence = fence.id.take().unwrap();
    client_wait(ctxt, fence);
    delete_fence(ctxt, fence);
}

/// Destroys a fence, from within the commands context.
#[inline]
pub unsafe fn destroy_linear_sync_fence(ctxt: &mut CommandContext<'_>, mut fence: LinearSyncFence) {
    let fence = fence.id.take().unwrap();
    delete_fence(ctxt, fence);
}

/// Calls `glClientWaitSync` and returns the result.
///
/// Tries without flushing first, then with flushing.
///
/// # Unsafety
///
/// The fence object must exist.
///
unsafe fn client_wait(ctxt: &mut CommandContext<'_>, fence: gl::types::GLsync) -> gl::types::GLenum {
    // trying without flushing first
    let result = if ctxt.version >= &Version(Api::Gl, 3, 2) ||
                    ctxt.version >= &Version(Api::GlEs, 3, 0) || ctxt.extensions.gl_arb_sync
    {
        ctxt.gl.ClientWaitSync(fence, 0, 0)
    } else if ctxt.extensions.gl_apple_sync {
        ctxt.gl.ClientWaitSyncAPPLE(fence, 0, 0)
    } else {
        unreachable!();
    };

    match result {
        val @ gl::ALREADY_SIGNALED | val @ gl::CONDITION_SATISFIED => return val,
        gl::TIMEOUT_EXPIRED => (),
        gl::WAIT_FAILED => (),
        _ => unreachable!()
    };

    // waiting with a deadline of one year
    // the reason why the deadline is so long is because if you attach a GL debugger,
    // the wait can be blocked during a breaking point of the debugger
    if ctxt.version >= &Version(Api::Gl, 3, 2) ||
       ctxt.version >= &Version(Api::GlEs, 3, 0) || ctxt.extensions.gl_arb_sync
    {
        ctxt.gl.ClientWaitSync(fence, gl::SYNC_FLUSH_COMMANDS_BIT,
                               365 * 24 * 3600 * 1000 * 1000 * 1000)
    } else if ctxt.extensions.gl_apple_sync {
        ctxt.gl.ClientWaitSyncAPPLE(fence, gl::SYNC_FLUSH_COMMANDS_BIT_APPLE,
                                    365 * 24 * 3600 * 1000 * 1000 * 1000)
    } else {
        unreachable!();
    }
}

/// Deletes a fence.
///
/// # Unsafety
///
/// The fence object must exist.
///
#[inline]
unsafe fn delete_fence(ctxt: &mut CommandContext<'_>, fence: gl::types::GLsync) {
    if ctxt.version >= &Version(Api::Gl, 3, 2) ||
       ctxt.version >= &Version(Api::GlEs, 3, 0) || ctxt.extensions.gl_arb_sync
    {
        ctxt.gl.DeleteSync(fence);
    } else if ctxt.extensions.gl_apple_sync {
        ctxt.gl.DeleteSyncAPPLE(fence);
    } else {
        unreachable!();
    };
}
