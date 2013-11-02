/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLBodyElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLBodyElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node, ScriptView};

pub struct HTMLBodyElement {
    htmlelement: HTMLElement
}

impl HTMLBodyElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLBodyElement {
        HTMLBodyElement {
            htmlelement: HTMLElement::new_inherited(HTMLBodyElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode<ScriptView> {
        let element = HTMLBodyElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLBodyElementBinding::Wrap)
    }
}

impl HTMLBodyElement {
    pub fn Text(&self) -> DOMString {
        None
    }

    pub fn SetText(&mut self, _text: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Link(&self) -> DOMString {
        None
    }

    pub fn SetLink(&self, _link: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn VLink(&self) -> DOMString {
        None
    }

    pub fn SetVLink(&self, _v_link: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn ALink(&self) -> DOMString {
        None
    }

    pub fn SetALink(&self, _a_link: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn BgColor(&self) -> DOMString {
        None
    }

    pub fn SetBgColor(&self, _bg_color: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Background(&self) -> DOMString {
        None
    }

    pub fn SetBackground(&self, _background: &DOMString) -> ErrorResult {
        Ok(())
    }
}
