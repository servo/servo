/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLVideoElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLVideoElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLVideoElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlmediaelement::HTMLMediaElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLVideoElement {
    pub htmlmediaelement: HTMLMediaElement
}

impl HTMLVideoElementDerived for EventTarget {
    fn is_htmlvideoelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLVideoElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLVideoElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLVideoElement {
        HTMLVideoElement {
            htmlmediaelement: HTMLMediaElement::new_inherited(HTMLVideoElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLVideoElement> {
        let element = HTMLVideoElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLVideoElementBinding::Wrap)
    }
}

impl HTMLVideoElement {
    pub fn Width(&self) -> u32 {
        0
    }

    pub fn SetWidth(&mut self, _width: u32) -> ErrorResult {
        Ok(())
    }

    pub fn Height(&self) -> u32 {
        0
    }

    pub fn SetHeight(&mut self, _height: u32) -> ErrorResult {
        Ok(())
    }

    pub fn VideoWidth(&self) -> u32 {
        0
    }

    pub fn VideoHeight(&self) -> u32 {
        0
    }

    pub fn Poster(&self) -> DOMString {
        ~""
    }

    pub fn SetPoster(&mut self, _poster: DOMString) -> ErrorResult {
        Ok(())
    }
}
