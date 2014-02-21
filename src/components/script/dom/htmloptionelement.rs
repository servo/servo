/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLOptionElementBinding;
use dom::bindings::utils::ErrorResult;
use dom::document::AbstractDocument;
use dom::element::HTMLOptionElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};
use servo_util::str::DOMString;

pub struct HTMLOptionElement {
    htmlelement: HTMLElement
}

impl HTMLOptionElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLOptionElement {
        HTMLOptionElement {
            htmlelement: HTMLElement::new_inherited(HTMLOptionElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLOptionElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLOptionElementBinding::Wrap)
    }
}

impl HTMLOptionElement {
    pub fn Disabled(&self) -> bool {
        false
    }

    pub fn SetDisabled(&mut self, _disabled: bool) -> ErrorResult {
        Ok(())
    }

    pub fn GetForm(&self) -> Option<AbstractNode> {
        None
    }

    pub fn Label(&self) -> DOMString {
        ~""
    }

    pub fn SetLabel(&mut self, _label: DOMString) -> ErrorResult {
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
        ~""
    }

    pub fn SetValue(&mut self, _value: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Text(&self) -> DOMString {
        ~""
    }

    pub fn SetText(&mut self, _text: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Index(&self) -> i32 {
        0
    }
}
