/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLDListElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLDListElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};

pub struct HTMLDListElement {
    htmlelement: HTMLElement
}

impl HTMLDListElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLDListElement {
        HTMLDListElement {
            htmlelement: HTMLElement::new_inherited(HTMLDListElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLDListElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLDListElementBinding::Wrap)
    }
}

impl HTMLDListElement {
    pub fn Compact(&self) -> bool {
        false
    }

    pub fn SetCompact(&mut self, _compact: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> DOMString {
        ~""
    }

    pub fn SetType(&mut self, _type: DOMString) -> ErrorResult {
        Ok(())
    }
}
