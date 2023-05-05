/*!

This module handles the fences of a buffer.

*/
use smallvec::SmallVec;
use std::cell::{RefCell, RefMut};
use std::ops::Range;

use crate::context::CommandContext;
use crate::sync::{self, LinearSyncFence};

/// Contains a list of fences.
pub struct Fences {
    fences: RefCell<SmallVec<[(Range<usize>, LinearSyncFence); 16]>>,
}

impl Fences {
    /// Initialization.
    pub fn new() -> Fences {
        Fences {
            fences: RefCell::new(SmallVec::new()),
        }
    }

    /// Creates an `Inserter` that allows inserting a fence in the list for the given range.
    #[inline]
    pub fn inserter(&self, range: Range<usize>) -> Inserter<'_> {
        Inserter {
            fences: self,
            range,
        }
    }

    /// Waits until the given range is accessible.
    pub fn wait(&self, ctxt: &mut CommandContext<'_>, range: Range<usize>) {
        let mut existing_fences = self.fences.borrow_mut();
        let mut new_fences = SmallVec::new();

        for existing in existing_fences.drain(..) {
            if (existing.0.start >= range.start && existing.0.start < range.end) ||
               (existing.0.end > range.start && existing.0.end < range.end)
            {
                unsafe { sync::wait_linear_sync_fence_and_drop(existing.1, ctxt) };
            } else {
                new_fences.push(existing);
            }
        }

        *existing_fences = new_fences;
    }

    /// Cleans up all fences in the container. Must be called or you'll get a panic.
    pub fn clean(&mut self, ctxt: &mut CommandContext<'_>) {
        let mut fences = self.fences.borrow_mut();
        for (_, sync) in fences.drain(..) {
            unsafe { sync::destroy_linear_sync_fence(ctxt, sync) };
        }
    }
}

/// Allows inserting a fence in the list.
pub struct Inserter<'a> {
    fences: &'a Fences,
    range: Range<usize>,
}

impl<'a> Inserter<'a> {
    /// Inserts a new fence.
    pub fn insert(self, ctxt: &mut CommandContext<'_>) {
        let mut new_fences = SmallVec::new();

        let mut written = false;

        let mut existing_fences = self.fences.fences.borrow_mut();
        for existing in existing_fences.drain(..) {
            if existing.0.start < self.range.start && existing.0.end <= self.range.start {
                new_fences.push(existing);

            } else if existing.0.start < self.range.start && existing.0.end >= self.range.end {
                // we are stuck here, because we can't duplicate a fence
                // so instead we just extend the new fence to the existing one
                let new_fence = unsafe { sync::new_linear_sync_fence(ctxt).unwrap() };
                new_fences.push((existing.0.start .. self.range.start, existing.1));
                new_fences.push((self.range.start .. existing.0.end, new_fence));
                written = true;

            } else if existing.0.start < self.range.start && existing.0.end >= self.range.start {
                new_fences.push((existing.0.start .. self.range.start, existing.1));
                if !written {
                    let new_fence = unsafe { sync::new_linear_sync_fence(ctxt).unwrap() };
                    new_fences.push((self.range.clone(), new_fence));
                    written = true;
                }

            } else if existing.0.start >= self.range.start && existing.0.end <= self.range.end {
                unsafe { sync::destroy_linear_sync_fence(ctxt, existing.1) };
                if !written {
                    let new_fence = unsafe { sync::new_linear_sync_fence(ctxt).unwrap() };
                    new_fences.push((self.range.clone(), new_fence));
                    written = true;
                }

            } else if existing.0.start >= self.range.end {
                if !written {
                    let new_fence = unsafe { sync::new_linear_sync_fence(ctxt).unwrap() };
                    new_fences.push((self.range.clone(), new_fence));
                    written = true;
                }

                new_fences.push(existing);

            } else {
                if !written {
                    let new_fence = unsafe { sync::new_linear_sync_fence(ctxt).unwrap() };
                    new_fences.push((self.range.clone(), new_fence));
                    written = true;
                }

                new_fences.push((self.range.end .. existing.0.end, existing.1));
            }
        }

        if !written {
            let new_fence = unsafe { sync::new_linear_sync_fence(ctxt).unwrap() };
            new_fences.push((self.range, new_fence));
        }

        *existing_fences = new_fences;
    }
}

/// An encapsulated storage of fence inserters
/// Allows reusing a single Vec for fences without allocating new Vec's on every draw/compute call.
pub struct FenceInserters<'a>(RefMut<'a, Vec<Option<Inserter<'a>>>>);

impl<'a> FenceInserters<'a> {
    /// The constructor takes a reference to a RefCell with a 'static lifetime inside.
    /// This is because we want to keep a cached Vec of fences per Context.
    /// 
    /// Inside this constructor, the reference is downcast to a local lifetime,
    /// and after usage the Vec is cleared, and can be reused in the next draw/compute call.
    pub fn new(mut vec: RefMut<Vec<Option<Inserter<'static>>>>) -> Self {
        vec.clear();

        // Downcast lifetime from 'static to 'a.
        let vec = unsafe {
            std::mem::transmute::<RefMut<'_, Vec<Option<Inserter<'static>>>>, RefMut<'_, Vec<Option<Inserter<'a>>>>>(vec)
        };
        Self(vec)
    }

    /// Add a new fence
    pub fn push(&mut self, value: Inserter<'a>) {
        self.0.push(Some(value));
    }

    /// Fullfil all fences
    pub fn fulfill(&mut self, ctxt: &mut CommandContext) {
        for fence in self.0.iter_mut() {
            if let Some(fence) = std::mem::take(fence) {
                fence.insert(ctxt)
            }
        }

        // The underlying Vec will be returned to a cache to be reused in next draw/compute calls,
        // so we must clear it. Clear will not shrink the underlying memory of a vec,
        // reducing memory allocations.
        self.0.clear();
    }
}
