/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLLIElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLLIElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node, ScriptView};

pub struct HTMLLIElement {
    htmlelement: HTMLElement,
}

impl HTMLLIElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLLIElement {
        HTMLLIElement {
            htmlelement: HTMLElement::new_inherited(HTMLLIElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode<ScriptView> {
        let element = HTMLLIElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLLIElementBinding::Wrap)
    }
}

impl HTMLLIElement {
    pub fn Value(&self) -> i32 {
        0
    }

    pub fn SetValue(&mut self, _value: i32) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> DOMString {
        ~""
    }

    pub fn SetType(&mut self, _type: &DOMString) -> ErrorResult {
        Ok(())
    }
}
