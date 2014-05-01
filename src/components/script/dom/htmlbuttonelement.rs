/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLButtonElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLButtonElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLButtonElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::htmlformelement::HTMLFormElement;
use dom::node::{Node, ElementNodeTypeId};
use dom::validitystate::ValidityState;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLButtonElement {
    pub htmlelement: HTMLElement
}

impl HTMLButtonElementDerived for EventTarget {
    fn is_htmlbuttonelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLButtonElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLButtonElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLButtonElement {
        HTMLButtonElement {
            htmlelement: HTMLElement::new_inherited(HTMLButtonElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLButtonElement> {
        let element = HTMLButtonElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLButtonElementBinding::Wrap)
    }
}

impl HTMLButtonElement {
    pub fn Autofocus(&self) -> bool {
        false
    }

    pub fn SetAutofocus(&mut self, _autofocus: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Disabled(&self) -> bool {
        false
    }

    pub fn SetDisabled(&mut self, _disabled: bool) -> ErrorResult {
        Ok(())
    }

    pub fn GetForm(&self) -> Option<JS<HTMLFormElement>> {
        None
    }

    pub fn FormAction(&self) -> DOMString {
        ~""
    }

    pub fn SetFormAction(&mut self, _formaction: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn FormEnctype(&self) -> DOMString {
        ~""
    }

    pub fn SetFormEnctype(&mut self, _formenctype: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn FormMethod(&self) -> DOMString {
        ~""
    }

    pub fn SetFormMethod(&mut self, _formmethod: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn FormNoValidate(&self) -> bool {
        false
    }

    pub fn SetFormNoValidate(&mut self, _novalidate: bool) -> ErrorResult {
        Ok(())
    }

    pub fn FormTarget(&self) -> DOMString {
        ~""
    }

    pub fn SetFormTarget(&mut self, _formtarget: DOMString) -> ErrorResult {
        Ok(())
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

    pub fn SetType(&mut self, _type: DOMString) -> ErrorResult {
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
