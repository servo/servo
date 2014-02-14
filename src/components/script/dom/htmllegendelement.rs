/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLLegendElementBinding;
use dom::bindings::utils::ErrorResult;
use dom::document::AbstractDocument;
use dom::element::HTMLLegendElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};
use servo_util::str::DOMString;

pub struct HTMLLegendElement {
    htmlelement: HTMLElement,
}

impl HTMLLegendElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLLegendElement {
        HTMLLegendElement {
            htmlelement: HTMLElement::new_inherited(HTMLLegendElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLLegendElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLLegendElementBinding::Wrap)
    }
}

impl HTMLLegendElement {
    pub fn Align(&self) -> DOMString {
        ~""
    }

    pub fn SetAlign(&mut self, _align: DOMString) -> ErrorResult {
        Ok(())
    }
}
