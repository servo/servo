/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Routines for handling measuring the memory usage of arbitrary DOM nodes.

use std::os::raw::c_void;

use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};

/// Used by codegen to include the pointer to the `MallocSizeOf` implementation of each
/// IDL interface. This way we don't have to find the most-derived interface of DOM
/// objects by hand in code.
#[expect(unsafe_code)]
pub(crate) unsafe fn malloc_size_of_including_raw_self<T: MallocSizeOf>(
    ops: &mut MallocSizeOfOps,
    obj: *const c_void,
) -> usize {
    unsafe { ops.malloc_size_of(obj) + (*(obj as *const T)).size_of(ops) }
}

/// Like [`malloc_size_of_including_raw_self`], but for weak-referenceable DOM objects living
/// inside an `Rc`. The `obj` pointer must be adjusted to the allocation base before asking the
/// allocator, as not all allocators can handle pointers pointing inside the allocation.
#[expect(unsafe_code)]
pub(crate) unsafe fn malloc_size_of_including_raw_self_in_rc<T: MallocSizeOf>(
    ops: &mut MallocSizeOfOps,
    obj: *const c_void,
) -> usize {
    let base = malloc_size_of::refcounted_allocation_base(obj as *const T);
    unsafe { ops.malloc_size_of(base) + (*(obj as *const T)).size_of(ops) }
}
