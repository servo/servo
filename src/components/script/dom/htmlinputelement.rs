/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLInputElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLInputElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::{ErrorResult, Fallible};
use dom::document::Document;
use dom::element::HTMLInputElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLInputElement {
    pub htmlelement: HTMLElement,
}

impl HTMLInputElementDerived for EventTarget {
    fn is_htmlinputelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLInputElementTypeId))
    }
}

impl HTMLInputElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLInputElement {
        HTMLInputElement {
            htmlelement: HTMLElement::new_inherited(HTMLInputElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLInputElement> {
        let element = HTMLInputElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLInputElementBinding::Wrap)
    }
}

pub trait HTMLInputElementMethods {
    fn Accept(&self) -> DOMString;
    fn SetAccept(&self, _accept: DOMString) -> ErrorResult;
    fn Alt(&self) -> DOMString;
    fn SetAlt(&self, _alt: DOMString) -> ErrorResult;
    fn Autocomplete(&self) -> DOMString;
    fn SetAutocomplete(&self, _autocomple: DOMString) -> ErrorResult;
    fn Autofocus(&self) -> bool;
    fn SetAutofocus(&self, _autofocus: bool) -> ErrorResult;
    fn DefaultChecked(&self) -> bool;
    fn SetDefaultChecked(&self, _default_checked: bool) -> ErrorResult;
    fn Checked(&self) -> bool;
    fn SetChecked(&self, _checked: bool);
    fn Disabled(&self) -> bool;
    fn SetDisabled(&self, _disabled: bool) -> ErrorResult;
    fn FormAction(&self) -> DOMString;
    fn SetFormAction(&self, _form_action: DOMString) -> ErrorResult;
    fn FormEnctype(&self) -> DOMString;
    fn SetFormEnctype(&self, _form_enctype: DOMString) -> ErrorResult;
    fn FormMethod(&self) -> DOMString;
    fn SetFormMethod(&self, _form_method: DOMString) -> ErrorResult;
    fn FormNoValidate(&self) -> bool;
    fn SetFormNoValidate(&self, _form_no_validate: bool) -> ErrorResult;
    fn FormTarget(&self) -> DOMString;
    fn SetFormTarget(&self, _form_target: DOMString) -> ErrorResult;
    fn Height(&self) -> u32;
    fn SetHeight(&self, _height: u32) -> ErrorResult;
    fn Indeterminate(&self) -> bool;
    fn SetIndeterminate(&self, _indeterminate: bool);
    fn InputMode(&self) -> DOMString;
    fn SetInputMode(&self, _input_mode: DOMString) -> ErrorResult;
    fn Max(&self) -> DOMString;
    fn SetMax(&self, _max: DOMString) -> ErrorResult;
    fn MaxLength(&self) -> i32;
    fn SetMaxLength(&self, _max_length: i32) -> ErrorResult;
    fn Min(&self) -> DOMString;
    fn SetMin(&self, _min: DOMString) -> ErrorResult;
    fn Multiple(&self) -> bool;
    fn SetMultiple(&self, _multiple: bool) -> ErrorResult;
    fn Name(&self) -> DOMString;
    fn SetName(&self, _name: DOMString) -> ErrorResult;
    fn Pattern(&self) -> DOMString;
    fn SetPattern(&self, _pattern: DOMString) -> ErrorResult;
    fn Placeholder(&self) -> DOMString;
    fn SetPlaceholder(&self, _placeholder: DOMString) -> ErrorResult;
    fn ReadOnly(&self) -> bool;
    fn SetReadOnly(&self, _read_only: bool) -> ErrorResult;
    fn Required(&self) -> bool;
    fn SetRequired(&self, _required: bool) -> ErrorResult;
    fn Size(&self) -> u32;
    fn SetSize(&self, _size: u32) -> ErrorResult;
    fn Src(&self) -> DOMString;
    fn SetSrc(&self, _src: DOMString) -> ErrorResult;
    fn Step(&self) -> DOMString;
    fn SetStep(&self, _step: DOMString) -> ErrorResult;
    fn Type(&self) -> DOMString;
    fn SetType(&self, _type: DOMString) -> ErrorResult;
    fn DefaultValue(&self) -> DOMString;
    fn SetDefaultValue(&self, _default_value: DOMString) -> ErrorResult;
    fn Value(&self) -> DOMString;
    fn SetValue(&self, _value: DOMString) -> ErrorResult;
    fn Width(&self) -> u32;
    fn SetWidth(&self, _width: u32);
    fn WillValidate(&self) -> bool;
    fn SetWillValidate(&self, _will_validate: bool);
    fn GetValidationMessage(&self) -> Fallible<DOMString>;
    fn CheckValidity(&self) -> bool;
    fn SetCustomValidity(&self, _error: DOMString);
    fn Select(&self);
    fn GetSelectionStart(&self) -> Fallible<i32>;
    fn SetSelectionStart(&self, _selection_start: i32) -> ErrorResult;
    fn GetSelectionEnd(&self) -> Fallible<i32>;
    fn SetSelectionEnd(&self, _selection_end: i32) -> ErrorResult;
    fn GetSelectionDirection(&self) -> Fallible<DOMString>;
    fn SetSelectionDirection(&self, _selection_direction: DOMString) -> ErrorResult;
    fn Align(&self) -> DOMString;
    fn SetAlign(&self, _align: DOMString) -> ErrorResult;
    fn UseMap(&self) -> DOMString;
    fn SetUseMap(&self, _align: DOMString) -> ErrorResult;
}

impl<'a> HTMLInputElementMethods for JSRef<'a, HTMLInputElement> {
    fn Accept(&self) -> DOMString {
        "".to_owned()
    }

    fn SetAccept(&self, _accept: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Alt(&self) -> DOMString {
        "".to_owned()
    }

    fn SetAlt(&self, _alt: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Autocomplete(&self) -> DOMString {
        "".to_owned()
    }

    fn SetAutocomplete(&self, _autocomple: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Autofocus(&self) -> bool {
        false
    }

    fn SetAutofocus(&self, _autofocus: bool) -> ErrorResult {
        Ok(())
    }

    fn DefaultChecked(&self) -> bool {
        false
    }

    fn SetDefaultChecked(&self, _default_checked: bool) -> ErrorResult {
        Ok(())
    }

    fn Checked(&self) -> bool {
        false
    }

    fn SetChecked(&self, _checked: bool) {
    }

    fn Disabled(&self) -> bool {
        false
    }

    fn SetDisabled(&self, _disabled: bool) -> ErrorResult {
        Ok(())
    }

    fn FormAction(&self) -> DOMString {
        "".to_owned()
    }

    fn SetFormAction(&self, _form_action: DOMString) -> ErrorResult {
        Ok(())
    }

    fn FormEnctype(&self) -> DOMString {
        "".to_owned()
    }

    fn SetFormEnctype(&self, _form_enctype: DOMString) -> ErrorResult {
        Ok(())
    }

    fn FormMethod(&self) -> DOMString {
        "".to_owned()
    }

    fn SetFormMethod(&self, _form_method: DOMString) -> ErrorResult {
        Ok(())
    }

    fn FormNoValidate(&self) -> bool {
        false
    }

    fn SetFormNoValidate(&self, _form_no_validate: bool) -> ErrorResult {
        Ok(())
    }

    fn FormTarget(&self) -> DOMString {
        "".to_owned()
    }

    fn SetFormTarget(&self, _form_target: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Height(&self) -> u32 {
        0
    }

    fn SetHeight(&self, _height: u32) -> ErrorResult {
        Ok(())
    }

    fn Indeterminate(&self) -> bool {
        false
    }

    fn SetIndeterminate(&self, _indeterminate: bool) {
    }

    fn InputMode(&self) -> DOMString {
        "".to_owned()
    }

    fn SetInputMode(&self, _input_mode: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Max(&self) -> DOMString {
        "".to_owned()
    }

    fn SetMax(&self, _max: DOMString) -> ErrorResult {
        Ok(())
    }

    fn MaxLength(&self) -> i32 {
        0
    }

    fn SetMaxLength(&self, _max_length: i32) -> ErrorResult {
        Ok(())
    }

    fn Min(&self) -> DOMString {
        "".to_owned()
    }

    fn SetMin(&self, _min: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Multiple(&self) -> bool {
        false
    }

    fn SetMultiple(&self, _multiple: bool) -> ErrorResult {
        Ok(())
    }

    fn Name(&self) -> DOMString {
        "".to_owned()
    }

    fn SetName(&self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Pattern(&self) -> DOMString {
        "".to_owned()
    }

    fn SetPattern(&self, _pattern: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Placeholder(&self) -> DOMString {
        "".to_owned()
    }

    fn SetPlaceholder(&self, _placeholder: DOMString) -> ErrorResult {
        Ok(())
    }

    fn ReadOnly(&self) -> bool {
        false
    }

    fn SetReadOnly(&self, _read_only: bool) -> ErrorResult {
        Ok(())
    }

    fn Required(&self) -> bool {
        false
    }

    fn SetRequired(&self, _required: bool) -> ErrorResult {
        Ok(())
    }

    fn Size(&self) -> u32 {
        0
    }

    fn SetSize(&self, _size: u32) -> ErrorResult {
        Ok(())
    }

    fn Src(&self) -> DOMString {
        "".to_owned()
    }

    fn SetSrc(&self, _src: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Step(&self) -> DOMString {
        "".to_owned()
    }

    fn SetStep(&self, _step: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Type(&self) -> DOMString {
        "".to_owned()
    }

    fn SetType(&self, _type: DOMString) -> ErrorResult {
        Ok(())
    }

    fn DefaultValue(&self) -> DOMString {
        "".to_owned()
    }

    fn SetDefaultValue(&self, _default_value: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Value(&self) -> DOMString {
        "".to_owned()
    }

    fn SetValue(&self, _value: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Width(&self) -> u32 {
        0
    }

    fn SetWidth(&self, _width: u32) {
    }

    fn WillValidate(&self) -> bool {
        false
    }

    fn SetWillValidate(&self, _will_validate: bool) {
    }

    fn GetValidationMessage(&self) -> Fallible<DOMString> {
        Ok("".to_owned())
    }

    fn CheckValidity(&self) -> bool {
        false
    }

    fn SetCustomValidity(&self, _error: DOMString) {
    }

    fn Select(&self) {
    }

    fn GetSelectionStart(&self) -> Fallible<i32> {
        Ok(0)
    }

    fn SetSelectionStart(&self, _selection_start: i32) -> ErrorResult {
        Ok(())
    }

    fn GetSelectionEnd(&self) -> Fallible<i32> {
        Ok(0)
    }

    fn SetSelectionEnd(&self, _selection_end: i32) -> ErrorResult {
        Ok(())
    }

    fn GetSelectionDirection(&self) -> Fallible<DOMString> {
        Ok("".to_owned())
    }

    fn SetSelectionDirection(&self, _selection_direction: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Align(&self) -> DOMString {
        "".to_owned()
    }

    fn SetAlign(&self, _align: DOMString) -> ErrorResult {
        Ok(())
    }

    fn UseMap(&self) -> DOMString {
        "".to_owned()
    }

    fn SetUseMap(&self, _align: DOMString) -> ErrorResult {
        Ok(())
    }
}
