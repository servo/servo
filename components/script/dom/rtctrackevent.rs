/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::codegen::Bindings::RTCTrackEventBinding::{self, RTCTrackEventMethods};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::globalscope::GlobalScope;
use crate::dom::mediastreamtrack::MediaStreamTrack;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct RTCTrackEvent {
    event: Event,
    track: Dom<MediaStreamTrack>,
}

impl RTCTrackEvent {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(track: &MediaStreamTrack) -> RTCTrackEvent {
        RTCTrackEvent {
            event: Event::new_inherited(),
            track: Dom::from_ref(track),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        track: &MediaStreamTrack,
        can_gc: CanGc,
    ) -> DomRoot<RTCTrackEvent> {
        Self::new_with_proto(global, None, type_, bubbles, cancelable, track, can_gc)
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        track: &MediaStreamTrack,
        can_gc: CanGc,
    ) -> DomRoot<RTCTrackEvent> {
        let trackevent = reflect_dom_object_with_proto(
            Box::new(RTCTrackEvent::new_inherited(track)),
            global,
            proto,
            can_gc,
        );
        {
            let event = trackevent.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        trackevent
    }
}

impl RTCTrackEventMethods<crate::DomTypeHolder> for RTCTrackEvent {
    // https://w3c.github.io/webrtc-pc/#dom-rtctrackevent-constructor
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        type_: DOMString,
        init: &RTCTrackEventBinding::RTCTrackEventInit,
    ) -> Fallible<DomRoot<RTCTrackEvent>> {
        Ok(RTCTrackEvent::new_with_proto(
            &window.global(),
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            &init.track,
            can_gc,
        ))
    }

    // https://w3c.github.io/webrtc-pc/#dom-rtctrackevent-track
    fn Track(&self) -> DomRoot<MediaStreamTrack> {
        DomRoot::from_ref(&*self.track)
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
