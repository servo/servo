/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::RGBA;
use dom::bindings::codegen::Bindings::HTMLTableSectionElementBinding::{self, HTMLTableSectionElementMethods};
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::root::{DomRoot, LayoutDom, RootedReference};
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::{Element, RawLayoutElementHelpers};
use dom::htmlcollection::{CollectionFilter, HTMLCollection};
use dom::htmlelement::HTMLElement;
use dom::htmltablerowelement::HTMLTableRowElement;
use dom::node::{Node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use std::marker::PhantomData;
use style::attr::AttrValue;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLTableSectionElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement<TH>,
}

impl<TH: TypeHolderTrait> HTMLTableSectionElement<TH> {
    fn new_inherited(local_name: LocalName, prefix: Option<Prefix>, document: &Document<TH>)
                     -> HTMLTableSectionElement<TH> {
        HTMLTableSectionElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName, prefix: Option<Prefix>, document: &Document<TH>)
               -> DomRoot<HTMLTableSectionElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLTableSectionElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLTableSectionElementBinding::Wrap)
    }
}

#[derive(JSTraceable)]
struct RowsFilter<TH: TypeHolderTrait>(PhantomData<TH>);
impl<TH: TypeHolderTrait> CollectionFilter<TH> for RowsFilter<TH> {
    fn filter(&self, elem: &Element<TH>, root: &Node<TH>) -> bool {
        elem.is::<HTMLTableRowElement<TH>>() &&
            elem.upcast::<Node<TH>>().GetParentNode().r() == Some(root)
    }
}

impl<TH: TypeHolderTrait> HTMLTableSectionElementMethods<TH> for HTMLTableSectionElement<TH> {
    // https://html.spec.whatwg.org/multipage/#dom-tbody-rows
    fn Rows(&self) -> DomRoot<HTMLCollection<TH>> {
        HTMLCollection::create(&window_from_node(self), self.upcast(), Box::new(RowsFilter(Default::default())))
    }

    // https://html.spec.whatwg.org/multipage/#dom-tbody-insertrow
    fn InsertRow(&self, index: i32) -> Fallible<DomRoot<HTMLElement<TH>>> {
        let node = self.upcast::<Node<TH>>();
        node.insert_cell_or_row(
            index,
            || self.Rows(),
            || HTMLTableRowElement::new(local_name!("tr"), None, &node.owner_doc()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-tbody-deleterow
    fn DeleteRow(&self, index: i32) -> ErrorResult {
        let node = self.upcast::<Node<TH>>();
        node.delete_cell_or_row(
            index,
            || self.Rows(),
            |n| n.is::<HTMLTableRowElement<TH>>())
    }
}

pub trait HTMLTableSectionElementLayoutHelpers {
    fn get_background_color(&self) -> Option<RGBA>;
}

#[allow(unsafe_code)]
impl<TH: TypeHolderTrait> HTMLTableSectionElementLayoutHelpers for LayoutDom<HTMLTableSectionElement<TH>> {
    fn get_background_color(&self) -> Option<RGBA> {
        unsafe {
            (&*self.upcast::<Element<TH>>().unsafe_get())
                .get_attr_for_layout(&ns!(), &local_name!("bgcolor"))
                .and_then(AttrValue::as_color)
                .cloned()
        }
    }
}

impl<TH: TypeHolderTrait> VirtualMethods<TH> for HTMLTableSectionElement<TH> {
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
