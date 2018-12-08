/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::TextTrackListBinding::{self, TextTrackListMethods};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::texttrack::TextTrack;
use crate::dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct TextTrackList {
    eventtarget: EventTarget,
    dom_tracks: DomRefCell<Vec<Dom<TextTrack>>>,
}

impl TextTrackList {
    pub fn new_inherited(tracks: &[&TextTrack]) -> TextTrackList {
        TextTrackList {
            eventtarget: EventTarget::new_inherited(),
            dom_tracks: DomRefCell::new(tracks.iter().map(|g| Dom::from_ref(&**g)).collect()),
        }
    }

    pub fn new(window: &Window, tracks: &[&TextTrack]) -> DomRoot<TextTrackList> {
        reflect_dom_object(
            Box::new(TextTrackList::new_inherited(tracks)),
            window,
            TextTrackListBinding::Wrap,
        )
    }

    pub fn item(&self, idx: usize) -> Option<DomRoot<TextTrack>> {
        self.dom_tracks
            .borrow()
            .get(idx as usize)
            .map(|t| DomRoot::from_ref(&**t))
    }

    pub fn find(&self, track: &TextTrack) -> Option<usize> {
        self.dom_tracks
            .borrow()
            .iter()
            .enumerate()
            .filter(|(_, t)| **t == track)
            .next()
            .map(|(i, _)| i)
    }

    pub fn add(&self, track: &TextTrack) {
        // Only add a track if it does not exist in the list
        if self.find(track).is_none() {
            self.dom_tracks.borrow_mut().push(Dom::from_ref(track))
        };
        self.upcast::<EventTarget>()
            .fire_event(atom!("addtrack"));
    }

    pub fn remove(&self, idx: usize) {
        self.dom_tracks.borrow_mut().remove(idx);
        self.upcast::<EventTarget>()
            .fire_event(atom!("removetrack"));
    }
}

impl TextTrackListMethods for TextTrackList {
    // https://html.spec.whatwg.org/multipage/#dom-texttracklist-length
    fn Length(&self) -> u32 {
        self.dom_tracks.borrow().len() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-texttracklist-item
    fn IndexedGetter(&self, idx: u32) -> Option<DomRoot<TextTrack>> {
        self.item(idx as usize)
    }

    // https://html.spec.whatwg.org/multipage/#dom-texttracklist-gettrackbyid
    fn GetTrackById(&self, id: DOMString) -> Option<DomRoot<TextTrack>> {
        let id_str = String::from(id.clone());
        self.dom_tracks
            .borrow()
            .iter()
            .filter(|track| track.id() == &id_str)
            .next()
            .map(|t| DomRoot::from_ref(&**t))
    }

    // https://html.spec.whatwg.org/multipage/#handler-texttracklist-onchange
    event_handler!(onchange, GetOnchange, SetOnchange);

    // https://html.spec.whatwg.org/multipage/#handler-texttracklist-onaddtrack
    event_handler!(onchange, GetOnaddtrack, SetOnaddtrack);

    // https://html.spec.whatwg.org/multipage/#handler-texttracklist-onremovetrack
    event_handler!(onchange, GetOnremovetrack, SetOnremovetrack);
}
