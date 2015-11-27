/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::RGBA;
use dom::attr::AttrValue;
use dom::bindings::codegen::Bindings::HTMLFontElementBinding;
use dom::bindings::codegen::Bindings::HTMLFontElementBinding::HTMLFontElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{LayoutJS, Root};
use dom::document::Document;
use dom::element::{Element, RawLayoutElementHelpers};
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom::virtualmethods::VirtualMethods;
use string_cache::Atom;
use style::values::specified;
use util::str::{DOMString, parse_legacy_font_size};

#[dom_struct]
pub struct HTMLFontElement {
    htmlelement: HTMLElement,
}


impl HTMLFontElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: &Document) -> HTMLFontElement {
        HTMLFontElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document),
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
    make_legacy_color_setter!(SetColor, "color");

    // https://html.spec.whatwg.org/multipage/#dom-font-face
    make_getter!(Face, "face");

    // https://html.spec.whatwg.org/multipage/#dom-font-face
    make_atomic_setter!(SetFace, "face");

    // https://html.spec.whatwg.org/multipage/#dom-font-size
    make_getter!(Size, "size");

    // https://html.spec.whatwg.org/multipage/#dom-font-size
    fn SetSize(&self, value: DOMString) {
        let element = self.upcast::<Element>();
        let length = parse_length(&value);
        element.set_attribute(&atom!("size"), AttrValue::Length(value, length));
    }
}

impl VirtualMethods for HTMLFontElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("face") => AttrValue::from_atomic(value),
            &atom!("color") => AttrValue::from_legacy_color(value),
            &atom!("size") => {
                let length = parse_length(&value);
                AttrValue::Length(value, length)
            },
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }
}

pub trait HTMLFontElementLayoutHelpers {
    fn get_color(&self) -> Option<RGBA>;
    fn get_face(&self) -> Option<Atom>;
    fn get_size(&self) -> Option<specified::Length>;
}

impl HTMLFontElementLayoutHelpers for LayoutJS<HTMLFontElement> {
    #[allow(unsafe_code)]
    fn get_color(&self) -> Option<RGBA> {
        unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &atom!("color"))
                .and_then(AttrValue::as_color)
                .cloned()
        }
    }

    #[allow(unsafe_code)]
    fn get_face(&self) -> Option<Atom> {
        unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &atom!("face"))
                .map(AttrValue::as_atom)
                .cloned()
        }
    }

    #[allow(unsafe_code)]
    fn get_size(&self) -> Option<specified::Length> {
        unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &atom!("size"))
                .and_then(AttrValue::as_length)
                .cloned()
        }
    }
}

fn parse_length(value: &str) -> Option<specified::Length> {
    parse_legacy_font_size(&value).and_then(|parsed| specified::Length::from_str(&parsed))
}
