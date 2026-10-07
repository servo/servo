/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object};

use crate::dom::bindings::codegen::Bindings::VideoTrackBinding::VideoTrackMethods;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::RootedDomString;
use crate::dom::videotracklist::VideoTrackList;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct VideoTrack {
    reflector_: Reflector,
    id: RootedDomString,
    kind: RootedDomString,
    label: RootedDomString,
    language: RootedDomString,
    selected: Cell<bool>,
    track_list: DomRefCell<Option<Dom<VideoTrackList>>>,
}

impl VideoTrack {
    pub(crate) fn new_inherited(
        id: RootedDomString,
        kind: RootedDomString,
        label: RootedDomString,
        language: RootedDomString,
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
        cx: &mut JSContext,
        window: &Window,
        id: RootedDomString,
        kind: RootedDomString,
        label: RootedDomString,
        language: RootedDomString,
        track_list: Option<&VideoTrackList>,
    ) -> DomRoot<VideoTrack> {
        reflect_dom_object(
            cx,
            Box::new(VideoTrack::new_inherited(
                id, kind, label, language, track_list,
            )),
            window,
        )
    }

    pub(crate) fn id(&self) -> RootedDomString {
        self.id.clone()
    }

    pub(crate) fn kind(&self) -> RootedDomString {
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
    /// <https://html.spec.whatwg.org/multipage/#dom-videotrack-id>
    fn Id(&self) -> RootedDomString {
        self.id()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-videotrack-kind>
    fn Kind(&self) -> RootedDomString {
        self.kind()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-videotrack-label>
    fn Label(&self) -> RootedDomString {
        self.label.clone()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-videotrack-language>
    fn Language(&self) -> RootedDomString {
        self.language.clone()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-videotrack-selected>
    fn Selected(&self) -> bool {
        self.selected()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-videotrack-selected>
    fn SetSelected(&self, value: bool) {
        if let Some(list) = self.track_list.borrow().as_ref() &&
            let Some(idx) = list.find(self)
        {
            list.set_selected(idx, value);
        }
        self.set_selected(value);
    }
}
