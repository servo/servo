/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::BindingDeclarations::HTMLBodyElementBinding;
use dom::bindings::codegen::InheritTypes::EventTargetCast;
use dom::bindings::codegen::InheritTypes::{HTMLBodyElementDerived, HTMLElementCast};
use dom::bindings::error::ErrorResult;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::Reflectable;
use dom::document::Document;
use dom::element::HTMLBodyElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId, EventTargetHelpers};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId, window_from_node};
use dom::virtualmethods::VirtualMethods;
use dom::window::WindowMethods;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLBodyElement {
    pub htmlelement: HTMLElement
}

impl HTMLBodyElementDerived for EventTarget {
    fn is_htmlbodyelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLBodyElementTypeId))
    }
}

impl HTMLBodyElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLBodyElement {
        HTMLBodyElement {
            htmlelement: HTMLElement::new_inherited(HTMLBodyElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLBodyElement> {
        let element = HTMLBodyElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLBodyElementBinding::Wrap)
    }
}

pub trait HTMLBodyElementMethods {
    fn Text(&self) -> DOMString;
    fn SetText(&mut self, _text: DOMString) -> ErrorResult;
    fn Link(&self) -> DOMString;
    fn SetLink(&self, _link: DOMString) -> ErrorResult;
    fn VLink(&self) -> DOMString;
    fn SetVLink(&self, _v_link: DOMString) -> ErrorResult;
    fn ALink(&self) -> DOMString;
    fn SetALink(&self, _a_link: DOMString) -> ErrorResult;
    fn BgColor(&self) -> DOMString;
    fn SetBgColor(&self, _bg_color: DOMString) -> ErrorResult;
    fn Background(&self) -> DOMString;
    fn SetBackground(&self, _background: DOMString) -> ErrorResult;
    fn GetOnunload(&self) -> Option<EventHandlerNonNull>;
    fn SetOnunload(&mut self, listener: Option<EventHandlerNonNull>);
}

impl<'a> HTMLBodyElementMethods for JSRef<'a, HTMLBodyElement> {
    fn Text(&self) -> DOMString {
        "".to_owned()
    }

    fn SetText(&mut self, _text: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Link(&self) -> DOMString {
        "".to_owned()
    }

    fn SetLink(&self, _link: DOMString) -> ErrorResult {
        Ok(())
    }

    fn VLink(&self) -> DOMString {
        "".to_owned()
    }

    fn SetVLink(&self, _v_link: DOMString) -> ErrorResult {
        Ok(())
    }

    fn ALink(&self) -> DOMString {
        "".to_owned()
    }

    fn SetALink(&self, _a_link: DOMString) -> ErrorResult {
        Ok(())
    }

    fn BgColor(&self) -> DOMString {
        "".to_owned()
    }

    fn SetBgColor(&self, _bg_color: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Background(&self) -> DOMString {
        "".to_owned()
    }

    fn SetBackground(&self, _background: DOMString) -> ErrorResult {
        Ok(())
    }

    fn GetOnunload(&self) -> Option<EventHandlerNonNull> {
        let win = window_from_node(self).root();
        win.deref().GetOnunload()
    }

    fn SetOnunload(&mut self, listener: Option<EventHandlerNonNull>) {
        let mut win = window_from_node(self).root();
        win.SetOnunload(listener)
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLBodyElement> {
    fn super_type<'a>(&'a mut self) -> Option<&'a mut VirtualMethods:> {
        let element: &mut JSRef<HTMLElement> = HTMLElementCast::from_mut_ref(self);
        Some(element as &mut VirtualMethods:)
    }

    fn after_set_attr(&mut self, name: DOMString, value: DOMString) {
        match self.super_type() {
            Some(ref mut s) => s.after_set_attr(name.clone(), value.clone()),
            _ => (),
        }

        if name.starts_with("on") {
            static forwarded_events: &'static [&'static str] =
                &["onfocus", "onload", "onscroll", "onafterprint", "onbeforeprint",
                  "onbeforeunload", "onhashchange", "onlanguagechange", "onmessage",
                  "onoffline", "ononline", "onpagehide", "onpageshow", "onpopstate",
                  "onstorage", "onresize", "onunload", "onerror"];
            let mut window = window_from_node(self).root();
            let (cx, url, reflector) = (window.get_cx(),
                                        window.get_url(),
                                        window.reflector().get_jsobject());
            let evtarget: &mut JSRef<EventTarget> =
                if forwarded_events.iter().any(|&event| name.as_slice() == event) {
                    EventTargetCast::from_mut_ref(&mut *window)
                } else {
                    EventTargetCast::from_mut_ref(self)
                };
            evtarget.set_event_handler_uncompiled(cx, url, reflector,
                                                  name.slice_from(2).to_owned(),
                                                  value);
        }
    }
}
