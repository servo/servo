/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{local_name, namespace_url, ns, LocalName, Prefix};
use js::rust::HandleObject;
use style::attr::{AttrValue, LengthOrPercentageOrAuto};
use style::color::AbsoluteColor;
use style::context::QuirksMode;

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

const DEFAULT_COLSPAN: u32 = 1;
const DEFAULT_ROWSPAN: u32 = 1;

#[dom_struct]
pub struct HTMLTableCellElement {
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

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLTableCellElement> {
        let n = Node::reflect_node_with_proto(
            Box::new(HTMLTableCellElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        );

        n.upcast::<Node>().set_weird_parser_insertion_mode();
        n
    }
}

impl HTMLTableCellElementMethods for HTMLTableCellElement {
    // https://html.spec.whatwg.org/multipage/#dom-tdth-colspan
    make_uint_getter!(ColSpan, "colspan", DEFAULT_COLSPAN);

    // https://html.spec.whatwg.org/multipage/#dom-tdth-colspan
    make_uint_setter!(SetColSpan, "colspan", DEFAULT_COLSPAN);

    // https://html.spec.whatwg.org/multipage/#dom-tdth-rowspan
    make_uint_getter!(RowSpan, "rowspan", DEFAULT_ROWSPAN);

    // https://html.spec.whatwg.org/multipage/#dom-tdth-rowspan
    make_uint_setter!(SetRowSpan, "rowspan", DEFAULT_ROWSPAN);

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

pub trait HTMLTableCellElementLayoutHelpers<'dom> {
    fn get_background_color(self) -> Option<AbsoluteColor>;
    fn get_colspan(self) -> Option<u32>;
    fn get_rowspan(self) -> Option<u32>;
    fn get_table(self) -> Option<LayoutDom<'dom, HTMLTableElement>>;
    fn get_width(self) -> LengthOrPercentageOrAuto;
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
}

impl VirtualMethods for HTMLTableCellElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn parse_plain_attribute(&self, local_name: &LocalName, value: DOMString) -> AttrValue {
        match *local_name {
            local_name!("colspan") => {
                let mut attr = AttrValue::from_u32(value.into(), DEFAULT_COLSPAN);
                if let AttrValue::UInt(_, ref mut val) = attr {
                    if *val == 0 {
                        *val = 1;
                    }
                }
                attr
            },
            local_name!("rowspan") => {
                let mut attr = AttrValue::from_u32(value.into(), DEFAULT_ROWSPAN);
                if let AttrValue::UInt(_, ref mut val) = attr {
                    if *val == 0 {
                        let node = self.upcast::<Node>();
                        let doc = node.owner_doc();
                        // rowspan = 0 is not supported in quirks mode
                        if doc.quirks_mode() != QuirksMode::NoQuirks {
                            *val = 1;
                        }
                    }
                }
                attr
            },
            local_name!("bgcolor") => AttrValue::from_legacy_color(value.into()),
            local_name!("width") => AttrValue::from_nonzero_dimension(value.into()),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(local_name, value),
        }
    }
}
