/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::MediaStreamTrackBinding;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use servo_media::streams::registry::MediaStreamId;

#[dom_struct]
pub struct MediaStreamTrack {
    eventtarget: EventTarget,
    #[ignore_malloc_size_of = "defined in servo-media"]
    id: MediaStreamId,
}

impl MediaStreamTrack {
    pub fn new_inherited(id: MediaStreamId) -> MediaStreamTrack {
        MediaStreamTrack {
            eventtarget: EventTarget::new_inherited(),
            id,
        }
    }

    pub fn new(global: &GlobalScope, id: MediaStreamId) -> DomRoot<MediaStreamTrack> {
        reflect_dom_object(
            Box::new(MediaStreamTrack::new_inherited(id)),
            global,
            MediaStreamTrackBinding::Wrap,
        )
    }
}
