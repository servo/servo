/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, ScriptView};
use dom::validitystate::ValidityState;

pub struct HTMLButtonElement {
    parent: HTMLElement
}

impl HTMLButtonElement {
    pub fn Autofocus(&self) -> bool {
        false
    }

    pub fn SetAutofocus(&mut self, _autofocus: bool, _rv: &mut ErrorResult) {
    }

    pub fn Disabled(&self) -> bool {
        false
    }

    pub fn SetDisabled(&mut self, _disabled: bool, _rv: &mut ErrorResult) {
    }

    pub fn GetForm(&self) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn FormAction(&self) -> DOMString {
        None
    }

    pub fn SetFormAction(&mut self, _formaction: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn FormEnctype(&self) -> DOMString {
        None
    }

    pub fn SetFormEnctype(&mut self, _formenctype: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn FormMethod(&self) -> DOMString {
        None
    }

    pub fn SetFormMethod(&mut self, _formmethod: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn FormNoValidate(&self) -> bool {
        false
    }

    pub fn SetFormNoValidate(&mut self, _novalidate: bool, _rv: &mut ErrorResult) {
    }

    pub fn FormTarget(&self) -> DOMString {
        None
    }

    pub fn SetFormTarget(&mut self, _formtarget: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Name(&self) -> DOMString {
        None
    }

    pub fn SetName(&mut self, _name: &DOMString, _rv: &mut ErrorResult) {
    }
    
    pub fn Type(&self) -> DOMString {
        None
    }

    pub fn SetType(&mut self, _type: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Value(&self) -> DOMString {
        None
    }

    pub fn SetValue(&mut self, _value: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn WillValidate(&self) -> bool {
        false
    }

    pub fn SetWillValidate(&mut self, _will_validate: bool) {
    }

    pub fn Validity(&self) -> @mut ValidityState {
        @mut ValidityState::valid()
    }

    pub fn SetValidity(&mut self, _validity: @mut ValidityState) {
    }

    pub fn ValidationMessage(&self) -> DOMString {
        None
    }

    pub fn SetValidationMessage(&mut self, _message: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn CheckValidity(&self) -> bool {
        true
    }

    pub fn SetCustomValidity(&mut self, _error: &DOMString) {
    }
}