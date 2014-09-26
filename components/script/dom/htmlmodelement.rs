/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLModElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLModElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLModElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[jstraceable]
#[must_root]
pub struct HTMLModElement {
    pub htmlelement: HTMLElement
}

impl HTMLModElementDerived for EventTarget {
    fn is_htmlmodelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLModElementTypeId))
    }
}

impl HTMLModElement {
    fn new_inherited(localName: DOMString, document: JSRef<Document>) -> HTMLModElement {
        HTMLModElement {
            htmlelement: HTMLElement::new_inherited(HTMLModElementTypeId, localName, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, document: JSRef<Document>) -> Temporary<HTMLModElement> {
        let element = HTMLModElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLModElementBinding::Wrap)
    }
}

impl Reflectable for HTMLModElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
