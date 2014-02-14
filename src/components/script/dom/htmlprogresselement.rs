/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLProgressElementBinding;
use dom::bindings::utils::{ErrorResult, Fallible};
use dom::document::AbstractDocument;
use dom::element::HTMLProgressElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};
use servo_util::str::DOMString;

pub struct HTMLProgressElement {
    htmlelement: HTMLElement,
}

impl HTMLProgressElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLProgressElement {
        HTMLProgressElement {
            htmlelement: HTMLElement::new_inherited(HTMLProgressElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLProgressElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLProgressElementBinding::Wrap)
    }
}

impl HTMLProgressElement {
    pub fn Value(&self) -> f64 {
        0f64
    }

    pub fn SetValue(&mut self, _value: f64) -> ErrorResult {
        Ok(())
    }

    pub fn Max(&self) -> f64 {
        0f64
    }

    pub fn SetMax(&mut self, _max: f64) -> ErrorResult {
        Ok(())
    }

    pub fn Position(&self) -> f64 {
        0f64
    }

    pub fn GetPositiom(&self) -> Fallible<f64> {
        Ok(0f64)
    }
}
