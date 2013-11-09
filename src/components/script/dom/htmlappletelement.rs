/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLAppletElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLAppletElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node, ScriptView};

pub struct HTMLAppletElement {
    htmlelement: HTMLElement
}

impl HTMLAppletElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLAppletElement {
        HTMLAppletElement {
            htmlelement: HTMLElement::new_inherited(HTMLAppletElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode<ScriptView> {
        let element = HTMLAppletElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLAppletElementBinding::Wrap)
    }
}

impl HTMLAppletElement {
    pub fn Align(&self) -> Option<DOMString> {
        None
    }

    pub fn SetAlign(&mut self, _align: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Alt(&self) -> Option<DOMString> {
        None
    }

    pub fn SetAlt(&self, _alt: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Archive(&self) -> Option<DOMString> {
        None
    }

    pub fn SetArchive(&self, _archive: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Code(&self) -> Option<DOMString> {
        None
    }

    pub fn SetCode(&self, _code: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn CodeBase(&self) -> Option<DOMString> {
        None
    }

    pub fn SetCodeBase(&self, _code_base: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Height(&self) -> Option<DOMString> {
        None
    }

    pub fn SetHeight(&self, _height: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Hspace(&self) -> u32 {
        0
    }

    pub fn SetHspace(&mut self, _hspace: u32) -> ErrorResult {
        Ok(())
    }

    pub fn Name(&self) -> Option<DOMString> {
        None
    }

    pub fn SetName(&mut self, _name: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Object(&self) -> Option<DOMString> {
        None
    }

    pub fn SetObject(&mut self, _object: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Vspace(&self) -> u32 {
        0
    }

    pub fn SetVspace(&mut self, _vspace: u32) -> ErrorResult {
        Ok(())
    }

    pub fn Width(&self) -> Option<DOMString> {
        None
    }

    pub fn SetWidth(&mut self, _width: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }
}
