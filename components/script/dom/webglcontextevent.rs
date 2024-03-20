/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::WebGLContextEventBinding::{
    WebGLContextEventInit, WebGLContextEventMethods,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::window::Window;

#[dom_struct]
pub struct WebGLContextEvent {
    event: Event,
    status_message: DOMString,
}

impl WebGLContextEventMethods for WebGLContextEvent {
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.15
    fn StatusMessage(&self) -> DOMString {
        self.status_message.clone()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}

impl WebGLContextEvent {
    fn new_inherited(status_message: DOMString) -> WebGLContextEvent {
        WebGLContextEvent {
            event: Event::new_inherited(),
            status_message,
        }
    }

    pub fn new(
        window: &Window,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        status_message: DOMString,
    ) -> DomRoot<WebGLContextEvent> {
        Self::new_with_proto(window, None, type_, bubbles, cancelable, status_message)
    }

    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        status_message: DOMString,
    ) -> DomRoot<WebGLContextEvent> {
        let event = reflect_dom_object_with_proto(
            Box::new(WebGLContextEvent::new_inherited(status_message)),
            window,
            proto,
        );

        {
            let parent = event.upcast::<Event>();
            parent.init_event(type_, bool::from(bubbles), bool::from(cancelable));
        }

        event
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &WebGLContextEventInit,
    ) -> Fallible<DomRoot<WebGLContextEvent>> {
        let status_message = match init.statusMessage.as_ref() {
            Some(message) => message.clone(),
            None => DOMString::new(),
        };

        let bubbles = EventBubbles::from(init.parent.bubbles);

        let cancelable = EventCancelable::from(init.parent.cancelable);

        Ok(WebGLContextEvent::new_with_proto(
            window,
            proto,
            Atom::from(type_),
            bubbles,
            cancelable,
            status_message,
        ))
    }
}
