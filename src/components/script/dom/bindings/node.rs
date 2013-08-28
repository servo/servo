/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::element;
use dom::bindings::text;
use dom::bindings::utils;
use dom::bindings::utils::{CacheableWrapper, WrapperCache, DerivedWrapper};
use dom::element::{HTMLElementTypeId,
                   HTMLAnchorElementTypeId, HTMLAppletElementTypeId,
                   HTMLAreaElementTypeId, HTMLBaseElementTypeId,
                   HTMLBodyElementTypeId, HTMLBRElementTypeId, HTMLButtonElementTypeId,
                   HTMLCanvasElementTypeId, HTMLDataElementTypeId, HTMLDataListElementTypeId,
                   HTMLDirectoryElementTypeId, HTMLDivElementTypeId, HTMLEmbedElementTypeId,
                   HTMLFieldSetElementTypeId, HTMLFontElementTypeId, HTMLFrameElementTypeId,
                   HTMLFrameSetElementTypeId, HTMLHeadElementTypeId, HTMLHeadingElementTypeId,
                   HTMLHRElementTypeId, HTMLHtmlElementTypeId, HTMLIframeElementTypeId,
                   HTMLImageElementTypeId, HTMLInputElementTypeId, HTMLLIElementTypeId,
                   HTMLLinkElementTypeId, HTMLMapElementTypeId, HTMLMetaElementTypeId,
                   HTMLOListElementTypeId, HTMLParagraphElementTypeId,
                   HTMLProgressElementTypeId, HTMLQuoteElementTypeId, HTMLScriptElementTypeId,
                   HTMLSpanElementTypeId, HTMLSourceElementTypeId,
                   HTMLStyleElementTypeId, HTMLTextAreaElementTypeId,
                   HTMLTableElementTypeId, HTMLTableCaptionElementTypeId, HTMLTableCellElementTypeId,
                   HTMLTableColElementTypeId,
                   HTMLTableRowElementTypeId, HTMLTableSectionElementTypeId, HTMLTimeElementTypeId,
                   HTMLTitleElementTypeId, HTMLUListElementTypeId, HTMLDListElementTypeId};
use dom::element::{HTMLHeadElement,HTMLHtmlElement, HTMLDivElement, HTMLParagraphElement, HTMLSpanElement};
use dom::htmlelement::HTMLElement;
use dom::htmlanchorelement::HTMLAnchorElement;
use dom::htmlappletelement::HTMLAppletElement;
use dom::htmlareaelement::HTMLAreaElement;
use dom::htmlbaseelement::HTMLBaseElement;
use dom::htmlbodyelement::HTMLBodyElement;
use dom::htmlbuttonelement::HTMLButtonElement;
use dom::htmlhrelement::HTMLHRElement;
use dom::htmlbrelement::HTMLBRElement;
use dom::htmlcanvaselement::HTMLCanvasElement;
use dom::htmldataelement::HTMLDataElement;
use dom::htmldatalistelement::HTMLDataListElement;
use dom::htmldirectoryelement::HTMLDirectoryElement;
use dom::htmldlistelement::HTMLDListElement;
use dom::htmlembedelement::HTMLEmbedElement;
use dom::htmlfieldsetelement::HTMLFieldSetElement;
use dom::htmlfontelement::HTMLFontElement;
use dom::htmlframeelement::HTMLFrameElement;
use dom::htmlframesetelement::HTMLFrameSetElement;
use dom::htmlheadingelement::HTMLHeadingElement;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::htmlimageelement::HTMLImageElement;
use dom::htmlinputelement::HTMLInputElement;
use dom::htmllielement::HTMLLIElement;
use dom::htmllinkelement::HTMLLinkElement;
use dom::htmlmapelement::HTMLMapElement;
use dom::htmlmetaelement::HTMLMetaElement;
use dom::htmlolistelement::HTMLOListElement;
use dom::htmlprogresselement::HTMLProgressElement;
use dom::htmlquoteelement::HTMLQuoteElement;
use dom::htmlscriptelement::HTMLScriptElement;
use dom::htmlsourceelement::HTMLSourceElement;
use dom::htmlstyleelement::HTMLStyleElement;
use dom::htmltableelement::HTMLTableElement;
use dom::htmltablecaptionelement::HTMLTableCaptionElement;
use dom::htmltablecellelement::HTMLTableCellElement;
use dom::htmltablecolelement::HTMLTableColElement;
use dom::htmltablerowelement::HTMLTableRowElement;
use dom::htmltablesectionelement::HTMLTableSectionElement;
use dom::htmltextareaelement::HTMLTextAreaElement;
use dom::htmltimeelement::HTMLTimeElement;
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

#[fixed_stack_segment]
pub fn init(compartment: @mut Compartment) {
    let obj = utils::define_empty_prototype(~"Node", None, compartment);

    let attrs = @~[
        JSPropertySpec {
         name: compartment.add_name(~"firstChild"),
         tinyid: 0,
         flags: (JSPROP_SHARED | JSPROP_ENUMERATE | JSPROP_NATIVE_ACCESSORS) as u8,
         getter: JSPropertyOpWrapper {op: Some(getFirstChild), info: null()},
         setter: JSStrictPropertyOpWrapper {op: None, info: null()}},

        JSPropertySpec {
         name: compartment.add_name(~"nextSibling"),
         tinyid: 0,
         flags: (JSPROP_SHARED | JSPROP_ENUMERATE | JSPROP_NATIVE_ACCESSORS) as u8,
         getter: JSPropertyOpWrapper {op: Some(getNextSibling), info: null()},
         setter: JSStrictPropertyOpWrapper {op: None, info: null()}},

        JSPropertySpec {
         name: compartment.add_name(~"nodeType"),
         tinyid: 0,
         flags: (JSPROP_SHARED | JSPROP_ENUMERATE | JSPROP_NATIVE_ACCESSORS) as u8,
         getter: JSPropertyOpWrapper {op: Some(getNodeType), info: null()},
         setter: JSStrictPropertyOpWrapper {op: None, info: null()}},
        
        JSPropertySpec {
         name: null(),
         tinyid: 0,
         flags: (JSPROP_SHARED | JSPROP_ENUMERATE | JSPROP_NATIVE_ACCESSORS) as u8,
         getter: JSPropertyOpWrapper {op: None, info: null()},
         setter: JSStrictPropertyOpWrapper {op: None, info: null()}}];
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

pub fn create(cx: *JSContext, node: &mut AbstractNode<ScriptView>) -> *JSObject {
    match node.type_id() {
        ElementNodeTypeId(HTMLElementTypeId) => generate_element!(HTMLElement),
        ElementNodeTypeId(HTMLAnchorElementTypeId) => generate_element!(HTMLAnchorElement),
        ElementNodeTypeId(HTMLAppletElementTypeId) => generate_element!(HTMLAppletElement),
        ElementNodeTypeId(HTMLAreaElementTypeId) => generate_element!(HTMLAreaElement),
        ElementNodeTypeId(HTMLBaseElementTypeId) => generate_element!(HTMLBaseElement),
        ElementNodeTypeId(HTMLBodyElementTypeId) => generate_element!(HTMLBodyElement),
        ElementNodeTypeId(HTMLBRElementTypeId) => generate_element!(HTMLBRElement),
        ElementNodeTypeId(HTMLButtonElementTypeId) => generate_element!(HTMLButtonElement),
        ElementNodeTypeId(HTMLCanvasElementTypeId) => generate_element!(HTMLCanvasElement),
        ElementNodeTypeId(HTMLDataElementTypeId) => generate_element!(HTMLDataElement),
        ElementNodeTypeId(HTMLDataListElementTypeId) => generate_element!(HTMLDataListElement),
        ElementNodeTypeId(HTMLDirectoryElementTypeId) => generate_element!(HTMLDirectoryElement),
        ElementNodeTypeId(HTMLDListElementTypeId) => generate_element!(HTMLDListElement),
        ElementNodeTypeId(HTMLDivElementTypeId) => generate_element!(HTMLDivElement),
        ElementNodeTypeId(HTMLEmbedElementTypeId) => generate_element!(HTMLEmbedElement),
        ElementNodeTypeId(HTMLFieldSetElementTypeId) => generate_element!(HTMLFieldSetElement),
        ElementNodeTypeId(HTMLFontElementTypeId) => generate_element!(HTMLFontElement),
        ElementNodeTypeId(HTMLFrameElementTypeId) => generate_element!(HTMLFrameElement),
        ElementNodeTypeId(HTMLFrameSetElementTypeId) => generate_element!(HTMLFrameSetElement),
        ElementNodeTypeId(HTMLHeadElementTypeId) => generate_element!(HTMLHeadElement),
        ElementNodeTypeId(HTMLHeadingElementTypeId) => generate_element!(HTMLHeadingElement),
        ElementNodeTypeId(HTMLHRElementTypeId) => generate_element!(HTMLHRElement),
        ElementNodeTypeId(HTMLHtmlElementTypeId) => generate_element!(HTMLHtmlElement),
        ElementNodeTypeId(HTMLIframeElementTypeId) => generate_element!(HTMLIFrameElement),
        ElementNodeTypeId(HTMLImageElementTypeId) => generate_element!(HTMLImageElement),
        ElementNodeTypeId(HTMLInputElementTypeId) => generate_element!(HTMLInputElement),
        ElementNodeTypeId(HTMLLIElementTypeId) => generate_element!(HTMLLIElement),
        ElementNodeTypeId(HTMLLinkElementTypeId) => generate_element!(HTMLLinkElement),
        ElementNodeTypeId(HTMLMapElementTypeId) => generate_element!(HTMLMapElement),
        ElementNodeTypeId(HTMLMetaElementTypeId) => generate_element!(HTMLMetaElement),
        ElementNodeTypeId(HTMLOListElementTypeId) => generate_element!(HTMLOListElement),
        ElementNodeTypeId(HTMLParagraphElementTypeId) => generate_element!(HTMLParagraphElement),
        ElementNodeTypeId(HTMLProgressElementTypeId) => generate_element!(HTMLProgressElement),
        ElementNodeTypeId(HTMLQuoteElementTypeId) => generate_element!(HTMLQuoteElement),
        ElementNodeTypeId(HTMLScriptElementTypeId) => generate_element!(HTMLScriptElement),
        ElementNodeTypeId(HTMLSourceElementTypeId) => generate_element!(HTMLSourceElement),
        ElementNodeTypeId(HTMLSpanElementTypeId) => generate_element!(HTMLSpanElement),
        ElementNodeTypeId(HTMLStyleElementTypeId) => generate_element!(HTMLStyleElement),
        ElementNodeTypeId(HTMLTableElementTypeId) => generate_element!(HTMLTableElement),
        ElementNodeTypeId(HTMLTableCellElementTypeId) => generate_element!(HTMLTableCellElement),
        ElementNodeTypeId(HTMLTableCaptionElementTypeId) => generate_element!(HTMLTableCaptionElement),
        ElementNodeTypeId(HTMLTableColElementTypeId) => generate_element!(HTMLTableColElement),
        ElementNodeTypeId(HTMLTableRowElementTypeId) => generate_element!(HTMLTableRowElement),
        ElementNodeTypeId(HTMLTableSectionElementTypeId) => generate_element!(HTMLTableSectionElement),
        ElementNodeTypeId(HTMLTextAreaElementTypeId) => generate_element!(HTMLTextAreaElement),
        ElementNodeTypeId(HTMLTimeElementTypeId) => generate_element!(HTMLTimeElement),
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
