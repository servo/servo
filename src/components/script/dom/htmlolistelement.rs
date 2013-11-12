/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLOListElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLOListElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node, ScriptView};

pub struct HTMLOListElement {
    htmlelement: HTMLElement,
}

impl HTMLOListElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLOListElement {
        HTMLOListElement {
            htmlelement: HTMLElement::new_inherited(HTMLOListElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode<ScriptView> {
        let element = HTMLOListElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLOListElementBinding::Wrap)
    }
}

impl HTMLOListElement {
    pub fn Reversed(&self) -> bool {
        false
    }

    pub fn SetReversed(&self, _reversed: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Start(&self) -> i32 {
        0
    }

    pub fn SetStart(&mut self, _start: i32) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> DOMString {
        ~""
    }

    pub fn SetType(&mut self, _type: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Compact(&self) -> bool {
        false
    }

    pub fn SetCompact(&self, _compact: bool) -> ErrorResult {
        Ok(())
    }
}
