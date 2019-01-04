/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::audiotrack::AudioTrack;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::AudioTrackListBinding::{self, AudioTrackListMethods};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct AudioTrackList {
    eventtarget: EventTarget,
    tracks: DomRefCell<Vec<Dom<AudioTrack>>>,
}

impl AudioTrackList {
    pub fn new_inherited(tracks: &[&AudioTrack]) -> AudioTrackList {
        AudioTrackList {
            eventtarget: EventTarget::new_inherited(),
            tracks: DomRefCell::new(tracks.iter().map(|track| Dom::from_ref(&**track)).collect()),
        }
    }

    pub fn new(window: &Window, tracks: &[&AudioTrack]) -> DomRoot<AudioTrackList> {
        reflect_dom_object(
            Box::new(AudioTrackList::new_inherited(tracks)),
            window,
            AudioTrackListBinding::Wrap,
        )
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.borrow().is_empty().clone()
    }

    pub fn item(&self, idx: usize) -> Option<DomRoot<AudioTrack>> {
        self.tracks
            .borrow()
            .get(idx as usize)
            .map(|track| DomRoot::from_ref(&**track))
    }

    pub fn enabled_index(&self) -> i32 {
        self.tracks
            .borrow()
            .iter()
            .enumerate()
            .filter(|&(_, track)| track.enabled())
            .map(|(i, _)| i as i32)
            .next()
            .unwrap_or(-1)
    }

    pub fn set_enabled(&self, idx: usize, value: bool) {
        self.item(idx).unwrap().set_enabled(value);
        self.upcast::<EventTarget>().fire_event(atom!("change"));
    }

    pub fn add(&self, track: &AudioTrack) {
        self.tracks.borrow_mut().push(Dom::from_ref(track));
        self.upcast::<EventTarget>().fire_event(atom!("addtrack"));
    }
}

impl AudioTrackListMethods for AudioTrackList {
    // https://html.spec.whatwg.org/multipage/#dom-audiotracklist-length
    fn Length(&self) -> u32 {
        self.tracks.borrow().len() as u32
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
            .filter(|track| track.id() == id)
            .next()
            .map(|track| DomRoot::from_ref(&**track))
    }

    // https://html.spec.whatwg.org/multipage/#handler-tracklist-onchange
    event_handler!(onchange, GetOnchange, SetOnchange);

    // https://html.spec.whatwg.org/multipage/#handler-tracklist-onaddtrack
    event_handler!(onaddtrack, GetOnaddtrack, SetOnaddtrack);

    // https://html.spec.whatwg.org/multipage/#handler-tracklist-onremovetrack
    event_handler!(onremovetrack, GetOnremovetrack, SetOnremovetrack);
}
