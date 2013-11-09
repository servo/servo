/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLTrackElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLTrackElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node, ScriptView};

pub struct HTMLTrackElement {
    htmlelement: HTMLElement,
}

impl HTMLTrackElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLTrackElement {
        HTMLTrackElement {
            htmlelement: HTMLElement::new_inherited(HTMLTrackElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode<ScriptView> {
        let element = HTMLTrackElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLTrackElementBinding::Wrap)
    }
}

impl HTMLTrackElement {
    pub fn Kind(&self) -> Option<DOMString> {
        None
    }

    pub fn SetKind(&mut self, _kind: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Src(&self) -> Option<DOMString> {
        None
    }

    pub fn SetSrc(&mut self, _src: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Srclang(&self) -> Option<DOMString> {
        None
    }

    pub fn SetSrclang(&mut self, _srclang: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Label(&self) -> Option<DOMString> {
        None
    }

    pub fn SetLabel(&mut self, _label: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Default(&self) -> bool {
        false
    }

    pub fn SetDefault(&mut self, _default: bool) -> ErrorResult {
        Ok(())
    }

    pub fn ReadyState(&self) -> u16 {
        0
    }
}
