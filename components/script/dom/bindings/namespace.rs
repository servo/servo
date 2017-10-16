/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Machinery to initialise namespace objects.

use dom::bindings::guard::Guard;
use dom::bindings::interface::{create_object, define_on_global_object};
use js::jsapi::{HandleObject, JSClass, JSContext, JSFunctionSpec, MutableHandleObject};
use libc;

/// The class of a namespace object.
#[derive(Clone, Copy)]
pub struct NamespaceObjectClass(JSClass);

unsafe impl Sync for NamespaceObjectClass {}

impl NamespaceObjectClass {
    /// Create a new `NamespaceObjectClass` structure.
    pub const unsafe fn new(name: &'static [u8]) -> Self {
        NamespaceObjectClass(JSClass {
            name: name as *const _ as *const libc::c_char,
            flags: 0,
            cOps: 0 as *mut _,
            reserved: [0 as *mut _; 3],
        })
    }
}

/// Create a new namespace object.
pub unsafe fn create_namespace_object(
        cx: *mut JSContext,
        global: HandleObject,
        proto: HandleObject,
        class: &'static NamespaceObjectClass,
        methods: &[Guard<&'static [JSFunctionSpec]>],
        name: &[u8],
        rval: MutableHandleObject) {
    create_object(cx, proto, &class.0, methods, &[], &[], rval);
    define_on_global_object(cx, global, name, rval.handle());
}
