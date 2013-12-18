/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLMainElementBinding;
use dom::document::AbstractDocument;
use dom::element::HTMLMainElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};

pub struct HTMLMainElement {
    htmlelement: HTMLElement
}

impl HTMLMainElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLMainElement {
        HTMLMainElement {
            htmlelement: HTMLElement::new_inherited(HTMLMainElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode {
        let element = HTMLMainElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLMainElementBinding::Wrap)
    }
}
