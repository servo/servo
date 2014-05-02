/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLTextAreaElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTextAreaElementDerived;
use dom::bindings::js::JS;
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
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLTextAreaElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLTextAreaElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLTextAreaElement {
        HTMLTextAreaElement {
            htmlelement: HTMLElement::new_inherited(HTMLTextAreaElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLTextAreaElement> {
        let element = HTMLTextAreaElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLTextAreaElementBinding::Wrap)
    }
}

impl HTMLTextAreaElement {
    pub fn Autofocus(&self) -> bool {
        false
    }

    pub fn SetAutofocus(&mut self, _autofocus: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Cols(&self) -> u32 {
        0
    }

    pub fn SetCols(&self, _cols: u32) -> ErrorResult {
        Ok(())
    }

    pub fn Disabled(&self) -> bool {
        false
    }

    pub fn SetDisabled(&mut self, _disabled: bool) -> ErrorResult {
        Ok(())
    }

    pub fn MaxLength(&self) -> i32 {
        0
    }

    pub fn SetMaxLength(&self, _max_length: i32) -> ErrorResult {
        Ok(())
    }

    pub fn Name(&self) -> DOMString {
        ~""
    }

    pub fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Placeholder(&self) -> DOMString {
        ~""
    }

    pub fn SetPlaceholder(&mut self, _placeholder: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn ReadOnly(&self) -> bool {
        false
    }

    pub fn SetReadOnly(&mut self, _read_only: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Required(&self) -> bool {
        false
    }

    pub fn SetRequired(&mut self, _required: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Rows(&self) -> u32 {
        0
    }

    pub fn SetRows(&self, _rows: u32) -> ErrorResult {
        Ok(())
    }

    pub fn Wrap(&self) -> DOMString {
        ~""
    }

    pub fn SetWrap(&mut self, _wrap: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> DOMString {
        ~""
    }

    pub fn SetType(&mut self, _type: DOMString) {
    }

    pub fn DefaultValue(&self) -> DOMString {
        ~""
    }

    pub fn SetDefaultValue(&mut self, _default_value: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Value(&self) -> DOMString {
        ~""
    }

    pub fn SetValue(&mut self, _value: DOMString) {
    }

    pub fn TextLength(&self) -> u32 {
        0
    }

    pub fn SetTextLength(&self, _text_length: u32) -> ErrorResult {
        Ok(())
    }

    pub fn WillValidate(&self) -> bool {
        false
    }

    pub fn SetWillValidate(&mut self, _will_validate: bool) -> ErrorResult {
        Ok(())
    }

    pub fn ValidationMessage(&self) -> DOMString {
        ~""
    }

    pub fn CheckValidity(&self) -> bool {
        false
    }

    pub fn SetCustomValidity(&self, _error: DOMString) {
    }

    pub fn Select(&self) {
    }

    pub fn GetSelectionStart(&self) -> Fallible<u32> {
        Ok(0)
    }

    pub fn SetSelectionStart(&self, _selection_start: u32) -> ErrorResult {
        Ok(())
    }

    pub fn GetSelectionEnd(&self) -> Fallible<u32> {
        Ok(0)
    }

    pub fn SetSelectionEnd(&self, _selection_end: u32) -> ErrorResult {
        Ok(())
    }

    pub fn GetSelectionDirection(&self) -> Fallible<DOMString> {
        Ok(~"")
    }

    pub fn SetSelectionDirection(&self, _selection_direction: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn SetRangeText(&self, _replacement: DOMString) {
    }
}
