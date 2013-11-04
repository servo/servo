/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLMapElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlcollection::HTMLCollection;
use dom::document::AbstractDocument;
use dom::element::HTMLMapElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node, ScriptView};

pub struct HTMLMapElement {
    htmlelement: HTMLElement
}

impl HTMLMapElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLMapElement {
        HTMLMapElement {
            htmlelement: HTMLElement::new_inherited(HTMLMapElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode<ScriptView> {
        let element = HTMLMapElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLMapElementBinding::Wrap)
    }
}

impl HTMLMapElement {
    pub fn Name(&self) -> DOMString {
        None
    }

    pub fn SetName(&mut self, _name: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Areas(&self) -> @mut HTMLCollection {
        let window = self.htmlelement.element.node.owner_doc().document().window;
        HTMLCollection::new(window, ~[])
    }
}
