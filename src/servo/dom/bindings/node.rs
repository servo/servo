import js::rust::{bare_compartment, methods, jsobj};
import js::{JS_ARGV, JSCLASS_HAS_RESERVED_SLOTS, JSPROP_ENUMERATE, JSPROP_SHARED, JSVAL_NULL,
            JS_THIS_OBJECT, JS_SET_RVAL};
import js::jsapi::{JSContext, jsval, JSObject, JSBool, jsid, JSClass, JSFreeOp, JSPropertySpec};
import js::jsapi::bindgen::{JS_ValueToString, JS_GetStringCharsZAndLength, JS_ReportError,
                            JS_GetReservedSlot, JS_SetReservedSlot, JS_NewStringCopyN,
                            JS_DefineFunctions, JS_DefineProperty, JS_GetContextPrivate};
import js::jsapi::bindgen::*;
import js::glue::bindgen::*;
import js::crust::{JS_PropertyStub, JS_StrictPropertyStub, JS_EnumerateStub, JS_ConvertStub};

import dom::base::{Node, Element, Text};
import utils::{rust_box, squirrel_away_unique, get_compartment, domstring_to_jsval, str};
import libc::c_uint;
import ptr::null;

fn init(compartment: bare_compartment) {
    let obj = utils::define_empty_prototype(~"Node", none, compartment);

    let attrs = @~[
        {name: compartment.add_name(~"firstChild"),
         tinyid: 0,
         flags: 0,
         getter: getFirstChild,
         setter: null()},

        {name: compartment.add_name(~"nextSibling"),
         tinyid: 0,
         flags: 0,
         getter: getNextSibling,
         setter: null()}];
    vec::push(compartment.global_props, attrs);
    vec::as_buf(*attrs, |specs, _len| {
        JS_DefineProperties(compartment.cx.ptr, obj.ptr, specs);
    });
}

fn create(cx: *JSContext, node: Node) -> jsobj unsafe {
    do node.read |nd| {
        match nd.kind {
            ~Element(ed) => {
              element::create(cx, node)
            }

            ~Text(s) => {
              fail ~"no text node bindings yet";
            }
        }
    }
}

unsafe fn unwrap(obj: *JSObject) -> *rust_box<Node> {
    let val = JS_GetReservedSlot(obj, 0);
    unsafe::reinterpret_cast(RUST_JSVAL_TO_PRIVATE(val))
}

extern fn getFirstChild(cx: *JSContext, obj: *JSObject, _id: jsid, rval: *mut jsval) -> JSBool {
    unsafe {
        (*unwrap(obj)).payload.read(|nd| {
            match nd.tree.first_child {
              some(n) => {
                let obj = create(cx, n).ptr;
                *rval = RUST_OBJECT_TO_JSVAL(obj);
              }
              none => {
                *rval = JSVAL_NULL;
              }
            }
        });
    }
    return 1;
}

extern fn getNextSibling(cx: *JSContext, obj: *JSObject, _id: jsid, rval: *mut jsval) -> JSBool {
    unsafe {
        (*unwrap(obj)).payload.read(|nd| {
            match nd.tree.next_sibling {
              some(n) => {
                let obj = create(cx, n).ptr;
                *rval = RUST_OBJECT_TO_JSVAL(obj);
              }
              none => {
                *rval = JSVAL_NULL;
              }
            }
        });
    }
    return 1;
}
