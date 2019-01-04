/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::EventBinding::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::TrackEventBinding;
use crate::dom::bindings::codegen::Bindings::TrackEventBinding::TrackEventMethods;
use crate::dom::bindings::codegen::UnionTypes::VideoTrackOrAudioTrackOrTextTrack;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use servo_atoms::Atom;

#[dom_struct]
pub struct TrackEvent {
    event: Event,
    #[ignore_malloc_size_of = "Defined in rust-mozjs"]
    track: Option<VideoTrackOrAudioTrackOrTextTrack>,
}

impl TrackEvent {
    pub fn new_inherited(_track: &Option<VideoTrackOrAudioTrackOrTextTrack>) -> TrackEvent {
        unimplemented!();
    }

    pub fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        track: &Option<VideoTrackOrAudioTrackOrTextTrack>,
    ) -> DomRoot<TrackEvent> {
        let te = reflect_dom_object(
            Box::new(TrackEvent::new_inherited(&track)),
            global,
            TrackEventBinding::Wrap,
        );
        {
            let event = te.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        te
    }

    pub fn Constructor(
        window: &Window,
        type_: DOMString,
        init: &TrackEventBinding::TrackEventInit,
    ) -> Fallible<DomRoot<TrackEvent>> {
        Ok(TrackEvent::new(
            &window.global(),
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            &init.track,
        ))
    }
}

impl TrackEventMethods for TrackEvent {
    // https://html.spec.whatwg.org/multipage/#dom-trackevent-track
    fn GetTrack(&self) -> Option<VideoTrackOrAudioTrackOrTextTrack> {
        unimplemented!();
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
