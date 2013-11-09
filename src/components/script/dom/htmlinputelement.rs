/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLInputElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult, Fallible};
use dom::document::AbstractDocument;
use dom::element::HTMLInputElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node, ScriptView};

pub struct HTMLInputElement {
    htmlelement: HTMLElement,
}

impl HTMLInputElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLInputElement {
        HTMLInputElement {
            htmlelement: HTMLElement::new_inherited(HTMLInputElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode<ScriptView> {
        let element = HTMLInputElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLInputElementBinding::Wrap)
    }
}

impl HTMLInputElement {
    pub fn Accept(&self) -> Option<DOMString> {
        None
    }

    pub fn SetAccept(&mut self, _accept: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Alt(&self) -> Option<DOMString> {
        None
    }

    pub fn SetAlt(&mut self, _alt: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Autocomplete(&self) -> Option<DOMString> {
        None
    }

    pub fn SetAutocomplete(&mut self, _autocomple: &Option<DOMString>) -> ErrorResult {
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

    pub fn FormAction(&self) -> Option<DOMString> {
        None
    }

    pub fn SetFormAction(&mut self, _form_action: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn FormEnctype(&self) -> Option<DOMString> {
        None
    }

    pub fn SetFormEnctype(&mut self, _form_enctype: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn FormMethod(&self) -> Option<DOMString> {
        None
    }

    pub fn SetFormMethod(&mut self, _form_method: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn FormNoValidate(&self) -> bool {
        false
    }

    pub fn SetFormNoValidate(&mut self, _form_no_validate: bool) -> ErrorResult {
        Ok(())
    }

    pub fn FormTarget(&self) -> Option<DOMString> {
        None
    }

    pub fn SetFormTarget(&mut self, _form_target: &Option<DOMString>) -> ErrorResult {
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

    pub fn InputMode(&self) -> Option<DOMString> {
        None
    }

    pub fn SetInputMode(&mut self, _input_mode: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Max(&self) -> Option<DOMString> {
        None
    }

    pub fn SetMax(&mut self, _max: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn MaxLength(&self) -> i32 {
        0
    }

    pub fn SetMaxLength(&mut self, _max_length: i32) -> ErrorResult {
        Ok(())
    }

    pub fn Min(&self) -> Option<DOMString> {
        None
    }

    pub fn SetMin(&mut self, _min: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Multiple(&self) -> bool {
        false
    }

    pub fn SetMultiple(&mut self, _multiple: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Name(&self) -> Option<DOMString> {
        None
    }

    pub fn SetName(&mut self, _name: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Pattern(&self) -> Option<DOMString> {
        None
    }

    pub fn SetPattern(&mut self, _pattern: &Option<DOMString>) -> ErrorResult {
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

    pub fn Size(&self) -> u32 {
        0
    }

    pub fn SetSize(&mut self, _size: u32) -> ErrorResult {
        Ok(())
    }

    pub fn Src(&self) -> Option<DOMString> {
        None
    }

    pub fn SetSrc(&mut self, _src: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Step(&self) -> Option<DOMString> {
        None
    }

    pub fn SetStep(&mut self, _step: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> Option<DOMString> {
        None
    }

    pub fn SetType(&mut self, _type: &Option<DOMString>) -> ErrorResult {
        Ok(())
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

    pub fn SetValue(&mut self, _value: &Option<DOMString>) -> ErrorResult {
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

    pub fn GetValidationMessage(&self) -> Fallible<Option<DOMString>> {
        Ok(None)
    }

    pub fn CheckValidity(&self) -> bool {
        false
    }

    pub fn SetCustomValidity(&self, _error: &Option<DOMString>) {
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

    pub fn GetSelectionDirection(&self) -> Fallible<Option<DOMString>> {
        Ok(None)
    }

    pub fn SetSelectionDirection(&mut self, _selection_direction: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Align(&self) -> Option<DOMString> {
        None
    }

    pub fn SetAlign(&mut self, _align: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn UseMap(&self) -> Option<DOMString> {
        None
    }

    pub fn SetUseMap(&mut self, _align: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }
}
