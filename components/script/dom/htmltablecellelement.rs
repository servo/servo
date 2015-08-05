/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrHelpers, AttrValue};
use dom::bindings::codegen::Bindings::HTMLTableCellElementBinding::HTMLTableCellElementMethods;
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLTableCellElementDerived};
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::NodeTypeId;
use dom::virtualmethods::VirtualMethods;

use util::str::{self, DOMString, LengthOrPercentageOrAuto};

use cssparser::RGBA;
use string_cache::Atom;

use std::cell::Cell;
use std::cmp::max;

const DEFAULT_COLSPAN: u32 = 1;

#[derive(JSTraceable, Copy, Clone, Debug, HeapSizeOf)]
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
#[derive(HeapSizeOf)]
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

impl<'a> HTMLTableCellElementMethods for &'a HTMLTableCellElement {
    // https://html.spec.whatwg.org/multipage/#dom-tdth-colspan
    make_uint_getter!(ColSpan, "colspan", DEFAULT_COLSPAN);
    make_uint_setter!(SetColSpan, "colspan");
}

pub trait HTMLTableCellElementHelpers {
    fn get_background_color(self) -> Option<RGBA>;
    fn get_colspan(self) -> Option<u32>;
    fn get_width(self) -> LengthOrPercentageOrAuto;
}

impl<'a> HTMLTableCellElementHelpers for &'a HTMLTableCellElement {
    fn get_background_color(self) -> Option<RGBA> {
        self.background_color.get()
    }

    fn get_colspan(self) -> Option<u32> {
        self.colspan.get()
    }

    fn get_width(self) -> LengthOrPercentageOrAuto {
        self.width.get()
    }
}

impl<'a> VirtualMethods for &'a HTMLTableCellElement {
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
                self.background_color.set(str::parse_legacy_color(&attr.value()).ok())
            }
            &atom!("colspan") => {
                match *attr.value() {
                    AttrValue::UInt(_, colspan) => {
                        self.colspan.set(Some(max(DEFAULT_COLSPAN, colspan)))
                    },
                    _ => unreachable!(),
                }
            },
            &atom!("width") => self.width.set(str::parse_length(&attr.value())),
            _ => ()
        }
    }

    fn before_remove_attr(&self, attr: &Attr) {
        if let Some(ref s) = self.super_type() {
            s.before_remove_attr(attr);
        }

        match attr.local_name() {
            &atom!("bgcolor") => self.background_color.set(None),
            &atom!("colspan") => self.colspan.set(None),
            &atom!("width") => self.width.set(LengthOrPercentageOrAuto::Auto),
            _ => ()
        }
    }

    fn parse_plain_attribute(&self, local_name: &Atom, value: DOMString) -> AttrValue {
        match local_name {
            &atom!("colspan") => AttrValue::from_u32(value, DEFAULT_COLSPAN),
            _ => self.super_type().unwrap().parse_plain_attribute(local_name, value),
        }
    }
}

