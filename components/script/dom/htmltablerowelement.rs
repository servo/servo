/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::RGBA;
use dom::attr::Attr;
use dom::bindings::codegen::Bindings::HTMLTableRowElementBinding::{self, HTMLTableRowElementMethods};
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::conversions::Castable;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::js::{JS, MutNullableHeap, Root, RootedReference};
use dom::document::Document;
use dom::element::{AttributeMutation, Element};
use dom::htmlcollection::{CollectionFilter, HTMLCollection};
use dom::htmlelement::HTMLElement;
use dom::htmltabledatacellelement::HTMLTableDataCellElement;
use dom::htmltableheadercellelement::HTMLTableHeaderCellElement;
use dom::node::{Node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use std::cell::Cell;
use util::str::{self, DOMString};


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
    background_color: Cell<Option<RGBA>>,
}

impl HTMLTableRowElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: &Document)
                     -> HTMLTableRowElement {
        HTMLTableRowElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document),
            cells: Default::default(),
            background_color: Cell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: &Document)
               -> Root<HTMLTableRowElement> {
        Node::reflect_node(box HTMLTableRowElement::new_inherited(localName, prefix, document),
                           document,
                           HTMLTableRowElementBinding::Wrap)
    }

    pub fn get_background_color(&self) -> Option<RGBA> {
        self.background_color.get()
    }
}

impl HTMLTableRowElementMethods for HTMLTableRowElement {
    // https://html.spec.whatwg.org/multipage/#dom-tr-bgcolor
    make_getter!(BgColor);

    // https://html.spec.whatwg.org/multipage/#dom-tr-bgcolor
    make_setter!(SetBgColor, "bgcolor");

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
            || HTMLTableDataCellElement::new("td".to_owned(), None, node.owner_doc().r()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-tr-deletecell
    fn DeleteCell(&self, index: i32) -> ErrorResult {
        let node = self.upcast::<Node>();
        node.delete_cell_or_row(
            index,
            || self.Cells(),
            |n| n.is::<HTMLTableDataCellElement>())
    }
}

impl VirtualMethods for HTMLTableRowElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &atom!(bgcolor) => {
                self.background_color.set(mutation.new_value(attr).and_then(|value| {
                    str::parse_legacy_color(&value).ok()
                }));
            },
            _ => {},
        }
    }
}
