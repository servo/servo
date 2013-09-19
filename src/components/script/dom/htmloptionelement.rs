/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, ScriptView};

pub struct HTMLOptionElement {
    htmlelement: HTMLElement
}

impl HTMLOptionElement {
    pub fn Disabled(&self) -> bool {
        false
    }

    pub fn SetDisabled(&mut self, _disabled: bool) -> ErrorResult {
        Ok(())
    }

    pub fn GetForm(&self) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn Label(&self) -> DOMString {
        None
    }

    pub fn SetLabel(&mut self, _label: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn DefaultSelected(&self) -> bool {
        false
    }

    pub fn SetDefaultSelected(&mut self, _default_selected: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Selected(&self) -> bool {
        false
    }

    pub fn SetSelected(&mut self, _selected: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Value(&self) -> DOMString {
        None
    }

    pub fn SetValue(&mut self, _value: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Text(&self) -> DOMString {
        None
    }

    pub fn SetText(&mut self, _text: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Index(&self) -> i32 {
        0
    }
}
