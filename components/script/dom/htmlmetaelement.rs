/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLMetaElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLMetaElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLMetaElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
#[must_root]
pub struct HTMLMetaElement {
    pub htmlelement: HTMLElement,
}

impl HTMLMetaElementDerived for EventTarget {
    fn is_htmlmetaelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLMetaElementTypeId))
    }
}

impl HTMLMetaElement {
    pub fn new_inherited(localName: DOMString, document: JSRef<Document>) -> HTMLMetaElement {
        HTMLMetaElement {
            htmlelement: HTMLElement::new_inherited(HTMLMetaElementTypeId, localName, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, document: JSRef<Document>) -> Temporary<HTMLMetaElement> {
        let element = HTMLMetaElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLMetaElementBinding::Wrap)
    }
}

impl Reflectable for HTMLMetaElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
