/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLHeadElementBinding;
use dom::document::AbstractDocument;
use dom::element::HTMLHeadElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};
use servo_util::str::DOMString;

pub struct HTMLHeadElement {
    htmlelement: HTMLElement
}

impl HTMLHeadElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLHeadElement {
        HTMLHeadElement {
            htmlelement: HTMLElement::new_inherited(HTMLHeadElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLHeadElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLHeadElementBinding::Wrap)
    }
}
