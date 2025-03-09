/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name};
use js::rust::HandleObject;
use style::attr::AttrValue;

use crate::dom::bindings::codegen::Bindings::HTMLLIElementBinding::HTMLLIElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::Node;
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLLIElement {
    htmlelement: HTMLElement,
}

impl HTMLLIElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLLIElement {
        HTMLLIElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLLIElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLLIElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
            can_gc,
        )
    }
}

impl HTMLLIElementMethods<crate::DomTypeHolder> for HTMLLIElement {
    // https://html.spec.whatwg.org/multipage/#dom-li-value
    make_int_getter!(Value, "value");

    // https://html.spec.whatwg.org/multipage/#dom-li-value
    make_int_setter!(SetValue, "value");
}

impl VirtualMethods for HTMLLIElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("value") => AttrValue::from_i32(value.into(), 0),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }
}
