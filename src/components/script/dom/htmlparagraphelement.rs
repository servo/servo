/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLParagraphElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLParagraphElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLParagraphElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLParagraphElement {
    pub htmlelement: HTMLElement
}

impl HTMLParagraphElementDerived for EventTarget {
    fn is_htmlparagraphelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLParagraphElementTypeId))
    }
}

impl HTMLParagraphElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLParagraphElement {
        HTMLParagraphElement {
            htmlelement: HTMLElement::new_inherited(HTMLParagraphElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLParagraphElement> {
        let element = HTMLParagraphElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLParagraphElementBinding::Wrap)
    }
}

pub trait HTMLParagraphElementMethods {
    fn Align(&self) -> DOMString;
    fn SetAlign(&mut self, _align: DOMString) -> ErrorResult;
}

impl<'a> HTMLParagraphElementMethods for JSRef<'a, HTMLParagraphElement> {
    fn Align(&self) -> DOMString {
        "".to_owned()
    }

    fn SetAlign(&mut self, _align: DOMString) -> ErrorResult {
        Ok(())
    }
}
