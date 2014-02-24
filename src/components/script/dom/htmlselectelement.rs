/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLSelectElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLSelectElementDerived;
use dom::bindings::codegen::UnionTypes::{HTMLElementOrLong, HTMLOptionElementOrHTMLOptGroupElement};
use dom::bindings::js::JS;
use dom::bindings::utils::ErrorResult;
use dom::document::Document;
use dom::element::{Element, HTMLSelectElementTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::htmlformelement::HTMLFormElement;
use dom::node::{Node, ElementNodeTypeId};
use dom::htmloptionelement::HTMLOptionElement;
use dom::validitystate::ValidityState;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLSelectElement {
    htmlelement: HTMLElement
}

impl HTMLSelectElementDerived for EventTarget {
    fn is_htmlselectelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLSelectElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLSelectElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLSelectElement {
        HTMLSelectElement {
            htmlelement: HTMLElement::new_inherited(HTMLSelectElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLSelectElement> {
        let element = HTMLSelectElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLSelectElementBinding::Wrap)
    }
}

impl HTMLSelectElement {
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

    pub fn Multiple(&self) -> bool {
        false
    }

    pub fn SetMultiple(&mut self, _multiple: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Name(&self) -> DOMString {
        ~""
    }

    pub fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Required(&self) -> bool {
        false
    }

    pub fn SetRequired(&mut self, _multiple: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Size(&self) -> u32 {
        0
    }

    pub fn SetSize(&mut self, _size: u32) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> DOMString {
        ~""
    }

    pub fn Length(&self) -> u32 {
        0
    }

    pub fn SetLength(&mut self, _length: u32) -> ErrorResult {
        Ok(())
    }

    pub fn Item(&self, _index: u32) -> Option<JS<Element>> {
        None
    }

    pub fn NamedItem(&self, _name: DOMString) -> Option<JS<HTMLOptionElement>> {
        None
    }

    pub fn IndexedGetter(&self, _index: u32, _found: &mut bool) -> Option<JS<Element>> {
        None
    }

    pub fn IndexedSetter(&mut self, _index: u32, _option: Option<JS<HTMLOptionElement>>) -> ErrorResult {
        Ok(())
    }

    pub fn Remove_(&self) {
    }

    pub fn Remove(&self, _index: i32) {
    }

    pub fn SelectedIndex(&self) -> i32 {
        0
    }

    pub fn SetSelectedIndex(&mut self, _index: i32) -> ErrorResult {
        Ok(())
    }

    pub fn Value(&self) -> DOMString {
        ~""
    }

    pub fn SetValue(&mut self, _value: DOMString) {
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

    pub fn Add(&self, _element: HTMLOptionElementOrHTMLOptGroupElement, _before: Option<HTMLElementOrLong>) -> ErrorResult {
        Ok(())
    }
}
