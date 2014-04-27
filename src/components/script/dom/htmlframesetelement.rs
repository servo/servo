/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLFrameSetElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLFrameSetElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLFrameSetElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLFrameSetElement {
    pub htmlelement: HTMLElement
}

impl HTMLFrameSetElementDerived for EventTarget {
    fn is_htmlframesetelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLFrameSetElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLFrameSetElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLFrameSetElement {
        HTMLFrameSetElement {
            htmlelement: HTMLElement::new_inherited(HTMLFrameSetElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLFrameSetElement> {
        let element = HTMLFrameSetElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLFrameSetElementBinding::Wrap)
    }
}

impl HTMLFrameSetElement {
    pub fn Cols(&self) -> DOMString {
        ~""
    }

    pub fn SetCols(&mut self, _cols: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Rows(&self) -> DOMString {
        ~""
    }

    pub fn SetRows(&mut self, _rows: DOMString) -> ErrorResult {
        Ok(())
    }
}
