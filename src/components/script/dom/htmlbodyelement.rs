/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLBodyElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLBodyElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};

pub struct HTMLBodyElement {
    htmlelement: HTMLElement
}

impl HTMLBodyElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLBodyElement {
        HTMLBodyElement {
            htmlelement: HTMLElement::new_inherited(HTMLBodyElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLBodyElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLBodyElementBinding::Wrap)
    }
}

impl HTMLBodyElement {
    pub fn Text(&self) -> DOMString {
        ~""
    }

    pub fn SetText(&mut self, _text: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Link(&self) -> DOMString {
        ~""
    }

    pub fn SetLink(&self, _link: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn VLink(&self) -> DOMString {
        ~""
    }

    pub fn SetVLink(&self, _v_link: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn ALink(&self) -> DOMString {
        ~""
    }

    pub fn SetALink(&self, _a_link: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn BgColor(&self) -> DOMString {
        ~""
    }

    pub fn SetBgColor(&self, _bg_color: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Background(&self) -> DOMString {
        ~""
    }

    pub fn SetBackground(&self, _background: DOMString) -> ErrorResult {
        Ok(())
    }
}
