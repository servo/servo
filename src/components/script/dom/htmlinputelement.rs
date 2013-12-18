/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLInputElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult, Fallible};
use dom::document::AbstractDocument;
use dom::element::HTMLInputElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};

pub struct HTMLInputElement {
    htmlelement: HTMLElement,
}

impl HTMLInputElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLInputElement {
        HTMLInputElement {
            htmlelement: HTMLElement::new_inherited(HTMLInputElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode {
        let element = HTMLInputElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLInputElementBinding::Wrap)
    }
}

impl HTMLInputElement {
    pub fn Accept(&self) -> DOMString {
        ~""
    }

    pub fn SetAccept(&mut self, _accept: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Alt(&self) -> DOMString {
        ~""
    }

    pub fn SetAlt(&mut self, _alt: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Autocomplete(&self) -> DOMString {
        ~""
    }

    pub fn SetAutocomplete(&mut self, _autocomple: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Autofocus(&self) -> bool {
        false
    }

    pub fn SetAutofocus(&mut self, _autofocus: bool) -> ErrorResult {
        Ok(())
    }

    pub fn DefaultChecked(&self) -> bool {
        false
    }

    pub fn SetDefaultChecked(&mut self, _default_checked: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Checked(&self) -> bool {
        false
    }

    pub fn SetChecked(&mut self, _checked: bool) {
    }

    pub fn Disabled(&self) -> bool {
        false
    }

    pub fn SetDisabled(&mut self, _disabled: bool) -> ErrorResult {
        Ok(())
    }

    pub fn FormAction(&self) -> DOMString {
        ~""
    }

    pub fn SetFormAction(&mut self, _form_action: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn FormEnctype(&self) -> DOMString {
        ~""
    }

    pub fn SetFormEnctype(&mut self, _form_enctype: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn FormMethod(&self) -> DOMString {
        ~""
    }

    pub fn SetFormMethod(&mut self, _form_method: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn FormNoValidate(&self) -> bool {
        false
    }

    pub fn SetFormNoValidate(&mut self, _form_no_validate: bool) -> ErrorResult {
        Ok(())
    }

    pub fn FormTarget(&self) -> DOMString {
        ~""
    }

    pub fn SetFormTarget(&mut self, _form_target: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Height(&self) -> u32 {
        0
    }

    pub fn SetHeight(&mut self, _height: u32) -> ErrorResult {
        Ok(())
    }

    pub fn Indeterminate(&self) -> bool {
        false
    }

    pub fn SetIndeterminate(&mut self, _indeterminate: bool) {
    }

    pub fn InputMode(&self) -> DOMString {
        ~""
    }

    pub fn SetInputMode(&mut self, _input_mode: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Max(&self) -> DOMString {
        ~""
    }

    pub fn SetMax(&mut self, _max: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn MaxLength(&self) -> i32 {
        0
    }

    pub fn SetMaxLength(&mut self, _max_length: i32) -> ErrorResult {
        Ok(())
    }

    pub fn Min(&self) -> DOMString {
        ~""
    }

    pub fn SetMin(&mut self, _min: DOMString) -> ErrorResult {
        Ok(())
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

    pub fn Pattern(&self) -> DOMString {
        ~""
    }

    pub fn SetPattern(&mut self, _pattern: DOMString) -> ErrorResult {
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

    pub fn Size(&self) -> u32 {
        0
    }

    pub fn SetSize(&mut self, _size: u32) -> ErrorResult {
        Ok(())
    }

    pub fn Src(&self) -> DOMString {
        ~""
    }

    pub fn SetSrc(&mut self, _src: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Step(&self) -> DOMString {
        ~""
    }

    pub fn SetStep(&mut self, _step: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> DOMString {
        ~""
    }

    pub fn SetType(&mut self, _type: DOMString) -> ErrorResult {
        Ok(())
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

    pub fn SetValue(&mut self, _value: DOMString) -> ErrorResult {
        Ok(())
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

    pub fn GetValidationMessage(&self) -> Fallible<DOMString> {
        Ok(~"")
    }

    pub fn CheckValidity(&self) -> bool {
        false
    }

    pub fn SetCustomValidity(&self, _error: DOMString) {
    }

    pub fn Select(&self) {
    }

    pub fn GetSelectionStart(&self) -> Fallible<i32> {
        Ok(0)
    }

    pub fn SetSelectionStart(&mut self, _selection_start: i32) -> ErrorResult {
        Ok(())
    }

    pub fn GetSelectionEnd(&self) -> Fallible<i32> {
        Ok(0)
    }

    pub fn SetSelectionEnd(&mut self, _selection_end: i32) -> ErrorResult {
        Ok(())
    }

    pub fn GetSelectionDirection(&self) -> Fallible<DOMString> {
        Ok(~"")
    }

    pub fn SetSelectionDirection(&mut self, _selection_direction: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Align(&self) -> DOMString {
        ~""
    }

    pub fn SetAlign(&mut self, _align: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn UseMap(&self) -> DOMString {
        ~""
    }

    pub fn SetUseMap(&mut self, _align: DOMString) -> ErrorResult {
        Ok(())
    }
}
