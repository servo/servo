/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::HTMLTableSectionElementBinding;
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLTableSectionElementDerived};
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId};
use dom::virtualmethods::VirtualMethods;

use cssparser::RGBA;
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
}


impl HTMLTableSectionElement {
    pub fn get_background_color(&self) -> Option<RGBA> {
        self.background_color.get()
    }
}

impl VirtualMethods for HTMLTableSectionElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &HTMLElement = HTMLElementCast::from_ref(self);
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
