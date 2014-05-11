/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLHeadingElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLHeadingElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::document::Document;
use dom::element::HTMLHeadingElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub enum HeadingLevel {
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
}

#[deriving(Encodable)]
pub struct HTMLHeadingElement {
    pub htmlelement: HTMLElement,
    pub level: HeadingLevel,
}

impl HTMLHeadingElementDerived for EventTarget {
    fn is_htmlheadingelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLHeadingElementTypeId))
    }
}

impl HTMLHeadingElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>, level: HeadingLevel) -> HTMLHeadingElement {
        HTMLHeadingElement {
            htmlelement: HTMLElement::new_inherited(HTMLHeadingElementTypeId, localName, document),
            level: level,
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>, level: HeadingLevel) -> Temporary<HTMLHeadingElement> {
        let element = HTMLHeadingElement::new_inherited(localName, document, level);
        Node::reflect_node(box element, document, HTMLHeadingElementBinding::Wrap)
    }
}

pub trait HTMLHeadingElementMethods {
    fn Align(&self) -> DOMString;
    fn SetAlign(&mut self, _align: DOMString);
}

impl<'a> HTMLHeadingElementMethods for JSRef<'a, HTMLHeadingElement> {
    fn Align(&self) -> DOMString {
        "".to_owned()
    }

    fn SetAlign(&mut self, _align: DOMString) {
    }
}
