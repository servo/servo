/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTableSectionElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTableSectionElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLTableSectionElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[jstraceable]
#[must_root]
pub struct HTMLTableSectionElement {
    pub htmlelement: HTMLElement,
}

impl HTMLTableSectionElementDerived for EventTarget {
    fn is_htmltablesectionelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLTableSectionElementTypeId))
    }
}

impl HTMLTableSectionElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLTableSectionElement {
        HTMLTableSectionElement {
            htmlelement: HTMLElement::new_inherited(HTMLTableSectionElementTypeId, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLTableSectionElement> {
        let element = HTMLTableSectionElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLTableSectionElementBinding::Wrap)
    }
}

impl Reflectable for HTMLTableSectionElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
