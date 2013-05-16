/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, rust_box, squirrel_away, str};
use dom::bindings::utils::{WrapperCache, DerivedWrapper};
use dom::bindings::utils::{jsval_to_str, WrapNewBindingObject, CacheableWrapper};
use dom::bindings::utils;
use dom::document::Document;
use dom::htmlcollection::HTMLCollection;
use js::glue::bindgen::*;
use js::glue::{PROPERTY_STUB, STRICT_PROPERTY_STUB};
use js::jsapi::bindgen::{JS_DefineProperties};
use js::jsapi::bindgen::{JS_GetReservedSlot, JS_SetReservedSlot, JS_DefineFunctions};
use js::jsapi::{JSContext, JSVal, JSObject, JSBool, JSFreeOp, JSPropertySpec, JSPropertyOpWrapper};
use js::jsapi::{JSStrictPropertyOpWrapper, JSNativeWrapper, JSFunctionSpec};
use js::rust::{Compartment, jsobj};
use js::{JSPROP_NATIVE_ACCESSORS};
use js::{JS_ARGV, JSPROP_ENUMERATE, JSPROP_SHARED, JSVAL_NULL, JS_THIS_OBJECT, JS_SET_RVAL};
use scripting::script_task::task_from_context;

use core::libc::c_uint;
use core::ptr::null;

extern fn getDocumentElement(cx: *JSContext, _argc: c_uint, vp: *mut JSVal) -> JSBool {
    unsafe {
        let obj = JS_THIS_OBJECT(cx, cast::transmute(vp));
        if obj.is_null() {
            return 0;
        }

        let doc = &mut (*unwrap(obj)).payload;
        let root = &mut doc.root;
        assert!(root.is_element());
        root.wrap(cx, ptr::null(), vp); //XXXjdm proper scope at some point
        return 1;
    }
}

extern fn getElementsByTagName(cx: *JSContext, _argc: c_uint, vp: *JSVal) -> JSBool {
    unsafe {
        let obj = JS_THIS_OBJECT(cx, vp);

        let argv = JS_ARGV(cx, cast::transmute(vp));

        let arg0: DOMString;
        let strval = jsval_to_str(cx, (*argv.offset(0)));
        if strval.is_err() {
            return 0;
        }
        arg0 = str(strval.get());

        let doc = &mut (*unwrap(obj)).payload;
        let rval: Option<@mut HTMLCollection>;
        rval = doc.getElementsByTagName(arg0);
        if rval.is_none() {
            JS_SET_RVAL(cx, vp, JSVAL_NULL);
        } else {
            let cache = doc.get_wrappercache();
            let rval = rval.get() as @mut CacheableWrapper;
            assert!(WrapNewBindingObject(cx, cache.get_wrapper(),
                                         rval,
                                         cast::transmute(vp)));
        }
        return 1;
    }
}

unsafe fn unwrap(obj: *JSObject) -> *mut rust_box<Document> {
    //TODO: some kind of check if this is a Document object
    let val = JS_GetReservedSlot(obj, 0);
    RUST_JSVAL_TO_PRIVATE(val) as *mut rust_box<Document>
}

extern fn finalize(_fop: *JSFreeOp, obj: *JSObject) {
    debug!("document finalize!");
    unsafe {
        let val = JS_GetReservedSlot(obj, 0);
        let _doc: @Document = cast::transmute(RUST_JSVAL_TO_PRIVATE(val));
    }
}

pub fn init(compartment: @mut Compartment) {
    let obj = utils::define_empty_prototype(~"Document", None, compartment);

    let attrs = @~[
        JSPropertySpec {
         name: compartment.add_name(~"documentElement"),
         tinyid: 0,
         flags: (JSPROP_SHARED | JSPROP_ENUMERATE | JSPROP_NATIVE_ACCESSORS) as u8,
         getter: JSPropertyOpWrapper {op: getDocumentElement, info: null()},
         setter: JSStrictPropertyOpWrapper {op: null(), info: null()}},
        JSPropertySpec {
         name: null(),
         tinyid: 0,
         flags: (JSPROP_SHARED | JSPROP_ENUMERATE | JSPROP_NATIVE_ACCESSORS) as u8,
         getter: JSPropertyOpWrapper {op: null(), info: null()},
         setter: JSStrictPropertyOpWrapper {op: null(), info: null()}}];
    vec::push(&mut compartment.global_props, attrs);
    vec::as_imm_buf(*attrs, |specs, _len| {
        assert!(JS_DefineProperties(compartment.cx.ptr, obj.ptr, specs) == 1);
    });

    let methods = @~[JSFunctionSpec {name: compartment.add_name(~"getElementsByTagName"),
                                     call: JSNativeWrapper {op: getElementsByTagName, info: null()},
                                     nargs: 0,
                                     flags: 0,
                                     selfHostedName: null()},
                     JSFunctionSpec {name: null(),
                                     call: JSNativeWrapper {op: null(), info: null()},
                                     nargs: 0,
                                     flags: 0,
                                     selfHostedName: null()}];
    vec::as_imm_buf(*methods, |fns, _len| {
        JS_DefineFunctions(compartment.cx.ptr, obj.ptr, fns);
    });

    compartment.register_class(utils::instance_jsclass(~"DocumentInstance",
                                                       finalize,
                                                       ptr::null()));
}

pub fn create(compartment: @mut Compartment, doc: @mut Document) -> *JSObject {
    let instance : jsobj = result::unwrap(
        compartment.new_object_with_proto(~"DocumentInstance", ~"Document",
                                          compartment.global_obj.ptr));
    doc.wrapper.set_wrapper(instance.ptr);

    unsafe {
        let raw_ptr: *libc::c_void = cast::transmute(squirrel_away(doc));
        JS_SetReservedSlot(instance.ptr, 0, RUST_PRIVATE_TO_JSVAL(raw_ptr));
    }

    compartment.define_property(~"document", RUST_OBJECT_TO_JSVAL(instance.ptr),
                                GetJSClassHookStubPointer(PROPERTY_STUB) as *u8,
                                GetJSClassHookStubPointer(STRICT_PROPERTY_STUB) as *u8,
                                JSPROP_ENUMERATE);

    instance.ptr
}

impl CacheableWrapper for Document {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        unsafe { cast::transmute(&self.wrapper) }
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, _scope: *JSObject) -> *JSObject {
        let script_context = task_from_context(cx);
        unsafe {
            create((*script_context).js_compartment, self)
        }
    }
}

