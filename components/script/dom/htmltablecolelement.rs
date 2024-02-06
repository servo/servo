/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{local_name, namespace_url, ns, LocalName, Prefix};
use js::rust::HandleObject;
use style::attr::AttrValue;

use super::bindings::root::LayoutDom;
use super::element::Element;
use crate::dom::bindings::codegen::Bindings::HTMLTableColElementBinding::HTMLTableColElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::LayoutElementHelpers;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::Node;
use crate::dom::virtualmethods::VirtualMethods;

const DEFAULT_SPAN: u32 = 1;

#[dom_struct]
pub struct HTMLTableColElement {
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

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLTableColElement> {
        let n = Node::reflect_node_with_proto(
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

impl HTMLTableColElementMethods for HTMLTableColElement {
    // <https://html.spec.whatwg.org/multipage/#attr-col-span>
    make_uint_getter!(Span, "span", DEFAULT_SPAN);
    // <https://html.spec.whatwg.org/multipage/#attr-col-span>
    make_uint_setter!(SetSpan, "span", DEFAULT_SPAN);
}

pub trait HTMLTableColElementLayoutHelpers<'dom> {
    fn get_span(self) -> Option<u32>;
}

impl<'dom> HTMLTableColElementLayoutHelpers<'dom> for LayoutDom<'dom, HTMLTableColElement> {
    fn get_span(self) -> Option<u32> {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("span"))
            .map(AttrValue::as_uint)
    }
}

impl VirtualMethods for HTMLTableColElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn parse_plain_attribute(&self, local_name: &LocalName, value: DOMString) -> AttrValue {
        match *local_name {
            local_name!("span") => {
                let mut attr = AttrValue::from_u32(value.into(), DEFAULT_SPAN);
                if let AttrValue::UInt(_, ref mut val) = attr {
                    if *val == 0 {
                        *val = 1;
                    }
                }
                attr
            },
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(local_name, value),
        }
    }
}
