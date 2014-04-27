/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLTrackElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTrackElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLTrackElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLTrackElement {
    pub htmlelement: HTMLElement,
}

impl HTMLTrackElementDerived for EventTarget {
    fn is_htmltrackelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLTrackElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLTrackElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLTrackElement {
        HTMLTrackElement {
            htmlelement: HTMLElement::new_inherited(HTMLTrackElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLTrackElement> {
        let element = HTMLTrackElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLTrackElementBinding::Wrap)
    }
}

impl HTMLTrackElement {
    pub fn Kind(&self) -> DOMString {
        ~""
    }

    pub fn SetKind(&mut self, _kind: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Src(&self) -> DOMString {
        ~""
    }

    pub fn SetSrc(&mut self, _src: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Srclang(&self) -> DOMString {
        ~""
    }

    pub fn SetSrclang(&mut self, _srclang: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Label(&self) -> DOMString {
        ~""
    }

    pub fn SetLabel(&mut self, _label: DOMString) -> ErrorResult {
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
