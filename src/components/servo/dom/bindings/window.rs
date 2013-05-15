/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// DOM bindings for the Window object.

use dom::bindings::utils::{rust_box, squirrel_away, CacheableWrapper};
use dom::bindings::utils::{WrapperCache};
use dom::window::Window;
use super::utils;

use core::libc::c_uint;
use core::ptr::null;
use core::ptr;
use js::crust::{JS_PropertyStub, JS_StrictPropertyStub};
use js::global::jsval_to_rust_str;
use js::glue::bindgen::*;
use js::glue::bindgen::RUST_JSVAL_TO_INT;
use js::jsapi::bindgen::{JS_DefineFunctions, JS_GC, JS_GetRuntime};
use js::jsapi::bindgen::{JS_GetReservedSlot, JS_SetReservedSlot};
use js::jsapi::bindgen::{JS_ValueToString};
use js::jsapi::{JSContext, JSVal, JSObject, JSBool, JSFreeOp, JSFunctionSpec};
use js::jsapi::{JSNativeWrapper};
use js::rust::Compartment;
use js::{JS_ARGV, JSPROP_ENUMERATE, JSVAL_NULL};
use js::{JS_THIS_OBJECT, JS_SET_RVAL};

extern fn alert(cx: *JSContext, argc: c_uint, vp: *JSVal) -> JSBool {
  unsafe {
    let argv = JS_ARGV(cx, vp);
    assert!(argc == 1);
    // Abstract this pattern and use it in debug, too?
    let jsstr = JS_ValueToString(cx, *ptr::offset(argv, 0));
    if jsstr.is_null() {
        return 0;
    }
    
    (*unwrap(JS_THIS_OBJECT(cx, vp))).payload.alert(jsval_to_rust_str(cx, jsstr));

    JS_SET_RVAL(cx, vp, JSVAL_NULL);
    return 1;
  }
}

extern fn setTimeout(cx: *JSContext, argc: c_uint, vp: *JSVal) -> JSBool {
    unsafe {
        let argv = JS_ARGV(cx, vp);
        assert!(argc >= 2);

        //TODO: don't crash when passed a non-integer value for the timeout

        (*unwrap(JS_THIS_OBJECT(cx, vp))).payload.setTimeout(
            RUST_JSVAL_TO_INT(*ptr::offset(argv, 1)) as int,
            argc, argv);

        JS_SET_RVAL(cx, vp, JSVAL_NULL);
        return 1;
    }
}

extern fn close(cx: *JSContext, _argc: c_uint, vp: *JSVal) -> JSBool {
    unsafe {
        (*unwrap(JS_THIS_OBJECT(cx, vp))).payload.close();
        JS_SET_RVAL(cx, vp, JSVAL_NULL);
        return 1;
    }
}

extern fn gc(cx: *JSContext, _argc: c_uint, _vp: *JSVal) -> JSBool {
    let runtime = JS_GetRuntime(cx);
    JS_GC(runtime);
    return 1;
}

unsafe fn unwrap(obj: *JSObject) -> *rust_box<Window> {
    let val = JS_GetReservedSlot(obj, 0);
    cast::transmute(RUST_JSVAL_TO_PRIVATE(val))
}

extern fn finalize(_fop: *JSFreeOp, obj: *JSObject) {
    debug!("finalize!");
    unsafe {
        let val = JS_GetReservedSlot(obj, 0);
        let _: @Window = cast::transmute(RUST_JSVAL_TO_PRIVATE(val));
    }
}

pub fn init(compartment: @mut Compartment) {
    let proto = utils::define_empty_prototype(~"Window", None, compartment);
    compartment.register_class(utils::instance_jsclass(~"WindowInstance", finalize, null()));

    /* Define methods on a window */
    let methods = [
        JSFunctionSpec {
            name: compartment.add_name(~"alert"),
            call: JSNativeWrapper { op: alert, info: null() },
            nargs: 1,
            flags: 0,
            selfHostedName: null()
        },
        JSFunctionSpec {
            name: compartment.add_name(~"setTimeout"),
            call: JSNativeWrapper { op: setTimeout, info: null() },
            nargs: 2,
            flags: 0,
            selfHostedName: null()
        },
        JSFunctionSpec {
            name: compartment.add_name(~"close"),
            call: JSNativeWrapper { op: close, info: null() },
            nargs: 0,
            flags: 0,
            selfHostedName: null()
        },
        JSFunctionSpec {
            name: compartment.add_name(~"_trigger_gc"),
            call: JSNativeWrapper { op: gc, info: null() },
            nargs: 0,
            flags: 0,
            selfHostedName: null()
        },
        JSFunctionSpec {
            name: null(),
            call: JSNativeWrapper { op: null(), info: null() },
            nargs: 0,
            flags: 0,
            selfHostedName: null()
        }
    ];

    JS_DefineFunctions(compartment.cx.ptr, proto.ptr, &methods[0]);
}

pub fn create(compartment: @mut Compartment, win: @mut Window) {
    let obj = result::unwrap(
                 compartment.new_object_with_proto(~"WindowInstance",
                                                   ~"Window", null()));

    win.get_wrappercache().set_wrapper(obj.ptr);

    unsafe {
        let raw_ptr: *libc::c_void = cast::transmute(squirrel_away(win));
        JS_SetReservedSlot(obj.ptr, 0, RUST_PRIVATE_TO_JSVAL(raw_ptr));
    }

    //TODO: All properties/methods on Window need to be available on the global
    //      object as well. We probably want a special JSClass with a resolve hook.
    compartment.define_property(~"window", RUST_OBJECT_TO_JSVAL(obj.ptr),
                                JS_PropertyStub, JS_StrictPropertyStub,
                                JSPROP_ENUMERATE);
}

impl CacheableWrapper for Window {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        unsafe { cast::transmute(&self.wrapper) }
    }

    fn wrap_object_shared(@mut self, _cx: *JSContext, _scope: *JSObject) -> *JSObject {
        fail!(~"should this be called?");
    }
}
