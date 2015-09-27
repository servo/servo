/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::RGBA;
use dom::attr::{Attr, AttrValue};
use dom::bindings::codegen::Bindings::HTMLTableCellElementBinding::HTMLTableCellElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLElementTypeId};
use dom::bindings::codegen::InheritTypes::{HTMLTableCellElementDerived, HTMLTableCellElementTypeId};
use dom::bindings::codegen::InheritTypes::{HTMLTableRowElementDerived, NodeCast};
use dom::bindings::js::LayoutJS;
use dom::document::Document;
use dom::element::AttributeMutation;
use dom::htmlelement::HTMLElement;
use dom::virtualmethods::VirtualMethods;
use std::cell::Cell;
use std::cmp::max;
use string_cache::Atom;
use util::str::{self, DOMString, LengthOrPercentageOrAuto};

const DEFAULT_COLSPAN: u32 = 1;

#[dom_struct]
pub struct HTMLTableCellElement {
    htmlelement: HTMLElement,
    background_color: Cell<Option<RGBA>>,
    colspan: Cell<Option<u32>>,
    width: Cell<LengthOrPercentageOrAuto>,
}

impl HTMLTableCellElement {
    pub fn new_inherited(type_id: HTMLTableCellElementTypeId,
                         tag_name: DOMString,
                         prefix: Option<DOMString>,
                         document: &Document)
                         -> HTMLTableCellElement {
        HTMLTableCellElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLTableCellElement(type_id),
                                                    tag_name, prefix, document),
            background_color: Cell::new(None),
            colspan: Cell::new(None),
            width: Cell::new(LengthOrPercentageOrAuto::Auto),
        }
    }

    #[inline]
    pub fn htmlelement(&self) -> &HTMLElement {
        &self.htmlelement
    }
}

impl HTMLTableCellElementMethods for HTMLTableCellElement {
    // https://html.spec.whatwg.org/multipage/#dom-tdth-colspan
    make_uint_getter!(ColSpan, "colspan", DEFAULT_COLSPAN);

    // https://html.spec.whatwg.org/multipage/#dom-tdth-colspan
    make_uint_setter!(SetColSpan, "colspan");

    // https://html.spec.whatwg.org/multipage/#dom-tdth-cellindex
    fn CellIndex(&self) -> i32 {
        let self_node = NodeCast::from_ref(self);

        let parent_children = match self_node.GetParentNode() {
            Some(ref parent_node) if parent_node.is_htmltablerowelement() => parent_node.children(),
            _ => return -1,
        };

        parent_children.filter(|c| c.is_htmltablecellelement())
                       .position(|c| c.r() == self_node)
                       .map(|p| p as i32).unwrap_or(-1)
    }
}


pub trait HTMLTableCellElementLayoutHelpers {
    fn get_background_color(&self) -> Option<RGBA>;
    fn get_colspan(&self) -> Option<u32>;
    fn get_width(&self) -> LengthOrPercentageOrAuto;
}

#[allow(unsafe_code)]
impl HTMLTableCellElementLayoutHelpers for LayoutJS<HTMLTableCellElement> {
    fn get_background_color(&self) -> Option<RGBA> {
        unsafe {
            (*self.unsafe_get()).background_color.get()
        }
    }

    fn get_colspan(&self) -> Option<u32> {
        unsafe {
            (*self.unsafe_get()).colspan.get()
        }
    }

    fn get_width(&self) -> LengthOrPercentageOrAuto {
        unsafe {
            (*self.unsafe_get()).width.get()
        }
    }
}

impl VirtualMethods for HTMLTableCellElement {
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
            &atom!(colspan) => {
                self.colspan.set(mutation.new_value(attr).map(|value| {
                    max(DEFAULT_COLSPAN, value.as_uint())
                }));
            },
            &atom!(width) => {
                let width = mutation.new_value(attr).map(|value| {
                    str::parse_length(&value)
                });
                self.width.set(width.unwrap_or(LengthOrPercentageOrAuto::Auto));
            },
            _ => {},
        }
    }

    fn parse_plain_attribute(&self, local_name: &Atom, value: DOMString) -> AttrValue {
        match local_name {
            &atom!("colspan") => AttrValue::from_u32(value, DEFAULT_COLSPAN),
            _ => self.super_type().unwrap().parse_plain_attribute(local_name, value),
        }
    }
}
