/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use js::context::JSContext;
use js::jsapi::JS::{HeapState, RuntimeHeapState};

thread_local!(
    static THREAD_ACTIVE: Cell<bool> = const { Cell::new(true) };
);

pub fn runtime_is_alive() -> bool {
    THREAD_ACTIVE.with(|t| t.get())
}

/// Whether a GC collection is in progress.
/// Mainly useful for (debug) assertions.
pub fn during_gc_collection() -> bool {
    // SAFETY: `RuntimeHeapState` only reads thread-local runtime state and has no preconditions.
    matches!(
        unsafe { RuntimeHeapState() },
        HeapState::MajorCollecting | HeapState::MinorCollecting
    )
}

pub fn mark_runtime_dead() {
    THREAD_ACTIVE.with(|t| t.set(false));
}

/// Get the current JSContext for the running thread.
///
/// ## Safety
/// Using this function is unsafe because no other JSContext may be constructed apart from initial ones,
/// but because we are still working on passing down &mut SafeJSContext references,
/// this function is provided as temporary workaround/placeholder.
///
/// As such all it's usages will need to be eventually replaced with proper &mut SafeJSContext references.
pub unsafe fn temp_cx() -> JSContext {
    unsafe { JSContext::from_ptr(js::rust::Runtime::get().unwrap()) }
}
