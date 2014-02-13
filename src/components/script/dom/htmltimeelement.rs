/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLTimeElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLTimeElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};

pub struct HTMLTimeElement {
    htmlelement: HTMLElement
}

impl HTMLTimeElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLTimeElement {
        HTMLTimeElement {
            htmlelement: HTMLElement::new_inherited(HTMLTimeElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLTimeElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLTimeElementBinding::Wrap)
    }
}

impl HTMLTimeElement {
    pub fn DateTime(&self) -> DOMString {
        ~""
    }
    
    pub fn SetDateTime(&mut self, _dateTime: DOMString) -> ErrorResult {
        Ok(())
    }
}
