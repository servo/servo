use std::ptr;

use js::jsapi::{
    GCContext, Handle, JSAtomState, JSCLASS_RESERVED_SLOTS_SHIFT, JSClass, JSClassOps, JSContext,
    JSObject, JSTracer, MutableHandleIdVector, PropertyKey,
};
use js::{JSCLASS_IS_DOMJSCLASS, JSCLASS_RESERVED_SLOTS_MASK};

use crate::lock::ThreadUnsafeOnceLock;
use crate::utils::{DOMClass, DOMJSClass};

pub(crate) struct InitClassConfig {
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

pub(crate) fn init_class_ops(
    class_ops: &ThreadUnsafeOnceLock<JSClassOps>,
    config: InitClassConfig,
) {
    class_ops.set(JSClassOps {
        addProperty: None,
        delProperty: None,
        enumerate: None,
        newEnumerate: config.enumerate_hook,
        resolve: config.resolve_hook,
        mayResolve: config.may_resolve_hook,
        finalize: Some(config.finalize_hook),
        call: None,
        construct: None,
        trace: Some(config.trace_hook),
    })
}

pub(crate) struct DomJSClassConfig {
    pub(crate) name: *const i8,
    pub(crate) flags: u32,
    pub(crate) slots: u32,
    pub(crate) class: DOMClass,
}

pub(crate) fn init_domjs_class(
    js_class: &ThreadUnsafeOnceLock<JSClassOps>,
    js_class_config: InitClassConfig,
    class: &ThreadUnsafeOnceLock<DOMJSClass>,
    domjs_class_config: DomJSClassConfig,
) {
    {
        init_class_ops(js_class, js_class_config);
        class.set(DOMJSClass {
        base: JSClass {
            name: domjs_class_config.name,
            flags: JSCLASS_IS_DOMJSCLASS | domjs_class_config.flags |
                   (((domjs_class_config.slots) & JSCLASS_RESERVED_SLOTS_MASK) << JSCLASS_RESERVED_SLOTS_SHIFT)
                   /* JSCLASS_HAS_RESERVED_SLOTS({args['slots']}) */,
            cOps: unsafe { js_class.get() },
            spec: ptr::null(),
            ext: ptr::null(),
            oOps: ptr::null(),
        },
        dom_class: domjs_class_config.class,
    });
    }
}
