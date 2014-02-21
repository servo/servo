/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLDirectoryElementBinding;
use dom::bindings::utils::ErrorResult;
use dom::document::AbstractDocument;
use dom::element::HTMLDirectoryElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};
use servo_util::str::DOMString;

pub struct HTMLDirectoryElement {
    htmlelement: HTMLElement
}

impl HTMLDirectoryElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLDirectoryElement {
        HTMLDirectoryElement {
            htmlelement: HTMLElement::new_inherited(HTMLDirectoryElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLDirectoryElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLDirectoryElementBinding::Wrap)
    }
}

impl HTMLDirectoryElement {
    pub fn Compact(&self) -> bool {
        false
    }

    pub fn SetCompact(&mut self, _compact: bool) -> ErrorResult {
        Ok(())
    }
}
