/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::RGBA;
use dom::bindings::codegen::Bindings::HTMLHRElementBinding::{self, HTMLHRElementMethods};
use dom::bindings::inheritance::Castable;
use dom::bindings::root::{DomRoot, LayoutDom};
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::{Element, RawLayoutElementHelpers};
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use style::attr::{AttrValue, LengthOrPercentageOrAuto};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLHRElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement<TH>,
}

impl<TH: TypeHolderTrait> HTMLHRElement<TH> {
    fn new_inherited(local_name: LocalName, prefix: Option<Prefix>, document: &Document<TH>) -> HTMLHRElement<TH> {
        HTMLHRElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>) -> DomRoot<HTMLHRElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLHRElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLHRElementBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> HTMLHRElementMethods for HTMLHRElement<TH> {
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
    fn get_color(&self) -> Option<RGBA>;
    fn get_width(&self) -> LengthOrPercentageOrAuto;
}

impl<TH: TypeHolderTrait> HTMLHRLayoutHelpers for LayoutDom<HTMLHRElement<TH>> {
    #[allow(unsafe_code)]
    fn get_color(&self) -> Option<RGBA> {
        unsafe {
            (&*self.upcast::<Element<TH>>().unsafe_get())
                .get_attr_for_layout(&ns!(), &local_name!("color"))
                .and_then(AttrValue::as_color)
                .cloned()
        }
    }

    #[allow(unsafe_code)]
    fn get_width(&self) -> LengthOrPercentageOrAuto {
        unsafe {
            (&*self.upcast::<Element<TH>>().unsafe_get())
                .get_attr_for_layout(&ns!(), &local_name!("width"))
                .map(AttrValue::as_dimension)
                .cloned()
                .unwrap_or(LengthOrPercentageOrAuto::Auto)
        }
    }
}


impl<TH: TypeHolderTrait> VirtualMethods<TH> for HTMLHRElement<TH> {
    fn super_type(&self) -> Option<&VirtualMethods<TH>> {
        Some(self.upcast::<HTMLElement<TH>>() as &VirtualMethods<TH>)
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("align") => AttrValue::from_dimension(value.into()),
            &local_name!("color") => AttrValue::from_legacy_color(value.into()),
            &local_name!("width") => AttrValue::from_dimension(value.into()),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }
}
