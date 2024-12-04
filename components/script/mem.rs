/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Routines for handling measuring the memory usage of arbitrary DOM nodes.

use std::os::raw::c_void;

use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};

/// Used by codegen to include the pointer to the `MallocSizeOf` implementation of each
/// IDL interface. This way we don't have to find the most-derived interface of DOM
/// objects by hand in code.
#[allow(unsafe_code)]
pub unsafe fn malloc_size_of_including_raw_self<T: MallocSizeOf>(
    ops: &mut MallocSizeOfOps,
    obj: *const c_void,
) -> usize {
    ops.malloc_size_of(obj) + (*(obj as *const T)).size_of(ops)
}
