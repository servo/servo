/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLParamElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLParamElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLParamElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[jstraceable]
#[must_root]
pub struct HTMLParamElement {
    pub htmlelement: HTMLElement
}

impl HTMLParamElementDerived for EventTarget {
    fn is_htmlparamelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLParamElementTypeId))
    }
}

impl HTMLParamElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLParamElement {
        HTMLParamElement {
            htmlelement: HTMLElement::new_inherited(HTMLParamElementTypeId, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLParamElement> {
        let element = HTMLParamElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLParamElementBinding::Wrap)
    }
}

impl Reflectable for HTMLParamElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
