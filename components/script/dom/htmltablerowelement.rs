/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name, ns};
use js::rust::HandleObject;
use style::attr::{AttrValue, LengthOrPercentageOrAuto};
use style::color::AbsoluteColor;

use crate::dom::bindings::codegen::Bindings::HTMLTableElementBinding::HTMLTableElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLTableRowElementBinding::HTMLTableRowElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLTableSectionElementBinding::HTMLTableSectionElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::error::{ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, LayoutDom, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{Element, LayoutElementHelpers};
use crate::dom::htmlcollection::HTMLCollection;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmltablecellelement::HTMLTableCellElement;
use crate::dom::htmltableelement::HTMLTableElement;
use crate::dom::htmltablesectionelement::HTMLTableSectionElement;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLTableRowElement {
    htmlelement: HTMLElement,
    cells: MutNullableDom<HTMLCollection>,
}

impl HTMLTableRowElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLTableRowElement {
        HTMLTableRowElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            cells: Default::default(),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLTableRowElement> {
        let n = Node::reflect_node_with_proto(
            Box::new(HTMLTableRowElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        );

        n.upcast::<Node>().set_weird_parser_insertion_mode();
        n
    }

    /// Determine the index for this `HTMLTableRowElement` within the given
    /// `HTMLCollection`. Returns `-1` if not found within collection.
    fn row_index(&self, collection: DomRoot<HTMLCollection>) -> i32 {
        collection
            .elements_iter()
            .position(|elem| (&elem as &Element) == self.upcast())
            .map_or(-1, |i| i as i32)
    }
}

impl HTMLTableRowElementMethods<crate::DomTypeHolder> for HTMLTableRowElement {
    // https://html.spec.whatwg.org/multipage/#dom-tr-bgcolor
    make_getter!(BgColor, "bgcolor");

    // https://html.spec.whatwg.org/multipage/#dom-tr-bgcolor
    make_legacy_color_setter!(SetBgColor, "bgcolor");

    // https://html.spec.whatwg.org/multipage/#dom-tr-cells
    fn Cells(&self) -> DomRoot<HTMLCollection> {
        self.cells.or_init(|| {
            HTMLCollection::new_with_filter_fn(
                &self.owner_window(),
                self.upcast(),
                |element, root| {
                    (element.is::<HTMLTableCellElement>()) &&
                        element.upcast::<Node>().GetParentNode().as_deref() == Some(root)
                },
                CanGc::note(),
            )
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-tr-insertcell
    fn InsertCell(&self, index: i32, can_gc: CanGc) -> Fallible<DomRoot<HTMLElement>> {
        let node = self.upcast::<Node>();
        node.insert_cell_or_row(
            index,
            || self.Cells(),
            || HTMLTableCellElement::new(local_name!("td"), None, &node.owner_doc(), None, can_gc),
            can_gc,
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-tr-deletecell
    fn DeleteCell(&self, index: i32) -> ErrorResult {
        let node = self.upcast::<Node>();
        node.delete_cell_or_row(
            index,
            || self.Cells(),
            |n| n.is::<HTMLTableCellElement>(),
            CanGc::note(),
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-tr-rowindex
    fn RowIndex(&self) -> i32 {
        let parent = match self.upcast::<Node>().GetParentNode() {
            Some(parent) => parent,
            None => return -1,
        };
        if let Some(table) = parent.downcast::<HTMLTableElement>() {
            return self.row_index(table.Rows());
        }
        if !parent.is::<HTMLTableSectionElement>() {
            return -1;
        }
        let grandparent = match parent.upcast::<Node>().GetParentNode() {
            Some(parent) => parent,
            None => return -1,
        };
        grandparent
            .downcast::<HTMLTableElement>()
            .map_or(-1, |table| self.row_index(table.Rows()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-tr-sectionrowindex
    fn SectionRowIndex(&self) -> i32 {
        let parent = match self.upcast::<Node>().GetParentNode() {
            Some(parent) => parent,
            None => return -1,
        };
        let collection = if let Some(table) = parent.downcast::<HTMLTableElement>() {
            table.Rows()
        } else if let Some(table_section) = parent.downcast::<HTMLTableSectionElement>() {
            table_section.Rows()
        } else {
            return -1;
        };
        self.row_index(collection)
    }
}

pub(crate) trait HTMLTableRowElementLayoutHelpers {
    fn get_background_color(self) -> Option<AbsoluteColor>;
    fn get_height(self) -> LengthOrPercentageOrAuto;
}

impl HTMLTableRowElementLayoutHelpers for LayoutDom<'_, HTMLTableRowElement> {
    fn get_background_color(self) -> Option<AbsoluteColor> {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("bgcolor"))
            .and_then(AttrValue::as_color)
            .cloned()
    }

    fn get_height(self) -> LengthOrPercentageOrAuto {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("height"))
            .map(AttrValue::as_dimension)
            .cloned()
            .unwrap_or(LengthOrPercentageOrAuto::Auto)
    }
}

impl VirtualMethods for HTMLTableRowElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn parse_plain_attribute(&self, local_name: &LocalName, value: DOMString) -> AttrValue {
        match *local_name {
            local_name!("bgcolor") => AttrValue::from_legacy_color(value.into()),
            local_name!("height") => AttrValue::from_dimension(value.into()),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(local_name, value),
        }
    }
}
