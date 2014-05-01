/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLTableCaptionElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTableCaptionElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLTableCaptionElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLTableCaptionElement {
    pub htmlelement: HTMLElement
}

impl HTMLTableCaptionElementDerived for EventTarget {
    fn is_htmltablecaptionelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLTableCaptionElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLTableCaptionElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLTableCaptionElement {
        HTMLTableCaptionElement {
            htmlelement: HTMLElement::new_inherited(HTMLTableCaptionElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLTableCaptionElement> {
        let element = HTMLTableCaptionElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLTableCaptionElementBinding::Wrap)
    }
}

impl HTMLTableCaptionElement {
    pub fn Align(&self) -> DOMString {
        ~""
    }

    pub fn SetAlign(&mut self, _align: DOMString) -> ErrorResult {
        Ok(())
    }
}
