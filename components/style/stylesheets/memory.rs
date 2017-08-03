/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Memory reporting for the style system when running inside of Gecko.

#[cfg(feature = "gecko")]
use gecko_bindings::bindings::Gecko_HaveSeenPtr;
#[cfg(feature = "gecko")]
use gecko_bindings::structs::SeenPtrs;
#[cfg(feature = "gecko")]
use servo_arc::Arc;
use shared_lock::SharedRwLockReadGuard;
use std::os::raw::c_void;

/// Like gecko_bindings::structs::MallocSizeOf, but without the Option<>
/// wrapper.
///
/// Note that functions of this type should not be called via
/// do_malloc_size_of(), rather than directly.
pub type MallocSizeOfFn = unsafe extern "C" fn(ptr: *const c_void) -> usize;

/// Servo-side counterpart to mozilla::SizeOfState. The only difference is that
/// this struct doesn't contain the SeenPtrs table, just a pointer to it.
#[cfg(feature = "gecko")]
pub struct SizeOfState {
    /// Function that measures the size of heap blocks.
    pub malloc_size_of: MallocSizeOfFn,
    /// Table recording heap blocks that have already been measured.
    pub seen_ptrs: *mut SeenPtrs,
}

/// Call malloc_size_of on ptr, first checking that the allocation isn't empty.
pub unsafe fn is_empty<T>(ptr: *const T) -> bool {
    return ptr as usize <= ::std::mem::align_of::<T>();
}

/// Call malloc_size_of on ptr, first checking that the allocation isn't empty.
pub unsafe fn do_malloc_size_of<T>(malloc_size_of: MallocSizeOfFn, ptr: *const T) -> usize {
    if is_empty(ptr) {
        0
    } else {
        malloc_size_of(ptr as *const c_void)
    }
}

/// Trait for measuring the size of heap data structures.
pub trait MallocSizeOf {
    /// Measure the size of any heap-allocated structures that hang off this
    /// value, but not the space taken up by the value itself.
    fn malloc_size_of_children(&self, malloc_size_of: MallocSizeOfFn) -> usize;
}

/// Like MallocSizeOf, but takes a SizeOfState which allows it to measure
/// graph-like structures such as those containing Arcs.
#[cfg(feature = "gecko")]
pub trait MallocSizeOfWithRepeats {
    /// Measure the size of any heap-allocated structures that hang off this
    /// value, but not the space taken up by the value itself.
    fn malloc_size_of_children(&self, state: &mut SizeOfState) -> usize;
}

/// Like MallocSizeOf, but operates with the global SharedRwLockReadGuard
/// locked.
pub trait MallocSizeOfWithGuard {
    /// Like MallocSizeOf::malloc_size_of_children, but with a |guard| argument.
    fn malloc_size_of_children(
        &self,
        guard: &SharedRwLockReadGuard,
        malloc_size_of: MallocSizeOfFn
    ) -> usize;
}

impl<A: MallocSizeOf, B: MallocSizeOf> MallocSizeOf for (A, B) {
    fn malloc_size_of_children(&self, malloc_size_of: MallocSizeOfFn) -> usize {
        self.0.malloc_size_of_children(malloc_size_of) +
            self.1.malloc_size_of_children(malloc_size_of)
    }
}

impl<T: MallocSizeOf> MallocSizeOf for Vec<T> {
    fn malloc_size_of_children(&self, malloc_size_of: MallocSizeOfFn) -> usize {
        self.iter().fold(
            unsafe { do_malloc_size_of(malloc_size_of, self.as_ptr()) },
            |n, elem| n + elem.malloc_size_of_children(malloc_size_of))
    }
}

#[cfg(feature = "gecko")]
impl<T: MallocSizeOfWithRepeats> MallocSizeOfWithRepeats for Arc<T> {
    fn malloc_size_of_children(&self, state: &mut SizeOfState) -> usize {
        let mut n = 0;
        let heap_ptr = self.heap_ptr();
        if unsafe { !is_empty(heap_ptr) && !Gecko_HaveSeenPtr(state.seen_ptrs, heap_ptr) } {
            n += unsafe { (state.malloc_size_of)(heap_ptr) };
            n += (**self).malloc_size_of_children(state);
        }
        n
    }
}

impl<T: MallocSizeOfWithGuard> MallocSizeOfWithGuard for Vec<T> {
    fn malloc_size_of_children(
        &self,
        guard: &SharedRwLockReadGuard,
        malloc_size_of: MallocSizeOfFn,
    ) -> usize {
        self.iter().fold(
            unsafe { do_malloc_size_of(malloc_size_of, self.as_ptr()) },
            |n, elem| n + elem.malloc_size_of_children(guard, malloc_size_of))
    }
}
