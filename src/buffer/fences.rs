/*!

This module handles the fences of a buffer.

*/
use smallvec::SmallVec;
use std::cell::RefCell;
use std::ops::Range;

use context::CommandContext;
use sync::{self, LinearSyncFence};

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
    pub fn inserter(&self, range: Range<usize>) -> Inserter {
        Inserter {
            fences: self,
            range: range,
        }
    }

    /// Waits until the given range is accessible.
    pub fn wait(&self, ctxt: &mut CommandContext, range: Range<usize>) {
        let mut existing_fences = self.fences.borrow_mut();
        let mut new_fences = SmallVec::new();

        for existing in existing_fences.into_iter() {
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
    pub fn clean(&mut self, ctxt: &mut CommandContext) {
        let mut fences = self.fences.borrow_mut();
        for (_, sync) in fences.into_iter() {
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
    pub fn insert(self, ctxt: &mut CommandContext) {
        let mut new_fences = SmallVec::new();

        let mut written = false;

        let mut existing_fences = self.fences.fences.borrow_mut();
        for existing in existing_fences.into_iter() {
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
