/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLFrameSetElementBinding;
use dom::bindings::utils::ErrorResult;
use dom::document::AbstractDocument;
use dom::element::HTMLFrameSetElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};
use servo_util::str::DOMString;

pub struct HTMLFrameSetElement {
    htmlelement: HTMLElement
}

impl HTMLFrameSetElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLFrameSetElement {
        HTMLFrameSetElement {
            htmlelement: HTMLElement::new_inherited(HTMLFrameSetElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLFrameSetElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLFrameSetElementBinding::Wrap)
    }
}

impl HTMLFrameSetElement {
    pub fn Cols(&self) -> DOMString {
        ~""
    }

    pub fn SetCols(&mut self, _cols: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Rows(&self) -> DOMString {
        ~""
    }

    pub fn SetRows(&mut self, _rows: DOMString) -> ErrorResult {
        Ok(())
    }
}
