/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLScriptElementBinding;
use dom::bindings::codegen::Bindings::HTMLScriptElementBinding::HTMLScriptElementMethods;
use dom::bindings::codegen::InheritTypes::HTMLScriptElementDerived;
use dom::bindings::codegen::InheritTypes::ElementCast;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::{HTMLScriptElementTypeId, Element, AttributeHandlers};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLScriptElement {
    pub htmlelement: HTMLElement,
}

impl HTMLScriptElementDerived for EventTarget {
    fn is_htmlscriptelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLScriptElementTypeId))
    }
}

impl HTMLScriptElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLScriptElement {
        HTMLScriptElement {
            htmlelement: HTMLElement::new_inherited(HTMLScriptElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLScriptElement> {
        let element = HTMLScriptElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLScriptElementBinding::Wrap)
    }
}

impl<'a> HTMLScriptElementMethods for JSRef<'a, HTMLScriptElement> {
    fn Src(&self) -> DOMString {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.get_url_attribute("src")
    }
}

impl Reflectable for HTMLScriptElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
