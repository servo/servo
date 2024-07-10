/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Machinery to initialise namespace objects.

use std::ffi::CStr;
use std::ptr;

use js::jsapi::{JSClass, JSFunctionSpec};
use js::rust::{HandleObject, MutableHandleObject};

use super::constant::ConstantSpec;
use crate::dom::bindings::guard::Guard;
use crate::dom::bindings::interface::{create_object, define_on_global_object};
use crate::script_runtime::JSContext;

/// The class of a namespace object.
#[derive(Clone, Copy)]
pub struct NamespaceObjectClass(JSClass);

unsafe impl Sync for NamespaceObjectClass {}

impl NamespaceObjectClass {
    /// Create a new `NamespaceObjectClass` structure.
    pub const unsafe fn new(name: &'static CStr) -> Self {
        NamespaceObjectClass(JSClass {
            name: name.as_ptr(),
            flags: 0,
            cOps: 0 as *mut _,
            spec: ptr::null(),
            ext: ptr::null(),
            oOps: ptr::null(),
        })
    }
}

/// Create a new namespace object.
#[allow(clippy::too_many_arguments)]
pub fn create_namespace_object(
    cx: JSContext,
    global: HandleObject,
    proto: HandleObject,
    class: &'static NamespaceObjectClass,
    methods: &[Guard<&'static [JSFunctionSpec]>],
    constants: &[Guard<&'static [ConstantSpec]>],
    name: &CStr,
    rval: MutableHandleObject,
) {
    create_object(cx, global, proto, &class.0, methods, &[], constants, rval);
    define_on_global_object(cx, global, name, rval.handle());
}
