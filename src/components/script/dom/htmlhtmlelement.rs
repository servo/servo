/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLHtmlElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLHtmlElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLHtmlElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLHtmlElement {
    pub htmlelement: HTMLElement
}

impl HTMLHtmlElementDerived for EventTarget {
    fn is_htmlhtmlelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLHtmlElementTypeId))
    }
}

impl HTMLHtmlElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLHtmlElement {
        HTMLHtmlElement {
            htmlelement: HTMLElement::new_inherited(HTMLHtmlElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLHtmlElement> {
        let element = HTMLHtmlElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLHtmlElementBinding::Wrap)
    }
}

pub trait HTMLHtmlElementMethods {
    fn Version(&self) -> DOMString;
    fn SetVersion(&mut self, _version: DOMString) -> ErrorResult;
}

impl<'a> HTMLHtmlElementMethods for JSRef<'a, HTMLHtmlElement> {
    fn Version(&self) -> DOMString {
        "".to_owned()
    }

    fn SetVersion(&mut self, _version: DOMString) -> ErrorResult {
        Ok(())
    }
}
