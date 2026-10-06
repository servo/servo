/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object};

use crate::dom::audio::audiotracklist::AudioTrackList;
use crate::dom::bindings::codegen::Bindings::AudioTrackBinding::AudioTrackMethods;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::RootedDOMString;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct AudioTrack {
    reflector_: Reflector,
    id: RootedDOMString,
    kind: RootedDOMString,
    label: RootedDOMString,
    language: RootedDOMString,
    enabled: Cell<bool>,
    track_list: DomRefCell<Option<Dom<AudioTrackList>>>,
}

impl AudioTrack {
    pub(crate) fn new_inherited(
        id: RootedDOMString,
        kind: RootedDOMString,
        label: RootedDOMString,
        language: RootedDOMString,
        track_list: Option<&AudioTrackList>,
    ) -> AudioTrack {
        AudioTrack {
            reflector_: Reflector::new(),
            id,
            kind,
            label,
            language,
            enabled: Cell::new(false),
            track_list: DomRefCell::new(track_list.map(Dom::from_ref)),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        id: RootedDOMString,
        kind: RootedDOMString,
        label: RootedDOMString,
        language: RootedDOMString,
        track_list: Option<&AudioTrackList>,
    ) -> DomRoot<AudioTrack> {
        reflect_dom_object(
            cx,
            Box::new(AudioTrack::new_inherited(
                id, kind, label, language, track_list,
            )),
            window,
        )
    }

    pub(crate) fn id(&self) -> RootedDOMString {
        self.id.clone()
    }

    pub(crate) fn kind(&self) -> RootedDOMString {
        self.kind.clone()
    }

    pub(crate) fn enabled(&self) -> bool {
        self.enabled.get()
    }

    pub(crate) fn set_enabled(&self, value: bool) {
        self.enabled.set(value);
    }

    pub(crate) fn add_track_list(&self, track_list: &AudioTrackList) {
        *self.track_list.borrow_mut() = Some(Dom::from_ref(track_list));
    }

    pub(crate) fn remove_track_list(&self) {
        *self.track_list.borrow_mut() = None;
    }
}

impl AudioTrackMethods<crate::DomTypeHolder> for AudioTrack {
    /// <https://html.spec.whatwg.org/multipage/#dom-audiotrack-id>
    fn Id(&self) -> RootedDOMString {
        self.id()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-audiotrack-kind>
    fn Kind(&self) -> RootedDOMString {
        self.kind()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-audiotrack-label>
    fn Label(&self) -> RootedDOMString {
        self.label.clone()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-audiotrack-language>
    fn Language(&self) -> RootedDOMString {
        self.language.clone()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-audiotrack-enabled>
    fn Enabled(&self) -> bool {
        self.enabled()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-audiotrack-enabled>
    fn SetEnabled(&self, value: bool) {
        if let Some(list) = self.track_list.borrow().as_ref() &&
            let Some(idx) = list.find(self)
        {
            list.set_enabled(idx, value);
        }
        self.set_enabled(value);
    }
}
