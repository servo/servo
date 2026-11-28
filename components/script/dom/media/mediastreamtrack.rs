/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use script_bindings::reflector::reflect_dom_object;
use servo_media::streams::MediaStreamType;
use servo_media::streams::registry::MediaStreamId;

use crate::dom::bindings::codegen::Bindings::MediaStreamTrackBinding::{
    MediaStreamTrackMethods, MediaStreamTrackState,
};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub(crate) struct MediaStreamTrack {
    eventtarget: EventTarget,
    #[no_trace]
    id: MediaStreamId,
    #[no_trace]
    ty: MediaStreamType,
    /// Whether the source is a capture device managed by this document.
    capture: bool,
    ended: Cell<bool>,
}

impl MediaStreamTrack {
    pub(crate) fn new_inherited(
        id: MediaStreamId,
        ty: MediaStreamType,
        capture: bool,
    ) -> MediaStreamTrack {
        MediaStreamTrack {
            eventtarget: EventTarget::new_inherited(),
            id,
            ty,
            capture,
            ended: Cell::new(false),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        global: &GlobalScope,
        id: MediaStreamId,
        ty: MediaStreamType,
    ) -> DomRoot<MediaStreamTrack> {
        reflect_dom_object(
            cx,
            Box::new(MediaStreamTrack::new_inherited(id, ty, false)),
            global,
        )
    }

    pub(crate) fn new_capture(
        cx: &mut js::context::JSContext,
        global: &GlobalScope,
        id: MediaStreamId,
        ty: MediaStreamType,
    ) -> DomRoot<MediaStreamTrack> {
        reflect_dom_object(
            cx,
            Box::new(MediaStreamTrack::new_inherited(id, ty, true)),
            global,
        )
    }

    pub(crate) fn is_ended(&self) -> bool {
        self.ended.get()
    }

    /// End the track without releasing its source; the caller owns cleanup.
    pub(crate) fn end(&self) {
        self.ended.set(true);
    }

    pub(crate) fn id(&self) -> MediaStreamId {
        self.id
    }

    pub(crate) fn ty(&self) -> MediaStreamType {
        self.ty
    }
}

impl MediaStreamTrackMethods<crate::DomTypeHolder> for MediaStreamTrack {
    /// <https://w3c.github.io/mediacapture-main/#dom-mediastreamtrack-kind>
    fn Kind(&self) -> DOMString {
        match self.ty {
            MediaStreamType::Video => DOMString::from_static("video"),
            MediaStreamType::Audio => DOMString::from_static("audio"),
        }
    }

    /// <https://w3c.github.io/mediacapture-main/#dom-mediastreamtrack-id>
    fn Id(&self) -> DOMString {
        self.id.id().to_string().into()
    }

    /// <https://w3c.github.io/mediacapture-main/#dom-mediastreamtrack-clone>
    fn Clone(&self, cx: &mut js::context::JSContext) -> DomRoot<MediaStreamTrack> {
        let clone = if self.capture {
            MediaStreamTrack::new_capture(cx, &self.global(), self.id, self.ty)
        } else {
            MediaStreamTrack::new(cx, &self.global(), self.id, self.ty)
        };
        clone.ended.set(self.ended.get());
        if self.capture &&
            !clone.is_ended() &&
            let Some(devices) = self.global().as_window().maybe_media_devices()
        {
            devices.register_capture_track(&clone);
        }
        clone
    }

    /// <https://w3c.github.io/mediacapture-main/#dom-mediastreamtrack-readystate>
    fn ReadyState(&self) -> MediaStreamTrackState {
        if self.ended.get() {
            MediaStreamTrackState::Ended
        } else {
            MediaStreamTrackState::Live
        }
    }

    /// <https://w3c.github.io/mediacapture-main/#dom-mediastreamtrack-stop>
    fn Stop(&self) {
        if self.ended.replace(true) || !self.capture {
            return;
        }
        if let Some(devices) = self.global().as_window().maybe_media_devices() {
            devices.release_source_if_unused(self.id);
        }
    }
}
