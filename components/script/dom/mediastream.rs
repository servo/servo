/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_media::streams::registry::MediaStreamId;
use servo_media::streams::MediaStreamType;

use crate::dom::bindings::cell::{DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::MediaStreamBinding::MediaStreamMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::mediastreamtrack::MediaStreamTrack;
use crate::dom::window::Window;

#[dom_struct]
pub struct MediaStream {
    eventtarget: EventTarget,
    tracks: DomRefCell<Vec<Dom<MediaStreamTrack>>>,
}

#[allow(non_snake_case)]
impl MediaStream {
    pub fn new_inherited() -> MediaStream {
        MediaStream {
            eventtarget: EventTarget::new_inherited(),
            tracks: DomRefCell::new(vec![]),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<MediaStream> {
        Self::new_with_proto(global, None)
    }

    fn new_with_proto(global: &GlobalScope, proto: Option<HandleObject>) -> DomRoot<MediaStream> {
        reflect_dom_object_with_proto(Box::new(MediaStream::new_inherited()), global, proto)
    }

    pub fn new_single(
        global: &GlobalScope,
        id: MediaStreamId,
        ty: MediaStreamType,
    ) -> DomRoot<MediaStream> {
        let this = Self::new(global);
        let track = MediaStreamTrack::new(global, id, ty);
        this.AddTrack(&track);
        this
    }

    pub fn Constructor(
        global: &Window,
        proto: Option<HandleObject>,
    ) -> Fallible<DomRoot<MediaStream>> {
        Ok(MediaStream::new_with_proto(&global.global(), proto))
    }

    pub fn Constructor_(
        _: &Window,
        proto: Option<HandleObject>,
        stream: &MediaStream,
    ) -> Fallible<DomRoot<MediaStream>> {
        Ok(stream.clone_with_proto(proto))
    }

    pub fn Constructor__(
        global: &Window,
        proto: Option<HandleObject>,
        tracks: Vec<DomRoot<MediaStreamTrack>>,
    ) -> Fallible<DomRoot<MediaStream>> {
        let new = MediaStream::new_with_proto(&global.global(), proto);
        for track in tracks {
            // this is quadratic, but shouldn't matter much
            // if this becomes a problem we can use a hash map
            new.AddTrack(&track)
        }
        Ok(new)
    }

    pub fn get_tracks(&self) -> Ref<[Dom<MediaStreamTrack>]> {
        Ref::map(self.tracks.borrow(), |tracks| &**tracks)
    }

    pub fn add_track(&self, track: &MediaStreamTrack) {
        self.tracks.borrow_mut().push(Dom::from_ref(track))
    }
}

impl MediaStreamMethods for MediaStream {
    /// <https://w3c.github.io/mediacapture-main/#dom-mediastream-gettracks>
    fn GetTracks(&self) -> Vec<DomRoot<MediaStreamTrack>> {
        self.tracks
            .borrow()
            .iter()
            .map(|x| DomRoot::from_ref(&**x))
            .collect()
    }

    /// <https://w3c.github.io/mediacapture-main/#dom-mediastream-getaudiotracks>
    fn GetAudioTracks(&self) -> Vec<DomRoot<MediaStreamTrack>> {
        self.tracks
            .borrow()
            .iter()
            .filter(|x| x.ty() == MediaStreamType::Audio)
            .map(|x| DomRoot::from_ref(&**x))
            .collect()
    }

    /// <https://w3c.github.io/mediacapture-main/#dom-mediastream-getvideotracks>
    fn GetVideoTracks(&self) -> Vec<DomRoot<MediaStreamTrack>> {
        self.tracks
            .borrow()
            .iter()
            .filter(|x| x.ty() == MediaStreamType::Video)
            .map(|x| DomRoot::from_ref(&**x))
            .collect()
    }

    /// <https://w3c.github.io/mediacapture-main/#dom-mediastream-gettrackbyid>
    fn GetTrackById(&self, id: DOMString) -> Option<DomRoot<MediaStreamTrack>> {
        self.tracks
            .borrow()
            .iter()
            .find(|x| x.id().id().to_string() == *id)
            .map(|x| DomRoot::from_ref(&**x))
    }

    /// <https://w3c.github.io/mediacapture-main/#dom-mediastream-addtrack>
    fn AddTrack(&self, track: &MediaStreamTrack) {
        let existing = self.tracks.borrow().iter().any(|x| x == &track);

        if existing {
            return;
        }
        self.add_track(track)
    }

    /// <https://w3c.github.io/mediacapture-main/#dom-mediastream-removetrack>
    fn RemoveTrack(&self, track: &MediaStreamTrack) {
        self.tracks.borrow_mut().retain(|x| *x != track);
    }

    /// <https://w3c.github.io/mediacapture-main/#dom-mediastream-clone>
    fn Clone(&self) -> DomRoot<MediaStream> {
        self.clone_with_proto(None)
    }
}

impl MediaStream {
    fn clone_with_proto(&self, proto: Option<HandleObject>) -> DomRoot<MediaStream> {
        let new = MediaStream::new_with_proto(&self.global(), proto);
        for track in &*self.tracks.borrow() {
            new.add_track(track)
        }
        new
    }
}
