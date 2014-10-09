/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::HTMLBodyElementBinding;
use dom::bindings::codegen::Bindings::HTMLBodyElementBinding::HTMLBodyElementMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::EventTargetCast;
use dom::bindings::codegen::InheritTypes::{HTMLBodyElementDerived, HTMLElementCast};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLBodyElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId, EventTargetHelpers};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId, window_from_node};
use dom::virtualmethods::VirtualMethods;

use servo_util::str::DOMString;
use string_cache::Atom;

#[jstraceable]
#[must_root]
pub struct HTMLBodyElement {
    pub htmlelement: HTMLElement
}

impl HTMLBodyElementDerived for EventTarget {
    fn is_htmlbodyelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLBodyElementTypeId))
    }
}

impl HTMLBodyElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLBodyElement {
        HTMLBodyElement {
            htmlelement: HTMLElement::new_inherited(HTMLBodyElementTypeId, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLBodyElement> {
        let element = HTMLBodyElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLBodyElementBinding::Wrap)
    }
}

impl<'a> HTMLBodyElementMethods for JSRef<'a, HTMLBodyElement> {
    fn GetOnunload(self) -> Option<EventHandlerNonNull> {
        let win = window_from_node(self).root();
        win.GetOnunload()
    }

    fn SetOnunload(self, listener: Option<EventHandlerNonNull>) {
        let win = window_from_node(self).root();
        win.SetOnunload(listener)
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLBodyElement> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods> {
        let element: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(element as &VirtualMethods)
    }

    fn after_set_attr(&self, name: &Atom, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(name, value.clone()),
            _ => (),
        }

        if name.as_slice().starts_with("on") {
            static forwarded_events: &'static [&'static str] =
                &["onfocus", "onload", "onscroll", "onafterprint", "onbeforeprint",
                  "onbeforeunload", "onhashchange", "onlanguagechange", "onmessage",
                  "onoffline", "ononline", "onpagehide", "onpageshow", "onpopstate",
                  "onstorage", "onresize", "onunload", "onerror"];
            let window = window_from_node(*self).root();
            let (cx, url, reflector) = (window.get_cx(),
                                        window.get_url(),
                                        window.reflector().get_jsobject());
            let evtarget: JSRef<EventTarget> =
                if forwarded_events.iter().any(|&event| name.as_slice() == event) {
                    EventTargetCast::from_ref(*window)
                } else {
                    EventTargetCast::from_ref(*self)
                };
            evtarget.set_event_handler_uncompiled(cx, url, reflector,
                                                  name.as_slice().slice_from(2),
                                                  value);
        }
    }
}

impl Reflectable for HTMLBodyElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
