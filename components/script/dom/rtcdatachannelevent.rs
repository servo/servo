/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::RTCDataChannelEventBinding::{
    RTCDataChannelEventInit, RTCDataChannelEventMethods,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::globalscope::GlobalScope;
use crate::dom::rtcdatachannel::RTCDataChannel;
use crate::dom::window::Window;

#[dom_struct]
pub struct RTCDataChannelEvent {
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

    pub fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        channel: &RTCDataChannel,
    ) -> DomRoot<RTCDataChannelEvent> {
        Self::new_with_proto(global, None, type_, bubbles, cancelable, channel)
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        channel: &RTCDataChannel,
    ) -> DomRoot<RTCDataChannelEvent> {
        let event = reflect_dom_object_with_proto(
            Box::new(RTCDataChannelEvent::new_inherited(channel)),
            global,
            proto,
        );
        {
            let event = event.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        event
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &RTCDataChannelEventInit,
    ) -> DomRoot<RTCDataChannelEvent> {
        RTCDataChannelEvent::new_with_proto(
            &window.global(),
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            &init.channel,
        )
    }
}

impl RTCDataChannelEventMethods for RTCDataChannelEvent {
    // https://www.w3.org/TR/webrtc/#dom-datachannelevent-channel
    fn Channel(&self) -> DomRoot<RTCDataChannel> {
        DomRoot::from_ref(&*self.channel)
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
