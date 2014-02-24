/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLTimeElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTimeElementDerived;
use dom::bindings::js::JS;
use dom::bindings::utils::ErrorResult;
use dom::document::Document;
use dom::element::HTMLTimeElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLTimeElement {
    htmlelement: HTMLElement
}

impl HTMLTimeElementDerived for EventTarget {
    fn is_htmltimeelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLTimeElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLTimeElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLTimeElement {
        HTMLTimeElement {
            htmlelement: HTMLElement::new_inherited(HTMLTimeElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLTimeElement> {
        let element = HTMLTimeElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLTimeElementBinding::Wrap)
    }
}

impl HTMLTimeElement {
    pub fn DateTime(&self) -> DOMString {
        ~""
    }
    
    pub fn SetDateTime(&mut self, _dateTime: DOMString) -> ErrorResult {
        Ok(())
    }
}
