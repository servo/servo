/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLDivElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLDivElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};

pub struct HTMLDivElement {
    htmlelement: HTMLElement
}

impl HTMLDivElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLDivElement {
        HTMLDivElement {
            htmlelement: HTMLElement::new_inherited(HTMLDivElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLDivElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLDivElementBinding::Wrap)
    }
}

impl HTMLDivElement {
    pub fn Align(&self) -> DOMString {
        ~""
    }

    pub fn SetAlign(&mut self, _align: DOMString) -> ErrorResult {
        Ok(())
    }
}
