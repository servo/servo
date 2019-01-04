/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::VideoTrackBinding::{self, VideoTrackMethods};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use std::cell::Cell;

#[dom_struct]
pub struct VideoTrack {
    reflector_: Reflector,
    id: DOMString,
    kind: DOMString,
    label: DOMString,
    language: DOMString,
    selected: Cell<bool>,
}

impl VideoTrack {
    pub fn new_inherited(
        id: DOMString,
        kind: DOMString,
        label: DOMString,
        language: DOMString,
        selected: bool,
    ) -> VideoTrack {
        VideoTrack {
            reflector_: Reflector::new(),
            id: id.into(),
            kind: kind.into(),
            label: label.into(),
            language: language.into(),
            selected: Cell::new(selected),
        }
    }

    pub fn new(
        window: &Window,
        id: DOMString,
        kind: DOMString,
        label: DOMString,
        language: DOMString,
        selected: bool,
    ) -> DomRoot<VideoTrack> {
        reflect_dom_object(
            Box::new(VideoTrack::new_inherited(
                id, kind, label, language, selected,
            )),
            window,
            VideoTrackBinding::Wrap,
        )
    }

    pub fn id(&self) -> DOMString {
        self.id.clone()
    }

    pub fn selected(&self) -> bool {
        self.selected.get().clone()
    }

    pub fn set_selected(&self, value: bool) {
        self.selected.set(value);
    }
}

impl VideoTrackMethods for VideoTrack {
    // https://html.spec.whatwg.org/multipage/#dom-videotrack-id
    fn Id(&self) -> DOMString {
        self.id()
    }

    // https://html.spec.whatwg.org/multipage/#dom-videotrack-kind
    fn Kind(&self) -> DOMString {
        self.kind.clone()
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
        self.set_selected(value);
    }
}
