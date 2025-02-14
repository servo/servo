/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_atoms::Atom;

use crate::dom::audiotrack::AudioTrack;
use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::codegen::Bindings::TrackEventBinding;
use crate::dom::bindings::codegen::Bindings::TrackEventBinding::TrackEventMethods;
use crate::dom::bindings::codegen::UnionTypes::VideoTrackOrAudioTrackOrTextTrack;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::globalscope::GlobalScope;
use crate::dom::texttrack::TextTrack;
use crate::dom::videotrack::VideoTrack;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(JSTraceable, MallocSizeOf)]
enum MediaTrack {
    Video(Dom<VideoTrack>),
    Audio(Dom<AudioTrack>),
    Text(Dom<TextTrack>),
}

#[dom_struct]
pub(crate) struct TrackEvent {
    event: Event,
    track: Option<MediaTrack>,
}

impl TrackEvent {
    #[allow(non_snake_case)]
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(track: &Option<VideoTrackOrAudioTrackOrTextTrack>) -> TrackEvent {
        let media_track = match track {
            Some(VideoTrackOrAudioTrackOrTextTrack::VideoTrack(VideoTrack)) => {
                Some(MediaTrack::Video(Dom::from_ref(VideoTrack)))
            },
            Some(VideoTrackOrAudioTrackOrTextTrack::AudioTrack(AudioTrack)) => {
                Some(MediaTrack::Audio(Dom::from_ref(AudioTrack)))
            },
            Some(VideoTrackOrAudioTrackOrTextTrack::TextTrack(TextTrack)) => {
                Some(MediaTrack::Text(Dom::from_ref(TextTrack)))
            },
            None => None,
        };

        TrackEvent {
            event: Event::new_inherited(),
            track: media_track,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        track: &Option<VideoTrackOrAudioTrackOrTextTrack>,
        can_gc: CanGc,
    ) -> DomRoot<TrackEvent> {
        Self::new_with_proto(global, None, type_, bubbles, cancelable, track, can_gc)
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        track: &Option<VideoTrackOrAudioTrackOrTextTrack>,
        can_gc: CanGc,
    ) -> DomRoot<TrackEvent> {
        let te = reflect_dom_object_with_proto(
            Box::new(TrackEvent::new_inherited(track)),
            global,
            proto,
            can_gc,
        );
        {
            let event = te.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        te
    }
}

impl TrackEventMethods<crate::DomTypeHolder> for TrackEvent {
    // https://html.spec.whatwg.org/multipage/#trackevent
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        type_: DOMString,
        init: &TrackEventBinding::TrackEventInit,
    ) -> Fallible<DomRoot<TrackEvent>> {
        Ok(TrackEvent::new_with_proto(
            &window.global(),
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            &init.track,
            can_gc,
        ))
    }

    // https://html.spec.whatwg.org/multipage/#dom-trackevent-track
    #[allow(non_snake_case)]
    fn GetTrack(&self) -> Option<VideoTrackOrAudioTrackOrTextTrack> {
        match &self.track {
            Some(MediaTrack::Video(VideoTrack)) => Some(
                VideoTrackOrAudioTrackOrTextTrack::VideoTrack(DomRoot::from_ref(VideoTrack)),
            ),
            Some(MediaTrack::Audio(AudioTrack)) => Some(
                VideoTrackOrAudioTrackOrTextTrack::AudioTrack(DomRoot::from_ref(AudioTrack)),
            ),
            Some(MediaTrack::Text(TextTrack)) => Some(
                VideoTrackOrAudioTrackOrTextTrack::TextTrack(DomRoot::from_ref(TextTrack)),
            ),
            None => None,
        }
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
