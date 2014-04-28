/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLFormElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLFormElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::{Element, HTMLFormElementTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlcollection::{HTMLCollection, Static};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLFormElement {
    pub htmlelement: HTMLElement
}

impl HTMLFormElementDerived for EventTarget {
    fn is_htmlformelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLFormElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLFormElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLFormElement {
        HTMLFormElement {
            htmlelement: HTMLElement::new_inherited(HTMLFormElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLFormElement> {
        let element = HTMLFormElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLFormElementBinding::Wrap)
    }
}

impl HTMLFormElement {
    pub fn AcceptCharset(&self) -> DOMString {
        ~""
    }

    pub fn SetAcceptCharset(&mut self, _accept_charset: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Action(&self) -> DOMString {
        ~""
    }

    pub fn SetAction(&mut self, _action: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Autocomplete(&self) -> DOMString {
        ~""
    }

    pub fn SetAutocomplete(&mut self, _autocomplete: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Enctype(&self) -> DOMString {
        ~""
    }

    pub fn SetEnctype(&mut self, _enctype: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Encoding(&self) -> DOMString {
        ~""
    }

    pub fn SetEncoding(&mut self, _encoding: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Method(&self) -> DOMString {
        ~""
    }

    pub fn SetMethod(&mut self, _method: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Name(&self) -> DOMString {
        ~""
    }

    pub fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn NoValidate(&self) -> bool {
        false
    }

    pub fn SetNoValidate(&mut self, _no_validate: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Target(&self) -> DOMString {
        ~""
    }

    pub fn SetTarget(&mut self, _target: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Elements(&self) -> JS<HTMLCollection> {
        // FIXME: https://github.com/mozilla/servo/issues/1844
        let doc = self.htmlelement.element.node.owner_doc();
        let doc = doc.get();
        HTMLCollection::new(&doc.window, Static(vec!()))
    }

    pub fn Length(&self) -> i32 {
        0
    }

    pub fn Submit(&self) -> ErrorResult {
        Ok(())
    }

    pub fn Reset(&self) {
    }

    pub fn CheckValidity(&self) -> bool {
        false
    }

    pub fn IndexedGetter(&self, _index: u32, _found: &mut bool) -> JS<Element> {
        fail!("Not implemented.")
    }
}
