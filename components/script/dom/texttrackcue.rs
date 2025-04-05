/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::TextTrackCueBinding::TextTrackCueMethods;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::texttrack::TextTrack;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct TextTrackCue {
    eventtarget: EventTarget,
    id: DomRefCell<DOMString>,
    track: Option<Dom<TextTrack>>,
    start_time: Cell<f64>,
    end_time: Cell<f64>,
    pause_on_exit: Cell<bool>,
}

impl TextTrackCue {
    pub(crate) fn new_inherited(
        id: DOMString,
        start_time: f64,
        end_time: f64,
        track: Option<&TextTrack>,
    ) -> TextTrackCue {
        TextTrackCue {
            eventtarget: EventTarget::new_inherited(),
            id: DomRefCell::new(id),
            track: track.map(Dom::from_ref),
            start_time: Cell::new(start_time),
            end_time: Cell::new(end_time),
            pause_on_exit: Cell::new(false),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn new(
        window: &Window,
        id: DOMString,
        start_time: f64,
        end_time: f64,
        track: Option<&TextTrack>,
        can_gc: CanGc,
    ) -> DomRoot<TextTrackCue> {
        reflect_dom_object(
            Box::new(TextTrackCue::new_inherited(id, start_time, end_time, track)),
            window,
            can_gc,
        )
    }

    pub(crate) fn id(&self) -> DOMString {
        self.id.borrow().clone()
    }

    pub(crate) fn get_track(&self) -> Option<DomRoot<TextTrack>> {
        self.track.as_ref().map(|t| DomRoot::from_ref(&**t))
    }
}

impl TextTrackCueMethods<crate::DomTypeHolder> for TextTrackCue {
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
