/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTableRowElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTableRowElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLTableRowElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[dom_struct]
pub struct HTMLTableRowElement {
    htmlelement: HTMLElement,
}

impl HTMLTableRowElementDerived for EventTarget {
    fn is_htmltablerowelement(&self) -> bool {
        *self.type_id() == NodeTargetTypeId(ElementNodeTypeId(HTMLTableRowElementTypeId))
    }
}

impl HTMLTableRowElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLTableRowElement {
        HTMLTableRowElement {
            htmlelement: HTMLElement::new_inherited(HTMLTableRowElementTypeId, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>)
               -> Temporary<HTMLTableRowElement> {
        Node::reflect_node(box HTMLTableRowElement::new_inherited(localName, prefix, document),
                           document,
                           HTMLTableRowElementBinding::Wrap)
    }
}

impl Reflectable for HTMLTableRowElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
