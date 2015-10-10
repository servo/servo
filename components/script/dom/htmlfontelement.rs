/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::RGBA;
use dom::attr::{Attr, AttrValue};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::HTMLFontElementBinding;
use dom::bindings::codegen::Bindings::HTMLFontElementBinding::HTMLFontElementMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast, HTMLFontElementDerived};
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::{AttributeMutation, ElementTypeId, RawLayoutElementHelpers};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId};
use dom::virtualmethods::VirtualMethods;
use std::cell::Cell;
use string_cache::Atom;
use style::values::specified;
use util::str::{self, DOMString, parse_legacy_font_size};

#[dom_struct]
pub struct HTMLFontElement {
    htmlelement: HTMLElement,
    color: Cell<Option<RGBA>>,
    face: DOMRefCell<Option<Atom>>,
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
            face: DOMRefCell::new(None),
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
    make_getter_setter!(Color, SetColor);

    // https://html.spec.whatwg.org/multipage/#dom-font-face
    make_atomic_getter_setter!(Face, SetFace);

    // https://html.spec.whatwg.org/multipage/#dom-font-size
    fn Size(&self) -> DOMString {
        let element = ElementCast::from_ref(self);
        element.get_string_attribute(&atom!("size"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-font-size
    fn SetSize(&self, value: DOMString) {
        let element = ElementCast::from_ref(self);
        let length = parse_length(&value);
        element.set_attribute(&atom!("size"), AttrValue::Length(value, length));
    }
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
            &atom!(face) => {
                *self.face.borrow_mut() =
                    mutation.new_value(attr)
                            .map(|value| value.as_atom().clone())
            },
            _ => {},
        }
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("face") => AttrValue::from_atomic(value),
            &atom!("size") => {
                let length = parse_length(&value);
                AttrValue::Length(value, length)
            },
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }
}


impl HTMLFontElement {
    pub fn get_color(&self) -> Option<RGBA> {
        self.color.get()
    }

    #[allow(unsafe_code)]
    pub fn get_face(&self) -> Option<Atom> {
        let face = unsafe { self.face.borrow_for_layout() };
        match *face {
            Some(ref s) => Some(s.clone()),
            None => None,
        }
    }

    #[allow(unsafe_code)]
    pub fn get_size(&self) -> Option<specified::Length> {
        unsafe {
            ElementCast::from_ref(self)
                .get_attr_for_layout(&ns!(""), &atom!("size"))
                .and_then(AttrValue::as_length)
                .cloned()
        }
    }
}

fn parse_length(value: &str) -> Option<specified::Length> {
    parse_legacy_font_size(&value).and_then(|parsed| specified::Length::from_str(&parsed))
}
