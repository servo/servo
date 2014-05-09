/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLDivElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLDivElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLDivElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLDivElement {
    pub htmlelement: HTMLElement
}

impl HTMLDivElementDerived for EventTarget {
    fn is_htmldivelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLDivElementTypeId))
    }
}

impl HTMLDivElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLDivElement {
        HTMLDivElement {
            htmlelement: HTMLElement::new_inherited(HTMLDivElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLDivElement> {
        let element = HTMLDivElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLDivElementBinding::Wrap)
    }
}

pub trait HTMLDivElementMethods {
    fn Align(&self) -> DOMString;
    fn SetAlign(&mut self, _align: DOMString) -> ErrorResult;
}

impl<'a> HTMLDivElementMethods for JSRef<'a, HTMLDivElement> {
    fn Align(&self) -> DOMString {
        "".to_owned()
    }

    fn SetAlign(&mut self, _align: DOMString) -> ErrorResult {
        Ok(())
    }
}
