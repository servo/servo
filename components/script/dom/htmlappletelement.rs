/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLAppletElementBinding;
use dom::bindings::codegen::Bindings::HTMLAppletElementBinding::HTMLAppletElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use style::attr::AttrValue;

#[dom_struct]
pub struct HTMLAppletElement {
    htmlelement: HTMLElement
}

impl HTMLAppletElement {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document) -> HTMLAppletElement {
        HTMLAppletElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document) -> Root<HTMLAppletElement> {
        Node::reflect_node(box HTMLAppletElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLAppletElementBinding::Wrap)
    }
}

impl HTMLAppletElementMethods for HTMLAppletElement {
    // https://html.spec.whatwg.org/multipage/#the-applet-element:dom-applet-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#the-applet-element:dom-applet-name
    make_atomic_setter!(SetName, "name");
}

impl VirtualMethods for HTMLAppletElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("name") => AttrValue::from_atomic(value.into()),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }
}
