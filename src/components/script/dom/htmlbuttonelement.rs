/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLButtonElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLButtonElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLButtonElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::htmlformelement::HTMLFormElement;
use dom::node::{Node, ElementNodeTypeId, window_from_node};
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
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLButtonElement {
        HTMLButtonElement {
            htmlelement: HTMLElement::new_inherited(HTMLButtonElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLButtonElement> {
        let element = HTMLButtonElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLButtonElementBinding::Wrap)
    }
}

pub trait HTMLButtonElementMethods {
    fn Autofocus(&self) -> bool;
    fn SetAutofocus(&mut self, _autofocus: bool) -> ErrorResult;
    fn Disabled(&self) -> bool;
    fn SetDisabled(&mut self, _disabled: bool) -> ErrorResult;
    fn GetForm(&self) -> Option<Temporary<HTMLFormElement>>;
    fn FormAction(&self) -> DOMString;
    fn SetFormAction(&mut self, _formaction: DOMString) -> ErrorResult;
    fn FormEnctype(&self) -> DOMString;
    fn SetFormEnctype(&mut self, _formenctype: DOMString) -> ErrorResult;
    fn FormMethod(&self) -> DOMString;
    fn SetFormMethod(&mut self, _formmethod: DOMString) -> ErrorResult;
    fn FormNoValidate(&self) -> bool;
    fn SetFormNoValidate(&mut self, _novalidate: bool) -> ErrorResult;
    fn FormTarget(&self) -> DOMString;
    fn SetFormTarget(&mut self, _formtarget: DOMString) -> ErrorResult;
    fn Name(&self) -> DOMString;
    fn SetName(&mut self, _name: DOMString) -> ErrorResult;
    fn Type(&self) -> DOMString;
    fn SetType(&mut self, _type: DOMString) -> ErrorResult;
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

impl<'a> HTMLButtonElementMethods for JSRef<'a, HTMLButtonElement> {
    fn Autofocus(&self) -> bool {
        false
    }

    fn SetAutofocus(&mut self, _autofocus: bool) -> ErrorResult {
        Ok(())
    }

    fn Disabled(&self) -> bool {
        false
    }

    fn SetDisabled(&mut self, _disabled: bool) -> ErrorResult {
        Ok(())
    }

    fn GetForm(&self) -> Option<Temporary<HTMLFormElement>> {
        None
    }

    fn FormAction(&self) -> DOMString {
        "".to_owned()
    }

    fn SetFormAction(&mut self, _formaction: DOMString) -> ErrorResult {
        Ok(())
    }

    fn FormEnctype(&self) -> DOMString {
        "".to_owned()
    }

    fn SetFormEnctype(&mut self, _formenctype: DOMString) -> ErrorResult {
        Ok(())
    }

    fn FormMethod(&self) -> DOMString {
        "".to_owned()
    }

    fn SetFormMethod(&mut self, _formmethod: DOMString) -> ErrorResult {
        Ok(())
    }

    fn FormNoValidate(&self) -> bool {
        false
    }

    fn SetFormNoValidate(&mut self, _novalidate: bool) -> ErrorResult {
        Ok(())
    }

    fn FormTarget(&self) -> DOMString {
        "".to_owned()
    }

    fn SetFormTarget(&mut self, _formtarget: DOMString) -> ErrorResult {
        Ok(())
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

    fn SetType(&mut self, _type: DOMString) -> ErrorResult {
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
