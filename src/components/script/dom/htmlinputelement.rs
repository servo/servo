/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, null_string, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLInputElement {
    parent: HTMLElement,
}

impl HTMLInputElement {
    pub fn Accept(&self) -> DOMString {
        null_string
    }

    pub fn SetAccept(&mut self, _accept: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Alt(&self) -> DOMString {
        null_string
    }

    pub fn SetAlt(&mut self, _alt: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Autocomplete(&self) -> DOMString {
        null_string
    }

    pub fn SetAutocomplete(&mut self, _autocomple: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Autofocus(&self) -> bool {
        false
    }

    pub fn SetAutofocus(&mut self, _autofocus: bool, _rv: &mut ErrorResult) {
    }

    pub fn DefaultChecked(&self) -> bool {
        false
    }

    pub fn SetDefaultChecked(&mut self, _default_checked: bool, _rv: &mut ErrorResult) {
    }

    pub fn Checked(&self) -> bool {
        false
    }

    pub fn SetChecked(&mut self, _checked: bool) {
    }

    pub fn Disabled(&self) -> bool {
        false
    }

    pub fn SetDisabled(&mut self, _disabled: bool, _rv: &mut ErrorResult) {
    }

    pub fn FormAction(&self) -> DOMString {
        null_string
    }

    pub fn SetFormAction(&mut self, _form_action: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn FormEnctype(&self) -> DOMString {
        null_string
    }

    pub fn SetFormEnctype(&mut self, _form_enctype: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn FormMethod(&self) -> DOMString {
        null_string
    }

    pub fn SetFormMethod(&mut self, _form_method: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn FormNoValidate(&self) -> bool {
        false
    }

    pub fn SetFormNoValidate(&mut self, _form_no_validate: bool, _rv: &mut ErrorResult) {
    }

    pub fn FormTarget(&self) -> DOMString {
        null_string
    }

    pub fn SetFormTarget(&mut self, _form_target: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Height(&self) -> u32 {
        0
    }

    pub fn SetHeight(&mut self, _height: u32, _rv: &mut ErrorResult) {
    }

    pub fn Indeterminate(&self) -> bool {
        false
    }

    pub fn SetIndeterminate(&mut self, _indeterminate: bool) {
    }

    pub fn InputMode(&self) -> DOMString {
        null_string
    }

    pub fn SetInputMode(&mut self, _input_mode: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Max(&self) -> DOMString {
        null_string
    }

    pub fn SetMax(&mut self, _max: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn MaxLength(&self) -> i32 {
        0
    }

    pub fn SetMaxLength(&mut self, _max_length: i32, _rv: &mut ErrorResult) {
    }

    pub fn Min(&self) -> DOMString {
        null_string
    }

    pub fn SetMin(&mut self, _min: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Multiple(&self) -> bool {
        false
    }

    pub fn SetMultiple(&mut self, _multiple: bool, _rv: &mut ErrorResult) {
    }

    pub fn Name(&self) -> DOMString {
        null_string
    }

    pub fn SetName(&mut self, _name: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Pattern(&self) -> DOMString {
        null_string
    }

    pub fn SetPattern(&mut self, _pattern: &DOMString, _rv: &mut ErrorResult) {
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

    pub fn Size(&self) -> u32 {
        0
    }

    pub fn SetSize(&mut self, _size: u32, _rv: &mut ErrorResult) {
    }

    pub fn Src(&self) -> DOMString {
        null_string
    }

    pub fn SetSrc(&mut self, _src: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Step(&self) -> DOMString {
        null_string
    }

    pub fn SetStep(&mut self, _step: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Type(&self) -> DOMString {
        null_string
    }

    pub fn SetType(&mut self, _type: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn DefaultValue(&self) -> DOMString {
        null_string
    }

    pub fn SetDefaultValue(&mut self, _default_value: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Value(&self) -> DOMString {
        null_string
    }

    pub fn SetValue(&mut self, _value: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Width(&self) -> u32 {
        0
    }

    pub fn SetWidth(&mut self, _width: u32) {
    }

    pub fn WillValidate(&self) -> bool {
        false
    }

    pub fn SetWillValidate(&self, _will_validate: bool) {
    }

    pub fn GetValidationMessage(&self, _rv: &mut ErrorResult) -> DOMString {
        null_string
    }

    pub fn CheckValidity(&self) -> bool {
        false
    }

    pub fn SetCustomValidity(&self, _error: &DOMString) {
    }

    pub fn Select(&self) {
    }

    pub fn GetSelectionStart(&self, _rv: &mut ErrorResult) -> i32 {
        0
    }

    pub fn SetSelectionStart(&mut self, _selection_start: i32, _rv: &mut ErrorResult) {
    }

    pub fn GetSelectionEnd(&self, _rv: &mut ErrorResult) -> i32 {
        0
    }

    pub fn SetSelectionEnd(&mut self, _selection_end: i32, _rv: &mut ErrorResult) {
    }

    pub fn GetSelectionDirection(&self, _rv: &mut ErrorResult) -> DOMString {
        null_string
    }

    pub fn SetSelectionDirection(&mut self, _selection_direction: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Align(&self) -> DOMString {
        null_string
    }

    pub fn SetAlign(&mut self, _align: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn UseMap(&self) -> DOMString {
        null_string
    }

    pub fn SetUseMap(&mut self, _align: &DOMString, _rv: &mut ErrorResult) {
    }
}
