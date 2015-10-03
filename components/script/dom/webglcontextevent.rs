/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::WebGLContextEventBinding;
use dom::bindings::codegen::Bindings::WebGLContextEventBinding::WebGLContextEventInit;
use dom::bindings::codegen::Bindings::WebGLContextEventBinding::WebGLContextEventMethods;
use dom::bindings::codegen::InheritTypes::{WebGLContextEventDerived, EventCast};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{Root, RootedReference};
use dom::bindings::utils::reflect_dom_object;
use dom::event::{Event, EventBubbles, EventCancelable, EventTypeId};
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

impl WebGLContextEventDerived for Event {
    fn is_webglcontextevent(&self) -> bool {
        *self.type_id() == EventTypeId::WebGLContextEvent
    }
}

impl WebGLContextEvent {
    pub fn new_inherited(type_id: EventTypeId, status_message: DOMString) -> WebGLContextEvent {
        WebGLContextEvent {
            event: Event::new_inherited(type_id),
            status_message: status_message,
        }
    }

    pub fn new(global: GlobalRef,
               type_: DOMString,
               bubbles: EventBubbles,
               cancelable: EventCancelable,
               status_message: DOMString) -> Root<WebGLContextEvent> {
        let event = reflect_dom_object(
                        box WebGLContextEvent::new_inherited(EventTypeId::WebGLContextEvent, status_message),
                        global,
                        WebGLContextEventBinding::Wrap);

        {
            let parent = EventCast::from_ref(event.r());
            parent.InitEvent(type_, bubbles == EventBubbles::Bubbles, cancelable == EventCancelable::Cancelable);
        }

        event
    }

    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &WebGLContextEventInit) -> Fallible<Root<WebGLContextEvent>> {
        let status_message = match init.statusMessage.as_ref() {
            Some(message) => message.clone(),
            None => "".to_owned(),
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
