/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLTableCaptionElementBinding;
use dom::bindings::utils::ErrorResult;
use dom::document::AbstractDocument;
use dom::element::HTMLTableCaptionElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};
use servo_util::str::DOMString;

pub struct HTMLTableCaptionElement {
    htmlelement: HTMLElement
}

impl HTMLTableCaptionElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLTableCaptionElement {
        HTMLTableCaptionElement {
            htmlelement: HTMLElement::new_inherited(HTMLTableCaptionElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLTableCaptionElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLTableCaptionElementBinding::Wrap)
    }
}

impl HTMLTableCaptionElement {
    pub fn Align(&self) -> DOMString {
        ~""
    }
    
    pub fn SetAlign(&mut self, _align: DOMString) -> ErrorResult {
        Ok(())
    }
}
