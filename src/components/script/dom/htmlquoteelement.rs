/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLQuoteElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLQuoteElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLQuoteElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLQuoteElement {
    pub htmlelement: HTMLElement,
}

impl HTMLQuoteElementDerived for EventTarget {
    fn is_htmlquoteelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLQuoteElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLQuoteElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLQuoteElement {
        HTMLQuoteElement {
            htmlelement: HTMLElement::new_inherited(HTMLQuoteElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLQuoteElement> {
        let element = HTMLQuoteElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLQuoteElementBinding::Wrap)
    }
}

impl HTMLQuoteElement {
    pub fn Cite(&self) -> DOMString {
        ~""
    }

    pub fn SetCite(&self, _cite: DOMString) -> ErrorResult {
        Ok(())
    }
}
