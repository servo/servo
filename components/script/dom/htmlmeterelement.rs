/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLMeterElementBinding;
use dom::bindings::codegen::InheritTypes::{ElementTypeId, EventTargetTypeId, HTMLElementTypeId};
use dom::bindings::codegen::InheritTypes::{HTMLMeterElementDerived, NodeTypeId};
use dom::bindings::js::Root;
use dom::bindings::utils::TopDOMClass;
use dom::document::Document;
use dom::eventtarget::EventTarget;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLMeterElement {
    htmlelement: HTMLElement
}

impl HTMLMeterElementDerived for EventTarget {
    fn is_htmlmeterelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLMeterElement)))
    }
}

impl HTMLMeterElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLMeterElement {
        HTMLMeterElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLMeterElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLMeterElement> {
        let element = HTMLMeterElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLMeterElementBinding::Wrap)
    }
}
