/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLHtmlElementBinding;
use dom::bindings::utils::ErrorResult;
use dom::document::AbstractDocument;
use dom::element::HTMLHtmlElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};
use servo_util::str::DOMString;

pub struct HTMLHtmlElement {
    htmlelement: HTMLElement
}

impl HTMLHtmlElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLHtmlElement {
        HTMLHtmlElement {
            htmlelement: HTMLElement::new_inherited(HTMLHtmlElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLHtmlElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLHtmlElementBinding::Wrap)
    }
}

impl HTMLHtmlElement {
    pub fn Version(&self) -> DOMString {
        ~""
    }

    pub fn SetVersion(&mut self, _version: DOMString) -> ErrorResult {
        Ok(())
    }
}
