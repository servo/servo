/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLElementBinding;
use dom::bindings::codegen::InheritTypes::ElementCast;
use dom::bindings::codegen::InheritTypes::HTMLElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::{ErrorResult, Fallible};
use dom::document::Document;
use dom::element::{Element, ElementTypeId, HTMLElementTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::node::{Node, ElementNodeTypeId};
use dom::virtualmethods::VirtualMethods;
use js::jsapi::JSContext;
use js::jsval::{JSVal, NullValue};
use servo_util::namespace;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLElement {
    pub element: Element
}

impl HTMLElementDerived for EventTarget {
    fn is_htmlelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(ElementTypeId)) => false,
            NodeTargetTypeId(ElementNodeTypeId(_)) => true,
            _ => false
        }
    }
}

impl HTMLElement {
    pub fn new_inherited(type_id: ElementTypeId, tag_name: DOMString, document: &JSRef<Document>) -> HTMLElement {
        HTMLElement {
            element: Element::new_inherited(type_id, tag_name, namespace::HTML, None, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLElement> {
        let element = HTMLElement::new_inherited(HTMLElementTypeId, localName, document);
        Node::reflect_node(~element, document, HTMLElementBinding::Wrap)
    }
}

pub trait HTMLElementMethods {
    fn Title(&self) -> DOMString;
    fn SetTitle(&mut self, _title: DOMString);
    fn Lang(&self) -> DOMString;
    fn SetLang(&mut self, _lang: DOMString);
    fn Dir(&self) -> DOMString;
    fn SetDir(&mut self, _dir: DOMString) -> ErrorResult;
    fn GetItemValue(&self, _cx: *JSContext) -> Fallible<JSVal>;
    fn SetItemValue(&mut self, _cx: *JSContext, _val: JSVal) -> ErrorResult;
    fn Hidden(&self) -> bool;
    fn SetHidden(&mut self, _hidden: bool) -> ErrorResult;
    fn Click(&self);
    fn TabIndex(&self) -> i32;
    fn SetTabIndex(&mut self, _index: i32) -> ErrorResult;
    fn Focus(&self) -> ErrorResult;
    fn Blur(&self) -> ErrorResult;
    fn AccessKey(&self) -> DOMString;
    fn SetAccessKey(&self, _key: DOMString) -> ErrorResult;
    fn AccessKeyLabel(&self) -> DOMString;
    fn Draggable(&self) -> bool;
    fn SetDraggable(&mut self, _draggable: bool) -> ErrorResult;
    fn ContentEditable(&self) -> DOMString;
    fn SetContentEditable(&mut self, _val: DOMString) -> ErrorResult;
    fn IsContentEditable(&self) -> bool;
    fn Spellcheck(&self) -> bool;
    fn SetSpellcheck(&self, _val: bool) -> ErrorResult;
    fn GetOffsetParent(&self) -> Option<Temporary<Element>>;
    fn OffsetTop(&self) -> i32;
    fn OffsetLeft(&self) -> i32;
    fn OffsetWidth(&self) -> i32;
    fn OffsetHeight(&self) -> i32;
}

impl<'a> HTMLElementMethods for JSRef<'a, HTMLElement> {
    fn Title(&self) -> DOMString {
        ~""
    }

    fn SetTitle(&mut self, _title: DOMString) {
    }

    fn Lang(&self) -> DOMString {
        ~""
    }

    fn SetLang(&mut self, _lang: DOMString) {
    }

    fn Dir(&self) -> DOMString {
        ~""
    }

    fn SetDir(&mut self, _dir: DOMString) -> ErrorResult {
        Ok(())
    }

    fn GetItemValue(&self, _cx: *JSContext) -> Fallible<JSVal> {
        Ok(NullValue())
    }

    fn SetItemValue(&mut self, _cx: *JSContext, _val: JSVal) -> ErrorResult {
        Ok(())
    }

    fn Hidden(&self) -> bool {
        false
    }

    fn SetHidden(&mut self, _hidden: bool) -> ErrorResult {
        Ok(())
    }

    fn Click(&self) {
    }

    fn TabIndex(&self) -> i32 {
        0
    }

    fn SetTabIndex(&mut self, _index: i32) -> ErrorResult {
        Ok(())
    }

    fn Focus(&self) -> ErrorResult {
        Ok(())
    }

    fn Blur(&self) -> ErrorResult {
        Ok(())
    }

    fn AccessKey(&self) -> DOMString {
        ~""
    }

    fn SetAccessKey(&self, _key: DOMString) -> ErrorResult {
        Ok(())
    }

    fn AccessKeyLabel(&self) -> DOMString {
        ~""
    }

    fn Draggable(&self) -> bool {
        false
    }

    fn SetDraggable(&mut self, _draggable: bool) -> ErrorResult {
        Ok(())
    }

    fn ContentEditable(&self) -> DOMString {
        ~""
    }

    fn SetContentEditable(&mut self, _val: DOMString) -> ErrorResult {
        Ok(())
    }

    fn IsContentEditable(&self) -> bool {
        false
    }

    fn Spellcheck(&self) -> bool {
        false
    }

    fn SetSpellcheck(&self, _val: bool) -> ErrorResult {
        Ok(())
    }

    fn GetOffsetParent(&self) -> Option<Temporary<Element>> {
        None
    }

    fn OffsetTop(&self) -> i32 {
        0
    }

    fn OffsetLeft(&self) -> i32 {
        0
    }

    fn OffsetWidth(&self) -> i32 {
        0
    }

    fn OffsetHeight(&self) -> i32 {
        0
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLElement> {
    fn super_type<'a>(&'a mut self) -> Option<&'a mut VirtualMethods:> {
        let element: &mut JSRef<Element> = ElementCast::from_mut_ref(self);
        Some(element as &mut VirtualMethods:)
    }
}
