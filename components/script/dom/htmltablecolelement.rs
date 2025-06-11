/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name, ns};
use js::rust::HandleObject;
use style::attr::{AttrValue, LengthOrPercentageOrAuto};

use super::attr::Attr;
use super::bindings::root::LayoutDom;
use super::element::{AttributeMutation, Element};
use super::node::NodeDamage;
use crate::dom::bindings::codegen::Bindings::HTMLTableColElementBinding::HTMLTableColElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::LayoutElementHelpers;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::Node;
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLTableColElement {
    htmlelement: HTMLElement,
}

impl HTMLTableColElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLTableColElement {
        HTMLTableColElement {
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
    ) -> DomRoot<HTMLTableColElement> {
        let n = Node::reflect_node_with_proto(
            Box::new(HTMLTableColElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        );

        n.upcast::<Node>().set_weird_parser_insertion_mode();
        n
    }
}

impl HTMLTableColElementMethods<crate::DomTypeHolder> for HTMLTableColElement {
    // <https://html.spec.whatwg.org/multipage/#attr-col-span>
    make_uint_getter!(Span, "span", 1);
    // <https://html.spec.whatwg.org/multipage/#attr-col-span>
    // > The span IDL attribute must reflect the content attribute of the same name. It is clamped
    // > to the range [1, 1000], and its default value is 1.
    make_clamped_uint_setter!(SetSpan, "span", 1, 1000, 1);
}

pub(crate) trait HTMLTableColElementLayoutHelpers<'dom> {
    fn get_span(self) -> Option<u32>;
    fn get_width(self) -> LengthOrPercentageOrAuto;
}

impl<'dom> HTMLTableColElementLayoutHelpers<'dom> for LayoutDom<'dom, HTMLTableColElement> {
    fn get_span(self) -> Option<u32> {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("span"))
            .map(AttrValue::as_uint)
    }

    fn get_width(self) -> LengthOrPercentageOrAuto {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("width"))
            .map(AttrValue::as_dimension)
            .cloned()
            .unwrap_or(LengthOrPercentageOrAuto::Auto)
    }
}

impl VirtualMethods for HTMLTableColElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, can_gc: CanGc) {
        if let Some(super_type) = self.super_type() {
            super_type.attribute_mutated(attr, mutation, can_gc);
        }

        if matches!(*attr.local_name(), local_name!("span")) {
            self.upcast::<Node>().dirty(NodeDamage::Other);
        }
    }

    fn parse_plain_attribute(&self, local_name: &LocalName, value: DOMString) -> AttrValue {
        match *local_name {
            local_name!("span") => {
                let mut attr = AttrValue::from_u32(value.into(), 1);
                if let AttrValue::UInt(_, ref mut val) = attr {
                    // From <https://html.spec.whatwg.org/multipage/#attr-col-span>:
                    // > The span IDL attribute must reflect the content attribute of the same name.
                    // > It is clamped to the range [1, 1000], and its default value is 1.
                    *val = (*val).clamp(1, 1000);
                }
                attr
            },
            local_name!("width") => AttrValue::from_dimension(value.into()),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(local_name, value),
        }
    }
}
