/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLPreElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLPreElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLPreElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[jstraceable]
#[must_root]
pub struct HTMLPreElement {
    pub htmlelement: HTMLElement,
}

impl HTMLPreElementDerived for EventTarget {
    fn is_htmlpreelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLPreElementTypeId))
    }
}

impl HTMLPreElement {
    pub fn new_inherited(localName: DOMString, document: JSRef<Document>) -> HTMLPreElement {
        HTMLPreElement {
            htmlelement: HTMLElement::new_inherited(HTMLPreElementTypeId, localName, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, document: JSRef<Document>) -> Temporary<HTMLPreElement> {
        let element = HTMLPreElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLPreElementBinding::Wrap)
    }
}

impl Reflectable for HTMLPreElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
