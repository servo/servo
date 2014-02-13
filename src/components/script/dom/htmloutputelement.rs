/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLOutputElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLOutputElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};
use dom::validitystate::ValidityState;

pub struct HTMLOutputElement {
    htmlelement: HTMLElement
}

impl HTMLOutputElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLOutputElement {
        HTMLOutputElement {
            htmlelement: HTMLElement::new_inherited(HTMLOutputElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLOutputElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLOutputElementBinding::Wrap)
    }
}

impl HTMLOutputElement {
    pub fn GetForm(&self) -> Option<AbstractNode> {
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

    pub fn Validity(&self) -> @mut ValidityState {
        let global = self.htmlelement.element.node.owner_doc().document().window;
        ValidityState::new(global)
    }

    pub fn SetValidity(&mut self, _validity: @mut ValidityState) {
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
