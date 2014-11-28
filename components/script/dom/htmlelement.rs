/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::attr::AttrHelpers;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::HTMLElementBinding;
use dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLFrameSetElementDerived};
use dom::bindings::codegen::InheritTypes::EventTargetCast;
use dom::bindings::codegen::InheritTypes::{HTMLElementDerived, HTMLBodyElementDerived};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::{Element, ElementTypeId, ElementTypeId_, HTMLElementTypeId};
use dom::eventtarget::{EventTarget, EventTargetHelpers, NodeTargetTypeId};
use dom::node::{Node, ElementNodeTypeId, window_from_node};
use dom::virtualmethods::VirtualMethods;

use servo_util::str::DOMString;

use string_cache::Atom;

#[dom_struct]
pub struct HTMLElement {
    element: Element
}

impl HTMLElementDerived for EventTarget {
    fn is_htmlelement(&self) -> bool {
        match *self.type_id() {
            NodeTargetTypeId(ElementNodeTypeId(ElementTypeId_)) => false,
            NodeTargetTypeId(ElementNodeTypeId(_)) => true,
            _ => false
        }
    }
}

impl HTMLElement {
    pub fn new_inherited(type_id: ElementTypeId, tag_name: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLElement {
        HTMLElement {
            element: Element::new_inherited(type_id, tag_name, ns!(HTML), prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLElement> {
        let element = HTMLElement::new_inherited(HTMLElementTypeId, localName, prefix, document);
        Node::reflect_node(box element, document, HTMLElementBinding::Wrap)
    }

    #[inline]
    pub fn element<'a>(&'a self) -> &'a Element {
        &self.element
    }
}

trait PrivateHTMLElementHelpers {
    fn is_body_or_frameset(self) -> bool;
}

impl<'a> PrivateHTMLElementHelpers for JSRef<'a, HTMLElement> {
    fn is_body_or_frameset(self) -> bool {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.is_htmlbodyelement() || eventtarget.is_htmlframesetelement()
    }
}

impl<'a> HTMLElementMethods for JSRef<'a, HTMLElement> {
    make_getter!(Title)
    make_setter!(SetTitle, "title")

    make_getter!(Lang)
    make_setter!(SetLang, "lang")

    // http://html.spec.whatwg.org/multipage/#dom-hidden
    make_bool_getter!(Hidden)
    make_bool_setter!(SetHidden, "hidden")

    event_handler!(click, GetOnclick, SetOnclick)

    fn GetOnload(self) -> Option<EventHandlerNonNull> {
        if self.is_body_or_frameset() {
            let win = window_from_node(self).root();
            win.GetOnload()
        } else {
            None
        }
    }

    fn SetOnload(self, listener: Option<EventHandlerNonNull>) {
        if self.is_body_or_frameset() {
            let win = window_from_node(self).root();
            win.SetOnload(listener)
        }
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLElement> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods> {
        let element: &JSRef<Element> = ElementCast::from_borrowed_ref(self);
        Some(element as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(attr),
            _ => ()
        }

        let name = attr.local_name().as_slice();
        if name.starts_with("on") {
            let window = window_from_node(*self).root();
            let (cx, url, reflector) = (window.get_cx(),
                                        window.get_url(),
                                        window.reflector().get_jsobject());
            let evtarget: JSRef<EventTarget> = EventTargetCast::from_ref(*self);
            evtarget.set_event_handler_uncompiled(cx, url, reflector,
                                                  name.slice_from(2),
                                                  attr.value().as_slice().to_string());
        }
    }
}

impl Reflectable for HTMLElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.element.reflector()
    }
}
