/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::MediaStreamBinding::{self, MediaStreamMethods};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::mediastreamtrack::MediaStreamTrack;
use dom_struct::dom_struct;
use servo_media::streams::MediaStreamType;
use std::cell::Ref;

#[dom_struct]
pub struct MediaStream {
    eventtarget: EventTarget,
    tracks: DomRefCell<Vec<Dom<MediaStreamTrack>>>,
}

impl MediaStream {
    pub fn new_inherited() -> MediaStream {
        MediaStream {
            eventtarget: EventTarget::new_inherited(),
            tracks: DomRefCell::new(vec![]),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<MediaStream> {
        reflect_dom_object(
            Box::new(MediaStream::new_inherited()),
            global,
            MediaStreamBinding::Wrap,
        )
    }

    pub fn get_tracks(&self) -> Ref<[Dom<MediaStreamTrack>]> {
        Ref::map(self.tracks.borrow(), |tracks| &**tracks)
    }

    pub fn add_track(&self, track: &MediaStreamTrack) {
        self.tracks.borrow_mut().push(Dom::from_ref(track))
    }
}

impl MediaStreamMethods for MediaStream {
    /// https://w3c.github.io/mediacapture-main/#dom-mediastream-gettracks
    fn GetTracks(&self) -> Vec<DomRoot<MediaStreamTrack>> {
        self.tracks
            .borrow()
            .iter()
            .map(|x| DomRoot::from_ref(&**x))
            .collect()
    }

    /// https://w3c.github.io/mediacapture-main/#dom-mediastream-getaudiotracks
    fn GetAudioTracks(&self) -> Vec<DomRoot<MediaStreamTrack>> {
        self.tracks
            .borrow()
            .iter()
            .filter(|x| x.ty() == MediaStreamType::Audio)
            .map(|x| DomRoot::from_ref(&**x))
            .collect()
    }

    /// https://w3c.github.io/mediacapture-main/#dom-mediastream-getvideotracks
    fn GetVideoTracks(&self) -> Vec<DomRoot<MediaStreamTrack>> {
        self.tracks
            .borrow()
            .iter()
            .filter(|x| x.ty() == MediaStreamType::Video)
            .map(|x| DomRoot::from_ref(&**x))
            .collect()
    }

    /// https://w3c.github.io/mediacapture-main/#dom-mediastream-gettrackbyid
    fn GetTrackById(&self, id: DOMString) -> Option<DomRoot<MediaStreamTrack>> {
        self.tracks
            .borrow()
            .iter()
            .find(|x| x.id().id().to_string() == &*id)
            .map(|x| DomRoot::from_ref(&**x))
    }

    /// https://w3c.github.io/mediacapture-main/#dom-mediastream-addtrack
    fn AddTrack(&self, track: &MediaStreamTrack) {
        let existing = self.tracks.borrow().iter().find(|x| *x == &track).is_some();

        if existing {
            return;
        }
        self.add_track(track)
    }

    /// https://w3c.github.io/mediacapture-main/#dom-mediastream-addtrack
    fn RemoveTrack(&self, track: &MediaStreamTrack) {
        self.tracks.borrow_mut().retain(|x| *x != track);
    }

    /// https://w3c.github.io/mediacapture-main/#dom-mediastream-clone
    fn Clone(&self) -> DomRoot<MediaStream> {
        let new = MediaStream::new(&self.global());
        for track in &*self.tracks.borrow_mut() {
            new.add_track(&track)
        }
        new
    }
}
