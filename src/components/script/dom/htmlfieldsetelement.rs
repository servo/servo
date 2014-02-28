/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLFieldSetElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLFieldSetElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLFieldSetElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlformelement::HTMLFormElement;
use dom::htmlcollection::HTMLCollection;
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use dom::validitystate::ValidityState;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLFieldSetElement {
    htmlelement: HTMLElement
}

impl HTMLFieldSetElementDerived for EventTarget {
    fn is_htmlfieldsetelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLFieldSetElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLFieldSetElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLFieldSetElement {
        HTMLFieldSetElement {
            htmlelement: HTMLElement::new_inherited(HTMLFieldSetElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLFieldSetElement> {
        let element = HTMLFieldSetElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLFieldSetElementBinding::Wrap)
    }
}

impl HTMLFieldSetElement {
    pub fn Disabled(&self) -> bool {
        false
    }

    pub fn SetDisabled(&mut self, _disabled: bool) -> ErrorResult {
        Ok(())
    }

    pub fn GetForm(&self) -> Option<JS<HTMLFormElement>> {
        None
    }

    pub fn Name(&self) -> DOMString {
        ~""
    }

    pub fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> DOMString {
        ~""
    }

    pub fn Elements(&self) -> JS<HTMLCollection> {
        let doc = self.htmlelement.element.node.owner_doc();
        let doc = doc.get();
        HTMLCollection::new(&doc.window, ~[])
    }

    pub fn WillValidate(&self) -> bool {
        false
    }

    pub fn Validity(&self) -> JS<ValidityState> {
        let doc = self.htmlelement.element.node.owner_doc();
        let doc = doc.get();
        ValidityState::new(&doc.window)
    }

    pub fn ValidationMessage(&self) -> DOMString {
        ~""
    }

    pub fn CheckValidity(&self) -> bool {
        false
    }

    pub fn SetCustomValidity(&mut self, _error: DOMString) {
    }
}
