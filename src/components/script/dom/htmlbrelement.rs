/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLBRElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLBRElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};

pub struct HTMLBRElement {
    htmlelement: HTMLElement,
}

impl HTMLBRElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLBRElement {
        HTMLBRElement {
            htmlelement: HTMLElement::new_inherited(HTMLBRElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLBRElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLBRElementBinding::Wrap)
    }
}

impl HTMLBRElement {
    pub fn Clear(&self) -> DOMString {
        ~""
    }

    pub fn SetClear(&mut self, _text: DOMString) -> ErrorResult {
        Ok(())
    }
}
