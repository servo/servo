/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLTextAreaElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTextAreaElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::{ErrorResult, Fallible};
use dom::document::Document;
use dom::element::HTMLTextAreaElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLTextAreaElement {
    pub htmlelement: HTMLElement,
}

impl HTMLTextAreaElementDerived for EventTarget {
    fn is_htmltextareaelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLTextAreaElementTypeId))
    }
}

impl HTMLTextAreaElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLTextAreaElement {
        HTMLTextAreaElement {
            htmlelement: HTMLElement::new_inherited(HTMLTextAreaElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLTextAreaElement> {
        let element = HTMLTextAreaElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLTextAreaElementBinding::Wrap)
    }
}

pub trait HTMLTextAreaElementMethods {
    fn Autofocus(&self) -> bool;
    fn SetAutofocus(&mut self, _autofocus: bool) -> ErrorResult;
    fn Cols(&self) -> u32;
    fn SetCols(&self, _cols: u32) -> ErrorResult;
    fn Disabled(&self) -> bool;
    fn SetDisabled(&mut self, _disabled: bool) -> ErrorResult;
    fn MaxLength(&self) -> i32;
    fn SetMaxLength(&self, _max_length: i32) -> ErrorResult;
    fn Name(&self) -> DOMString;
    fn SetName(&mut self, _name: DOMString) -> ErrorResult;
    fn Placeholder(&self) -> DOMString;
    fn SetPlaceholder(&mut self, _placeholder: DOMString) -> ErrorResult;
    fn ReadOnly(&self) -> bool;
    fn SetReadOnly(&mut self, _read_only: bool) -> ErrorResult;
    fn Required(&self) -> bool;
    fn SetRequired(&mut self, _required: bool) -> ErrorResult;
    fn Rows(&self) -> u32;
    fn SetRows(&self, _rows: u32) -> ErrorResult;
    fn Wrap(&self) -> DOMString;
    fn SetWrap(&mut self, _wrap: DOMString) -> ErrorResult;
    fn Type(&self) -> DOMString;
    fn SetType(&mut self, _type: DOMString);
    fn DefaultValue(&self) -> DOMString;
    fn SetDefaultValue(&mut self, _default_value: DOMString) -> ErrorResult;
    fn Value(&self) -> DOMString;
    fn SetValue(&mut self, _value: DOMString);
    fn TextLength(&self) -> u32;
    fn SetTextLength(&self, _text_length: u32) -> ErrorResult;
    fn WillValidate(&self) -> bool;
    fn SetWillValidate(&mut self, _will_validate: bool) -> ErrorResult;
    fn ValidationMessage(&self) -> DOMString;
    fn CheckValidity(&self) -> bool;
    fn SetCustomValidity(&self, _error: DOMString);
    fn Select(&self);
    fn GetSelectionStart(&self) -> Fallible<u32>;
    fn SetSelectionStart(&self, _selection_start: u32) -> ErrorResult;
    fn GetSelectionEnd(&self) -> Fallible<u32>;
    fn SetSelectionEnd(&self, _selection_end: u32) -> ErrorResult;
    fn GetSelectionDirection(&self) -> Fallible<DOMString>;
    fn SetSelectionDirection(&self, _selection_direction: DOMString) -> ErrorResult;
    fn SetRangeText(&self, _replacement: DOMString);
}

impl<'a> HTMLTextAreaElementMethods for JSRef<'a, HTMLTextAreaElement> {
    fn Autofocus(&self) -> bool {
        false
    }

    fn SetAutofocus(&mut self, _autofocus: bool) -> ErrorResult {
        Ok(())
    }

    fn Cols(&self) -> u32 {
        0
    }

    fn SetCols(&self, _cols: u32) -> ErrorResult {
        Ok(())
    }

    fn Disabled(&self) -> bool {
        false
    }

    fn SetDisabled(&mut self, _disabled: bool) -> ErrorResult {
        Ok(())
    }

    fn MaxLength(&self) -> i32 {
        0
    }

    fn SetMaxLength(&self, _max_length: i32) -> ErrorResult {
        Ok(())
    }

    fn Name(&self) -> DOMString {
        "".to_owned()
    }

    fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Placeholder(&self) -> DOMString {
        "".to_owned()
    }

    fn SetPlaceholder(&mut self, _placeholder: DOMString) -> ErrorResult {
        Ok(())
    }

    fn ReadOnly(&self) -> bool {
        false
    }

    fn SetReadOnly(&mut self, _read_only: bool) -> ErrorResult {
        Ok(())
    }

    fn Required(&self) -> bool {
        false
    }

    fn SetRequired(&mut self, _required: bool) -> ErrorResult {
        Ok(())
    }

    fn Rows(&self) -> u32 {
        0
    }

    fn SetRows(&self, _rows: u32) -> ErrorResult {
        Ok(())
    }

    fn Wrap(&self) -> DOMString {
        "".to_owned()
    }

    fn SetWrap(&mut self, _wrap: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Type(&self) -> DOMString {
        "".to_owned()
    }

    fn SetType(&mut self, _type: DOMString) {
    }

    fn DefaultValue(&self) -> DOMString {
        "".to_owned()
    }

    fn SetDefaultValue(&mut self, _default_value: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Value(&self) -> DOMString {
        "".to_owned()
    }

    fn SetValue(&mut self, _value: DOMString) {
    }

    fn TextLength(&self) -> u32 {
        0
    }

    fn SetTextLength(&self, _text_length: u32) -> ErrorResult {
        Ok(())
    }

    fn WillValidate(&self) -> bool {
        false
    }

    fn SetWillValidate(&mut self, _will_validate: bool) -> ErrorResult {
        Ok(())
    }

    fn ValidationMessage(&self) -> DOMString {
        "".to_owned()
    }

    fn CheckValidity(&self) -> bool {
        false
    }

    fn SetCustomValidity(&self, _error: DOMString) {
    }

    fn Select(&self) {
    }

    fn GetSelectionStart(&self) -> Fallible<u32> {
        Ok(0)
    }

    fn SetSelectionStart(&self, _selection_start: u32) -> ErrorResult {
        Ok(())
    }

    fn GetSelectionEnd(&self) -> Fallible<u32> {
        Ok(0)
    }

    fn SetSelectionEnd(&self, _selection_end: u32) -> ErrorResult {
        Ok(())
    }

    fn GetSelectionDirection(&self) -> Fallible<DOMString> {
        Ok("".to_owned())
    }

    fn SetSelectionDirection(&self, _selection_direction: DOMString) -> ErrorResult {
        Ok(())
    }

    fn SetRangeText(&self, _replacement: DOMString) {
    }
}
