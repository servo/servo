/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLElementBinding;
use dom::bindings::codegen::InheritTypes::ElementCast;
use dom::bindings::codegen::InheritTypes::HTMLElementDerived;
use dom::bindings::js::JS;
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
    pub fn new_inherited(type_id: ElementTypeId, tag_name: DOMString, document: JS<Document>) -> HTMLElement {
        HTMLElement {
            element: Element::new_inherited(type_id, tag_name, namespace::HTML, None, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLElement> {
        let element = HTMLElement::new_inherited(HTMLElementTypeId, localName, document.clone());
        Node::reflect_node(~element, document, HTMLElementBinding::Wrap)
    }
}

impl HTMLElement {
    pub fn Title(&self) -> DOMString {
        ~""
    }

    pub fn SetTitle(&mut self, _title: DOMString) {
    }

    pub fn Lang(&self) -> DOMString {
        ~""
    }

    pub fn SetLang(&mut self, _lang: DOMString) {
    }

    pub fn Dir(&self) -> DOMString {
        ~""
    }

    pub fn SetDir(&mut self, _dir: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn GetItemValue(&self, _cx: *JSContext) -> Fallible<JSVal> {
        Ok(NullValue())
    }

    pub fn SetItemValue(&mut self, _cx: *JSContext, _val: JSVal) -> ErrorResult {
        Ok(())
    }

    pub fn Hidden(&self) -> bool {
        false
    }

    pub fn SetHidden(&mut self, _hidden: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Click(&self) {
    }

    pub fn TabIndex(&self) -> i32 {
        0
    }

    pub fn SetTabIndex(&mut self, _index: i32) -> ErrorResult {
        Ok(())
    }

    pub fn Focus(&self) -> ErrorResult {
        Ok(())
    }

    pub fn Blur(&self) -> ErrorResult {
        Ok(())
    }

    pub fn AccessKey(&self) -> DOMString {
        ~""
    }

    pub fn SetAccessKey(&self, _key: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn AccessKeyLabel(&self) -> DOMString {
        ~""
    }

    pub fn Draggable(&self) -> bool {
        false
    }

    pub fn SetDraggable(&mut self, _draggable: bool) -> ErrorResult {
        Ok(())
    }

    pub fn ContentEditable(&self) -> DOMString {
        ~""
    }

    pub fn SetContentEditable(&mut self, _val: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn IsContentEditable(&self) -> bool {
        false
    }

    pub fn Spellcheck(&self) -> bool {
        false
    }

    pub fn SetSpellcheck(&self, _val: bool) -> ErrorResult {
        Ok(())
    }

    pub fn GetOffsetParent(&self) -> Option<JS<Element>> {
        None
    }

    pub fn OffsetTop(&self) -> i32 {
        0
    }

    pub fn OffsetLeft(&self) -> i32 {
        0
    }

    pub fn OffsetWidth(&self) -> i32 {
        0
    }

    pub fn OffsetHeight(&self) -> i32 {
        0
    }
}

impl VirtualMethods for JS<HTMLElement> {
    fn super_type(&self) -> Option<~VirtualMethods:> {
        let element: JS<Element> = ElementCast::from(self);
        Some(~element as ~VirtualMethods:)
    }
}
