/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLCanvasElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLCanvasElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::{ErrorResult};
use dom::document::Document;
use dom::element::HTMLCanvasElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLCanvasElement {
    pub htmlelement: HTMLElement,
}

impl HTMLCanvasElementDerived for EventTarget {
    fn is_htmlcanvaselement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLCanvasElementTypeId))
    }
}

impl HTMLCanvasElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLCanvasElement {
        HTMLCanvasElement {
            htmlelement: HTMLElement::new_inherited(HTMLCanvasElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLCanvasElement> {
        let element = HTMLCanvasElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLCanvasElementBinding::Wrap)
    }
}

pub trait HTMLCanvasElementMethods {
    fn Width(&self) -> u32;
    fn SetWidth(&mut self, _width: u32) -> ErrorResult;
    fn Height(&self) -> u32;
    fn SetHeight(&mut self, _height: u32) -> ErrorResult;
}

impl<'a> HTMLCanvasElementMethods for JSRef<'a, HTMLCanvasElement> {
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
}
