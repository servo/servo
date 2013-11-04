/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLDataElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLDataElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node, ScriptView};

pub struct HTMLDataElement {
    htmlelement: HTMLElement
}

impl HTMLDataElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLDataElement {
        HTMLDataElement {
            htmlelement: HTMLElement::new_inherited(HTMLDataElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode<ScriptView> {
        let element = HTMLDataElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLDataElementBinding::Wrap)
    }
}

impl HTMLDataElement {
    pub fn Value(&self) -> DOMString {
        None
    }

    pub fn SetValue(&mut self, _value: &DOMString) -> ErrorResult {
        Ok(())
    }
}
