/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, null_string, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLTextAreaElement {
    parent: HTMLElement,
}

impl HTMLTextAreaElement {
    pub fn Autofocus(&self) -> bool {
        false
    }

    pub fn SetAutofocus(&mut self, _autofocus: bool, _rv: &mut ErrorResult) {
    }

    pub fn Cols(&self) -> u32 {
        0
    }

    pub fn SetCols(&self, _cols: u32, _rv: &mut ErrorResult) {
    }

    pub fn Disabled(&self) -> bool {
        false
    }

    pub fn SetDisabled(&mut self, _disabled: bool, _rv: &mut ErrorResult) {
    }

    pub fn MaxLength(&self) -> i32 {
        0
    }

    pub fn SetMaxLength(&self, _max_length: i32, _rv: &mut ErrorResult) {
    }

    pub fn Name(&self) -> DOMString {
        null_string
    }

    pub fn SetName(&mut self, _name: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Placeholder(&self) -> DOMString {
        null_string
    }

    pub fn SetPlaceholder(&mut self, _placeholder: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn ReadOnly(&self) -> bool {
        false
    }

    pub fn SetReadOnly(&mut self, _read_only: bool, _rv: &mut ErrorResult) {
    }

    pub fn Required(&self) -> bool {
        false
    }

    pub fn SetRequired(&mut self, _required: bool, _rv: &mut ErrorResult) {
    }

    pub fn Rows(&self) -> u32 {
        0
    }

    pub fn SetRows(&self, _rows: u32, _rv: &mut ErrorResult) {
    }

    pub fn Wrap(&self) -> DOMString {
        null_string
    }

    pub fn SetWrap(&mut self, _wrap: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Type(&self) -> DOMString {
        null_string
    }

    pub fn SetType(&mut self, _type: &DOMString) {
    }

    pub fn DefaultValue(&self) -> DOMString {
        null_string
    }

    pub fn SetDefaultValue(&mut self, _default_value: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Value(&self) -> DOMString {
        null_string
    }

    pub fn SetValue(&mut self, _value: &DOMString) {
    }

    pub fn TextLength(&self) -> u32 {
        0
    }

    pub fn SetTextLength(&self, _text_length: u32, _rv: &mut ErrorResult) {
    }

    pub fn WillValidate(&self) -> bool {
        false
    }

    pub fn SetWillValidate(&mut self, _will_validate: bool, _rv: &mut ErrorResult) {
    }

    pub fn ValidationMessage(&self) -> DOMString {
        null_string
    }

    pub fn CheckValidity(&self) -> bool {
        false
    }

    pub fn SetCustomValidity(&self, _error: &DOMString) {
    }

    pub fn Select(&self) {
    }

    pub fn GetSelectionStart(&self, _rv: &mut ErrorResult) -> u32 {
        0
    }

    pub fn SetSelectionStart(&self, _selection_start: u32, _rv: &mut ErrorResult) {
    }

    pub fn GetSelectionEnd(&self, _rv: &mut ErrorResult) -> u32 {
        0
    }

    pub fn SetSelectionEnd(&self, _selection_end: u32, _rv: &mut ErrorResult) {
    }

    pub fn GetSelectionDirection(&self, _rv: &mut ErrorResult) -> DOMString {
        null_string
    }

    pub fn SetSelectionDirection(&self, _selection_direction: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn SetRangeText(&self, _replacement: &DOMString) {
    }
}
