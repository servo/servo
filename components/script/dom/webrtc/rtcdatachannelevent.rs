/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::reflect_dom_object_with_proto_and_cx;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::RTCDataChannelEventBinding::{
    RTCDataChannelEventInit, RTCDataChannelEventMethods,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::rtcdatachannel::RTCDataChannel;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct RTCDataChannelEvent {
    event: Event,
    channel: Dom<RTCDataChannel>,
}

impl RTCDataChannelEvent {
    fn new_inherited(channel: &RTCDataChannel) -> RTCDataChannelEvent {
        RTCDataChannelEvent {
            event: Event::new_inherited(),
            channel: Dom::from_ref(channel),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        channel: &RTCDataChannel,
    ) -> DomRoot<RTCDataChannelEvent> {
        Self::new_with_proto(cx, window, None, type_, bubbles, cancelable, channel)
    }

    fn new_with_proto(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        channel: &RTCDataChannel,
    ) -> DomRoot<RTCDataChannelEvent> {
        let event = reflect_dom_object_with_proto_and_cx(
            Box::new(RTCDataChannelEvent::new_inherited(channel)),
            window,
            proto,
            cx,
        );
        {
            let event = event.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        event
    }
}

impl RTCDataChannelEventMethods<crate::DomTypeHolder> for RTCDataChannelEvent {
    /// <https://www.w3.org/TR/webrtc/#dom-rtcdatachannelevent-constructor>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &RTCDataChannelEventInit,
    ) -> DomRoot<RTCDataChannelEvent> {
        RTCDataChannelEvent::new_with_proto(
            cx,
            window,
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            &init.channel,
        )
    }

    /// <https://www.w3.org/TR/webrtc/#dom-datachannelevent-channel>
    fn Channel(&self) -> DomRoot<RTCDataChannel> {
        DomRoot::from_ref(&*self.channel)
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
