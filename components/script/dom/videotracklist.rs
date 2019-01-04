/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::VideoTrackListBinding::{self, VideoTrackListMethods};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::Dom;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::videotrack::VideoTrack;
use crate::dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct VideoTrackList {
    eventtarget: EventTarget,
    tracks: DomRefCell<Vec<Dom<VideoTrack>>>,
}

impl VideoTrackList {
    pub fn new_inherited(tracks: &[&VideoTrack]) -> VideoTrackList {
        VideoTrackList {
            eventtarget: EventTarget::new_inherited(),
            tracks: DomRefCell::new(tracks.iter().map(|track| Dom::from_ref(&**track)).collect()),
        }
    }

    pub fn new(window: &Window, tracks: &[&VideoTrack]) -> DomRoot<VideoTrackList> {
        reflect_dom_object(
            Box::new(VideoTrackList::new_inherited(tracks)),
            window,
            VideoTrackListBinding::Wrap,
        )
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.borrow().is_empty().clone()
    }

    pub fn item(&self, idx: usize) -> Option<DomRoot<VideoTrack>> {
        self.tracks
            .borrow()
            .get(idx as usize)
            .map(|track| DomRoot::from_ref(&**track))
    }

    pub fn selected_index(&self) -> i32 {
        self.tracks
            .borrow()
            .iter()
            .enumerate()
            .filter(|&(_, track)| track.selected())
            .map(|(i, _)| i as i32)
            .next()
            .unwrap_or(-1)
    }

    pub fn set_selected(&self, idx: usize, value: bool) {
        self.item(idx).unwrap().set_selected(value);
        self.upcast::<EventTarget>().fire_event(atom!("change"));
    }

    pub fn add(&self, track: &VideoTrack) {
        if track.selected() {
            let selected_idx = self.selected_index();
            if selected_idx == -1 {
                self.tracks.borrow_mut().push(Dom::from_ref(track));
            } else {
                self.set_selected(selected_idx as usize, false);
                self.tracks.borrow_mut().push(Dom::from_ref(track));
            }
        } else {
            self.tracks.borrow_mut().push(Dom::from_ref(track));
        }
        self.upcast::<EventTarget>().fire_event(atom!("addtrack"));
    }
}

impl VideoTrackListMethods for VideoTrackList {
    // https://html.spec.whatwg.org/multipage/#dom-videotracklist-length
    fn Length(&self) -> u32 {
        self.tracks.borrow().len() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-tracklist-item
    fn IndexedGetter(&self, idx: u32) -> Option<DomRoot<VideoTrack>> {
        self.item(idx as usize)
    }

    // https://html.spec.whatwg.org/multipage/#dom-videotracklist-gettrackbyid
    fn GetTrackById(&self, id: DOMString) -> Option<DomRoot<VideoTrack>> {
        self.tracks
            .borrow()
            .iter()
            .filter(|track| track.id() == id)
            .next()
            .map(|t| DomRoot::from_ref(&**t))
    }

    // https://html.spec.whatwg.org/multipage/#dom-videotrack-selected
    fn SelectedIndex(&self) -> i32 {
        self.selected_index()
    }

    // https://html.spec.whatwg.org/multipage/#handler-tracklist-onchange
    event_handler!(onchange, GetOnchange, SetOnchange);

    // https://html.spec.whatwg.org/multipage/#handler-tracklist-onaddtrack
    event_handler!(onaddtrack, GetOnaddtrack, SetOnaddtrack);

    // https://html.spec.whatwg.org/multipage/#handler-tracklist-onremovetrack
    event_handler!(onremovetrack, GetOnremovetrack, SetOnremovetrack);
}
