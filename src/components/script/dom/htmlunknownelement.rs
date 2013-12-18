/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLUnknownElementBinding;
use dom::document::AbstractDocument;
use dom::element::HTMLUnknownElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};

pub struct HTMLUnknownElement {
    htmlelement: HTMLElement
}

impl HTMLUnknownElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLUnknownElement {
        HTMLUnknownElement {
            htmlelement: HTMLElement::new_inherited(HTMLUnknownElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode {
        let element = HTMLUnknownElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLUnknownElementBinding::Wrap)
    }
}
