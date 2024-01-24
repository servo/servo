/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use servo_media::streams::registry::MediaStreamId;
use servo_media::streams::MediaStreamType;

use crate::dom::bindings::codegen::Bindings::MediaStreamTrackBinding::MediaStreamTrackMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub struct MediaStreamTrack {
    eventtarget: EventTarget,
    #[ignore_malloc_size_of = "defined in servo-media"]
    #[no_trace]
    id: MediaStreamId,
    #[ignore_malloc_size_of = "defined in servo-media"]
    #[no_trace]
    ty: MediaStreamType,
}

impl MediaStreamTrack {
    pub fn new_inherited(id: MediaStreamId, ty: MediaStreamType) -> MediaStreamTrack {
        MediaStreamTrack {
            eventtarget: EventTarget::new_inherited(),
            id,
            ty,
        }
    }

    pub fn new(
        global: &GlobalScope,
        id: MediaStreamId,
        ty: MediaStreamType,
    ) -> DomRoot<MediaStreamTrack> {
        reflect_dom_object(Box::new(MediaStreamTrack::new_inherited(id, ty)), global)
    }

    pub fn id(&self) -> MediaStreamId {
        self.id
    }

    pub fn ty(&self) -> MediaStreamType {
        self.ty
    }
}

impl MediaStreamTrackMethods for MediaStreamTrack {
    /// <https://w3c.github.io/mediacapture-main/#dom-mediastreamtrack-kind>
    fn Kind(&self) -> DOMString {
        match self.ty {
            MediaStreamType::Video => "video".into(),
            MediaStreamType::Audio => "audio".into(),
        }
    }

    /// <https://w3c.github.io/mediacapture-main/#dom-mediastreamtrack-id>
    fn Id(&self) -> DOMString {
        self.id.id().to_string().into()
    }

    /// <https://w3c.github.io/mediacapture-main/#dom-mediastreamtrack-clone>
    fn Clone(&self) -> DomRoot<MediaStreamTrack> {
        MediaStreamTrack::new(&self.global(), self.id, self.ty)
    }
}
