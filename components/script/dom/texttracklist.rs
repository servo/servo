/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::TextTrackListBinding::TextTrackListMethods;
use crate::dom::bindings::codegen::UnionTypes::VideoTrackOrAudioTrackOrTextTrack;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::texttrack::TextTrack;
use crate::dom::trackevent::TrackEvent;
use crate::dom::window::Window;
use crate::task_source::TaskSource;

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
        reflect_dom_object(Box::new(TextTrackList::new_inherited(tracks)), window)
    }

    pub fn item(&self, idx: usize) -> Option<DomRoot<TextTrack>> {
        self.dom_tracks
            .borrow()
            .get(idx)
            .map(|t| DomRoot::from_ref(&**t))
    }

    pub fn find(&self, track: &TextTrack) -> Option<usize> {
        self.dom_tracks
            .borrow()
            .iter()
            .enumerate()
            .find(|(_, t)| **t == track)
            .map(|(i, _)| i)
    }

    pub fn add(&self, track: &TextTrack) {
        // Only add a track if it does not exist in the list
        if self.find(track).is_none() {
            self.dom_tracks.borrow_mut().push(Dom::from_ref(track));

            let this = Trusted::new(self);
            let (source, canceller) = &self
                .global()
                .as_window()
                .task_manager()
                .media_element_task_source_with_canceller();

            let idx = match self.find(track) {
                Some(t) => t,
                None => return,
            };

            let _ = source.queue_with_canceller(
                task!(track_event_queue: move || {
                    let this = this.root();

                    if let Some(track) = this.item(idx) {
                        let event = TrackEvent::new(
                            &this.global(),
                            atom!("addtrack"),
                            false,
                            false,
                            &Some(VideoTrackOrAudioTrackOrTextTrack::TextTrack(
                                DomRoot::from_ref(&track)
                            )),
                        );

                        event.upcast::<Event>().fire(this.upcast::<EventTarget>());
                    }
                }),
                canceller,
            );
            track.add_track_list(self);
        }
    }

    // FIXME(#22314, dlrobertson) allow TextTracks to be
    // removed from the TextTrackList.
    #[allow(dead_code)]
    pub fn remove(&self, idx: usize) {
        if let Some(track) = self.dom_tracks.borrow().get(idx) {
            track.remove_track_list();
        }
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
            .find(|track| track.id() == id_str)
            .map(|t| DomRoot::from_ref(&**t))
    }

    // https://html.spec.whatwg.org/multipage/#handler-texttracklist-onchange
    event_handler!(change, GetOnchange, SetOnchange);

    // https://html.spec.whatwg.org/multipage/#handler-texttracklist-onaddtrack
    event_handler!(addtrack, GetOnaddtrack, SetOnaddtrack);

    // https://html.spec.whatwg.org/multipage/#handler-texttracklist-onremovetrack
    event_handler!(removetrack, GetOnremovetrack, SetOnremovetrack);
}
