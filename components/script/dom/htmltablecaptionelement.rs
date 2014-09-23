/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTableCaptionElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTableCaptionElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLTableCaptionElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[jstraceable]
#[must_root]
pub struct HTMLTableCaptionElement {
    pub htmlelement: HTMLElement
}

impl HTMLTableCaptionElementDerived for EventTarget {
    fn is_htmltablecaptionelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLTableCaptionElementTypeId))
    }
}

impl HTMLTableCaptionElement {
    pub fn new_inherited(localName: DOMString, document: JSRef<Document>) -> HTMLTableCaptionElement {
        HTMLTableCaptionElement {
            htmlelement: HTMLElement::new_inherited(HTMLTableCaptionElementTypeId, localName, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, document: JSRef<Document>) -> Temporary<HTMLTableCaptionElement> {
        let element = HTMLTableCaptionElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLTableCaptionElementBinding::Wrap)
    }
}

impl Reflectable for HTMLTableCaptionElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
