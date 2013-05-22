/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::element;
use dom::bindings::text;
use dom::bindings::utils;
use dom::bindings::utils::{CacheableWrapper, WrapperCache, DerivedWrapper};
use dom::node::{AbstractNode, Node, ElementNodeTypeId, TextNodeTypeId, CommentNodeTypeId};
use dom::node::{DoctypeNodeTypeId, ScriptView};

use core::libc::c_uint;
use core::ptr::null;
use js::jsapi::bindgen::*;
use js::jsapi::{JSContext, JSVal, JSObject, JSBool, JSPropertySpec};
use js::jsapi::{JSPropertyOpWrapper, JSStrictPropertyOpWrapper};
use js::jsval::{INT_TO_JSVAL};
use js::rust::{Compartment, jsobj};
use js::{JSPROP_ENUMERATE, JSPROP_SHARED, JSVAL_NULL};
use js::{JS_THIS_OBJECT, JSPROP_NATIVE_ACCESSORS};
use servo_util::tree::TreeNodeRef;

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
pub fn create(cx: *JSContext, node: &mut AbstractNode<ScriptView>) -> jsobj {
    match node.type_id() {
        ElementNodeTypeId(_) => element::create(cx, node),
        TextNodeTypeId |
        CommentNodeTypeId |
        DoctypeNodeTypeId => text::create(cx, node),
     }
}

pub unsafe fn unwrap(obj: *JSObject) -> AbstractNode<ScriptView> {
    let raw = utils::unwrap::<*mut Node<ScriptView>>(obj);
    AbstractNode::from_raw(raw)
}

#[allow(non_implicitly_copyable_typarams)]
extern fn getFirstChild(cx: *JSContext, _argc: c_uint, vp: *mut JSVal) -> JSBool {
    unsafe {
        let obj = JS_THIS_OBJECT(cx, cast::transmute(vp));
        if obj.is_null() {
            return 0;
        }

        let node = unwrap(obj);
        let rval = do node.with_mut_base |base| {
            base.getFirstChild()
        };
        match rval {
            Some(n) => {
                n.wrap(cx, ptr::null(), vp); //XXXjdm pass a real scope
            }
            None => *vp = JSVAL_NULL
        };
    }
    return 1;
}

#[allow(non_implicitly_copyable_typarams)]
extern fn getNextSibling(cx: *JSContext, _argc: c_uint, vp: *mut JSVal) -> JSBool {
    unsafe {
        let obj = JS_THIS_OBJECT(cx, cast::transmute(vp));
        if obj.is_null() {
            return 0;
        }

        let node = unwrap(obj);
        let rval = do node.with_mut_base |base| {
            base.getNextSibling()
        };
        match rval {
            Some(n) => {
                n.wrap(cx, ptr::null(), vp); //XXXjdm pass a real scope
            }
            None => *vp = JSVAL_NULL
        };
    }
    return 1;
}

impl Node<ScriptView> {
    fn getNodeType(&self) -> i32 {
        match self.type_id {
            ElementNodeTypeId(_) => 1,
            TextNodeTypeId       => 3,
            CommentNodeTypeId    => 8,
            DoctypeNodeTypeId    => 10
        }
    }

    fn getNextSibling(&mut self) -> Option<&mut AbstractNode<ScriptView>> {
        match self.next_sibling {
            // transmute because the compiler can't deduce that the reference
            // is safe outside of with_mut_base blocks.
            Some(ref mut n) => Some(unsafe { cast::transmute(n) }),
            None => None
        }
    }

    fn getFirstChild(&mut self) -> Option<&mut AbstractNode<ScriptView>> {
        match self.first_child {
            // transmute because the compiler can't deduce that the reference
            // is safe outside of with_mut_base blocks.
            Some(ref mut n) => Some(unsafe { cast::transmute(n) }),
            None => None
        }
    }
 }

extern fn getNodeType(cx: *JSContext, _argc: c_uint, vp: *mut JSVal) -> JSBool {
    unsafe {
        let obj = JS_THIS_OBJECT(cx, cast::transmute(vp));
        if obj.is_null() {
            return 0;
        }

        let node = unwrap(obj);
        let rval = do node.with_base |base| {
            base.getNodeType()
        };
        *vp = INT_TO_JSVAL(rval);
    }
    return 1;
}

impl CacheableWrapper for AbstractNode<ScriptView> {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        do self.with_mut_base |base| {
            unsafe {
                cast::transmute(&base.wrapper)
            }
        }
    }

    fn wrap_object_shared(@mut self, _cx: *JSContext, _scope: *JSObject) -> *JSObject {
        fail!(~"need to implement wrapping");
    }
}
