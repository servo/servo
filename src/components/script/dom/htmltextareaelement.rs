/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLTextAreaElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult, Fallible};
use dom::document::AbstractDocument;
use dom::element::HTMLTextAreaElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node, ScriptView};

pub struct HTMLTextAreaElement {
    htmlelement: HTMLElement,
}

impl HTMLTextAreaElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLTextAreaElement {
        HTMLTextAreaElement {
            htmlelement: HTMLElement::new_inherited(HTMLTextAreaElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode<ScriptView> {
        let element = HTMLTextAreaElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLTextAreaElementBinding::Wrap)
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

    pub fn Name(&self) -> Option<DOMString> {
        None
    }

    pub fn SetName(&mut self, _name: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Placeholder(&self) -> Option<DOMString> {
        None
    }

    pub fn SetPlaceholder(&mut self, _placeholder: &Option<DOMString>) -> ErrorResult {
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

    pub fn Wrap(&self) -> Option<DOMString> {
        None
    }

    pub fn SetWrap(&mut self, _wrap: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> Option<DOMString> {
        None
    }

    pub fn SetType(&mut self, _type: &Option<DOMString>) {
    }

    pub fn DefaultValue(&self) -> Option<DOMString> {
        None
    }

    pub fn SetDefaultValue(&mut self, _default_value: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Value(&self) -> Option<DOMString> {
        None
    }

    pub fn SetValue(&mut self, _value: &Option<DOMString>) {
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

    pub fn ValidationMessage(&self) -> Option<DOMString> {
        None
    }

    pub fn CheckValidity(&self) -> bool {
        false
    }

    pub fn SetCustomValidity(&self, _error: &Option<DOMString>) {
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

    pub fn GetSelectionDirection(&self) -> Fallible<Option<DOMString>> {
        Ok(None)
    }

    pub fn SetSelectionDirection(&self, _selection_direction: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn SetRangeText(&self, _replacement: &Option<DOMString>) {
    }
}
