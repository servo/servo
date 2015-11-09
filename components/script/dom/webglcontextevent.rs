/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::WebGLContextEventBinding;
use dom::bindings::codegen::Bindings::WebGLContextEventBinding::WebGLContextEventInit;
use dom::bindings::codegen::Bindings::WebGLContextEventBinding::WebGLContextEventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::event::{Event, EventBubbles, EventCancelable};
use util::str::DOMString;

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
}

impl WebGLContextEvent {
    pub fn new_inherited(status_message: DOMString) -> WebGLContextEvent {
        WebGLContextEvent {
            event: Event::new_inherited(),
            status_message: status_message,
        }
    }

    pub fn new(global: GlobalRef,
               type_: DOMString,
               bubbles: EventBubbles,
               cancelable: EventCancelable,
               status_message: DOMString) -> Root<WebGLContextEvent> {
        let event = reflect_dom_object(
                        box WebGLContextEvent::new_inherited(status_message),
                        global,
                        WebGLContextEventBinding::Wrap);

        {
            let parent = event.upcast::<Event>();
            parent.InitEvent(type_, bubbles == EventBubbles::Bubbles, cancelable == EventCancelable::Cancelable);
        }

        event
    }

    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &WebGLContextEventInit) -> Fallible<Root<WebGLContextEvent>> {
        let status_message = match init.statusMessage.as_ref() {
            Some(message) => message.clone(),
            None => DOMString::new(),
        };

        let bubbles = if init.parent.bubbles {
            EventBubbles::Bubbles
        } else {
            EventBubbles::DoesNotBubble
        };

        let cancelable = if init.parent.cancelable {
            EventCancelable::Cancelable
        } else {
            EventCancelable::NotCancelable
        };

        Ok(WebGLContextEvent::new(global, type_,
                                  bubbles,
                                  cancelable,
                                  status_message))
    }
}
