/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrHelpers};
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::HTMLBodyElementBinding::{self, HTMLBodyElementMethods};
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::EventTargetCast;
use dom::bindings::codegen::InheritTypes::{HTMLBodyElementDerived, HTMLElementCast};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::Reflectable;
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId, EventTargetHelpers};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId, window_from_node};
use dom::virtualmethods::VirtualMethods;

use cssparser::RGBA;
use util::str::{self, DOMString};

use std::borrow::ToOwned;
use std::cell::Cell;

#[dom_struct]
pub struct HTMLBodyElement {
    htmlelement: HTMLElement,
    background_color: Cell<Option<RGBA>>,
}

impl HTMLBodyElementDerived for EventTarget {
    fn is_htmlbodyelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBodyElement)))
    }
}

impl HTMLBodyElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>)
                     -> HTMLBodyElement {
        HTMLBodyElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLBodyElement,
                                                    localName,
                                                    prefix,
                                                    document),
            background_color: Cell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>)
               -> Temporary<HTMLBodyElement> {
        let element = HTMLBodyElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLBodyElementBinding::Wrap)
    }
}

impl<'a> HTMLBodyElementMethods for JSRef<'a, HTMLBodyElement> {
    fn GetOnunload(self) -> Option<EventHandlerNonNull> {
        let win = window_from_node(self).root();
        win.r().GetOnunload()
    }

    fn SetOnunload(self, listener: Option<EventHandlerNonNull>) {
        let win = window_from_node(self).root();
        win.r().SetOnunload(listener)
    }
}

pub trait HTMLBodyElementHelpers {
    fn get_background_color(&self) -> Option<RGBA>;
}

impl HTMLBodyElementHelpers for HTMLBodyElement {
    fn get_background_color(&self) -> Option<RGBA> {
        self.background_color.get()
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLBodyElement> {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let element: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(element as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(attr),
            _ => (),
        }

        let name = attr.local_name().as_slice();
        if name.starts_with("on") {
            static FORWARDED_EVENTS: &'static [&'static str] =
                &["onfocus", "onload", "onscroll", "onafterprint", "onbeforeprint",
                  "onbeforeunload", "onhashchange", "onlanguagechange", "onmessage",
                  "onoffline", "ononline", "onpagehide", "onpageshow", "onpopstate",
                  "onstorage", "onresize", "onunload", "onerror"];
            let window = window_from_node(*self).root();
            let (cx, url, reflector) = (window.r().get_cx(),
                                        window.r().get_url(),
                                        window.r().reflector().get_jsobject());
            let evtarget: JSRef<EventTarget> =
                if FORWARDED_EVENTS.iter().any(|&event| name == event) {
                    EventTargetCast::from_ref(window.r())
                } else {
                    EventTargetCast::from_ref(*self)
                };
            evtarget.set_event_handler_uncompiled(cx, url, reflector,
                                                  &name[2..],
                                                  attr.value().as_slice().to_owned());
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
            _ => {}
        }

        match attr.local_name() {
            &atom!("bgcolor") => self.background_color.set(None),
            _ => {}
        }
    }
}

