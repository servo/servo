/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLSelectElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLSelectElementDerived;
use dom::bindings::codegen::UnionTypes::{HTMLElementOrLong, HTMLOptionElementOrHTMLOptGroupElement};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::{Element, HTMLSelectElementTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::htmlformelement::HTMLFormElement;
use dom::node::{Node, ElementNodeTypeId, window_from_node};
use dom::htmloptionelement::HTMLOptionElement;
use dom::validitystate::ValidityState;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLSelectElement {
    pub htmlelement: HTMLElement
}

impl HTMLSelectElementDerived for EventTarget {
    fn is_htmlselectelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLSelectElementTypeId))
    }
}

impl HTMLSelectElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLSelectElement {
        HTMLSelectElement {
            htmlelement: HTMLElement::new_inherited(HTMLSelectElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLSelectElement> {
        let element = HTMLSelectElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLSelectElementBinding::Wrap)
    }
}

pub trait HTMLSelectElementMethods {
    fn Autofocus(&self) -> bool;
    fn SetAutofocus(&mut self, _autofocus: bool) -> ErrorResult;
    fn Disabled(&self) -> bool;
    fn SetDisabled(&mut self, _disabled: bool) -> ErrorResult;
    fn GetForm(&self) -> Option<Temporary<HTMLFormElement>>;
    fn Multiple(&self) -> bool;
    fn SetMultiple(&mut self, _multiple: bool) -> ErrorResult;
    fn Name(&self) -> DOMString;
    fn SetName(&mut self, _name: DOMString) -> ErrorResult;
    fn Required(&self) -> bool;
    fn SetRequired(&mut self, _multiple: bool) -> ErrorResult;
    fn Size(&self) -> u32;
    fn SetSize(&mut self, _size: u32) -> ErrorResult;
    fn Type(&self) -> DOMString;
    fn Length(&self) -> u32;
    fn SetLength(&mut self, _length: u32) -> ErrorResult;
    fn Item(&self, _index: u32) -> Option<Temporary<Element>>;
    fn NamedItem(&self, _name: DOMString) -> Option<Temporary<HTMLOptionElement>>;
    fn IndexedGetter(&self, _index: u32, _found: &mut bool) -> Option<Temporary<Element>>;
    fn IndexedSetter(&mut self, _index: u32, _option: Option<JSRef<HTMLOptionElement>>) -> ErrorResult;
    fn Remove_(&self);
    fn Remove(&self, _index: i32);
    fn SelectedIndex(&self) -> i32;
    fn SetSelectedIndex(&mut self, _index: i32) -> ErrorResult;
    fn Value(&self) -> DOMString;
    fn SetValue(&mut self, _value: DOMString);
    fn WillValidate(&self) -> bool;
    fn SetWillValidate(&mut self, _will_validate: bool);
    fn Validity(&self) -> Temporary<ValidityState>;
    fn ValidationMessage(&self) -> DOMString;
    fn SetValidationMessage(&mut self, _message: DOMString) -> ErrorResult;
    fn CheckValidity(&self) -> bool;
    fn SetCustomValidity(&mut self, _error: DOMString);
    fn Add(&self, _element: HTMLOptionElementOrHTMLOptGroupElement, _before: Option<HTMLElementOrLong>) -> ErrorResult;
}

impl<'a> HTMLSelectElementMethods for JSRef<'a, HTMLSelectElement> {
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

    fn Multiple(&self) -> bool {
        false
    }

    fn SetMultiple(&mut self, _multiple: bool) -> ErrorResult {
        Ok(())
    }

    fn Name(&self) -> DOMString {
        "".to_owned()
    }

    fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Required(&self) -> bool {
        false
    }

    fn SetRequired(&mut self, _multiple: bool) -> ErrorResult {
        Ok(())
    }

    fn Size(&self) -> u32 {
        0
    }

    fn SetSize(&mut self, _size: u32) -> ErrorResult {
        Ok(())
    }

    fn Type(&self) -> DOMString {
        "".to_owned()
    }

    fn Length(&self) -> u32 {
        0
    }

    fn SetLength(&mut self, _length: u32) -> ErrorResult {
        Ok(())
    }

    fn Item(&self, _index: u32) -> Option<Temporary<Element>> {
        None
    }

    fn NamedItem(&self, _name: DOMString) -> Option<Temporary<HTMLOptionElement>> {
        None
    }

    fn IndexedGetter(&self, _index: u32, _found: &mut bool) -> Option<Temporary<Element>> {
        None
    }

    fn IndexedSetter(&mut self, _index: u32, _option: Option<JSRef<HTMLOptionElement>>) -> ErrorResult {
        Ok(())
    }

    fn Remove_(&self) {
    }

    fn Remove(&self, _index: i32) {
    }

    fn SelectedIndex(&self) -> i32 {
        0
    }

    fn SetSelectedIndex(&mut self, _index: i32) -> ErrorResult {
        Ok(())
    }

    fn Value(&self) -> DOMString {
        "".to_owned()
    }

    fn SetValue(&mut self, _value: DOMString) {
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

    fn Add(&self, _element: HTMLOptionElementOrHTMLOptGroupElement, _before: Option<HTMLElementOrLong>) -> ErrorResult {
        Ok(())
    }
}
