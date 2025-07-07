/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name, ns};
use js::rust::HandleObject;
use style::attr::{AttrValue, LengthOrPercentageOrAuto};
use style::color::AbsoluteColor;

use super::attr::Attr;
use super::element::AttributeMutation;
use super::node::NodeDamage;
use crate::dom::bindings::codegen::Bindings::HTMLTableCellElementBinding::HTMLTableCellElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, LayoutDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{Element, LayoutElementHelpers};
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmltableelement::HTMLTableElement;
use crate::dom::htmltablerowelement::HTMLTableRowElement;
use crate::dom::htmltablesectionelement::HTMLTableSectionElement;
use crate::dom::node::{LayoutNodeHelpers, Node};
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

const DEFAULT_COLSPAN: u32 = 1;
const DEFAULT_ROWSPAN: u32 = 1;

#[dom_struct]
pub(crate) struct HTMLTableCellElement {
    htmlelement: HTMLElement,
}

impl HTMLTableCellElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLTableCellElement {
        HTMLTableCellElement {
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
    ) -> DomRoot<HTMLTableCellElement> {
        let n = Node::reflect_node_with_proto(
            Box::new(HTMLTableCellElement::new_inherited(
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

impl HTMLTableCellElementMethods<crate::DomTypeHolder> for HTMLTableCellElement {
    // https://html.spec.whatwg.org/multipage/#dom-tdth-colspan
    make_uint_getter!(ColSpan, "colspan", DEFAULT_COLSPAN);

    // https://html.spec.whatwg.org/multipage/#dom-tdth-colspan
    // > The colSpan IDL attribute must reflect the colspan content attribute. It is clamped to
    // > the range [1, 1000], and its default value is 1.
    make_clamped_uint_setter!(SetColSpan, "colspan", 1, 1000, 1);

    // https://html.spec.whatwg.org/multipage/#dom-tdth-rowspan
    make_uint_getter!(RowSpan, "rowspan", DEFAULT_ROWSPAN);

    // https://html.spec.whatwg.org/multipage/#dom-tdth-rowspan
    // > The rowSpan IDL attribute must reflect the rowspan content attribute. It is clamped to
    // > the range [0, 65534], and its default value is 1.
    make_clamped_uint_setter!(SetRowSpan, "rowspan", 0, 65534, 1);

    // https://html.spec.whatwg.org/multipage/#dom-tdth-bgcolor
    make_getter!(BgColor, "bgcolor");

    // https://html.spec.whatwg.org/multipage/#dom-tdth-bgcolor
    make_legacy_color_setter!(SetBgColor, "bgcolor");

    // https://html.spec.whatwg.org/multipage/#dom-tdth-width
    make_getter!(Width, "width");

    // https://html.spec.whatwg.org/multipage/#dom-tdth-width
    make_nonzero_dimension_setter!(SetWidth, "width");

    // https://html.spec.whatwg.org/multipage/#dom-tdth-cellindex
    fn CellIndex(&self) -> i32 {
        let self_node = self.upcast::<Node>();

        let parent_children = match self_node.GetParentNode() {
            Some(ref parent_node) if parent_node.is::<HTMLTableRowElement>() => {
                parent_node.children()
            },
            _ => return -1,
        };

        parent_children
            .filter(|c| c.is::<HTMLTableCellElement>())
            .position(|c| &*c == self_node)
            .map_or(-1, |p| p as i32)
    }
}

pub(crate) trait HTMLTableCellElementLayoutHelpers<'dom> {
    fn get_background_color(self) -> Option<AbsoluteColor>;
    fn get_colspan(self) -> Option<u32>;
    fn get_rowspan(self) -> Option<u32>;
    fn get_table(self) -> Option<LayoutDom<'dom, HTMLTableElement>>;
    fn get_width(self) -> LengthOrPercentageOrAuto;
    fn get_height(self) -> LengthOrPercentageOrAuto;
}

impl<'dom> HTMLTableCellElementLayoutHelpers<'dom> for LayoutDom<'dom, HTMLTableCellElement> {
    fn get_background_color(self) -> Option<AbsoluteColor> {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("bgcolor"))
            .and_then(AttrValue::as_color)
            .cloned()
    }

    fn get_colspan(self) -> Option<u32> {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("colspan"))
            .map(AttrValue::as_uint)
    }

    fn get_rowspan(self) -> Option<u32> {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("rowspan"))
            .map(AttrValue::as_uint)
    }

    fn get_table(self) -> Option<LayoutDom<'dom, HTMLTableElement>> {
        let row = self.upcast::<Node>().composed_parent_node_ref()?;
        row.downcast::<HTMLTableRowElement>()?;
        let section = row.composed_parent_node_ref()?;
        section.downcast::<HTMLTableElement>().or_else(|| {
            section.downcast::<HTMLTableSectionElement>()?;
            let table = section.composed_parent_node_ref()?;
            table.downcast::<HTMLTableElement>()
        })
    }

    fn get_width(self) -> LengthOrPercentageOrAuto {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("width"))
            .map(AttrValue::as_dimension)
            .cloned()
            .unwrap_or(LengthOrPercentageOrAuto::Auto)
    }

    fn get_height(self) -> LengthOrPercentageOrAuto {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("height"))
            .map(AttrValue::as_dimension)
            .cloned()
            .unwrap_or(LengthOrPercentageOrAuto::Auto)
    }
}

impl VirtualMethods for HTMLTableCellElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, can_gc: CanGc) {
        if let Some(super_type) = self.super_type() {
            super_type.attribute_mutated(attr, mutation, can_gc);
        }

        if matches!(*attr.local_name(), local_name!("colspan")) {
            self.upcast::<Node>().dirty(NodeDamage::Other);
        }
        if matches!(*attr.local_name(), local_name!("rowspan")) {
            self.upcast::<Node>().dirty(NodeDamage::Other);
        }
    }

    fn parse_plain_attribute(&self, local_name: &LocalName, value: DOMString) -> AttrValue {
        match *local_name {
            local_name!("colspan") => {
                let mut attr = AttrValue::from_u32(value.into(), DEFAULT_COLSPAN);
                if let AttrValue::UInt(_, ref mut value) = attr {
                    // From <https://html.spec.whatwg.org/multipage/#dom-tdth-colspan>:
                    // > The colSpan IDL attribute must reflect the colspan content attribute. It is clamped to
                    // > the range [1, 1000], and its default value is 1.
                    *value = (*value).clamp(1, 1000);
                }
                attr
            },
            local_name!("rowspan") => {
                let mut attr = AttrValue::from_u32(value.into(), DEFAULT_ROWSPAN);
                if let AttrValue::UInt(_, ref mut value) = attr {
                    // From <https://html.spec.whatwg.org/multipage/#dom-tdth-rowspan>:
                    // > The rowSpan IDL attribute must reflect the rowspan content attribute. It is clamped to
                    // > the range [0, 65534], and its default value is 1.
                    // Note Firefox floors by 1 in quirks mode, but like Chrome and Safari we don't do that.
                    *value = (*value).clamp(0, 65534);
                }
                attr
            },
            local_name!("bgcolor") => AttrValue::from_legacy_color(value.into()),
            local_name!("width") => AttrValue::from_nonzero_dimension(value.into()),
            local_name!("height") => AttrValue::from_nonzero_dimension(value.into()),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(local_name, value),
        }
    }
}
