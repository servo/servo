use dom::bindings::utils::{rust_box, squirrel_away_unique, get_compartment, domstring_to_jsval};
use dom::bindings::utils::{str};
use dom::node::{AbstractNode, Node};
use super::element;
use super::utils;

use core::cast::transmute;
use core::libc::c_uint;
use core::ptr::null;
use js::glue::bindgen::*;
use js::jsapi::bindgen::*;
use js::jsapi::bindgen::{JS_DefineFunctions, JS_DefineProperty, JS_GetContextPrivate};
use js::jsapi::bindgen::{JS_GetReservedSlot, JS_SetReservedSlot, JS_NewStringCopyN};
use js::jsapi::bindgen::{JS_ValueToString, JS_GetStringCharsZAndLength, JS_ReportError};
use js::jsapi::{JSContext, JSVal, JSObject, JSBool, jsid, JSClass, JSFreeOp, JSPropertySpec};
use js::jsapi::{JSPropertyOpWrapper, JSStrictPropertyOpWrapper};
use js::jsval::{INT_TO_JSVAL, JSVAL_TO_PRIVATE};
use js::rust::{Compartment, jsobj};
use js::{JS_ARGV, JSCLASS_HAS_RESERVED_SLOTS, JSPROP_ENUMERATE, JSPROP_SHARED, JSVAL_NULL};
use js::{JS_THIS_OBJECT, JS_SET_RVAL, JSPROP_NATIVE_ACCESSORS};
use js;

pub fn init(compartment: @mut Compartment) {
/*
    let obj = utils::define_empty_prototype(~"Node", None, compartment);

    let attrs = @~[
        JSPropertySpec {
         name: compartment.add_name(~"firstChild"),
         tinyid: 0,
         flags: (JSPROP_SHARED | JSPROP_ENUMERATE | JSPROP_NATIVE_ACCESSORS) as u8,
         getter: JSPropertyOpWrapper {op: getFirstChild, info: null()},
         setter: JSStrictPropertyOpWrapper {op: null(), info: null()}},

        JSPropertySpec {
         name: compartment.add_name(~"nextSibling"),
         tinyid: 0,
         flags: (JSPROP_SHARED | JSPROP_ENUMERATE | JSPROP_NATIVE_ACCESSORS) as u8,
         getter: JSPropertyOpWrapper {op: getNextSibling, info: null()},
         setter: JSStrictPropertyOpWrapper {op: null(), info: null()}},

        JSPropertySpec {
         name: compartment.add_name(~"nodeType"),
         tinyid: 0,
         flags: (JSPROP_SHARED | JSPROP_ENUMERATE | JSPROP_NATIVE_ACCESSORS) as u8,
         getter: JSPropertyOpWrapper {op: getNodeType, info: null()},
         setter: JSStrictPropertyOpWrapper {op: null(), info: null()}},
        
        JSPropertySpec {
         name: null(),
         tinyid: 0,
         flags: (JSPROP_SHARED | JSPROP_ENUMERATE | JSPROP_NATIVE_ACCESSORS) as u8,
         getter: JSPropertyOpWrapper {op: null(), info: null()},
         setter: JSStrictPropertyOpWrapper {op: null(), info: null()}}];
    vec::push(&mut compartment.global_props, attrs);
    vec::as_imm_buf(*attrs, |specs, _len| {
        JS_DefineProperties(compartment.cx.ptr, obj.ptr, specs);
    });*/
}

#[allow(non_implicitly_copyable_typarams)]
pub fn create(cx: *JSContext, node: Node) -> jsobj {
    /*
    do scope.write(&node) |nd| {
        match nd.kind {
            ~Element(*) => element::create(cx, node, scope),
            ~Text(*) => fail!(~"no text node bindings yet"),
            ~Comment(*) => fail!(~"no comment node bindings yet"),
            ~Doctype(*) => fail!(~"no doctype node bindings yet")
        }
    }
    */
    unsafe {
        transmute(0)
    }
}

pub struct NodeBundle {
    node: AbstractNode,
}

pub fn NodeBundle(n: AbstractNode) -> NodeBundle {
    NodeBundle {
        node: n,
    }
}

pub unsafe fn unwrap(obj: *JSObject) -> *rust_box<NodeBundle> {
    let val = js::GetReservedSlot(obj, 0);
    cast::reinterpret_cast(&JSVAL_TO_PRIVATE(val))
}

#[allow(non_implicitly_copyable_typarams)]
extern fn getFirstChild(cx: *JSContext, _argc: c_uint, vp: *mut JSVal) -> JSBool {
    /*
    unsafe {
        let obj = JS_THIS_OBJECT(cx, cast::reinterpret_cast(&vp));
        if obj.is_null() {
            return 0;
        }

        let bundle = unwrap(obj);
        do (*bundle).payload.scope.write(&(*bundle).payload.node) |nd| {
            match nd.tree.first_child {
              Some(n) => {
                let obj = create(cx, n, (*bundle).payload.scope).ptr;
                *vp = RUST_OBJECT_TO_JSVAL(obj);
              }
              None => {
                *vp = JSVAL_NULL;
              }
            }
        };
    }*/
    return 1;
}

#[allow(non_implicitly_copyable_typarams)]
extern fn getNextSibling(cx: *JSContext, _argc: c_uint, vp: *mut JSVal) -> JSBool {
    /*
    unsafe {
        let obj = JS_THIS_OBJECT(cx, cast::reinterpret_cast(&vp));
        if obj.is_null() {
            return 0;
        }

        let bundle = unwrap(obj);
        do (*bundle).payload.scope.write(&(*bundle).payload.node) |nd| {
            match nd.tree.next_sibling {
              Some(n) => {
                let obj = create(cx, n, (*bundle).payload.scope).ptr;
                *vp = RUST_OBJECT_TO_JSVAL(obj);
              }
              None => {
                *vp = JSVAL_NULL;
              }
            }
        };
    }*/
    return 1;
}

impl NodeBundle {
    fn getNodeType() -> i32 {
        /*
        do self.node.read |nd| {
            match nd.kind {
              ~Element(*) => 1,
              ~Text(*)    => 3,
              ~Comment(*) => 8,
              ~Doctype(*) => 10
            }
        }*/
        0
    }
}

extern fn getNodeType(cx: *JSContext, _argc: c_uint, vp: *mut JSVal) -> JSBool {
    /*
    unsafe {
        let obj = JS_THIS_OBJECT(cx, cast::reinterpret_cast(&vp));
        if obj.is_null() {
            return 0;
        }

        let bundle = unwrap(obj);
        let nodeType = (*bundle).payload.getNodeType();
        *vp = INT_TO_JSVAL(nodeType);
    }*/
    return 1;
}
