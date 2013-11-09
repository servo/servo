/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLButtonElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLButtonElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node, ScriptView};
use dom::validitystate::ValidityState;

pub struct HTMLButtonElement {
    htmlelement: HTMLElement
}

impl HTMLButtonElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLButtonElement {
        HTMLButtonElement {
            htmlelement: HTMLElement::new_inherited(HTMLButtonElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode<ScriptView> {
        let element = HTMLButtonElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLButtonElementBinding::Wrap)
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

    pub fn GetForm(&self) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn FormAction(&self) -> Option<DOMString> {
        None
    }

    pub fn SetFormAction(&mut self, _formaction: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn FormEnctype(&self) -> Option<DOMString> {
        None
    }

    pub fn SetFormEnctype(&mut self, _formenctype: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn FormMethod(&self) -> Option<DOMString> {
        None
    }

    pub fn SetFormMethod(&mut self, _formmethod: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn FormNoValidate(&self) -> bool {
        false
    }

    pub fn SetFormNoValidate(&mut self, _novalidate: bool) -> ErrorResult {
        Ok(())
    }

    pub fn FormTarget(&self) -> Option<DOMString> {
        None
    }

    pub fn SetFormTarget(&mut self, _formtarget: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Name(&self) -> Option<DOMString> {
        None
    }

    pub fn SetName(&mut self, _name: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }
    
    pub fn Type(&self) -> Option<DOMString> {
        None
    }

    pub fn SetType(&mut self, _type: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Value(&self) -> Option<DOMString> {
        None
    }

    pub fn SetValue(&mut self, _value: &Option<DOMString>) -> ErrorResult {
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

    pub fn ValidationMessage(&self) -> Option<DOMString> {
        None
    }

    pub fn SetValidationMessage(&mut self, _message: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn CheckValidity(&self) -> bool {
        true
    }

    pub fn SetCustomValidity(&mut self, _error: &Option<DOMString>) {
    }
}
