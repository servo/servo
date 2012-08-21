import js::rust::{bare_compartment, methods};
import js::{JS_ARGV, JSCLASS_HAS_RESERVED_SLOTS, JSPROP_ENUMERATE, JSPROP_SHARED, JSVAL_NULL, JS_THIS_OBJECT,
            JS_SET_RVAL};
import js::jsapi::{JSContext, jsval, JSObject, JSBool, jsid, JSClass, JSFreeOp};
import js::jsapi::bindgen::{JS_ValueToString, JS_GetStringCharsZAndLength, JS_ReportError,
                            JS_GetReservedSlot, JS_SetReservedSlot, JS_NewStringCopyN,
    JS_DefineFunctions, JS_DefineProperty, JS_DefineProperties, JS_EncodeString, JS_free};
import js::glue::bindgen::*;
import js::global::jsval_to_rust_str;
import js::crust::{JS_PropertyStub, JS_StrictPropertyStub, JS_EnumerateStub, JS_ConvertStub, JS_ResolveStub};
import result::{result, ok, err};
import ptr::null;
import libc::c_uint;
import utils::{DOMString, domstring_to_jsval, rust_box, squirrel_away, str,
    Document_class};
import bindings::node::create;
import base::{Node, Window};

enum DOMException {
    INVALID_CHARACTER_ERR
}

enum Element = int;

extern fn alert(cx: *JSContext, argc: c_uint, vp: *jsval) -> JSBool {
  unsafe {
    let argv = JS_ARGV(cx, vp);
    assert (argc == 1);
    // Abstract this pattern and use it in debug, too?
    let jsstr = JS_ValueToString(cx, *ptr::offset(argv, 0));
    // Right now, just print to the console
    io::println(#fmt("ALERT: %s", jsval_to_rust_str(cx, jsstr)));
    JS_SET_RVAL(cx, vp, JSVAL_NULL);
  }
  1_i32
}

// Unfortunately duplicated in document and window.
// Generalizing it triggers a trans bug
extern fn getDocumentElement(cx: *JSContext, obj: *JSObject,
      _id: jsid, rval: *mut jsval) -> JSBool unsafe {
  let node = (*unwrap(obj)).payload.root;
    *rval = RUST_OBJECT_TO_JSVAL(node::create(cx, node).ptr);
    return 1;
}

unsafe fn unwrap(obj: *JSObject) -> *rust_box<Window> {
    let val = JS_GetReservedSlot(obj, 0);
    unsafe::reinterpret_cast(RUST_JSVAL_TO_PRIVATE(val))
}

extern fn finalize(_fop: *JSFreeOp, obj: *JSObject) {
    #debug("finalize!");
    unsafe {
        let val = JS_GetReservedSlot(obj, 0);
        let _: @Window = unsafe::reinterpret_cast(RUST_JSVAL_TO_PRIVATE(val));
    }
}

fn init(compartment: bare_compartment, win: @Window) {

    compartment.register_class(|c| Document_class(c, ~"DOMWindow",
                                                  finalize));

    let obj = result::unwrap(
                 compartment.new_object(~"DOMWindow", null(), null()));

    /* Define methods on a window */
    let methods = ~[{name: compartment.add_name(~"alert"),
                     call: alert,
                     nargs: 1,
                     flags: 0}];
    
    vec::as_buf(methods, |fns, _len| {
        JS_DefineFunctions(compartment.cx.ptr, obj.ptr, fns);
      });

    let attrs = @~[
        {name: compartment.add_name(~"DOMWindow"),
         tinyid: 0, // ???
         flags: 0,
         getter: getDocumentElement,
         setter: null()}];
    vec::push(compartment.global_props, attrs);
        vec::as_buf(*attrs, |specs, _len| {
        JS_DefineProperties(compartment.cx.ptr, obj.ptr, specs);
    });

    unsafe {
        let raw_ptr: *libc::c_void = unsafe::reinterpret_cast(squirrel_away(win));
        JS_SetReservedSlot(obj.ptr, 0, RUST_PRIVATE_TO_JSVAL(raw_ptr));
    }

    compartment.define_property(~"window", RUST_OBJECT_TO_JSVAL(obj.ptr),
                                JS_PropertyStub, JS_StrictPropertyStub,
                                JSPROP_ENUMERATE);
}
