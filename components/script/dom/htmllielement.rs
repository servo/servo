/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLLIElementBinding;
use dom::bindings::codegen::Bindings::HTMLLIElementBinding::HTMLLIElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom::virtualmethods::VirtualMethods;
use string_cache::Atom;
use style::attr::AttrValue;

#[dom_struct]
pub struct HTMLLIElement {
    htmlelement: HTMLElement,
}

impl HTMLLIElement {
    fn new_inherited(localName: Atom, prefix: Option<DOMString>, document: &Document) -> HTMLLIElement {
        HTMLLIElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLLIElement> {
        Node::reflect_node(box HTMLLIElement::new_inherited(localName, prefix, document),
                           document,
                           HTMLLIElementBinding::Wrap)
    }
}

impl HTMLLIElementMethods for HTMLLIElement {
    // https://html.spec.whatwg.org/multipage/#dom-li-value
    make_int_getter!(Value, "value");

    // https://html.spec.whatwg.org/multipage/#dom-li-value
    make_int_setter!(SetValue, "value");
}

impl VirtualMethods for HTMLLIElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("value") => AttrValue::from_i32(value.into(), 0),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }
}
