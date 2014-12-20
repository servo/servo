/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrHelpers};
use dom::bindings::codegen::Bindings::HTMLTableSectionElementBinding;
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLTableSectionElementDerived};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, NodeTypeId};
use dom::virtualmethods::VirtualMethods;

use cssparser::RGBA;
use servo_util::str::{mod, DOMString};
use std::cell::Cell;

#[dom_struct]
pub struct HTMLTableSectionElement {
    htmlelement: HTMLElement,
    background_color: Cell<Option<RGBA>>,
}

impl HTMLTableSectionElementDerived for EventTarget {
    fn is_htmltablesectionelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLTableSectionElement))
    }
}

impl HTMLTableSectionElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>)
                     -> HTMLTableSectionElement {
        HTMLTableSectionElement {
            htmlelement: HTMLElement::new_inherited(ElementTypeId::HTMLTableSectionElement,
                                                    localName,
                                                    prefix,
                                                    document),
            background_color: Cell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>)
               -> Temporary<HTMLTableSectionElement> {
        let element = HTMLTableSectionElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLTableSectionElementBinding::Wrap)
    }
}

impl Reflectable for HTMLTableSectionElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}

pub trait HTMLTableSectionElementHelpers {
    fn get_background_color(&self) -> Option<RGBA>;
}

impl HTMLTableSectionElementHelpers for HTMLTableSectionElement {
    fn get_background_color(&self) -> Option<RGBA> {
        self.background_color.get()
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLTableSectionElement> {
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

