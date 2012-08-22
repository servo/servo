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
import node::unwrap;
import dom::base::{HTMLImageElement, HTMLScriptElement, HTMLHeadElement, HTMLDivElement,
                   UnknownElement};
import gfx::geometry::{au_to_px, px_to_au};

extern fn finalize(_fop: *JSFreeOp, obj: *JSObject) {
    #debug("element finalize!");
    unsafe {
        let val = JS_GetReservedSlot(obj, 0);
        let _node: ~Node = unsafe::reinterpret_cast(RUST_JSVAL_TO_PRIVATE(val));
    }
}

fn init(compartment: bare_compartment) {
    let obj = utils::define_empty_prototype(~"Element", some(~"Node"), compartment);
    let attrs = @~[
        {name: compartment.add_name(~"tagName"),
         tinyid: 0,
         flags: 0,
         getter: getTagName,
         setter: null()}];
    vec::push(compartment.global_props, attrs);
    vec::as_buf(*attrs, |specs, _len| {
        JS_DefineProperties(compartment.cx.ptr, obj.ptr, specs);
    });

    compartment.register_class(utils::instance_jsclass(~"GenericElementInstance",
                                                       finalize));

    let _ = utils::define_empty_prototype(~"HTMLElement", some(~"Element"), compartment);
    let _ = utils::define_empty_prototype(~"HTMLDivElement", some(~"HTMLElement"), compartment);
    let _ = utils::define_empty_prototype(~"HTMLScriptElement", some(~"HTMLElement"), compartment);
    let _ = utils::define_empty_prototype(~"HTMLHeadElement", some(~"HTMLElement"), compartment);

    let obj = utils::define_empty_prototype(~"HTMLImageElement", some(~"HTMLElement"), compartment);
    let attrs = @~[
        {name: compartment.add_name(~"width"),
         tinyid: 0,
         flags: (JSPROP_SHARED | JSPROP_ENUMERATE) as u8,
         getter: HTMLImageElement_getWidth,
         setter: HTMLImageElement_setWidth}];
    vec::push(compartment.global_props, attrs);
    vec::as_buf(*attrs, |specs, _len| {
        JS_DefineProperties(compartment.cx.ptr, obj.ptr, specs);
    });
}

extern fn HTMLImageElement_getWidth(_cx: *JSContext, obj: *JSObject, _id: jsid,
                                    rval: *mut jsval) -> JSBool unsafe {
    let width = (*unwrap(obj)).payload.read(|nd| {
        match nd.kind {
          ~Element(ed) => {
            match ed.kind {
              ~HTMLImageElement(img) => img.size.width,
              _ => fail ~"why is this not an image element?"
            }
          }
          _ => fail ~"why is this not an element?"
        }
    });
    *rval = RUST_INT_TO_JSVAL(
              (au_to_px(width) & (i32::max_value as int)) as libc::c_int);
    return 1;
}

extern fn HTMLImageElement_setWidth(_cx: *JSContext, obj: *JSObject, _id: jsid,
                                    _strict: JSBool, vp: *jsval) -> JSBool unsafe {
    let width = (*unwrap(obj)).payload.read(|nd| {
        match nd.kind {
          ~Element(ed) => {
            match ed.kind {
              ~HTMLImageElement(img) =>
                img.size.width = px_to_au(RUST_JSVAL_TO_INT(*vp) as int),
              _ => fail ~"why is this not an image element?"
            }
          }
          _ => fail ~"why is this not an element?"
        }
    });
    return 1;
}

extern fn getTagName(cx: *JSContext, obj: *JSObject, _id: jsid, rval: *mut jsval)
    -> JSBool {
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

fn create(cx: *JSContext, node: Node) -> jsobj unsafe {
    let proto = node.read(|nd| {
        match nd.kind {
          ~Element(ed) => {
            match ed.kind {
              ~HTMLDivElement(*) => ~"HTMLDivElement",
              ~HTMLHeadElement(*) => ~"HTMLHeadElement",
              ~HTMLImageElement(*) => ~"HTMLImageElement",
              ~HTMLScriptElement(*) => ~"HTMLScriptElement",
              ~UnknownElement(*) => ~"HTMLElement"
            }
          }
          _ => fail ~"element::create only handles elements"
        }
    });   

    //XXXjdm the parent should probably be the node parent instead of the global
    //TODO error checking
    let compartment = utils::get_compartment(cx);
    let obj = result::unwrap(
        (*compartment).new_object_with_proto(~"GenericElementInstance", proto,
                                             (*compartment).global_obj.ptr));
 
    unsafe {
        let raw_ptr: *libc::c_void = unsafe::reinterpret_cast(squirrel_away_unique(~node));
        JS_SetReservedSlot(obj.ptr, 0, RUST_PRIVATE_TO_JSVAL(raw_ptr));
    }
    return obj;
}
