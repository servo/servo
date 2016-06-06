/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Reference-counted pointers to flows.
//!
//! Eventually, with dynamically sized types in Rust, much of this code will
//! be superfluous. This design is largely duplicating logic of Arc<T> and
//! Weak<T>; please see comments there for details.


use flow::Flow;
use std::sync::{Arc, Weak};

pub type FlowRef = Arc<Flow>;
pub type WeakFlowRef = Weak<Flow>;

/// WARNING: This should only be used when there is no aliasing:
/// when the traversal ensures that no other threads accesses the same flow at the same time.
/// See https://github.com/servo/servo/issues/6503
/// Use Arc::get_mut instead when possible (e.g. on an Arc that was just created).
#[allow(unsafe_code)]
pub fn deref_mut<'a>(r: &'a mut FlowRef) -> &'a mut Flow {
    let ptr: *const Flow = &**r;
    unsafe {
        &mut *(ptr as *mut Flow)
    }
}
