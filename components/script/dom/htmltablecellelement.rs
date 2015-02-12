/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrHelpers};
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLTableCellElementDerived};
use dom::bindings::js::JSRef;
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::NodeTypeId;
use dom::virtualmethods::VirtualMethods;

use cssparser::RGBA;
use util::str::{self, DOMString, LengthOrPercentageOrAuto};
use std::cell::Cell;

#[derive(Copy, PartialEq, Debug)]
#[jstraceable]
pub enum HTMLTableCellElementTypeId {
    HTMLTableDataCellElement,
    HTMLTableHeaderCellElement,
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
            EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableCellElement(_)))) => true,
            _ => false
        }
    }
}

impl HTMLTableCellElement {
    pub fn new_inherited(type_id: HTMLTableCellElementTypeId,
                         tag_name: DOMString,
                         prefix: Option<DOMString>,
                         document: JSRef<Document>)
                         -> HTMLTableCellElement {
        HTMLTableCellElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLTableCellElement(type_id), tag_name, prefix, document),
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

pub trait HTMLTableCellElementHelpers {
    fn get_background_color(&self) -> Option<RGBA>;
    fn get_colspan(&self) -> Option<u32>;
    fn get_width(&self) -> LengthOrPercentageOrAuto;
}

impl HTMLTableCellElementHelpers for HTMLTableCellElement {
    fn get_background_color(&self) -> Option<RGBA> {
        self.background_color.get()
    }

    fn get_colspan(&self) -> Option<u32> {
        self.colspan.get()
    }

    fn get_width(&self) -> LengthOrPercentageOrAuto {
        self.width.get()
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLTableCellElement> {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(attr),
            _ => ()
        }

        match attr.local_name() {
            &atom!("bgcolor") => {
                self.background_color.set(str::parse_legacy_color(attr.value().as_slice()).ok())
            }
            &atom!("colspan") => {
                self.colspan.set(str::parse_unsigned_integer(attr.value().as_slice().chars()));
            }
            &atom!("width") => self.width.set(str::parse_length(attr.value().as_slice())),
            _ => ()
        }
    }

    fn before_remove_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.before_remove_attr(attr),
            _ => ()
        }

        match attr.local_name() {
            &atom!("bgcolor") => self.background_color.set(None),
            &atom!("colspan") => self.colspan.set(None),
            &atom!("width") => self.width.set(LengthOrPercentageOrAuto::Auto),
            _ => ()
        }
    }
}

