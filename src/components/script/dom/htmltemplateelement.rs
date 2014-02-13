/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLTemplateElementBinding;
use dom::bindings::utils::DOMString;
use dom::document::AbstractDocument;
use dom::element::HTMLTemplateElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};

pub struct HTMLTemplateElement {
    htmlelement: HTMLElement,
}

impl HTMLTemplateElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLTemplateElement {
        HTMLTemplateElement {
            htmlelement: HTMLElement::new_inherited(HTMLTemplateElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLTemplateElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLTemplateElementBinding::Wrap)
    }
}
