/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLParagraphElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLParagraphElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};

pub struct HTMLParagraphElement {
    htmlelement: HTMLElement
}

impl HTMLParagraphElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLParagraphElement {
        HTMLParagraphElement {
            htmlelement: HTMLElement::new_inherited(HTMLParagraphElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode {
        let element = HTMLParagraphElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLParagraphElementBinding::Wrap)
    }
}

impl HTMLParagraphElement {
    pub fn Align(&self) -> DOMString {
        ~""
    }

    pub fn SetAlign(&mut self, _align: DOMString) -> ErrorResult {
        Ok(())
    }
}
