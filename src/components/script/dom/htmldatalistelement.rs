/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLDataListElementBinding;
use dom::bindings::utils::DOMString;
use dom::document::AbstractDocument;
use dom::element::HTMLDataListElementTypeId;
use dom::htmlcollection::HTMLCollection;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};

pub struct HTMLDataListElement {
    htmlelement: HTMLElement
}

impl HTMLDataListElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLDataListElement {
        HTMLDataListElement {
            htmlelement: HTMLElement::new_inherited(HTMLDataListElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLDataListElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLDataListElementBinding::Wrap)
    }
}

impl HTMLDataListElement {
    pub fn Options(&self) -> @mut HTMLCollection {
        let window = self.htmlelement.element.node.owner_doc().document().window;
        HTMLCollection::new(window, ~[])
    }
}
