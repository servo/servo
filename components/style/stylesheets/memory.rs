/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Memory reporting for the style system when running inside of Gecko.

use shared_lock::SharedRwLockReadGuard;
use std::os::raw::c_void;

/// Like gecko_bindings::structs::MallocSizeOf, but without the Option<>
/// wrapper.
///
/// Note that functions of this type should not be called via
/// do_malloc_size_of(), rather than directly.
pub type MallocSizeOfFn = unsafe extern "C" fn(ptr: *const c_void) -> usize;

/// Call malloc_size_of on ptr, first checking that the allocation isn't empty.
pub unsafe fn do_malloc_size_of<T>(malloc_size_of: MallocSizeOfFn, ptr: *const T) -> usize {
    use std::mem::align_of;

    if ptr as usize <= align_of::<T>() {
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
