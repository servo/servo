/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::RGBA;
use dom::attr::AttrValue;
use dom::bindings::codegen::Bindings::HTMLHRElementBinding::{self, HTMLHRElementMethods};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{LayoutJS, Root};
use dom::document::Document;
use dom::element::{Element, RawLayoutElementHelpers};
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom::virtualmethods::VirtualMethods;
use string_cache::Atom;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLHRElement {
    htmlelement: HTMLElement,
}

impl HTMLHRElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: &Document) -> HTMLHRElement {
        HTMLHRElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLHRElement> {
        let element = HTMLHRElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLHRElementBinding::Wrap)
    }
}

impl HTMLHRElementMethods for HTMLHRElement {
    // https://html.spec.whatwg.org/multipage/#dom-hr-color
    make_getter!(Color);

    // https://html.spec.whatwg.org/multipage/#dom-hr-color
    fn SetColor(&self, value: DOMString) {
        self.upcast::<Element>()
            .set_attribute(&atom!("color"), AttrValue::from_legacy_color(value));
    }
}

pub trait HTMLHRLayoutHelpers {
    fn get_color(&self) -> Option<RGBA>;
}

impl HTMLHRLayoutHelpers for LayoutJS<HTMLHRElement> {
    #[allow(unsafe_code)]
    fn get_color(&self) -> Option<RGBA> {
        unsafe {
            (&*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(""), &atom!("color"))
                .and_then(AttrValue::as_color)
                .cloned()
        }
    }
}


impl VirtualMethods for HTMLHRElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("color") => AttrValue::from_legacy_color(value),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }
}
