/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::HTMLFontElementBinding;
use dom::bindings::codegen::Bindings::HTMLFontElementBinding::HTMLFontElementMethods;
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLFontElementDerived};
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::{AttributeMutation, ElementTypeId};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId};
use dom::virtualmethods::VirtualMethods;
use util::str::{self, DOMString};

use cssparser::RGBA;
use std::cell::Cell;

#[dom_struct]
pub struct HTMLFontElement {
    htmlelement: HTMLElement,
    color: Cell<Option<RGBA>>,
}

impl HTMLFontElementDerived for EventTarget {
    fn is_htmlfontelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFontElement)))
    }
}

impl HTMLFontElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: &Document) -> HTMLFontElement {
        HTMLFontElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLFontElement, localName, prefix, document),
            color: Cell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLFontElement> {
        let element = HTMLFontElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLFontElementBinding::Wrap)
    }
}

impl HTMLFontElementMethods for HTMLFontElement {
    // https://html.spec.whatwg.org/multipage/#dom-font-color
    make_getter!(Color, "color");

    // https://html.spec.whatwg.org/multipage/#dom-font-color
    make_setter!(SetColor, "color");
}

impl VirtualMethods for HTMLFontElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement = HTMLElementCast::from_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
                &atom!(color) => {
                self.color.set(mutation.new_value(attr).and_then(|value| {
                    str::parse_legacy_color(&value).ok()
                }));
            },
            _ => {},
        }
    }
}


impl HTMLFontElement {
    pub fn get_color(&self) -> Option<RGBA> {
        self.color.get()
    }
}
