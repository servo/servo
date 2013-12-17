/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLPreElementBinding;
use dom::bindings::utils::{ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLPreElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};

pub struct HTMLPreElement {
    htmlelement: HTMLElement,
}

impl HTMLPreElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLPreElement {
        HTMLPreElement {
            htmlelement: HTMLElement::new_inherited(HTMLPreElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode {
        let element = HTMLPreElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLPreElementBinding::Wrap)
    }
}

impl HTMLPreElement {
    pub fn Width(&self) -> i32 {
        0
    }

    pub fn SetWidth(&mut self, _width: i32) -> ErrorResult {
        Ok(())
    }
}
