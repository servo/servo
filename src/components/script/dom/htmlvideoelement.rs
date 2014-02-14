/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLVideoElementBinding;
use dom::bindings::utils::ErrorResult;
use dom::document::AbstractDocument;
use dom::element::HTMLVideoElementTypeId;
use dom::htmlmediaelement::HTMLMediaElement;
use dom::node::{AbstractNode, Node};
use servo_util::str::DOMString;

pub struct HTMLVideoElement {
    htmlmediaelement: HTMLMediaElement
}

impl HTMLVideoElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLVideoElement {
        HTMLVideoElement {
            htmlmediaelement: HTMLMediaElement::new_inherited(HTMLVideoElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLVideoElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLVideoElementBinding::Wrap)
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
