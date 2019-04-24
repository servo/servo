/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::TextTrackCueBinding::{self, TextTrackCueMethods};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::texttrack::TextTrack;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use std::cell::Cell;

#[dom_struct]
pub struct TextTrackCue {
    eventtarget: EventTarget,
    id: DomRefCell<DOMString>,
    track: Option<Dom<TextTrack>>,
    start_time: Cell<f64>,
    end_time: Cell<f64>,
    pause_on_exit: Cell<bool>,
}

impl TextTrackCue {
    // FIXME(#22314, dlrobertson) implement VTTCue.
    #[allow(dead_code)]
    pub fn new_inherited(id: DOMString, track: Option<&TextTrack>) -> TextTrackCue {
        TextTrackCue {
            eventtarget: EventTarget::new_inherited(),
            id: DomRefCell::new(id),
            track: track.map(Dom::from_ref),
            start_time: Cell::new(0.),
            end_time: Cell::new(0.),
            pause_on_exit: Cell::new(false),
        }
    }

    // FIXME(#22314, dlrobertson) implement VTTCue.
    #[allow(dead_code)]
    pub fn new(window: &Window, id: DOMString, track: Option<&TextTrack>) -> DomRoot<TextTrackCue> {
        reflect_dom_object(
            Box::new(TextTrackCue::new_inherited(id, track)),
            window,
            TextTrackCueBinding::Wrap,
        )
    }

    pub fn id(&self) -> DOMString {
        self.id.borrow().clone()
    }

    pub fn get_track(&self) -> Option<DomRoot<TextTrack>> {
        self.track.as_ref().map(|t| DomRoot::from_ref(&**t))
    }
}

impl TextTrackCueMethods for TextTrackCue {
    // https://html.spec.whatwg.org/multipage/#dom-texttrackcue-id
    fn Id(&self) -> DOMString {
        self.id()
    }

    // https://html.spec.whatwg.org/multipage/#dom-texttrackcue-id
    fn SetId(&self, value: DOMString) {
        *self.id.borrow_mut() = value;
    }

    // https://html.spec.whatwg.org/multipage/#dom-texttrackcue-track
    fn GetTrack(&self) -> Option<DomRoot<TextTrack>> {
        self.get_track()
    }

    // https://html.spec.whatwg.org/multipage/#dom-texttrackcue-starttime
    fn StartTime(&self) -> Finite<f64> {
        Finite::wrap(self.start_time.get())
    }

    // https://html.spec.whatwg.org/multipage/#dom-texttrackcue-starttime
    fn SetStartTime(&self, value: Finite<f64>) {
        self.start_time.set(*value);
    }

    // https://html.spec.whatwg.org/multipage/#dom-texttrackcue-endtime
    fn EndTime(&self) -> Finite<f64> {
        Finite::wrap(self.end_time.get())
    }

    // https://html.spec.whatwg.org/multipage/#dom-texttrackcue-endtime
    fn SetEndTime(&self, value: Finite<f64>) {
        self.end_time.set(*value);
    }

    // https://html.spec.whatwg.org/multipage/#dom-texttrackcue-pauseonexit
    fn PauseOnExit(&self) -> bool {
        self.pause_on_exit.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-texttrackcue-pauseonexit
    fn SetPauseOnExit(&self, value: bool) {
        self.pause_on_exit.set(value);
    }

    // https://html.spec.whatwg.org/multipage/#handler-texttrackcue-onenter
    event_handler!(enter, GetOnenter, SetOnenter);

    // https://html.spec.whatwg.org/multipage/#handler-texttrackcue-onexit
    event_handler!(exit, GetOnexit, SetOnexit);
}
