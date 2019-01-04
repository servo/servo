/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::VideoTrackListBinding::{self, VideoTrackListMethods};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::Dom;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::videotrack::VideoTrack;
use crate::dom::window::Window;
use crate::task_source::TaskSource;
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

    pub fn len(&self) -> usize {
        self.tracks.borrow().len()
    }

    pub fn item(&self, idx: usize) -> Option<DomRoot<VideoTrack>> {
        self.tracks
            .borrow()
            .get(idx)
            .map(|track| DomRoot::from_ref(&**track))
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.tracks
            .borrow()
            .iter()
            .position(|track| track.selected())
    }

    // TODO(#22799) Integrate DOM Audio and Video track selection with media player.
    pub fn set_selected(&self, idx: usize, value: bool) {
        let track = match self.item(idx) {
            Some(t) => t,
            None => return,
        };

        // If the chosen tracks selected status is the same as the new status, return early.
        if track.selected() == value {
            return;
        }

        if let Some(current) = self.selected_index() {
            if current != idx {
                // Set the tracks selected status.
                self.tracks.borrow()[current].set_selected(false);
                track.set_selected(true);

                // Queue a task to fire an event named change.
                let global = &self.global();
                let this = Trusted::new(self);
                let (source, canceller) = global
                    .as_window()
                    .task_manager()
                    .media_element_task_source_with_canceller();

                let _ = source.queue_with_canceller(
                    task!(media_track_change: move || {
                        let this = this.root();
                        this.upcast::<EventTarget>().fire_event(atom!("change"));
                    }),
                    &canceller,
                );
            }
        }
    }

    pub fn add(&self, track: &VideoTrack) {
        self.tracks.borrow_mut().push(Dom::from_ref(track));
        if track.selected() {
            if let Some(idx) = self.selected_index() {
                self.set_selected(idx, false);
            }
        }
    }

    pub fn clear(&self) {
        self.tracks.borrow_mut().clear();
    }
}

impl VideoTrackListMethods for VideoTrackList {
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
        return -1;
    }

    // https://html.spec.whatwg.org/multipage/#handler-tracklist-onchange
    event_handler!(change, GetOnchange, SetOnchange);

    // https://html.spec.whatwg.org/multipage/#handler-tracklist-onaddtrack
    event_handler!(addtrack, GetOnaddtrack, SetOnaddtrack);

    // https://html.spec.whatwg.org/multipage/#handler-tracklist-onremovetrack
    event_handler!(removetrack, GetOnremovetrack, SetOnremovetrack);
}
