/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::reflect_dom_object_with_proto;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::RTCErrorEventBinding::{
    RTCErrorEventInit, RTCErrorEventMethods,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::rtcerror::RTCError;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct RTCErrorEvent {
    event: Event,
    error: Dom<RTCError>,
}

impl RTCErrorEvent {
    fn new_inherited(error: &RTCError) -> RTCErrorEvent {
        RTCErrorEvent {
            event: Event::new_inherited(),
            error: Dom::from_ref(error),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        error: &RTCError,
    ) -> DomRoot<RTCErrorEvent> {
        Self::new_with_proto(cx, window, None, type_, bubbles, cancelable, error)
    }

    fn new_with_proto(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        error: &RTCError,
    ) -> DomRoot<RTCErrorEvent> {
        let event = reflect_dom_object_with_proto(
            cx,
            Box::new(RTCErrorEvent::new_inherited(error)),
            window,
            proto,
        );
        {
            let event = event.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        event
    }
}

impl RTCErrorEventMethods<crate::DomTypeHolder> for RTCErrorEvent {
    /// <https://www.w3.org/TR/webrtc/#dom-rtcerrorevent-constructor>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &RTCErrorEventInit,
    ) -> DomRoot<RTCErrorEvent> {
        RTCErrorEvent::new_with_proto(
            cx,
            window,
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            &init.error,
        )
    }

    /// <https://www.w3.org/TR/webrtc/#dom-rtcerrorevent-error>
    fn Error(&self) -> DomRoot<RTCError> {
        DomRoot::from_ref(&*self.error)
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
