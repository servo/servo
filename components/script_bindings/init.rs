/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr;

use js::jsapi::{
    GCContext, Handle, JSAtomState, JSCLASS_RESERVED_SLOTS_SHIFT, JSClass, JSClassOps, JSContext,
    JSObject, JSTracer, MutableHandleIdVector, PropertyKey,
};
use js::{JSCLASS_IS_DOMJSCLASS, JSCLASS_RESERVED_SLOTS_MASK};

use crate::lock::ThreadUnsafeOnceLock;
use crate::utils::{DOMClass, DOMJSClass};

pub(crate) struct InitClassOpsConfig {
    pub(crate) enumerate_hook: Option<
        unsafe extern "C" fn(
            *mut JSContext,
            Handle<*mut JSObject>,
            MutableHandleIdVector,
            bool,
        ) -> bool,
    >,
    pub(crate) resolve_hook: Option<
        unsafe extern "C" fn(
            *mut JSContext,
            Handle<*mut JSObject>,
            Handle<PropertyKey>,
            *mut bool,
        ) -> bool,
    >,
    pub(crate) may_resolve_hook:
        Option<unsafe extern "C" fn(*const JSAtomState, PropertyKey, *mut JSObject) -> bool>,
    pub(crate) finalize_hook: unsafe extern "C" fn(*mut GCContext, *mut JSObject),
    pub(crate) trace_hook: unsafe extern "C" fn(*mut JSTracer, *mut JSObject),
}

pub(crate) struct DomJSClassConfig {
    pub(crate) name: *const i8,
    pub(crate) flags: u32,
    pub(crate) slots: u32,
    pub(crate) class: DOMClass,
}

pub(crate) fn init_domjs_class(
    js_class: &ThreadUnsafeOnceLock<JSClassOps>,
    js_class_config: InitClassOpsConfig,
    class: &ThreadUnsafeOnceLock<DOMJSClass>,
    domjs_class_config: DomJSClassConfig,
) {
    js_class.set(JSClassOps {
        addProperty: None,
        delProperty: None,
        enumerate: None,
        newEnumerate: js_class_config.enumerate_hook,
        resolve: js_class_config.resolve_hook,
        mayResolve: js_class_config.may_resolve_hook,
        finalize: Some(js_class_config.finalize_hook),
        call: None,
        construct: None,
        trace: Some(js_class_config.trace_hook),
    });

    class.set(DOMJSClass {
        base: JSClass {
            name: domjs_class_config.name,
            flags: JSCLASS_IS_DOMJSCLASS |
                domjs_class_config.flags |
                (((domjs_class_config.slots) & JSCLASS_RESERVED_SLOTS_MASK) <<
                    JSCLASS_RESERVED_SLOTS_SHIFT), /* JSCLASS_HAS_RESERVED_SLOTS({args['slots']}) */
            cOps: unsafe { js_class.get() },
            spec: ptr::null(),
            ext: ptr::null(),
            oOps: ptr::null(),
        },
        dom_class: domjs_class_config.class,
    });
}
