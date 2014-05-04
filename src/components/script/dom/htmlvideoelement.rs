/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLVideoElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLVideoElementDerived;
use dom::bindings::js::{JSRef, Temporary};
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
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLVideoElement {
        HTMLVideoElement {
            htmlmediaelement: HTMLMediaElement::new_inherited(HTMLVideoElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLVideoElement> {
        let element = HTMLVideoElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLVideoElementBinding::Wrap)
    }
}

pub trait HTMLVideoElementMethods {
    fn Width(&self) -> u32;
    fn SetWidth(&mut self, _width: u32) -> ErrorResult;
    fn Height(&self) -> u32;
    fn SetHeight(&mut self, _height: u32) -> ErrorResult;
    fn VideoWidth(&self) -> u32;
    fn VideoHeight(&self) -> u32;
    fn Poster(&self) -> DOMString;
    fn SetPoster(&mut self, _poster: DOMString) -> ErrorResult;
}

impl<'a> HTMLVideoElementMethods for JSRef<'a, HTMLVideoElement> {
    fn Width(&self) -> u32 {
        0
    }

    fn SetWidth(&mut self, _width: u32) -> ErrorResult {
        Ok(())
    }

    fn Height(&self) -> u32 {
        0
    }

    fn SetHeight(&mut self, _height: u32) -> ErrorResult {
        Ok(())
    }

    fn VideoWidth(&self) -> u32 {
        0
    }

    fn VideoHeight(&self) -> u32 {
        0
    }

    fn Poster(&self) -> DOMString {
        "".to_owned()
    }

    fn SetPoster(&mut self, _poster: DOMString) -> ErrorResult {
        Ok(())
    }
}
