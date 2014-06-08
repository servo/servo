/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::HTMLElementBinding;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLFrameSetElementDerived};
use dom::bindings::codegen::InheritTypes::EventTargetCast;
use dom::bindings::codegen::InheritTypes::{HTMLElementDerived, HTMLBodyElementDerived};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::Reflectable;
use dom::document::Document;
use dom::element::{Element, ElementTypeId, HTMLElementTypeId};
use dom::eventtarget::{EventTarget, EventTargetHelpers, NodeTargetTypeId};
use dom::node::{Node, ElementNodeTypeId, window_from_node};
use dom::virtualmethods::VirtualMethods;
use dom::window::WindowMethods;
use servo_util::namespace;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLElement {
    pub element: Element
}

impl HTMLElementDerived for EventTarget {
    fn is_htmlelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(ElementTypeId)) => false,
            NodeTargetTypeId(ElementNodeTypeId(_)) => true,
            _ => false
        }
    }
}

impl HTMLElement {
    pub fn new_inherited(type_id: ElementTypeId, tag_name: DOMString, document: &JSRef<Document>) -> HTMLElement {
        HTMLElement {
            element: Element::new_inherited(type_id, tag_name, namespace::HTML, None, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLElement> {
        let element = HTMLElement::new_inherited(HTMLElementTypeId, localName, document);
        Node::reflect_node(box element, document, HTMLElementBinding::Wrap)
    }
}

trait PrivateHTMLElementHelpers {
    fn is_body_or_frameset(&self) -> bool;
}

impl<'a> PrivateHTMLElementHelpers for JSRef<'a, HTMLElement> {
    fn is_body_or_frameset(&self) -> bool {
        let eventtarget: &JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.is_htmlbodyelement() || eventtarget.is_htmlframesetelement()
    }
}

pub trait HTMLElementMethods {
    fn GetOnclick(&self) -> Option<EventHandlerNonNull>;
    fn SetOnclick(&mut self, listener: Option<EventHandlerNonNull>);
    fn GetOnload(&self) -> Option<EventHandlerNonNull>;
    fn SetOnload(&mut self, listener: Option<EventHandlerNonNull>);
}

impl<'a> HTMLElementMethods for JSRef<'a, HTMLElement> {
    fn GetOnclick(&self) -> Option<EventHandlerNonNull> {
        let eventtarget: &JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("click")
    }

    fn SetOnclick(&mut self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: &mut JSRef<EventTarget> = EventTargetCast::from_mut_ref(self);
        eventtarget.set_event_handler_common("click", listener)
    }

    fn GetOnload(&self) -> Option<EventHandlerNonNull> {
        if self.is_body_or_frameset() {
            let win = window_from_node(self).root();
            win.deref().GetOnload()
        } else {
            None
        }
    }

    fn SetOnload(&mut self, listener: Option<EventHandlerNonNull>) {
        if self.is_body_or_frameset() {
            let mut win = window_from_node(self).root();
            win.SetOnload(listener)
        }
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLElement> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods:> {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        Some(element as &VirtualMethods:)
    }

    fn after_set_attr(&self, name: DOMString, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(name.clone(), value.clone()),
            _ => (),
        }

        if name.as_slice().starts_with("on") {
            let mut window = window_from_node(self).root();
            let (cx, url, reflector) = (window.get_cx(),
                                        window.get_url(),
                                        window.reflector().get_jsobject());
            let mut self_alias = self.clone();
            let evtarget: &mut JSRef<EventTarget> =
                EventTargetCast::from_mut_ref(&mut self_alias);
            evtarget.set_event_handler_uncompiled(cx, url, reflector,
                                                  name.as_slice().slice_from(2),
                                                  value);
        }
    }
}
