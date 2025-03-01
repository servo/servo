/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::VideoTrackListBinding::VideoTrackListMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlmediaelement::HTMLMediaElement;
use crate::dom::videotrack::VideoTrack;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct VideoTrackList {
    eventtarget: EventTarget,
    tracks: DomRefCell<Vec<Dom<VideoTrack>>>,
    media_element: Option<Dom<HTMLMediaElement>>,
}

impl VideoTrackList {
    pub(crate) fn new_inherited(
        tracks: &[&VideoTrack],
        media_element: Option<&HTMLMediaElement>,
    ) -> VideoTrackList {
        VideoTrackList {
            eventtarget: EventTarget::new_inherited(),
            tracks: DomRefCell::new(tracks.iter().map(|track| Dom::from_ref(&**track)).collect()),
            media_element: media_element.map(Dom::from_ref),
        }
    }

    pub(crate) fn new(
        window: &Window,
        tracks: &[&VideoTrack],
        media_element: Option<&HTMLMediaElement>,
        can_gc: CanGc,
    ) -> DomRoot<VideoTrackList> {
        reflect_dom_object(
            Box::new(VideoTrackList::new_inherited(tracks, media_element)),
            window,
            can_gc,
        )
    }

    pub(crate) fn len(&self) -> usize {
        self.tracks.borrow().len()
    }

    pub(crate) fn find(&self, track: &VideoTrack) -> Option<usize> {
        self.tracks.borrow().iter().position(|t| &**t == track)
    }

    pub(crate) fn item(&self, idx: usize) -> Option<DomRoot<VideoTrack>> {
        self.tracks
            .borrow()
            .get(idx)
            .map(|track| DomRoot::from_ref(&**track))
    }

    pub(crate) fn selected_index(&self) -> Option<usize> {
        self.tracks
            .borrow()
            .iter()
            .position(|track| track.selected())
    }

    pub(crate) fn set_selected(&self, idx: usize, value: bool) {
        let track = match self.item(idx) {
            Some(t) => t,
            None => return,
        };

        // If the chosen tracks selected status is the same as the new status, return early.
        if track.selected() == value {
            return;
        }

        if let Some(current) = self.selected_index() {
            self.tracks.borrow()[current].set_selected(false);
        }

        track.set_selected(value);
        if let Some(media_element) = self.media_element.as_ref() {
            media_element.set_video_track(idx, value);
        }

        let this = Trusted::new(self);
        self.global()
            .task_manager()
            .media_element_task_source()
            .queue(task!(media_track_change: move || {
                let this = this.root();
                this.upcast::<EventTarget>().fire_event(atom!("change"), CanGc::note());
            }));
    }

    pub(crate) fn add(&self, track: &VideoTrack) {
        self.tracks.borrow_mut().push(Dom::from_ref(track));
        if track.selected() {
            if let Some(idx) = self.selected_index() {
                self.set_selected(idx, false);
            }
        }
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

impl VideoTrackListMethods<crate::DomTypeHolder> for VideoTrackList {
    // https://html.spec.whatwg.org/multipage/#dom-videotracklist-length
    fn Length(&self) -> u32 {
        self.len() as u32
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
            .find(|track| track.id() == id)
            .map(|track| DomRoot::from_ref(&**track))
    }

    // https://html.spec.whatwg.org/multipage/#dom-videotrack-selected
    fn SelectedIndex(&self) -> i32 {
        if let Some(idx) = self.selected_index() {
            return idx as i32;
        }
        -1
    }

    // https://html.spec.whatwg.org/multipage/#handler-tracklist-onchange
    event_handler!(change, GetOnchange, SetOnchange);

    // https://html.spec.whatwg.org/multipage/#handler-tracklist-onaddtrack
    event_handler!(addtrack, GetOnaddtrack, SetOnaddtrack);

    // https://html.spec.whatwg.org/multipage/#handler-tracklist-onremovetrack
    event_handler!(removetrack, GetOnremovetrack, SetOnremovetrack);
}
