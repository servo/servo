/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::RGBA;
use dom::attr::Attr;
use dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use dom::bindings::codegen::Bindings::HTMLTableSectionElementBinding::{self, HTMLTableSectionElementMethods};
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLTableRowElementDerived, HTMLTableSectionElementDerived};
use dom::bindings::error::Error::IndexSize;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::js::{Root, RootedReference};
use dom::document::Document;
use dom::element::{AttributeMutation, Element, ElementTypeId};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlcollection::{CollectionFilter, HTMLCollection};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::htmltablerowelement::HTMLTableRowElement;
use dom::node::{Node, NodeTypeId, window_from_node};
use dom::virtualmethods::VirtualMethods;
use std::cell::Cell;
use util::str::{self, DOMString};

#[dom_struct]
pub struct HTMLTableSectionElement {
    htmlelement: HTMLElement,
    background_color: Cell<Option<RGBA>>,
}

impl HTMLTableSectionElementDerived for EventTarget {
    fn is_htmltablesectionelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableSectionElement)))
    }
}

impl HTMLTableSectionElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: &Document)
                     -> HTMLTableSectionElement {
        HTMLTableSectionElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLTableSectionElement,
                                                    localName,
                                                    prefix,
                                                    document),
            background_color: Cell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: &Document)
               -> Root<HTMLTableSectionElement> {
        let element = HTMLTableSectionElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLTableSectionElementBinding::Wrap)
    }

    pub fn get_background_color(&self) -> Option<RGBA> {
        self.background_color.get()
    }
}

#[derive(JSTraceable)]
struct RowsFilter;
impl CollectionFilter for RowsFilter {
    fn filter(&self, elem: &Element, root: &Node) -> bool {
        elem.is_htmltablerowelement()
            && NodeCast::from_ref(elem).GetParentNode().r() == Some(root)
    }
}

impl HTMLTableSectionElementMethods for HTMLTableSectionElement {
    // https://html.spec.whatwg.org/multipage/#dom-tbody-rows
    fn Rows(&self) -> Root<HTMLCollection> {
        HTMLCollection::create(&window_from_node(self), NodeCast::from_ref(self), box RowsFilter)
    }

    // https://html.spec.whatwg.org/multipage/#dom-tbody-insertrow
    fn InsertRow(&self, index: i32) -> Fallible<Root<HTMLElement>> {
        let length = self.Rows().Length();

        let index = match index {
            index if index < -1 => return Err(IndexSize),
            -1 => length,
            index => index as u32,
        };

        let self_node = NodeCast::from_ref(self);
        let tr = HTMLTableRowElement::new(String::from("tr"), None, self_node.owner_doc().r());

        if index == length {
            let tr_node = NodeCast::from_ref(tr.r());
            try!(self_node.AppendChild(tr_node));
        } else if let Some(element) = self.Rows().Item(index) {
            let tr_node = NodeCast::from_ref(tr.r());
            try!(self_node.InsertBefore(tr_node, Some(NodeCast::from_root(element)).r()));
        } else {
            return Err(IndexSize);
        }

        Ok(HTMLElementCast::from_root(tr))
    }

    // https://html.spec.whatwg.org/multipage/#dom-tbody-deleterow
    fn DeleteRow(&self, index: i32) -> ErrorResult {
        let index = match index {
            index if index < -1 => return Err(IndexSize),
            -1 => self.Rows().Length() - 1,
            index => index as u32,
        };

        let element = match self.Rows().Item(index) {
            None => return Err(IndexSize),
            Some(element) => element,
        };

        NodeCast::from_ref(element.r()).remove_self();
        Ok(())
    }
}

impl VirtualMethods for HTMLTableSectionElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &HTMLElement = HTMLElementCast::from_ref(self);
        Some(htmlelement as &VirtualMethods)
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
