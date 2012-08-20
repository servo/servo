import js::rust::{bare_compartment, methods};
import js::{JS_ARGV, JSCLASS_HAS_RESERVED_SLOTS, JSPROP_ENUMERATE, JSPROP_SHARED, JSVAL_NULL, JS_THIS_OBJECT,
            JS_SET_RVAL};
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
import content::content_task::Document;

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

extern fn getDocumentElement(cx: *JSContext, obj: *JSObject, _id: jsid, rval: *mut jsval) -> JSBool unsafe {
    let node = (*unwrap(obj)).payload.root;
    *rval = RUST_OBJECT_TO_JSVAL(node::create(cx, node).ptr);
    return 1;
}

unsafe fn unwrap(obj: *JSObject) -> *rust_box<Document> {
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
    fn Document_class(compartment: bare_compartment) -> JSClass {
        {name: compartment.add_name(~"DOMDocument"),
         flags: JSCLASS_HAS_RESERVED_SLOTS(1),
         addProperty: JS_PropertyStub,
         delProperty: JS_PropertyStub,
         getProperty: JS_PropertyStub,
         setProperty: JS_StrictPropertyStub,
         enumerate: JS_EnumerateStub,
         resolve: JS_ResolveStub,
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
        compartment.new_object(Document_class, null(), null()));
    /*let methods = ~[
        {name: compartment.add_name("getDocumentURI"),
          call: getDocumentURI,
          nargs: 0,
          flags: 0}];
    vec::as_buf(methods, |fns| {
        JS_DefineFunctions(compartment.cx.ptr, obj.ptr, fns);
    });*/

    let attrs = @~[
        {name: compartment.add_name(~"documentElement"),
         tinyid: 0,
         flags: 0,
         getter: getDocumentElement,
         setter: null()}];
    vec::push(compartment.global_props, attrs);
        vec::as_buf(*attrs, |specs, _len| {
        JS_DefineProperties(compartment.cx.ptr, obj.ptr, specs);
    });

    unsafe {
        let raw_ptr: *libc::c_void = unsafe::reinterpret_cast(squirrel_away(doc));
        JS_SetReservedSlot(obj.ptr, 0, RUST_PRIVATE_TO_JSVAL(raw_ptr));
    }

    compartment.define_property(~"document", RUST_OBJECT_TO_JSVAL(obj.ptr),
                                JS_PropertyStub, JS_StrictPropertyStub,
                                JSPROP_ENUMERATE);
}
