/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name, ns};
use js::rust::HandleObject;
use style::attr::{AttrValue, LengthOrPercentageOrAuto};

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::HTMLTableColElementBinding::HTMLTableColElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, LayoutDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::node::{Node, NodeDamage};
use crate::dom::virtualmethods::VirtualMethods;

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

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLTableColElement> {
        let n = Node::reflect_node_with_proto(
            cx,
            Box::new(HTMLTableColElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
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

    // <https://html.spec.whatwg.org/multipage/#dom-col-width>
    make_getter!(Width, "width");

    // <https://html.spec.whatwg.org/multipage/#dom-col-width>
    make_dimension_setter!(SetWidth, "width");
}

impl<'dom> LayoutDom<'dom, HTMLTableColElement> {
    pub(crate) fn get_span(self) -> Option<u32> {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("span"))
            .map(AttrValue::as_uint)
    }

    pub(crate) fn get_width(self) -> LengthOrPercentageOrAuto {
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

    fn attribute_mutated(
        &self,
        cx: &mut js::context::JSContext,
        attr: &Attr,
        mutation: AttributeMutation,
    ) {
        if let Some(super_type) = self.super_type() {
            super_type.attribute_mutated(cx, attr, mutation);
        }

        if matches!(*attr.local_name(), local_name!("span")) {
            self.upcast::<Node>().dirty(NodeDamage::Other);
        }
    }

    fn attribute_affects_presentational_hints(&self, attr: &Attr) -> bool {
        match attr.local_name() {
            &local_name!("width") => true,
            _ => self
                .super_type()
                .unwrap()
                .attribute_affects_presentational_hints(attr),
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
