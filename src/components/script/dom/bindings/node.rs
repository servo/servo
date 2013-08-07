/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::element;
use dom::bindings::text;
use dom::bindings::utils;
use dom::bindings::utils::{CacheableWrapper, WrapperCache, DerivedWrapper};
use dom::element::{HTMLElementTypeId};
use dom::element::{HTMLHeadElementTypeId, HTMLHtmlElementTypeId, HTMLAnchorElementTypeId};
use dom::element::{HTMLDivElementTypeId, HTMLImageElementTypeId, HTMLSpanElementTypeId};
use dom::element::{HTMLBodyElementTypeId, HTMLHRElementTypeId, HTMLIframeElementTypeId};
use dom::element::{HTMLBRElementTypeId, HTMLTitleElementTypeId};
use dom::element::{HTMLParagraphElementTypeId, HTMLScriptElementTypeId, HTMLMetaElementTypeId};
use dom::element::{HTMLOListElementTypeId, HTMLStyleElementTypeId, HTMLTableElementTypeId};
use dom::element::{HTMLTableRowElementTypeId, HTMLTableSectionElementTypeId};
use dom::element::{HTMLTextAreaElementTypeId, HTMLUListElementTypeId};
use dom::element::{HTMLHeadElement, HTMLHtmlElement, HTMLDivElement, HTMLSpanElement};
use dom::element::{HTMLParagraphElement};
use dom::htmlelement::HTMLElement;
use dom::htmlanchorelement::HTMLAnchorElement;
use dom::htmlbodyelement::HTMLBodyElement;
use dom::htmlhrelement::HTMLHRElement;
use dom::htmlbrelement::HTMLBRElement;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::htmlimageelement::HTMLImageElement;
use dom::htmlmetaelement::HTMLMetaElement;
use dom::htmlolistelement::HTMLOListElement;
use dom::htmlscriptelement::HTMLScriptElement;
use dom::htmlstyleelement::HTMLStyleElement;
use dom::htmltableelement::HTMLTableElement;
use dom::htmltablerowelement::HTMLTableRowElement;
use dom::htmltablesectionelement::HTMLTableSectionElement;
use dom::htmltextareaelement::HTMLTextAreaElement;
use dom::htmltitleelement::HTMLTitleElement;
use dom::htmlulistelement::HTMLUListElement;
use dom::node::{AbstractNode, Node, ElementNodeTypeId, TextNodeTypeId, CommentNodeTypeId};
use dom::node::{DoctypeNodeTypeId, ScriptView, Text};

use std::cast;
use std::libc::c_uint;
use std::ptr;
use std::ptr::null;
use js::jsapi::*;
use js::jsapi::{JSContext, JSVal, JSObject, JSBool, JSPropertySpec};
use js::jsapi::{JSPropertyOpWrapper, JSStrictPropertyOpWrapper};
use js::jsval::{INT_TO_JSVAL};
use js::rust::{Compartment};
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
    compartment.global_props.push(attrs);
    do attrs.as_imm_buf |specs, _len| {
        unsafe {
            JS_DefineProperties(compartment.cx.ptr, obj.ptr, specs);
        }
    }
}

macro_rules! generate_element(
    ($name: ident) => ({
        let node: @mut $name = unsafe { cast::transmute(node.raw_object()) };
        node.wrap_object_shared(cx, ptr::null())
    })
)

#[allow(non_implicitly_copyable_typarams)]
pub fn create(cx: *JSContext, node: &mut AbstractNode<ScriptView>) -> *JSObject {
    match node.type_id() {
        ElementNodeTypeId(HTMLElementTypeId) => generate_element!(HTMLElement),
        ElementNodeTypeId(HTMLAnchorElementTypeId) => generate_element!(HTMLAnchorElement),
        ElementNodeTypeId(HTMLBodyElementTypeId) => generate_element!(HTMLBodyElement),
        ElementNodeTypeId(HTMLBRElementTypeId) => generate_element!(HTMLBRElement),
        ElementNodeTypeId(HTMLDivElementTypeId) => generate_element!(HTMLDivElement),
        ElementNodeTypeId(HTMLHeadElementTypeId) => generate_element!(HTMLHeadElement),
        ElementNodeTypeId(HTMLHRElementTypeId) => generate_element!(HTMLHRElement),
        ElementNodeTypeId(HTMLHtmlElementTypeId) => generate_element!(HTMLHtmlElement),
        ElementNodeTypeId(HTMLIframeElementTypeId) => generate_element!(HTMLIFrameElement),
        ElementNodeTypeId(HTMLImageElementTypeId) => generate_element!(HTMLImageElement),
        ElementNodeTypeId(HTMLMetaElementTypeId) => generate_element!(HTMLMetaElement),
        ElementNodeTypeId(HTMLOListElementTypeId) => generate_element!(HTMLOListElement),
        ElementNodeTypeId(HTMLParagraphElementTypeId) => generate_element!(HTMLParagraphElement),
        ElementNodeTypeId(HTMLScriptElementTypeId) => generate_element!(HTMLScriptElement),
        ElementNodeTypeId(HTMLSpanElementTypeId) => generate_element!(HTMLSpanElement),
        ElementNodeTypeId(HTMLStyleElementTypeId) => generate_element!(HTMLStyleElement),
        ElementNodeTypeId(HTMLTableElementTypeId) => generate_element!(HTMLTableElement),
        ElementNodeTypeId(HTMLTableRowElementTypeId) => generate_element!(HTMLTableRowElement),
        ElementNodeTypeId(HTMLTableSectionElementTypeId) => generate_element!(HTMLTableSectionElement),
        ElementNodeTypeId(HTMLTextAreaElementTypeId) => generate_element!(HTMLTextAreaElement),
        ElementNodeTypeId(HTMLTitleElementTypeId) => generate_element!(HTMLTitleElement),
        ElementNodeTypeId(HTMLUListElementTypeId) => generate_element!(HTMLUListElement),
        ElementNodeTypeId(_) => element::create(cx, node).ptr,
        CommentNodeTypeId |
        DoctypeNodeTypeId => text::create(cx, node).ptr,
        TextNodeTypeId => {
            let node: @mut Text = unsafe { cast::transmute(node.raw_object()) };
            node.wrap_object_shared(cx, ptr::null())
        }
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
