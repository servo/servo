/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLModElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLModElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};

pub struct HTMLModElement {
    htmlelement: HTMLElement
}

impl HTMLModElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLModElement {
        HTMLModElement {
            htmlelement: HTMLElement::new_inherited(HTMLModElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode {
        let element = HTMLModElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLModElementBinding::Wrap)
    }
}

impl HTMLModElement {
    pub fn Cite(&self) -> DOMString {
        ~""
    }

    pub fn SetCite(&mut self, _cite: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn DateTime(&self) -> DOMString {
        ~""
    }

    pub fn SetDateTime(&mut self, _datetime: DOMString) -> ErrorResult {
        Ok(())
    }
}
