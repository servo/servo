/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLTableSectionElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLTableSectionElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};

pub struct HTMLTableSectionElement {
    htmlelement: HTMLElement,
}

impl HTMLTableSectionElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLTableSectionElement {
        HTMLTableSectionElement {
            htmlelement: HTMLElement::new_inherited(HTMLTableSectionElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode {
        let element = HTMLTableSectionElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLTableSectionElementBinding::Wrap)
    }
}

impl HTMLTableSectionElement {
    pub fn DeleteRow(&mut self, _index: i32) -> ErrorResult {
        Ok(())
    }

    pub fn Align(&self) -> DOMString {
        ~""
    }

    pub fn SetAlign(&mut self, _align: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Ch(&self) -> DOMString {
        ~""
    }

    pub fn SetCh(&mut self, _ch: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn ChOff(&self) -> DOMString {
        ~""
    }

    pub fn SetChOff(&mut self, _ch_off: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn VAlign(&self) -> DOMString {
        ~""
    }

    pub fn SetVAlign(&mut self, _v_align: DOMString) -> ErrorResult {
        Ok(())
    }
}
