/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLLabelElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLLabelElementDerived;
use dom::bindings::js::JS;
use dom::document::Document;
use dom::element::HTMLLabelElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLLabelElement {
    pub htmlelement: HTMLElement,
}

impl HTMLLabelElementDerived for EventTarget {
    fn is_htmllabelelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLLabelElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLLabelElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLLabelElement {
        HTMLLabelElement {
            htmlelement: HTMLElement::new_inherited(HTMLLabelElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLLabelElement> {
        let element = HTMLLabelElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLLabelElementBinding::Wrap)
    }
}

impl HTMLLabelElement {
    pub fn HtmlFor(&self) -> DOMString {
        ~""
    }

    pub fn SetHtmlFor(&mut self, _html_for: DOMString) {
    }
}
