/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::reflect_dom_object_with_cx;

use crate::dom::bindings::codegen::Bindings::TextTrackBinding::{TextTrackKind, TextTrackMethods};
use crate::dom::bindings::codegen::Bindings::TextTrackListBinding::TextTrackListMethods;
use crate::dom::bindings::codegen::UnionTypes::VideoTrackOrAudioTrackOrTextTrack;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::html::htmlmediaelement::HTMLMediaElement;
use crate::dom::html::htmltrackelement::HTMLTrackElement;
use crate::dom::node::node::Node;
use crate::dom::texttrack::TextTrack;
use crate::dom::texttrackcue::TextTrackCue;
use crate::dom::trackevent::TrackEvent;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct TextTrackList {
    eventtarget: EventTarget,
    /// <https://html.spec.whatwg.org/multipage/#list-of-text-tracks>
    media_element: Dom<HTMLMediaElement>,
    dom_tracks: DomRefCell<Vec<Dom<TextTrack>>>,
}

impl TextTrackList {
    pub(crate) fn new_inherited(
        media_element: &HTMLMediaElement,
        tracks: &[&TextTrack],
    ) -> TextTrackList {
        TextTrackList {
            eventtarget: EventTarget::new_inherited(),
            media_element: Dom::from_ref(media_element),
            dom_tracks: DomRefCell::new(tracks.iter().map(|g| Dom::from_ref(&**g)).collect()),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        media_element: &HTMLMediaElement,
        window: &Window,
        tracks: &[&TextTrack],
    ) -> DomRoot<TextTrackList> {
        reflect_dom_object_with_cx(
            Box::new(TextTrackList::new_inherited(media_element, tracks)),
            window,
            cx,
        )
    }

    pub(crate) fn item(&self, idx: usize) -> Option<DomRoot<TextTrack>> {
        self.dom_tracks
            .borrow()
            .get(idx)
            .map(|t| DomRoot::from_ref(&**t))
    }

    pub(crate) fn find(&self, track: &TextTrack) -> Option<usize> {
        self.dom_tracks
            .borrow()
            .iter()
            .enumerate()
            .find(|(_, t)| **t == track)
            .map(|(i, _)| i)
    }

    pub(crate) fn tracks_for_kinds(
        &self,
        text_track_kinds: Vec<TextTrackKind>,
    ) -> Vec<DomRoot<TextTrack>> {
        self.dom_tracks
            .borrow()
            .iter()
            .filter(|track| text_track_kinds.contains(&track.Kind()))
            .map(|track| DomRoot::from_ref(&**track))
            .collect()
    }

    pub(crate) fn notify_media_element_for_added_cue(
        &self,
        cx: &mut JSContext,
        cue: &TextTrackCue,
    ) {
        self.media_element.add_newly_added_cue(cx, cue);
    }

    pub(crate) fn add(&self, cx: &mut JSContext, track: &TextTrack) {
        // Only add a track if it does not exist in the list
        if self.find(track).is_some() {
            return;
        }
        self.dom_tracks.borrow_mut().push(Dom::from_ref(track));

        track.add_track_list(self);
        self.media_element
            .was_added_to_list_of_text_tracks(cx, track);

        let this = Trusted::new(self);
        let track = Trusted::new(track);
        self.global()
            .task_manager()
            .media_element_task_source()
            .queue(task!(track_event_queue: move |cx| {
                let this = this.root();
                let track = track.root();

                let event = TrackEvent::new(
                    cx,
                    this.global().as_window(),
                    atom!("addtrack"),
                    false,
                    false,
                    &Some(VideoTrackOrAudioTrackOrTextTrack::TextTrack(
                        DomRoot::from_ref(&track)
                    )),
                );

                event.upcast::<Event>().fire(cx, this.upcast::<EventTarget>());
            }));
    }

    pub(crate) fn remove(&self, track: &TextTrack) {
        let Some(idx) = self.find(track) else {
            return;
        };
        self.dom_tracks.borrow_mut().remove(idx);
        track.remove_track_list();

        let this = Trusted::new(self);
        let track = Trusted::new(track);
        self.global()
            .task_manager()
            .media_element_task_source()
            .queue(task!(track_event_queue: move |cx| {
                let this = this.root();
                let track = track.root();

                let event = TrackEvent::new(
                    cx,
                    this.global().as_window(),
                    atom!("removetrack"),
                    false,
                    false,
                    &Some(VideoTrackOrAudioTrackOrTextTrack::TextTrack(
                        DomRoot::from_ref(&track)
                    )),
                );

                event.upcast::<Event>().fire(cx, this.upcast::<EventTarget>());
            }));
    }

    pub(crate) fn iter(&self) -> TextTrackListIterator {
        TextTrackListIterator {
            track_elements: self
                .media_element
                .upcast::<Node>()
                .children()
                .filter_map(DomRoot::downcast::<HTMLTrackElement>)
                .collect(),
            current_track_element_index: 0,
        }
    }
}

impl TextTrackListMethods<crate::DomTypeHolder> for TextTrackList {
    /// <https://html.spec.whatwg.org/multipage/#dom-texttracklist-length>
    fn Length(&self) -> u32 {
        self.dom_tracks.borrow().len() as u32
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-texttracklist-item>
    fn IndexedGetter(&self, idx: u32) -> Option<DomRoot<TextTrack>> {
        self.item(idx as usize)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-texttracklist-gettrackbyid>
    fn GetTrackById(&self, id: DOMString) -> Option<DomRoot<TextTrack>> {
        let id_str = String::from(id);
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

/// <https://html.spec.whatwg.org/multipage/#list-of-text-tracks>
pub(crate) struct TextTrackListIterator {
    track_elements: Vec<DomRoot<HTMLTrackElement>>,
    current_track_element_index: usize,
}

impl Iterator for TextTrackListIterator {
    type Item = DomRoot<TextTrack>;

    fn next(&mut self) -> Option<Self::Item> {
        // > The text tracks are sorted as follows:
        // Step 1. The text tracks corresponding to track element children of the media element, in tree order.
        if let Some(track_element) = self.track_elements.get(self.current_track_element_index) {
            self.current_track_element_index += 1;
            return Some(track_element.track());
        }
        // Step 2. Any text tracks added using the addTextTrack() method,
        // in the order they were added, oldest first.
        // TODO
        // Step 3. Any media-resource-specific text tracks
        // (text tracks corresponding to data in the media resource),
        // in the order defined by the media resource's format specification.
        // TODO
        None
    }
}
