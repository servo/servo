/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrValue};
use dom::bindings::codegen::Bindings::HTMLTableCellElementBinding::HTMLTableCellElementMethods;
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLTableCellElementDerived};
use dom::document::Document;
use dom::element::{AttributeMutation, ElementTypeId};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::NodeTypeId;
use dom::virtualmethods::VirtualMethods;

use util::str::{self, DOMString, LengthOrPercentageOrAuto};

use cssparser::RGBA;
use string_cache::Atom;

use std::cell::Cell;
use std::cmp::max;

const DEFAULT_COLSPAN: u32 = 1;

#[derive(Copy, Clone, Debug)]
pub enum HTMLTableCellElementTypeId {
    HTMLTableDataCellElement = 0,
    HTMLTableHeaderCellElement = 1,
}

impl PartialEq for HTMLTableCellElementTypeId {
    #[inline]
    fn eq(&self, other: &HTMLTableCellElementTypeId) -> bool {
        (*self as u8) == (*other as u8)
    }
}

#[dom_struct]
pub struct HTMLTableCellElement {
    htmlelement: HTMLElement,
    background_color: Cell<Option<RGBA>>,
    colspan: Cell<Option<u32>>,
    width: Cell<LengthOrPercentageOrAuto>,
}

impl HTMLTableCellElementDerived for EventTarget {
    fn is_htmltablecellelement(&self) -> bool {
        match *self.type_id() {
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableCellElement(_)))) => true,
            _ => false
        }
    }
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
}


impl HTMLTableCellElement {
    pub fn get_background_color(&self) -> Option<RGBA> {
        self.background_color.get()
    }

    pub fn get_colspan(&self) -> Option<u32> {
        self.colspan.get()
    }

    pub fn get_width(&self) -> LengthOrPercentageOrAuto {
        self.width.get()
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
