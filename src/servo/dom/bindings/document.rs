import js::rust::{compartment, bare_compartment, methods, jsobj};
import js::{JS_ARGV, JSCLASS_HAS_RESERVED_SLOTS, JSPROP_ENUMERATE, JSPROP_SHARED,
            JSVAL_NULL, JS_THIS_OBJECT, JS_SET_RVAL, JSPROP_NATIVE_ACCESSORS};
import js::jsapi::{JSContext, jsval, JSObject, JSBool, jsid, JSClass, JSFreeOp};
import js::jsapi::bindgen::{JS_ValueToString, JS_GetStringCharsZAndLength, JS_ReportError,
                            JS_GetReservedSlot, JS_SetReservedSlot, JS_NewStringCopyN,
                            JS_DefineFunctions, JS_DefineProperty, JS_DefineProperties};
import js::glue::bindgen::*;
import js::crust::{JS_PropertyStub, JS_StrictPropertyStub, JS_EnumerateStub, JS_ConvertStub, JS_ResolveStub};
import result::{result, ok, err};
import ptr::null;
import libc::c_uint;
import utils::{DOMString, domstring_to_jsval, rust_box, squirrel_away, str};
import bindings::node::create;
import base::Document;
import option::{some, none};

enum DOMException {
    INVALID_CHARACTER_ERR
}

enum Element = int;

/*extern fn getElementById(cx: *JSContext, argc: c_uint, vp: *jsval) -> JSBool {
    //XXX check if actually document object
    if argc != 1 {
        //XXX throw proper DOM exception
        str::as_c_str("Not enough arguments", |s| {
            JS_ReportError(cx, s);
        });
        return 0;
    }
    let id;
    unsafe {
        id = JS_ARGV(cx, vp)[0];
    }
    alt jsval_to_str(cx, id) {
      ok(s) {
        unsafe {
            let doc: *Document = unsafe::reinterpret_cast(JS_GetContextPrivate(cx));
            let elem = (*doc).getElementById(s);
        }
        //XXX wrap result
        return 1;
      }
      err(_) {
        str::as_c_str("???", |s| {
            JS_ReportError(cx, s);
        });
        return 0;
      }
    }
}*/

/*extern fn getDocumentURI(cx: *JSContext, _argc: c_uint, vp: *jsval) -> JSBool {
    unsafe {
        let uri = (*unwrap(JS_THIS_OBJECT(cx, vp))).payload.getDocumentURI();
        JS_SET_RVAL(cx, vp, domstring_to_jsval(cx, uri));
    }
    return 1;
}*/

extern fn getDocumentElement(cx: *JSContext, _argc: c_uint, vp: *mut jsval)
    -> JSBool unsafe {
    let obj = JS_THIS_OBJECT(cx, unsafe::reinterpret_cast(vp));
    if obj.is_null() {
        return 0;
    }

    let box = unwrap(obj);
    let node = (*box).payload.root;
    let scope = (*box).payload.scope;
    *vp = RUST_OBJECT_TO_JSVAL(node::create(cx, node, scope).ptr);
    return 1;
}

unsafe fn unwrap(obj: *JSObject) -> *rust_box<Document> {
    //TODO: some kind of check if this is a Document object
    let val = JS_GetReservedSlot(obj, 0);
    unsafe::reinterpret_cast(RUST_JSVAL_TO_PRIVATE(val))
}

extern fn finalize(_fop: *JSFreeOp, obj: *JSObject) {
    #debug("document finalize!");
    unsafe {
        let val = JS_GetReservedSlot(obj, 0);
        let _doc: @Document = unsafe::reinterpret_cast(RUST_JSVAL_TO_PRIVATE(val));
    }
}

fn init(compartment: bare_compartment, doc: @Document) {
    let obj = utils::define_empty_prototype(~"Document", none, compartment);

    let attrs = @~[
        {name: compartment.add_name(~"documentElement"),
         tinyid: 0,
         flags: (JSPROP_SHARED | JSPROP_ENUMERATE | JSPROP_NATIVE_ACCESSORS) as u8,
         getter: {op: getDocumentElement, info: null()},
         setter: {op: null(), info: null()}}];
    vec::push(compartment.global_props, attrs);
    vec::as_buf(*attrs, |specs, _len| {
        assert JS_DefineProperties(compartment.cx.ptr, obj.ptr, specs) == 1;
    });

    compartment.register_class(utils::instance_jsclass(~"DocumentInstance", finalize));

    let instance = result::unwrap(
        compartment.new_object_with_proto(~"DocumentInstance", ~"Document",
                                          compartment.global_obj.ptr));

    unsafe {
        let raw_ptr: *libc::c_void = unsafe::reinterpret_cast(squirrel_away(doc));
        JS_SetReservedSlot(instance.ptr, 0, RUST_PRIVATE_TO_JSVAL(raw_ptr));
    }

    compartment.define_property(~"document", RUST_OBJECT_TO_JSVAL(instance.ptr),
                                JS_PropertyStub, JS_StrictPropertyStub,
                                JSPROP_ENUMERATE);
}
