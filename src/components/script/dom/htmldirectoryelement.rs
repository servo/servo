/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLDirectoryElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLDirectoryElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLDirectoryElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLDirectoryElement {
    pub htmlelement: HTMLElement
}

impl HTMLDirectoryElementDerived for EventTarget {
    fn is_htmldirectoryelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLDirectoryElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLDirectoryElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLDirectoryElement {
        HTMLDirectoryElement {
            htmlelement: HTMLElement::new_inherited(HTMLDirectoryElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLDirectoryElement> {
        let element = HTMLDirectoryElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLDirectoryElementBinding::Wrap)
    }
}

impl HTMLDirectoryElement {
    pub fn Compact(&self) -> bool {
        false
    }

    pub fn SetCompact(&mut self, _compact: bool) -> ErrorResult {
        Ok(())
    }
}
