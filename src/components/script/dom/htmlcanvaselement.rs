/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLCanvasElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLCanvasElementDerived;
use dom::bindings::js::JS;
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
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLCanvasElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLCanvasElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLCanvasElement {
        HTMLCanvasElement {
            htmlelement: HTMLElement::new_inherited(HTMLCanvasElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLCanvasElement> {
        let element = HTMLCanvasElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLCanvasElementBinding::Wrap)
    }
}

impl HTMLCanvasElement {
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
}
