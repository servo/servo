/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLTitleElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLTitleElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};

pub struct HTMLTitleElement {
    htmlelement: HTMLElement,
}

impl HTMLTitleElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLTitleElement {
        HTMLTitleElement {
            htmlelement: HTMLElement::new_inherited(HTMLTitleElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode {
        let element = HTMLTitleElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLTitleElementBinding::Wrap)
    }
}

impl HTMLTitleElement {
    pub fn Text(&self) -> DOMString {
        ~""
    }

    pub fn SetText(&mut self, _text: DOMString) -> ErrorResult {
        Ok(())
    }
}
