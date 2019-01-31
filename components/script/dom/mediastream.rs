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
use servo_media::webrtc::MediaStream as BackendMediaStream;
use std::mem;

#[dom_struct]
pub struct MediaStream {
    eventtarget: EventTarget,
    #[ignore_malloc_size_of = "defined in servo-media"]
    tracks: DomRefCell<Vec<Box<BackendMediaStream>>>,
}

impl MediaStream {
    pub fn new_inherited(tracks: Vec<Box<BackendMediaStream>>) -> MediaStream {
        MediaStream {
            eventtarget: EventTarget::new_inherited(),
            tracks: DomRefCell::new(tracks),
        }
    }

    pub fn new(global: &GlobalScope, tracks: Vec<Box<BackendMediaStream>>) -> DomRoot<MediaStream> {
        reflect_dom_object(
            Box::new(MediaStream::new_inherited(tracks)),
            global,
            MediaStreamBinding::Wrap,
        )
    }

    pub fn get_tracks(&self) -> Vec<Box<BackendMediaStream>> {
        // XXXManishearth we have hard ownership constraints here so we actually empty the vec,
        // ideally we should only have a media stream id here, or servo-media hands
        // out Arcs
        let mut tracks = self.tracks.borrow_mut();
        mem::replace(&mut *tracks, vec![])
    }
}
