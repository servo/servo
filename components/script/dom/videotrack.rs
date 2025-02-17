/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::VideoTrackBinding::VideoTrackMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::videotracklist::VideoTrackList;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct VideoTrack {
    reflector_: Reflector,
    id: DOMString,
    kind: DOMString,
    label: DOMString,
    language: DOMString,
    selected: Cell<bool>,
    track_list: DomRefCell<Option<Dom<VideoTrackList>>>,
}

impl VideoTrack {
    pub(crate) fn new_inherited(
        id: DOMString,
        kind: DOMString,
        label: DOMString,
        language: DOMString,
        track_list: Option<&VideoTrackList>,
    ) -> VideoTrack {
        VideoTrack {
            reflector_: Reflector::new(),
            id,
            kind,
            label,
            language,
            selected: Cell::new(false),
            track_list: DomRefCell::new(track_list.map(Dom::from_ref)),
        }
    }

    pub(crate) fn new(
        window: &Window,
        id: DOMString,
        kind: DOMString,
        label: DOMString,
        language: DOMString,
        track_list: Option<&VideoTrackList>,
        can_gc: CanGc,
    ) -> DomRoot<VideoTrack> {
        reflect_dom_object(
            Box::new(VideoTrack::new_inherited(
                id, kind, label, language, track_list,
            )),
            window,
            can_gc,
        )
    }

    pub(crate) fn id(&self) -> DOMString {
        self.id.clone()
    }

    pub(crate) fn kind(&self) -> DOMString {
        self.kind.clone()
    }

    pub(crate) fn selected(&self) -> bool {
        self.selected.get()
    }

    pub(crate) fn set_selected(&self, value: bool) {
        self.selected.set(value);
    }

    pub(crate) fn add_track_list(&self, track_list: &VideoTrackList) {
        *self.track_list.borrow_mut() = Some(Dom::from_ref(track_list));
    }

    pub(crate) fn remove_track_list(&self) {
        *self.track_list.borrow_mut() = None;
    }
}

impl VideoTrackMethods<crate::DomTypeHolder> for VideoTrack {
    // https://html.spec.whatwg.org/multipage/#dom-videotrack-id
    fn Id(&self) -> DOMString {
        self.id()
    }

    // https://html.spec.whatwg.org/multipage/#dom-videotrack-kind
    fn Kind(&self) -> DOMString {
        self.kind()
    }

    // https://html.spec.whatwg.org/multipage/#dom-videotrack-label
    fn Label(&self) -> DOMString {
        self.label.clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-videotrack-language
    fn Language(&self) -> DOMString {
        self.language.clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-videotrack-selected
    fn Selected(&self) -> bool {
        self.selected()
    }

    // https://html.spec.whatwg.org/multipage/#dom-videotrack-selected
    fn SetSelected(&self, value: bool) {
        if let Some(list) = self.track_list.borrow().as_ref() {
            if let Some(idx) = list.find(self) {
                list.set_selected(idx, value);
            }
        }
        self.set_selected(value);
    }
}
