/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLSourceElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLSourceElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLSourceElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[jstraceable]
#[must_root]
pub struct HTMLSourceElement {
    pub htmlelement: HTMLElement
}

impl HTMLSourceElementDerived for EventTarget {
    fn is_htmlsourceelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLSourceElementTypeId))
    }
}

impl HTMLSourceElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLSourceElement {
        HTMLSourceElement {
            htmlelement: HTMLElement::new_inherited(HTMLSourceElementTypeId, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLSourceElement> {
        let element = HTMLSourceElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLSourceElementBinding::Wrap)
    }
}

impl Reflectable for HTMLSourceElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
