/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLOutputElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLOutputElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLOutputElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::htmlformelement::HTMLFormElement;
use dom::node::{Node, ElementNodeTypeId, window_from_node};
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
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLOutputElement {
        HTMLOutputElement {
            htmlelement: HTMLElement::new_inherited(HTMLOutputElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLOutputElement> {
        let element = HTMLOutputElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLOutputElementBinding::Wrap)
    }
}

pub trait HTMLOutputElementMethods {
    fn GetForm(&self) -> Option<Temporary<HTMLFormElement>>;
    fn Name(&self) -> DOMString;
    fn SetName(&mut self, _name: DOMString) -> ErrorResult;
    fn Type(&self) -> DOMString;
    fn DefaultValue(&self) -> DOMString;
    fn SetDefaultValue(&mut self, _value: DOMString) -> ErrorResult;
    fn Value(&self) -> DOMString;
    fn SetValue(&mut self, _value: DOMString) -> ErrorResult;
    fn WillValidate(&self) -> bool;
    fn SetWillValidate(&mut self, _will_validate: bool);
    fn Validity(&self) -> Temporary<ValidityState>;
    fn ValidationMessage(&self) -> DOMString;
    fn SetValidationMessage(&mut self, _message: DOMString) -> ErrorResult;
    fn CheckValidity(&self) -> bool;
    fn SetCustomValidity(&mut self, _error: DOMString);
}

impl<'a> HTMLOutputElementMethods for JSRef<'a, HTMLOutputElement> {
    fn GetForm(&self) -> Option<Temporary<HTMLFormElement>> {
        None
    }

    fn Name(&self) -> DOMString {
        "".to_owned()
    }

    fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Type(&self) -> DOMString {
        "".to_owned()
    }

    fn DefaultValue(&self) -> DOMString {
        "".to_owned()
    }

    fn SetDefaultValue(&mut self, _value: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Value(&self) -> DOMString {
        "".to_owned()
    }

    fn SetValue(&mut self, _value: DOMString) -> ErrorResult {
        Ok(())
    }

    fn WillValidate(&self) -> bool {
        false
    }

    fn SetWillValidate(&mut self, _will_validate: bool) {
    }

    fn Validity(&self) -> Temporary<ValidityState> {
        let window = window_from_node(self).root();
        ValidityState::new(&*window)
    }

    fn ValidationMessage(&self) -> DOMString {
        "".to_owned()
    }

    fn SetValidationMessage(&mut self, _message: DOMString) -> ErrorResult {
        Ok(())
    }

    fn CheckValidity(&self) -> bool {
        true
    }

    fn SetCustomValidity(&mut self, _error: DOMString) {
    }
}
