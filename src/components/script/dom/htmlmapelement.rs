/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLMapElementBinding;
use dom::bindings::utils::ErrorResult;
use dom::htmlcollection::HTMLCollection;
use dom::document::AbstractDocument;
use dom::element::HTMLMapElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};
use servo_util::str::DOMString;

pub struct HTMLMapElement {
    htmlelement: HTMLElement
}

impl HTMLMapElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLMapElement {
        HTMLMapElement {
            htmlelement: HTMLElement::new_inherited(HTMLMapElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLMapElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLMapElementBinding::Wrap)
    }
}

impl HTMLMapElement {
    pub fn Name(&self) -> DOMString {
        ~""
    }

    pub fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Areas(&self) -> @mut HTMLCollection {
        let window = self.htmlelement.element.node.owner_doc().document().window;
        HTMLCollection::new(window, ~[])
    }
}
