/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::audiotrack::AudioTrack;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::AudioTrackListBinding::AudioTrackListMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlmediaelement::HTMLMediaElement;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct AudioTrackList {
    eventtarget: EventTarget,
    tracks: DomRefCell<Vec<Dom<AudioTrack>>>,
    media_element: Option<Dom<HTMLMediaElement>>,
}

impl AudioTrackList {
    pub(crate) fn new_inherited(
        tracks: &[&AudioTrack],
        media_element: Option<&HTMLMediaElement>,
    ) -> AudioTrackList {
        AudioTrackList {
            eventtarget: EventTarget::new_inherited(),
            tracks: DomRefCell::new(tracks.iter().map(|track| Dom::from_ref(&**track)).collect()),
            media_element: media_element.map(Dom::from_ref),
        }
    }

    pub(crate) fn new(
        window: &Window,
        tracks: &[&AudioTrack],
        media_element: Option<&HTMLMediaElement>,
    ) -> DomRoot<AudioTrackList> {
        reflect_dom_object(
            Box::new(AudioTrackList::new_inherited(tracks, media_element)),
            window,
            CanGc::note(),
        )
    }

    pub(crate) fn len(&self) -> usize {
        self.tracks.borrow().len()
    }

    pub(crate) fn find(&self, track: &AudioTrack) -> Option<usize> {
        self.tracks.borrow().iter().position(|t| &**t == track)
    }

    pub(crate) fn item(&self, idx: usize) -> Option<DomRoot<AudioTrack>> {
        self.tracks
            .borrow()
            .get(idx)
            .map(|track| DomRoot::from_ref(&**track))
    }

    pub(crate) fn enabled_index(&self) -> Option<usize> {
        self.tracks
            .borrow()
            .iter()
            .position(|track| track.enabled())
    }

    pub(crate) fn set_enabled(&self, idx: usize, value: bool) {
        let track = match self.item(idx) {
            Some(t) => t,
            None => return,
        };

        // If the chosen tracks enabled status is the same as the new status, return early.
        if track.enabled() == value {
            return;
        }
        // Set the tracks enabled status.
        track.set_enabled(value);
        if let Some(media_element) = self.media_element.as_ref() {
            media_element.set_audio_track(idx, value);
        }

        // Queue a task to fire an event named change.
        let global = &self.global();
        let this = Trusted::new(self);
        let task_source = global.task_manager().media_element_task_source();
        task_source.queue(task!(media_track_change: move || {
            let this = this.root();
            this.upcast::<EventTarget>().fire_event(atom!("change"), CanGc::note());
        }));
    }

    pub(crate) fn add(&self, track: &AudioTrack) {
        self.tracks.borrow_mut().push(Dom::from_ref(track));
        track.add_track_list(self);
    }

    pub(crate) fn clear(&self) {
        self.tracks
            .borrow()
            .iter()
            .for_each(|t| t.remove_track_list());
        self.tracks.borrow_mut().clear();
    }
}

impl AudioTrackListMethods<crate::DomTypeHolder> for AudioTrackList {
    // https://html.spec.whatwg.org/multipage/#dom-audiotracklist-length
    fn Length(&self) -> u32 {
        self.len() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-tracklist-item
    fn IndexedGetter(&self, idx: u32) -> Option<DomRoot<AudioTrack>> {
        self.item(idx as usize)
    }

    // https://html.spec.whatwg.org/multipage/#dom-audiotracklist-gettrackbyid
    fn GetTrackById(&self, id: DOMString) -> Option<DomRoot<AudioTrack>> {
        self.tracks
            .borrow()
            .iter()
            .find(|track| track.id() == id)
            .map(|track| DomRoot::from_ref(&**track))
    }

    // https://html.spec.whatwg.org/multipage/#handler-tracklist-onchange
    event_handler!(change, GetOnchange, SetOnchange);

    // https://html.spec.whatwg.org/multipage/#handler-tracklist-onaddtrack
    event_handler!(addtrack, GetOnaddtrack, SetOnaddtrack);

    // https://html.spec.whatwg.org/multipage/#handler-tracklist-onremovetrack
    event_handler!(removetrack, GetOnremovetrack, SetOnremovetrack);
}
