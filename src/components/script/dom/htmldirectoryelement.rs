/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLDirectoryElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLDirectoryElementDerived;
use dom::bindings::js::{JSRef, Temporary};
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
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLDirectoryElementTypeId))
    }
}

impl HTMLDirectoryElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLDirectoryElement {
        HTMLDirectoryElement {
            htmlelement: HTMLElement::new_inherited(HTMLDirectoryElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLDirectoryElement> {
        let element = HTMLDirectoryElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLDirectoryElementBinding::Wrap)
    }
}

pub trait HTMLDirectoryElementMethods {
    fn Compact(&self) -> bool;
    fn SetCompact(&mut self, _compact: bool) -> ErrorResult;
}

impl<'a> HTMLDirectoryElementMethods for JSRef<'a, HTMLDirectoryElement> {
    fn Compact(&self) -> bool {
        false
    }

    fn SetCompact(&mut self, _compact: bool) -> ErrorResult {
        Ok(())
    }
}
