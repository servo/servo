use js::rust::{Compartment, jsobj};
use js::{JS_ARGV, JSCLASS_HAS_RESERVED_SLOTS, JSPROP_ENUMERATE, JSPROP_SHARED,
            JSVAL_NULL, JS_THIS_OBJECT, JS_SET_RVAL, JSPROP_NATIVE_ACCESSORS};
use js::jsapi::{JSContext, JSVal, JSObject, JSBool, jsid, JSClass, JSFreeOp,
                JSPropertySpec, JSPropertyOpWrapper, JSStrictPropertyOpWrapper,
                JSNativeWrapper, JSFunctionSpec};
use js::jsapi::bindgen::{JS_ValueToString, JS_GetStringCharsZAndLength, JS_ReportError,
                            JS_GetReservedSlot, JS_SetReservedSlot, JS_NewStringCopyN,
                            JS_DefineFunctions, JS_DefineProperty, JS_DefineProperties};
use js::glue::bindgen::*;
use js::glue::{PROPERTY_STUB, STRICT_PROPERTY_STUB};
use js::crust::{JS_PropertyStub, JS_StrictPropertyStub, JS_EnumerateStub, JS_ConvertStub, JS_ResolveStub};
use core::ptr::null;
use core::libc::c_uint;
use dom::bindings::utils::{DOMString, domstring_to_jsval, rust_box, squirrel_away, str};
use dom::bindings::utils::{jsval_to_str, WrapNewBindingObject, CacheableWrapper};
use dom::bindings::utils::WrapperCache;
use dom::bindings::node::create;

use dom::document::Document;
use dom::bindings::htmlcollection::HTMLCollection;
use dom::bindings::node;
use dom::bindings::utils;
use dom::node::Node;

extern fn getDocumentElement(cx: *JSContext, _argc: c_uint, vp: *mut JSVal) -> JSBool {
    unsafe {
        let obj = JS_THIS_OBJECT(cx, cast::reinterpret_cast(&vp));
        if obj.is_null() {
            return 0;
        }

        let doc = &mut (*unwrap(obj)).payload;
        *vp = RUST_OBJECT_TO_JSVAL(node::create(cx, &mut doc.root).ptr);
        return 1;
    }
}

extern fn getElementsByTagName(cx: *JSContext, argc: c_uint, vp: *JSVal) -> JSBool {
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
        let rval: Option<~HTMLCollection>;
        rval = doc.getElementsByTagName(arg0);
        if rval.is_none() {
            JS_SET_RVAL(cx, vp, JSVAL_NULL);
        } else {
            let cache = doc.get_wrappercache();
            fail_unless!(WrapNewBindingObject(cx, cache.get_wrapper(),
                                              rval.get(),
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
        let _doc: @Document = cast::reinterpret_cast(&RUST_JSVAL_TO_PRIVATE(val));
    }
}

pub fn init(compartment: @mut Compartment, doc: @mut Document) {
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
        fail_unless!(JS_DefineProperties(compartment.cx.ptr, obj.ptr, specs) == 1);
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

    compartment.register_class(utils::instance_jsclass(~"DocumentInstance", finalize));

    let instance : jsobj = result::unwrap(
        compartment.new_object_with_proto(~"DocumentInstance", ~"Document",
                                          compartment.global_obj.ptr));
    doc.wrapper.set_wrapper(instance.ptr);

    unsafe {
        let raw_ptr: *libc::c_void = cast::reinterpret_cast(&squirrel_away(doc));
        JS_SetReservedSlot(instance.ptr, 0, RUST_PRIVATE_TO_JSVAL(raw_ptr));
    }

    compartment.define_property(~"document", RUST_OBJECT_TO_JSVAL(instance.ptr),
                                GetJSClassHookStubPointer(PROPERTY_STUB) as *u8,
                                GetJSClassHookStubPointer(STRICT_PROPERTY_STUB) as *u8,
                                JSPROP_ENUMERATE);
}

impl CacheableWrapper for Document {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        unsafe { cast::transmute(&self.wrapper) }
    }

    fn wrap_object_unique(~self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        fail!(~"need to implement wrapping");
    }

    fn wrap_object_shared(@self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        fail!(~"need to implement wrapping");
    }
}
