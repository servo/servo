/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `Reflector` struct.

pub use bindings::reflector::*;

use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use js::jsapi::{HandleObject, JSContext, JSObject};
use std::cell::UnsafeCell;
use std::ptr;

/// Create the reflector for a new DOM object and yield ownership to the
/// reflector.
pub fn reflect_dom_object<T: Reflectable>(obj: Box<T>,
                                          global: GlobalRef,
                                          wrap_fn: fn(*mut JSContext, GlobalRef, Box<T>) -> Root<T>)
                                          -> Root<T> {
    wrap_fn(global.get_cx(), global, obj)
}
