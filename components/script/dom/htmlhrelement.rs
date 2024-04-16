/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{local_name, namespace_url, ns, LocalName, Prefix};
use js::rust::HandleObject;
use style::attr::{AttrValue, LengthOrPercentageOrAuto};
use style::color::AbsoluteColor;

use crate::dom::bindings::codegen::Bindings::HTMLHRElementBinding::HTMLHRElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, LayoutDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{Element, LayoutElementHelpers};
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::Node;
use crate::dom::virtualmethods::VirtualMethods;

#[dom_struct]
pub struct HTMLHRElement {
    htmlelement: HTMLElement,
}

impl HTMLHRElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLHRElement {
        HTMLHRElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLHRElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLHRElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
        )
    }
}

impl HTMLHRElementMethods for HTMLHRElement {
    // https://html.spec.whatwg.org/multipage/#dom-hr-align
    make_getter!(Align, "align");

    // https://html.spec.whatwg.org/multipage/#dom-hr-align
    make_atomic_setter!(SetAlign, "align");

    // https://html.spec.whatwg.org/multipage/#dom-hr-color
    make_getter!(Color, "color");

    // https://html.spec.whatwg.org/multipage/#dom-hr-color
    make_legacy_color_setter!(SetColor, "color");

    // https://html.spec.whatwg.org/multipage/#dom-hr-width
    make_getter!(Width, "width");

    // https://html.spec.whatwg.org/multipage/#dom-hr-width
    make_dimension_setter!(SetWidth, "width");
}

pub trait HTMLHRLayoutHelpers {
    fn get_color(self) -> Option<AbsoluteColor>;
    fn get_width(self) -> LengthOrPercentageOrAuto;
}

impl HTMLHRLayoutHelpers for LayoutDom<'_, HTMLHRElement> {
    fn get_color(self) -> Option<AbsoluteColor> {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("color"))
            .and_then(AttrValue::as_color)
            .cloned()
    }

    fn get_width(self) -> LengthOrPercentageOrAuto {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("width"))
            .map(AttrValue::as_dimension)
            .cloned()
            .unwrap_or(LengthOrPercentageOrAuto::Auto)
    }
}

impl VirtualMethods for HTMLHRElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match *name {
            local_name!("align") => AttrValue::from_dimension(value.into()),
            local_name!("color") => AttrValue::from_legacy_color(value.into()),
            local_name!("width") => AttrValue::from_dimension(value.into()),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }
}
