/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::MediaStreamBinding;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use servo_media::streams::registry::MediaStreamId;

#[dom_struct]
pub struct MediaStream {
    eventtarget: EventTarget,
    #[ignore_malloc_size_of = "defined in servo-media"]
    tracks: DomRefCell<Vec<MediaStreamId>>,
}

impl MediaStream {
    pub fn new_inherited(tracks: Vec<MediaStreamId>) -> MediaStream {
        MediaStream {
            eventtarget: EventTarget::new_inherited(),
            tracks: DomRefCell::new(tracks),
        }
    }

    pub fn new(global: &GlobalScope, tracks: Vec<MediaStreamId>) -> DomRoot<MediaStream> {
        reflect_dom_object(
            Box::new(MediaStream::new_inherited(tracks)),
            global,
            MediaStreamBinding::Wrap,
        )
    }

    pub fn get_tracks(&self) -> Vec<MediaStreamId> {
        self.tracks.borrow_mut().clone()
    }
}
