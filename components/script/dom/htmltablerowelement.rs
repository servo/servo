/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::RGBA;
use dom::attr::AttrValue;
use dom::bindings::codegen::Bindings::HTMLTableRowElementBinding::{self, HTMLTableRowElementMethods};
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, LayoutJS, MutNullableHeap, Root, RootedReference};
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
use string_cache::Atom;
use util::str::DOMString;


#[derive(JSTraceable)]
struct CellsFilter;
impl CollectionFilter for CellsFilter {
    fn filter(&self, elem: &Element, root: &Node) -> bool {
        (elem.is::<HTMLTableHeaderCellElement>() || elem.is::<HTMLTableDataCellElement>())
            && elem.upcast::<Node>().GetParentNode().r() == Some(root)
    }
}

#[dom_struct]
pub struct HTMLTableRowElement {
    htmlelement: HTMLElement,
    cells: MutNullableHeap<JS<HTMLCollection>>,
}

impl HTMLTableRowElement {
    fn new_inherited(localName: Atom, prefix: Option<DOMString>, document: &Document)
                     -> HTMLTableRowElement {
        HTMLTableRowElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document),
            cells: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom, prefix: Option<DOMString>, document: &Document)
               -> Root<HTMLTableRowElement> {
        Node::reflect_node(box HTMLTableRowElement::new_inherited(localName, prefix, document),
                           document,
                           HTMLTableRowElementBinding::Wrap)
    }
}

impl HTMLTableRowElementMethods for HTMLTableRowElement {
    // https://html.spec.whatwg.org/multipage/#dom-tr-bgcolor
    make_getter!(BgColor, "bgcolor");

    // https://html.spec.whatwg.org/multipage/#dom-tr-bgcolor
    make_legacy_color_setter!(SetBgColor, "bgcolor");

    // https://html.spec.whatwg.org/multipage/#dom-tr-cells
    fn Cells(&self) -> Root<HTMLCollection> {
        self.cells.or_init(|| {
            let window = window_from_node(self);
            let filter = box CellsFilter;
            HTMLCollection::create(window.r(), self.upcast(), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-tr-insertcell
    fn InsertCell(&self, index: i32) -> Fallible<Root<HTMLElement>> {
        let node = self.upcast::<Node>();
        node.insert_cell_or_row(
            index,
            || self.Cells(),
            || HTMLTableDataCellElement::new(atom!("td"), None, node.owner_doc().r()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-tr-deletecell
    fn DeleteCell(&self, index: i32) -> ErrorResult {
        let node = self.upcast::<Node>();
        node.delete_cell_or_row(
            index,
            || self.Cells(),
            |n| n.is::<HTMLTableDataCellElement>())
    }

    // https://html.spec.whatwg.org/multipage/#dom-tr-rowindex
    fn RowIndex(&self) -> i32 {
        let parent = match self.upcast::<Node>().GetParentNode() {
            Some(parent) => parent,
            None => return -1,
        };
        if let Some(table) = parent.downcast::<HTMLTableElement>() {
            return table.row_index(self).map_or(-1, |i| i as i32);
        }
        if !parent.is::<HTMLTableSectionElement>() {
            return -1;
        }
        let grandparent = match parent.upcast::<Node>().GetParentNode() {
            Some(parent) => parent,
            None => return -1,
        };
        grandparent.downcast::<HTMLTableElement>()
                   .and_then(|table| table.row_index(self))
                   .map_or(-1, |i| i as i32)
    }
}

pub trait HTMLTableRowElementLayoutHelpers {
    fn get_background_color(&self) -> Option<RGBA>;
}

#[allow(unsafe_code)]
impl HTMLTableRowElementLayoutHelpers for LayoutJS<HTMLTableRowElement> {
    fn get_background_color(&self) -> Option<RGBA> {
        unsafe {
            (&*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &atom!("bgcolor"))
                .and_then(AttrValue::as_color)
                .cloned()
        }
    }
}

impl VirtualMethods for HTMLTableRowElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn parse_plain_attribute(&self, local_name: &Atom, value: DOMString) -> AttrValue {
        match *local_name {
            atom!("bgcolor") => AttrValue::from_legacy_color(value),
            _ => self.super_type().unwrap().parse_plain_attribute(local_name, value),
        }
    }
}
