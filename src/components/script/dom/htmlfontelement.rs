/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLFontElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLFontElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node, ScriptView};

pub struct HTMLFontElement {
    htmlelement: HTMLElement
}

impl HTMLFontElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLFontElement {
        HTMLFontElement {
            htmlelement: HTMLElement::new_inherited(HTMLFontElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode<ScriptView> {
        let element = HTMLFontElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLFontElementBinding::Wrap)
    }
}

impl HTMLFontElement {
    pub fn Color(&self) -> DOMString {
        None
    }

    pub fn SetColor(&mut self, _color: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Face(&self) -> DOMString {
        None
    }

    pub fn SetFace(&mut self, _face: &DOMString) -> ErrorResult {
        Ok(())
    }
    
    pub fn Size(&self) -> DOMString {
        None
    }

    pub fn SetSize(&mut self, _size: &DOMString) -> ErrorResult {
        Ok(())
    }
}
