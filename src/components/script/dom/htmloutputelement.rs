/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLOutputElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLOutputElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLOutputElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::htmlformelement::HTMLFormElement;
use dom::node::{Node, ElementNodeTypeId};
use dom::validitystate::ValidityState;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLOutputElement {
    pub htmlelement: HTMLElement
}

impl HTMLOutputElementDerived for EventTarget {
    fn is_htmloutputelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLOutputElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLOutputElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLOutputElement {
        HTMLOutputElement {
            htmlelement: HTMLElement::new_inherited(HTMLOutputElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLOutputElement> {
        let element = HTMLOutputElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLOutputElementBinding::Wrap)
    }
}

impl HTMLOutputElement {
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

    pub fn DefaultValue(&self) -> DOMString {
        ~""
    }

    pub fn SetDefaultValue(&mut self, _value: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Value(&self) -> DOMString {
        ~""
    }

    pub fn SetValue(&mut self, _value: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn WillValidate(&self) -> bool {
        false
    }

    pub fn SetWillValidate(&mut self, _will_validate: bool) {
    }

    pub fn Validity(&self) -> JS<ValidityState> {
        let doc = self.htmlelement.element.node.owner_doc();
        let doc = doc.get();
        ValidityState::new(&doc.window)
    }

    pub fn SetValidity(&mut self, _validity: JS<ValidityState>) {
    }

    pub fn ValidationMessage(&self) -> DOMString {
        ~""
    }

    pub fn SetValidationMessage(&mut self, _message: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn CheckValidity(&self) -> bool {
        true
    }

    pub fn SetCustomValidity(&mut self, _error: DOMString) {
    }
}
