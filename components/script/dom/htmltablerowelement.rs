/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::RGBA;
use dom::bindings::codegen::Bindings::HTMLTableElementBinding::HTMLTableElementMethods;
use dom::bindings::codegen::Bindings::HTMLTableRowElementBinding::{self, HTMLTableRowElementMethods};
use dom::bindings::codegen::Bindings::HTMLTableSectionElementBinding::HTMLTableSectionElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::root::{DomRoot, LayoutDom, MutNullableDom, RootedReference};
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::{Element, RawLayoutElementHelpers};
use dom::htmlcollection::{CollectionFilter, HTMLCollection};
use dom::htmlelement::HTMLElement;
use dom::htmltabledatacellelement::HTMLTableDataCellElement;
use dom::htmltableelement::HTMLTableElement;
use dom::htmltableheadercellelement::HTMLTableHeaderCellElement;
use dom::htmltablesectionelement::HTMLTableSectionElement;
use dom::node::{Node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use std::marker::PhantomData;
use style::attr::AttrValue;
use typeholder::TypeHolderTrait;

#[derive(JSTraceable)]
struct CellsFilter<TH: TypeHolderTrait>(PhantomData<TH>);
impl<TH: TypeHolderTrait> CollectionFilter<TH> for CellsFilter<TH> {
    fn filter(&self, elem: &Element<TH>, root: &Node<TH>) -> bool {
        (elem.is::<HTMLTableHeaderCellElement<TH>>() || elem.is::<HTMLTableDataCellElement<TH>>()) &&
            elem.upcast::<Node<TH>>().GetParentNode().r() == Some(root)
    }
}

#[dom_struct]
pub struct HTMLTableRowElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement<TH>,
    cells: MutNullableDom<HTMLCollection<TH>>,
}

impl<TH: TypeHolderTrait> HTMLTableRowElement<TH> {
    fn new_inherited(local_name: LocalName, prefix: Option<Prefix>, document: &Document<TH>)
                     -> HTMLTableRowElement<TH> {
        HTMLTableRowElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            cells: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName, prefix: Option<Prefix>, document: &Document<TH>)
               -> DomRoot<HTMLTableRowElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLTableRowElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLTableRowElementBinding::Wrap)
    }

    /// Determine the index for this `HTMLTableRowElement` within the given
    /// `HTMLCollection`. Returns `-1` if not found within collection.
    fn row_index(&self, collection: DomRoot<HTMLCollection<TH>>) -> i32 {
        collection.elements_iter()
                  .position(|elem| (&elem as &Element<TH>) == self.upcast())
                  .map_or(-1, |i| i as i32)
    }
}

impl<TH: TypeHolderTrait> HTMLTableRowElementMethods<TH> for HTMLTableRowElement<TH> {
    // https://html.spec.whatwg.org/multipage/#dom-tr-bgcolor
    make_getter!(BgColor, "bgcolor");

    // https://html.spec.whatwg.org/multipage/#dom-tr-bgcolor
    make_legacy_color_setter!(SetBgColor, "bgcolor");

    // https://html.spec.whatwg.org/multipage/#dom-tr-cells
    fn Cells(&self) -> DomRoot<HTMLCollection<TH>> {
        self.cells.or_init(|| {
            let window = window_from_node(self);
            let filter = Box::new(CellsFilter(Default::default()));
            HTMLCollection::create(&window, self.upcast(), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-tr-insertcell
    fn InsertCell(&self, index: i32) -> Fallible<DomRoot<HTMLElement<TH>>> {
        let node = self.upcast::<Node<TH>>();
        node.insert_cell_or_row(
            index,
            || self.Cells(),
            || HTMLTableDataCellElement::new(local_name!("td"), None, &node.owner_doc()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-tr-deletecell
    fn DeleteCell(&self, index: i32) -> ErrorResult {
        let node = self.upcast::<Node<TH>>();
        node.delete_cell_or_row(
            index,
            || self.Cells(),
            |n| n.is::<HTMLTableDataCellElement<TH>>())
    }

    // https://html.spec.whatwg.org/multipage/#dom-tr-rowindex
    fn RowIndex(&self) -> i32 {
        let parent = match self.upcast::<Node<TH>>().GetParentNode() {
            Some(parent) => parent,
            None => return -1,
        };
        if let Some(table) = parent.downcast::<HTMLTableElement<TH>>() {
            return self.row_index(table.Rows());
        }
        if !parent.is::<HTMLTableSectionElement<TH>>() {
            return -1;
        }
        let grandparent = match parent.upcast::<Node<TH>>().GetParentNode() {
            Some(parent) => parent,
            None => return -1,
        };
        grandparent.downcast::<HTMLTableElement<TH>>()
                   .map_or(-1, |table| self.row_index(table.Rows()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-tr-sectionrowindex
    fn SectionRowIndex(&self) -> i32 {
        let parent = match self.upcast::<Node<TH>>().GetParentNode() {
            Some(parent) => parent,
            None => return -1,
        };
        let collection = if let Some(table) = parent.downcast::<HTMLTableElement<TH>>() {
            table.Rows()
        } else if let Some(table_section) = parent.downcast::<HTMLTableSectionElement<TH>>() {
            table_section.Rows()
        } else {
            return -1;
        };
        self.row_index(collection)
    }
}

pub trait HTMLTableRowElementLayoutHelpers {
    fn get_background_color(&self) -> Option<RGBA>;
}

#[allow(unsafe_code)]
impl<TH: TypeHolderTrait> HTMLTableRowElementLayoutHelpers for LayoutDom<HTMLTableRowElement<TH>> {
    fn get_background_color(&self) -> Option<RGBA> {
        unsafe {
            (&*self.upcast::<Element<TH>>().unsafe_get())
                .get_attr_for_layout(&ns!(), &local_name!("bgcolor"))
                .and_then(AttrValue::as_color)
                .cloned()
        }
    }
}

impl<TH: TypeHolderTrait> VirtualMethods<TH> for HTMLTableRowElement<TH> {
    fn super_type(&self) -> Option<&VirtualMethods<TH>> {
        Some(self.upcast::<HTMLElement<TH>>() as &VirtualMethods<TH>)
    }

    fn parse_plain_attribute(&self, local_name: &LocalName, value: DOMString) -> AttrValue {
        match *local_name {
            local_name!("bgcolor") => AttrValue::from_legacy_color(value.into()),
            _ => self.super_type().unwrap().parse_plain_attribute(local_name, value),
        }
    }
}
