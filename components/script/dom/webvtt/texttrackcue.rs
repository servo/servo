/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::cmp::Ordering;

use dom_struct::dom_struct;
use script_bindings::cell::DomRefCell;

use crate::dom::bindings::codegen::Bindings::TextTrackCueBinding::TextTrackCueMethods;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::texttrack::TextTrack;

#[dom_struct]
pub(crate) struct TextTrackCue {
    eventtarget: EventTarget,
    /// <https://html.spec.whatwg.org/multipage/#text-track-cue-identifier>
    id: DomRefCell<DOMString>,
    /// <https://html.spec.whatwg.org/multipage/#dom-texttrackcue-track>
    text_track: MutNullableDom<TextTrack>,
    /// <https://html.spec.whatwg.org/multipage/#text-track-cue-start-time>
    start_time: Cell<f64>,
    /// <https://html.spec.whatwg.org/multipage/#text-track-cue-end-time>
    end_time: Cell<f64>,
    /// <https://html.spec.whatwg.org/multipage/#text-track-cue-pause-on-exit-flag>
    pause_on_exit: Cell<bool>,
    /// <https://html.spec.whatwg.org/multipage/#text-track-cue-active-flag>
    active: Cell<bool>,
    /// <https://html.spec.whatwg.org/multipage/#text-track-cue-order>
    /// > in the order they were last added to their respective
    /// > text track list of cues, oldest first
    initial_index_in_list: Cell<usize>,
}

impl TextTrackCue {
    pub(crate) fn new_inherited(
        id: DOMString,
        start_time: f64,
        end_time: f64,
        text_track: Option<&TextTrack>,
    ) -> TextTrackCue {
        TextTrackCue {
            eventtarget: EventTarget::new_inherited(),
            id: DomRefCell::new(id),
            text_track: MutNullableDom::new(text_track),
            start_time: Cell::new(start_time),
            end_time: Cell::new(end_time),
            pause_on_exit: Cell::new(false),
            active: Default::default(),
            initial_index_in_list: Default::default(),
        }
    }

    pub(crate) fn id(&self) -> DOMString {
        self.id.borrow().clone()
    }

    pub(crate) fn get_text_track(&self) -> Option<DomRoot<TextTrack>> {
        self.text_track.get()
    }

    pub(crate) fn set_text_track(&self, text_track: Option<&TextTrack>) {
        self.text_track.set(text_track);
    }

    pub(crate) fn start_time(&self) -> f64 {
        self.start_time.get()
    }

    pub(crate) fn end_time(&self) -> f64 {
        self.end_time.get()
    }

    pub(crate) fn is_active(&self) -> bool {
        self.active.get()
    }

    pub(crate) fn set_active(&self, active: bool) {
        self.active.set(active)
    }

    pub(crate) fn set_initial_index_in_list(&self, initial_index_in_list: usize) {
        self.initial_index_in_list.set(initial_index_in_list);
    }
}

impl TextTrackCueMethods<crate::DomTypeHolder> for TextTrackCue {
    /// <https://html.spec.whatwg.org/multipage/#dom-texttrackcue-id>
    fn Id(&self) -> DOMString {
        self.id()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-texttrackcue-id>
    fn SetId(&self, value: DOMString) {
        *self.id.borrow_mut() = value;
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-texttrackcue-track>
    fn GetTrack(&self) -> Option<DomRoot<TextTrack>> {
        self.get_text_track()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-texttrackcue-starttime>
    fn StartTime(&self) -> Finite<f64> {
        Finite::wrap(self.start_time.get())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-texttrackcue-starttime>
    fn SetStartTime(&self, value: Finite<f64>) {
        self.start_time.set(*value);
        if let Some(text_track) = self.text_track.get() {
            text_track.sort_cue_list();
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-texttrackcue-endtime>
    fn EndTime(&self) -> Finite<f64> {
        Finite::wrap(self.end_time.get())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-texttrackcue-endtime>
    fn SetEndTime(&self, value: Finite<f64>) {
        self.end_time.set(*value);
        if let Some(text_track) = self.text_track.get() {
            text_track.sort_cue_list();
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-texttrackcue-pauseonexit>
    fn PauseOnExit(&self) -> bool {
        self.pause_on_exit.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-texttrackcue-pauseonexit>
    fn SetPauseOnExit(&self, value: bool) {
        self.pause_on_exit.set(value);
    }

    // https://html.spec.whatwg.org/multipage/#handler-texttrackcue-onenter
    event_handler!(enter, GetOnenter, SetOnenter);

    // https://html.spec.whatwg.org/multipage/#handler-texttrackcue-onexit
    event_handler!(exit, GetOnexit, SetOnexit);
}

impl PartialOrd for TextTrackCue {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TextTrackCue {
    /// <https://html.spec.whatwg.org/multipage/#text-track-cue-order>
    fn cmp(&self, other: &TextTrackCue) -> Ordering {
        // > cues must be sorted by their start time, earliest first;
        self.start_time
            .get()
            .total_cmp(&other.start_time.get())
            .then_with(|| {
                // > then, any cues with the same start time must be sorted by their end time, latest first;
                self.end_time
                    .get()
                    .total_cmp(&other.end_time.get())
                    .reverse()
                    .then_with(|| {
                        // > and finally, any cues with identical end times must be sorted in the order
                        // > they were last added to their respective text track list of cues,
                        // > oldest first (so e.g. for cues from a WebVTT file,
                        // > that would initially be the order in which the cues were listed in the file).
                        self.initial_index_in_list
                            .get()
                            .cmp(&other.initial_index_in_list.get())
                    })
            })
    }
}
