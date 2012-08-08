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

import dom::base::{Node, Element};
import utils::{rust_box, squirrel_away_unique, get_compartment, domstring_to_jsval, str};
import libc::c_uint;
import ptr::null;
import rcu::ReaderMethods;

extern fn finalize(_fop: *JSFreeOp, obj: *JSObject) {
    #debug("node finalize!");
    unsafe {
        let val = JS_GetReservedSlot(obj, 0);
        let _node: ~Node = unsafe::reinterpret_cast(RUST_JSVAL_TO_PRIVATE(val));
    }
}

fn create(cx: *JSContext, node: Node) -> jsobj unsafe {
    let compartment = get_compartment(cx);
    fn Node_class(compartment: bare_compartment) -> JSClass {
        {name: compartment.add_name(~"Node"),
         flags: JSCLASS_HAS_RESERVED_SLOTS(1),
         addProperty: JS_PropertyStub,
         delProperty: JS_PropertyStub,
         getProperty: JS_PropertyStub,
         setProperty: JS_StrictPropertyStub,
         enumerate: JS_EnumerateStub,
         resolve: JS_PropertyStub,
         convert: JS_ConvertStub,
         finalize: finalize,
         checkAccess: null(),
         call: null(),
         construct: null(),
         hasInstance: null(),
         trace: null(),
         reserved: (null(), null(), null(), null(), null(),  // 05
                    null(), null(), null(), null(), null(),  // 10
                    null(), null(), null(), null(), null(),  // 15
                    null(), null(), null(), null(), null(),  // 20
                    null(), null(), null(), null(), null(),  // 25
                    null(), null(), null(), null(), null(),  // 30
                    null(), null(), null(), null(), null(),  // 35
                    null(), null(), null(), null(), null())} // 40
    };

    let obj = result::unwrap(
        (*compartment).new_object(Node_class, null(),
                                  (*compartment).global_obj.ptr));
    let attrs = @~[
        {name: (*compartment).add_name(~"firstChild"),
         tinyid: 0,
         flags: 0,
         getter: getFirstChild,
         setter: null()},

        {name: (*compartment).add_name(~"nextSibling"),
         tinyid: 0,
         flags: 0,
         getter: getNextSibling,
         setter: null()},

        {name: (*compartment).add_name(~"tagName"),
         tinyid: 0,
         flags: 0,
         getter: getTagName,
         setter: null()}];
    vec::push((*compartment).global_props, attrs);
    vec::as_buf(*attrs, |specs, _len| {
        JS_DefineProperties((*compartment).cx.ptr, obj.ptr, specs);
    });

    unsafe {
        let raw_ptr: *libc::c_void = unsafe::reinterpret_cast(squirrel_away_unique(~node));
        JS_SetReservedSlot(obj.ptr, 0, RUST_PRIVATE_TO_JSVAL(raw_ptr));
    }

    return obj;
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

extern fn getTagName(cx: *JSContext, obj: *JSObject, _id: jsid, rval: *mut jsval) -> JSBool {
    unsafe {
        (*unwrap(obj)).payload.read(|nd| {
            match nd.kind {
              ~Element(ed) => {
                let s = str(copy ed.tag_name);
                *rval = domstring_to_jsval(cx, s);
              }
              _ => {
                //XXXjdm should probably read the spec to figure out what to do here
                *rval = JSVAL_NULL;
              }
            }
        });
    }
    return 1;
}
