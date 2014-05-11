/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLLabelElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLLabelElementDerived;
use dom::bindings::js::{JSRef, Temporary};
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
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLLabelElementTypeId))
    }
}

impl HTMLLabelElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLLabelElement {
        HTMLLabelElement {
            htmlelement: HTMLElement::new_inherited(HTMLLabelElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLLabelElement> {
        let element = HTMLLabelElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLLabelElementBinding::Wrap)
    }
}

pub trait HTMLLabelElementMethods {
    fn HtmlFor(&self) -> DOMString;
    fn SetHtmlFor(&mut self, _html_for: DOMString);
}

impl<'a> HTMLLabelElementMethods for JSRef<'a, HTMLLabelElement> {
    fn HtmlFor(&self) -> DOMString {
        "".to_owned()
    }

    fn SetHtmlFor(&mut self, _html_for: DOMString) {
    }
}
