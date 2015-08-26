/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrHelpers};
use dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use dom::bindings::codegen::Bindings::HTMLTableRowElementBinding;
use dom::bindings::codegen::Bindings::HTMLTableRowElementBinding::HTMLTableRowElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::HTMLElementCast;
use dom::bindings::codegen::InheritTypes::HTMLTableDataCellElementDerived;
use dom::bindings::codegen::InheritTypes::HTMLTableHeaderCellElementDerived;
use dom::bindings::codegen::InheritTypes::HTMLTableRowElementDerived;
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::error::Error::IndexSize;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::js::{JS, MutNullableHeap, Root, RootedReference};
use dom::document::Document;
use dom::element::{Element, ElementTypeId};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlcollection::{CollectionFilter, HTMLCollection};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::htmltabledatacellelement::HTMLTableDataCellElement;
use dom::node::{Node, NodeHelpers, NodeTypeId, window_from_node};
use dom::virtualmethods::VirtualMethods;

use cssparser::RGBA;
use std::cell::Cell;
use util::str::{self, DOMString};

#[derive(JSTraceable)]
struct CellsFilter;
impl CollectionFilter for CellsFilter {
    fn filter(&self, elem: &Element, root: &Node) -> bool {
        (elem.is_htmltableheadercellelement() || elem.is_htmltabledatacellelement())
            && NodeCast::from_ref(elem).GetParentNode().r() == Some(root)
    }
}

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct HTMLTableRowElement {
    htmlelement: HTMLElement,
    cells: MutNullableHeap<JS<HTMLCollection>>,
    background_color: Cell<Option<RGBA>>,
}

impl HTMLTableRowElementDerived for EventTarget {
    fn is_htmltablerowelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableRowElement)))
    }
}

impl HTMLTableRowElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: &Document)
                     -> HTMLTableRowElement {
        HTMLTableRowElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLTableRowElement,
                                                    localName,
                                                    prefix,
                                                    document),
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
}


pub trait HTMLTableRowElementHelpers {
    fn get_background_color(self) -> Option<RGBA>;
}

impl<'a> HTMLTableRowElementHelpers for &'a HTMLTableRowElement {
    fn get_background_color(self) -> Option<RGBA> {
        self.background_color.get()
    }
}

impl <'a> HTMLTableRowElementMethods for &'a HTMLTableRowElement {
    // https://html.spec.whatwg.org/multipage/#dom-tr-cells
    fn Cells(self) -> Root<HTMLCollection> {
        self.cells.or_init(|| {
            let window = window_from_node(self);
            let filter = box CellsFilter;
            HTMLCollection::create(window.r(), NodeCast::from_ref(self), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-tr-insertcell
    fn InsertCell(self, index: i32) -> Fallible<Root<HTMLElement>> {
        if index < -1 || index > self.Cells().Length() as i32 {
            return Err(IndexSize);
        }

        let this = NodeCast::from_ref(self);
        let td = HTMLTableDataCellElement::new("td".to_owned(), None, this.owner_doc().r());
        if index == -1 || index == self.Cells().Length() as i32 {
            try!(this.AppendChild(NodeCast::from_ref(td.r())));
        } else {
            let reference = NodeCast::from_root(self.Cells().Item(index as u32).unwrap());
            try!(this.InsertBefore(NodeCast::from_ref(td.r()), Some(reference.r())));
        }
        Ok(HTMLElementCast::from_root(td))
    }

    // https://html.spec.whatwg.org/multipage/#dom-tr-deletecell
    fn DeleteCell(self, index: i32) -> ErrorResult {
        if index < -1 || index >= self.Cells().Length() as i32 {
            return Err(IndexSize);
        }

        let index = if index == -1 {
            self.Cells().Length() as i32 - 1
        } else {
            index
        };

        NodeCast::from_ref(self.Cells().Item(index as u32).unwrap().r()).remove_self();
        Ok(())
    }
}

impl<'a> VirtualMethods for &'a HTMLTableRowElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &&HTMLElement = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: &Attr) {
        if let Some(ref s) = self.super_type() {
            s.after_set_attr(attr);
        }

        match attr.local_name() {
            &atom!("bgcolor") => {
                self.background_color.set(str::parse_legacy_color(&attr.value()).ok());
            },
            _ => ()
        }
    }

    fn before_remove_attr(&self, attr: &Attr) {
        if let Some(ref s) = self.super_type() {
            s.before_remove_attr(attr);
        }

        match attr.local_name() {
            &atom!("bgcolor") => {
                self.background_color.set(None);
            },
            _ => ()
        }
    }
}
