// DOM bindings for the Window object.

use dom::bindings::node::create;
use dom::bindings::utils::{rust_box, squirrel_away, jsval_to_str, CacheableWrapper};
use dom::bindings::utils::{WrapperCache};
use dom::node::Node;
use dom::window::{Window, TimerMessage_Fire};
use super::utils;

use core::libc::c_uint;
use core::ptr::null;
use core::ptr;
use js::crust::{JS_PropertyStub, JS_StrictPropertyStub, JS_EnumerateStub, JS_ConvertStub};
use js::crust::{JS_ResolveStub};
use js::global::jsval_to_rust_str;
use js::glue::bindgen::*;
use js::glue::bindgen::RUST_JSVAL_TO_INT;
use js::jsapi::bindgen::{JS_DefineFunctions, JS_DefineProperty, JS_DefineProperties};
use js::jsapi::bindgen::{JS_EncodeString, JS_free};
use js::jsapi::bindgen::{JS_GetReservedSlot, JS_SetReservedSlot, JS_NewStringCopyN};
use js::jsapi::bindgen::{JS_ValueToString, JS_GetStringCharsZAndLength, JS_ReportError};
use js::jsapi::{JSContext, JSVal, JSObject, JSBool, jsid, JSClass, JSFreeOp, JSFunctionSpec};
use js::jsapi::{JSNativeWrapper};
use js::rust::Compartment;
use js::{JS_ARGV, JSCLASS_HAS_RESERVED_SLOTS, JSPROP_ENUMERATE, JSPROP_SHARED, JSVAL_NULL};
use js::{JS_THIS_OBJECT, JS_SET_RVAL};

extern fn alert(cx: *JSContext, argc: c_uint, vp: *JSVal) -> JSBool {
  unsafe {
    let argv = JS_ARGV(cx, vp);
    fail_unless!(argc == 1);
    // Abstract this pattern and use it in debug, too?
    let jsstr = JS_ValueToString(cx, *ptr::offset(argv, 0));
    
    (*unwrap(JS_THIS_OBJECT(cx, vp))).payload.alert(jsval_to_rust_str(cx, jsstr));

    JS_SET_RVAL(cx, vp, JSVAL_NULL);
  }
  1_i32
}

extern fn setTimeout(cx: *JSContext, argc: c_uint, vp: *JSVal) -> JSBool {
    unsafe {
        let argv = JS_ARGV(cx, vp);
        fail_unless!(argc >= 2);

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

unsafe fn unwrap(obj: *JSObject) -> *rust_box<Window> {
    let val = JS_GetReservedSlot(obj, 0);
    cast::reinterpret_cast(&RUST_JSVAL_TO_PRIVATE(val))
}

extern fn finalize(_fop: *JSFreeOp, obj: *JSObject) {
    debug!("finalize!");
    unsafe {
        let val = JS_GetReservedSlot(obj, 0);
        let _: @Window = cast::reinterpret_cast(&RUST_JSVAL_TO_PRIVATE(val));
    }
}

pub fn init(compartment: @mut Compartment, win: @mut Window) {
    let proto = utils::define_empty_prototype(~"Window", None, compartment);
    compartment.register_class(utils::instance_jsclass(~"WindowInstance", finalize));

    let obj = result::unwrap(
                 compartment.new_object_with_proto(~"WindowInstance",
                                                   ~"Window", null()));

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
            nargs: 2,
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

    win.get_wrappercache().set_wrapper(obj.ptr);

    unsafe {
        JS_DefineFunctions(compartment.cx.ptr, proto.ptr, &methods[0]);

        let raw_ptr: *libc::c_void = cast::reinterpret_cast(&squirrel_away(win));
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

    fn wrap_object_unique(~self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        fail!(~"should this be called?");
    }

    fn wrap_object_shared(@self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        fail!(~"should this be called?");
    }
}
