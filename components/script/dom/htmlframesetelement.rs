/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLFrameSetElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLFrameSetElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, NodeTypeId};
use servo_util::str::DOMString;

#[dom_struct]
pub struct HTMLFrameSetElement {
    htmlelement: HTMLElement
}

impl HTMLFrameSetElementDerived for EventTarget {
    fn is_htmlframesetelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLFrameSetElement))
    }
}

impl HTMLFrameSetElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLFrameSetElement {
        HTMLFrameSetElement {
            htmlelement: HTMLElement::new_inherited(ElementTypeId::HTMLFrameSetElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLFrameSetElement> {
        let element = HTMLFrameSetElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLFrameSetElementBinding::Wrap)
    }
}

impl Reflectable for HTMLFrameSetElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
