/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLFormElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLFormElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLFormElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[jstraceable]
#[must_root]
pub struct HTMLFormElement {
    pub htmlelement: HTMLElement
}

impl HTMLFormElementDerived for EventTarget {
    fn is_htmlformelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLFormElementTypeId))
    }
}

impl HTMLFormElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLFormElement {
        HTMLFormElement {
            htmlelement: HTMLElement::new_inherited(HTMLFormElementTypeId, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLFormElement> {
        let element = HTMLFormElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLFormElementBinding::Wrap)
    }
}

impl Reflectable for HTMLFormElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
