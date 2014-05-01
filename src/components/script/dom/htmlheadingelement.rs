/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLHeadingElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLHeadingElementDerived;
use dom::bindings::js::JS;
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
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLHeadingElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLHeadingElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>, level: HeadingLevel) -> HTMLHeadingElement {
        HTMLHeadingElement {
            htmlelement: HTMLElement::new_inherited(HTMLHeadingElementTypeId, localName, document),
            level: level,
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>, level: HeadingLevel) -> JS<HTMLHeadingElement> {
        let element = HTMLHeadingElement::new_inherited(localName, document.clone(), level);
        Node::reflect_node(~element, document, HTMLHeadingElementBinding::Wrap)
    }
}

impl HTMLHeadingElement {
    pub fn Align(&self) -> DOMString {
        ~""
    }

    pub fn SetAlign(&mut self, _align: DOMString) {
    }
}
