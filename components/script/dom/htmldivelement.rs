/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLDivElementBinding::{self, HTMLDivElementMethods};
use dom::bindings::codegen::InheritTypes::{ElementTypeId, EventTargetTypeId};
use dom::bindings::codegen::InheritTypes::{HTMLDivElementDerived, HTMLElementTypeId, NodeTypeId};
use dom::bindings::js::Root;
use dom::document::Document;
use dom::eventtarget::EventTarget;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLDivElement {
    htmlelement: HTMLElement
}

impl HTMLDivElementDerived for EventTarget {
    fn is_htmldivelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLDivElement)))
    }
}

impl HTMLDivElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLDivElement {
        HTMLDivElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLDivElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLDivElement> {
        let element = HTMLDivElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLDivElementBinding::Wrap)
    }
}

impl HTMLDivElementMethods for HTMLDivElement {
    // https://html.spec.whatwg.org/multipage/#dom-div-align
    make_getter!(Align);

    // https://html.spec.whatwg.org/multipage/#dom-div-align
    make_setter!(SetAlign, "align");
}
