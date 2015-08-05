/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLAppletElementBinding;
use dom::bindings::codegen::Bindings::HTMLAppletElementBinding::HTMLAppletElementMethods;

use dom::attr::AttrValue;
use dom::bindings::codegen::InheritTypes::HTMLAppletElementDerived;
use dom::bindings::codegen::InheritTypes::HTMLElementCast;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::{AttributeHandlers, ElementTypeId};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId};
use dom::virtualmethods::VirtualMethods;

use string_cache::Atom;
use util::str::DOMString;

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct HTMLAppletElement {
    htmlelement: HTMLElement
}

impl HTMLAppletElementDerived for EventTarget {
    fn is_htmlappletelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAppletElement)))
    }
}

impl HTMLAppletElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLAppletElement {
        HTMLAppletElement {
            htmlelement:
                HTMLElement::new_inherited(HTMLElementTypeId::HTMLAppletElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLAppletElement> {
        let element = HTMLAppletElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLAppletElementBinding::Wrap)
    }
}

impl<'a> HTMLAppletElementMethods for &'a HTMLAppletElement {
    // https://html.spec.whatwg.org/#the-applet-element:dom-applet-name
    make_getter!(Name);
    make_atomic_setter!(SetName, "name");
}

impl<'a> VirtualMethods for &'a HTMLAppletElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        Some(HTMLElementCast::from_borrowed_ref(self) as &VirtualMethods)
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("name") => AttrValue::from_atomic(value),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }
}
