/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLHtmlElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLHtmlElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, NodeTypeId};
use servo_util::str::DOMString;

#[dom_struct]
pub struct HTMLHtmlElement {
    htmlelement: HTMLElement
}

impl HTMLHtmlElementDerived for EventTarget {
    fn is_htmlhtmlelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLHtmlElement))
    }
}

impl HTMLHtmlElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLHtmlElement {
        HTMLHtmlElement {
            htmlelement: HTMLElement::new_inherited(ElementTypeId::HTMLHtmlElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLHtmlElement> {
        let element = HTMLHtmlElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLHtmlElementBinding::Wrap)
    }
}

impl Reflectable for HTMLHtmlElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
