/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name};
use js::rust::HandleObject;
use style::attr::{AttrValue, LengthOrPercentageOrAuto};

use crate::dom::bindings::codegen::Bindings::HTMLMarqueeElementBinding::HTMLMarqueeElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, LayoutDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::element::attributes::storage::AttrRef;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::node::Node;
use crate::dom::node::virtualmethods::VirtualMethods;

#[dom_struct]
pub(crate) struct HTMLMarqueeElement {
    htmlelement: HTMLElement,
}

impl HTMLMarqueeElement {
    fn new_inherited(local_name: LocalName, prefix: Option<Prefix>, document: &Document) -> Self {
        Self {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<Self> {
        Node::reflect_node_with_proto(
            cx,
            Box::new(Self::new_inherited(local_name, prefix, document)),
            document,
            proto,
        )
    }
}

impl HTMLMarqueeElementMethods<crate::DomTypeHolder> for HTMLMarqueeElement {
    // <https://html.spec.whatwg.org/multipage/#dom-marquee-width>
    make_getter!(Width, "width");
    // <https://html.spec.whatwg.org/multipage/#dom-marquee-width>
    make_dimension_setter!(SetWidth, "width");

    // <https://html.spec.whatwg.org/multipage/#dom-marquee-height>
    make_getter!(Height, "height");
    // <https://html.spec.whatwg.org/multipage/#dom-marquee-height>
    make_dimension_setter!(SetHeight, "height");
}

impl VirtualMethods for HTMLMarqueeElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_affects_presentational_hints(&self, attr: AttrRef<'_>) -> bool {
        match attr.local_name() {
            &local_name!("width") | &local_name!("height") => true,
            _ => self
                .super_type()
                .unwrap()
                .attribute_affects_presentational_hints(attr),
        }
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match *name {
            local_name!("width") => AttrValue::from_dimension(value.into()),
            local_name!("height") => AttrValue::from_dimension(value.into()),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }
}

impl LayoutDom<'_, HTMLMarqueeElement> {
    pub(crate) fn width(self) -> LengthOrPercentageOrAuto {
        self.upcast::<Element>()
            .dimension_attr_value(local_name!("width"))
    }

    pub(crate) fn height(self) -> LengthOrPercentageOrAuto {
        self.upcast::<Element>()
            .dimension_attr_value(local_name!("height"))
    }
}
