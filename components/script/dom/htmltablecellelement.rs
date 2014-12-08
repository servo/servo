/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrHelpers};
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLTableCellElementDerived};
use dom::bindings::js::JSRef;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::{ElementTypeId, HTMLTableDataCellElementTypeId};
use dom::element::{HTMLTableHeaderCellElementTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::ElementNodeTypeId;
use dom::virtualmethods::VirtualMethods;

use servo_util::str::{AutoLpa, DOMString, LengthOrPercentageOrAuto};
use servo_util::str;
use std::cell::Cell;

#[dom_struct]
pub struct HTMLTableCellElement {
    htmlelement: HTMLElement,
    border: Cell<Option<u32>>,
    width: Cell<LengthOrPercentageOrAuto>,
}

impl HTMLTableCellElementDerived for EventTarget {
    fn is_htmltablecellelement(&self) -> bool {
        match *self.type_id() {
            NodeTargetTypeId(ElementNodeTypeId(HTMLTableDataCellElementTypeId)) |
            NodeTargetTypeId(ElementNodeTypeId(HTMLTableHeaderCellElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLTableCellElement {
    pub fn new_inherited(type_id: ElementTypeId,
                         tag_name: DOMString,
                         prefix: Option<DOMString>,
                         document: JSRef<Document>)
                         -> HTMLTableCellElement {
        HTMLTableCellElement {
            htmlelement: HTMLElement::new_inherited(type_id, tag_name, prefix, document),
            border: Cell::new(None),
            width: Cell::new(AutoLpa),
        }
    }

    #[inline]
    pub fn htmlelement(&self) -> &HTMLElement {
        &self.htmlelement
    }
}

pub trait HTMLTableCellElementHelpers {
    fn get_border(&self) -> Option<u32>;
    fn get_width(&self) -> LengthOrPercentageOrAuto;
}

impl HTMLTableCellElementHelpers for HTMLTableCellElement {
    fn get_border(&self) -> Option<u32> {
        self.border.get()
    }

    fn get_width(&self) -> LengthOrPercentageOrAuto {
        self.width.get()
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLTableCellElement> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(attr),
            _ => ()
        }

        match attr.local_name() {
            &atom!("border") => {
                // According to HTML5 § 14.3.9, invalid values map to 1px.
                self.border.set(Some(str::parse_unsigned_integer(attr.value()
                                                                     .as_slice()
                                                                     .chars()).unwrap_or(1)))
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
            &atom!("border") => self.border.set(None),
            &atom!("width") => self.width.set(AutoLpa),
            _ => ()
        }
    }
}

impl Reflectable for HTMLTableCellElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
