/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::str::FromStr;

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name, ns};
use js::rust::HandleObject;
use style::attr::{AttrValue, LengthOrPercentageOrAuto};
use style::color::AbsoluteColor;
use style::values::generics::NonNegative;
use style::values::specified::border::BorderSideWidth;
use style::values::specified::length::Size;
use style::values::specified::{LengthPercentage, NoCalcLength};

use crate::dom::bindings::codegen::Bindings::HTMLHRElementBinding::HTMLHRElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, LayoutDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{Element, LayoutElementHelpers};
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::Node;
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLHRElement {
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

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLHRElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLHRElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
            can_gc,
        )
    }
}

impl HTMLHRElementMethods<crate::DomTypeHolder> for HTMLHRElement {
    // https://html.spec.whatwg.org/multipage/#dom-hr-align
    make_getter!(Align, "align");

    // https://html.spec.whatwg.org/multipage/#dom-hr-align
    make_atomic_setter!(SetAlign, "align");

    // https://html.spec.whatwg.org/multipage/#dom-hr-color
    make_getter!(Color, "color");

    // https://html.spec.whatwg.org/multipage/#dom-hr-color
    make_legacy_color_setter!(SetColor, "color");

    // https://html.spec.whatwg.org/multipage/#dom-hr-noshade
    make_bool_getter!(NoShade, "noshade");

    // https://html.spec.whatwg.org/multipage/#dom-hr-noshade
    make_bool_setter!(SetNoShade, "noshade");

    // https://html.spec.whatwg.org/multipage/#dom-hr-size
    make_getter!(Size, "size");

    // https://html.spec.whatwg.org/multipage/#dom-hr-size
    make_dimension_setter!(SetSize, "size");

    // https://html.spec.whatwg.org/multipage/#dom-hr-width
    make_getter!(Width, "width");

    // https://html.spec.whatwg.org/multipage/#dom-hr-width
    make_dimension_setter!(SetWidth, "width");
}

/// The result of applying the the presentational hint for the `size` attribute.
///
/// (This attribute can mean different things depending on its value and other attributes)
#[allow(clippy::enum_variant_names)]
pub(crate) enum SizePresentationalHint {
    SetHeightTo(Size),
    SetAllBorderWidthValuesTo(BorderSideWidth),
    SetBottomBorderWidthToZero,
}

pub(crate) trait HTMLHRLayoutHelpers {
    fn get_color(self) -> Option<AbsoluteColor>;
    fn get_width(self) -> LengthOrPercentageOrAuto;
    fn get_size_info(self) -> Option<SizePresentationalHint>;
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

    fn get_size_info(self) -> Option<SizePresentationalHint> {
        // https://html.spec.whatwg.org/multipage/#the-hr-element-2
        let element = self.upcast::<Element>();
        let size_value = element
            .get_attr_val_for_layout(&ns!(), &local_name!("size"))
            .and_then(|value| usize::from_str(value).ok())
            .filter(|value| *value != 0)?;

        let hint = if element
            .get_attr_for_layout(&ns!(), &local_name!("color"))
            .is_some() ||
            element
                .get_attr_for_layout(&ns!(), &local_name!("noshade"))
                .is_some()
        {
            SizePresentationalHint::SetAllBorderWidthValuesTo(BorderSideWidth::from_px(
                size_value as f32 / 2.0,
            ))
        } else if size_value == 1 {
            SizePresentationalHint::SetBottomBorderWidthToZero
        } else {
            SizePresentationalHint::SetHeightTo(Size::LengthPercentage(NonNegative(
                LengthPercentage::Length(NoCalcLength::from_px((size_value - 2) as f32)),
            )))
        };

        Some(hint)
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
