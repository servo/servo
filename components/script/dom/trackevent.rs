/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::audiotrack::AudioTrack;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::TrackEventBinding;
use crate::dom::bindings::codegen::Bindings::TrackEventBinding::TrackEventMethods;
use crate::dom::bindings::codegen::UnionTypes::VideoTrackOrAudioTrackOrTextTrack;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::globalscope::GlobalScope;
use crate::dom::texttrack::TextTrack;
use crate::dom::videotrack::VideoTrack;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use servo_atoms::Atom;

#[must_root]
#[derive(JSTraceable, MallocSizeOf)]
enum MediaTrack {
    Video(Dom<VideoTrack>),
    Audio(Dom<AudioTrack>),
    Text(Dom<TextTrack>),
}

#[dom_struct]
pub struct TrackEvent {
    event: Event,
    track: Option<MediaTrack>,
}

impl TrackEvent {
    #[allow(unrooted_must_root)]
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
