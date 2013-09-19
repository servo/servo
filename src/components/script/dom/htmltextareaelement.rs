/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult, Fallible};
use dom::htmlelement::HTMLElement;

pub struct HTMLTextAreaElement {
    htmlelement: HTMLElement,
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
        None
    }

    pub fn SetName(&mut self, _name: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Placeholder(&self) -> DOMString {
        None
    }

    pub fn SetPlaceholder(&mut self, _placeholder: &DOMString) -> ErrorResult {
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
        None
    }

    pub fn SetWrap(&mut self, _wrap: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> DOMString {
        None
    }

    pub fn SetType(&mut self, _type: &DOMString) {
    }

    pub fn DefaultValue(&self) -> DOMString {
        None
    }

    pub fn SetDefaultValue(&mut self, _default_value: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Value(&self) -> DOMString {
        None
    }

    pub fn SetValue(&mut self, _value: &DOMString) {
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
        None
    }

    pub fn CheckValidity(&self) -> bool {
        false
    }

    pub fn SetCustomValidity(&self, _error: &DOMString) {
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
        Ok(None)
    }

    pub fn SetSelectionDirection(&self, _selection_direction: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn SetRangeText(&self, _replacement: &DOMString) {
    }
}
