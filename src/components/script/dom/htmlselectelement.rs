/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, ScriptView};
use dom::validitystate::ValidityState;

pub struct HTMLSelectElement {
    htmlelement: HTMLElement
}

impl HTMLSelectElement {
    pub fn Autofocus(&self) -> bool {
        false
    }

    pub fn SetAutofocus(&mut self, _autofocus: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Disabled(&self) -> bool {
        false
    }

    pub fn SetDisabled(&mut self, _disabled: bool) -> ErrorResult {
        Ok(())
    }

    pub fn GetForm(&self) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn Multiple(&self) -> bool {
        false
    }

    pub fn SetMultiple(&mut self, _multiple: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Name(&self) -> DOMString {
        None
    }

    pub fn SetName(&mut self, _name: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Required(&self) -> bool {
        false
    }

    pub fn SetRequired(&mut self, _multiple: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Size(&self) -> u32 {
        0
    }

    pub fn SetSize(&mut self, _size: u32) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> DOMString {
        None
    }

    pub fn Length(&self) -> u32 {
        0
    }

    pub fn SetLength(&mut self, _length: u32) -> ErrorResult {
        Ok(())
    }

    pub fn Item(&self, _index: u32) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn NamedItem(&self, _name: &DOMString) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn IndexedGetter(&self, _index: u32, _found: &mut bool) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn IndexedSetter(&mut self, _index: u32, _option: Option<AbstractNode<ScriptView>>) -> ErrorResult {
        Ok(())
    }

    pub fn Remove_(&self) {
    }

    pub fn Remove(&self, _index: i32) {
    }

    pub fn SelectedIndex(&self) -> i32 {
        0
    }

    pub fn SetSelectedIndex(&mut self, _index: i32) -> ErrorResult {
        Ok(())
    }

    pub fn Value(&self) -> DOMString {
        None
    }

    pub fn SetValue(&mut self, _value: &DOMString) {
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

    pub fn SetValidationMessage(&mut self, _message: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn CheckValidity(&self) -> bool {
        true
    }

    pub fn SetCustomValidity(&mut self, _error: &DOMString) {
    }
}
