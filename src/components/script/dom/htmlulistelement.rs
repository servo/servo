/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLUListElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLUListElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLUListElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLUListElement {
    pub htmlelement: HTMLElement
}

impl HTMLUListElementDerived for EventTarget {
    fn is_htmlulistelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLUListElementTypeId))
    }
}

impl HTMLUListElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLUListElement {
        HTMLUListElement {
            htmlelement: HTMLElement::new_inherited(HTMLUListElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLUListElement> {
        let element = HTMLUListElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLUListElementBinding::Wrap)
    }
}

pub trait HTMLUListElementMethods {
    fn Compact(&self) -> bool;
    fn SetCompact(&mut self, _compact: bool) -> ErrorResult;
    fn Type(&self) -> DOMString;
    fn SetType(&mut self, _type: DOMString) -> ErrorResult;
}

impl<'a> HTMLUListElementMethods for JSRef<'a, HTMLUListElement> {
    fn Compact(&self) -> bool {
        false
    }

    fn SetCompact(&mut self, _compact: bool) -> ErrorResult {
        Ok(())
    }

    fn Type(&self) -> DOMString {
        "".to_owned()
    }

    fn SetType(&mut self, _type: DOMString) -> ErrorResult {
        Ok(())
    }
}
