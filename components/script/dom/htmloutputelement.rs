/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLOutputElementBinding;
use dom::bindings::codegen::Bindings::HTMLOutputElementBinding::HTMLOutputElementMethods;
use dom::bindings::codegen::InheritTypes::HTMLOutputElementDerived;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId, window_from_node};
use dom::validitystate::ValidityState;
use util::str::DOMString;

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct HTMLOutputElement {
    htmlelement: HTMLElement
}

impl HTMLOutputElementDerived for EventTarget {
    fn is_htmloutputelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOutputElement)))
    }
}

impl HTMLOutputElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLOutputElement {
        HTMLOutputElement {
            htmlelement:
                HTMLElement::new_inherited(HTMLElementTypeId::HTMLOutputElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLOutputElement> {
        let element = HTMLOutputElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLOutputElementBinding::Wrap)
    }
}

impl<'a> HTMLOutputElementMethods for &'a HTMLOutputElement {
    // https://html.spec.whatwg.org/multipage/#dom-cva-validity
    fn Validity(self) -> Root<ValidityState> {
        let window = window_from_node(self);
        ValidityState::new(window.r())
    }
}

