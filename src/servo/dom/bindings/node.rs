use dom::bindings::utils::{rust_box, squirrel_away_unique, get_compartment};
use dom::bindings::utils::{str, domstring_to_jsval, CacheableWrapper, WrapperCache};
use dom::bindings::utils::{DOM_OBJECT_SLOT};
use dom::node::{AbstractNode, Node, ElementNodeTypeId, TextNodeTypeId, CommentNodeTypeId};
use dom::node::{DoctypeNodeTypeId};
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
    });
}

#[allow(non_implicitly_copyable_typarams)]
pub fn create(cx: *JSContext, node: &mut AbstractNode) -> jsobj {
    match node.type_id() {
        ElementNodeTypeId(_) => element::create(cx, node),
        TextNodeTypeId    => fail!(~"no text node bindings yet"),
        CommentNodeTypeId => fail!(~"no comment node bindings yet"),
        DoctypeNodeTypeId => fail!(~"no doctype node bindings yet")
     }
}

pub unsafe fn unwrap(obj: *JSObject) -> *AbstractNode {
    let val = js::GetReservedSlot(obj, DOM_OBJECT_SLOT as u64);
    cast::transmute(JSVAL_TO_PRIVATE(val))
}

#[allow(non_implicitly_copyable_typarams)]
extern fn getFirstChild(cx: *JSContext, _argc: c_uint, vp: *mut JSVal) -> JSBool {
    unsafe {
        let obj = JS_THIS_OBJECT(cx, cast::reinterpret_cast(&vp));
        if obj.is_null() {
            return 0;
        }

        let node = *unwrap(obj);
        let rval = do node.with_mut_node |node| {
            node.getFirstChild()
        };
        match rval {
            Some(n) => {
                let obj = create(cx, n).ptr;
                *vp = RUST_OBJECT_TO_JSVAL(obj)
            }
            None => *vp = JSVAL_NULL
        };
    }
    return 1;
}

#[allow(non_implicitly_copyable_typarams)]
extern fn getNextSibling(cx: *JSContext, _argc: c_uint, vp: *mut JSVal) -> JSBool {
    unsafe {
        let obj = JS_THIS_OBJECT(cx, cast::reinterpret_cast(&vp));
        if obj.is_null() {
            return 0;
        }

        let node = *unwrap(obj);
        let rval = do node.with_mut_node |node| {
            node.getNextSibling()
        };
        match rval {
            Some(n) => {
                let obj = create(cx, n).ptr;
                *vp = RUST_OBJECT_TO_JSVAL(obj)
            }
            None => *vp = JSVAL_NULL
        };
    }
    return 1;
}

impl Node {
    fn getNodeType(&self) -> i32 {
        match self.type_id {
            ElementNodeTypeId(_) => 1,
            TextNodeTypeId       => 3,
            CommentNodeTypeId    => 8,
            DoctypeNodeTypeId    => 10
        }
    }

    fn getNextSibling(&mut self) -> Option<&mut AbstractNode> {
        match self.next_sibling {
          // transmute because the compiler can't deduce that the reference
          // is safe outside of with_mut_node blocks.
          Some(ref mut n) => Some(unsafe { cast::transmute(n) }),
          None => None
        }
    }

    fn getFirstChild(&mut self) -> Option<&mut AbstractNode> {
        match self.first_child {
          // transmute because the compiler can't deduce that the reference
          // is safe outside of with_mut_node blocks.
          Some(ref mut n) => Some(unsafe { cast::transmute(n) }),
          None => None
        }
    }
 }

extern fn getNodeType(cx: *JSContext, _argc: c_uint, vp: *mut JSVal) -> JSBool {
    unsafe {
        let obj = JS_THIS_OBJECT(cx, cast::reinterpret_cast(&vp));
        if obj.is_null() {
            return 0;
        }

        let node = *unwrap(obj);
        let rval = do node.with_imm_node |node| {
            node.getNodeType()
        };
        *vp = INT_TO_JSVAL(rval);
    }
    return 1;
}

impl CacheableWrapper for AbstractNode {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        do self.with_mut_node |n| {
            unsafe { cast::transmute(&n.wrapper) }
        }
    }

    fn wrap_object_unique(~self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        fail!(~"need to implement wrapping");
    }

    fn wrap_object_shared(@self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        fail!(~"need to implement wrapping");
    }
}
