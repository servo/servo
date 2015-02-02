/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrHelpers};
use dom::bindings::codegen::Bindings::HTMLTableRowElementBinding;
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLTableRowElementDerived};
use dom::bindings::js::{JSRef, Temporary};
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId};
use dom::virtualmethods::VirtualMethods;

use cssparser::RGBA;
use util::str::{self, DOMString};
use std::cell::Cell;

#[dom_struct]
pub struct HTMLTableRowElement {
    htmlelement: HTMLElement,
    background_color: Cell<Option<RGBA>>,
}

impl HTMLTableRowElementDerived for EventTarget {
    fn is_htmltablerowelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableRowElement)))
    }
}

impl HTMLTableRowElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>)
                     -> HTMLTableRowElement {
        HTMLTableRowElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLTableRowElement,
                                                    localName,
                                                    prefix,
                                                    document),
            background_color: Cell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>)
               -> Temporary<HTMLTableRowElement> {
        Node::reflect_node(box HTMLTableRowElement::new_inherited(localName, prefix, document),
                           document,
                           HTMLTableRowElementBinding::Wrap)
    }
}


pub trait HTMLTableRowElementHelpers {
    fn get_background_color(&self) -> Option<RGBA>;
}

impl HTMLTableRowElementHelpers for HTMLTableRowElement {
    fn get_background_color(&self) -> Option<RGBA> {
        self.background_color.get()
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLTableRowElement> {
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
            _ => {}
        }
    }

    fn before_remove_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.before_remove_attr(attr),
            _ => ()
        }

        match attr.local_name() {
            &atom!("bgcolor") => self.background_color.set(None),
            _ => {}
        }
    }
}
