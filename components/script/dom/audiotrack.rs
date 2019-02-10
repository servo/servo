/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::AudioTrackBinding::{self, AudioTrackMethods};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use std::cell::Cell;

#[dom_struct]
pub struct AudioTrack {
    reflector_: Reflector,
    id: DOMString,
    kind: DOMString,
    label: DOMString,
    language: DOMString,
    enabled: Cell<bool>,
}

impl AudioTrack {
    pub fn new_inherited(
        id: DOMString,
        kind: DOMString,
        label: DOMString,
        language: DOMString,
    ) -> AudioTrack {
        AudioTrack {
            reflector_: Reflector::new(),
            id: id.into(),
            kind: kind.into(),
            label: label.into(),
            language: language.into(),
            enabled: Cell::new(false),
        }
    }

    pub fn new(
        window: &Window,
        id: DOMString,
        kind: DOMString,
        label: DOMString,
        language: DOMString,
    ) -> DomRoot<AudioTrack> {
        reflect_dom_object(
            Box::new(AudioTrack::new_inherited(id, kind, label, language)),
            window,
            AudioTrackBinding::Wrap,
        )
    }

    pub fn id(&self) -> DOMString {
        self.id.clone()
    }

    pub fn enabled(&self) -> bool {
        self.enabled.get()
    }

    pub fn set_enabled(&self, value: bool) {
        self.enabled.set(value);
    }
}

impl AudioTrackMethods for AudioTrack {
    // https://html.spec.whatwg.org/multipage/#dom-audiotrack-id
    fn Id(&self) -> DOMString {
        self.id()
    }

    // https://html.spec.whatwg.org/multipage/#dom-audiotrack-kind
    fn Kind(&self) -> DOMString {
        self.kind.clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-audiotrack-label
    fn Label(&self) -> DOMString {
        self.label.clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-audiotrack-language
    fn Language(&self) -> DOMString {
        self.language.clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-audiotrack-enabled
    fn Enabled(&self) -> bool {
        self.enabled()
    }

    // https://html.spec.whatwg.org/multipage/#dom-audiotrack-enabled
    fn SetEnabled(&self, value: bool) {
        self.set_enabled(value);
    }
}
