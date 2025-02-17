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
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::mediastreamtrack::MediaStreamTrack;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct MediaStream {
    eventtarget: EventTarget,
    tracks: DomRefCell<Vec<Dom<MediaStreamTrack>>>,
}

impl MediaStream {
    pub(crate) fn new_inherited() -> MediaStream {
        MediaStream {
            eventtarget: EventTarget::new_inherited(),
            tracks: DomRefCell::new(vec![]),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<MediaStream> {
        Self::new_with_proto(global, None, can_gc)
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<MediaStream> {
        reflect_dom_object_with_proto(
            Box::new(MediaStream::new_inherited()),
            global,
            proto,
            can_gc,
        )
    }

    pub(crate) fn new_single(
        global: &GlobalScope,
        id: MediaStreamId,
        ty: MediaStreamType,
        can_gc: CanGc,
    ) -> DomRoot<MediaStream> {
        let this = Self::new(global, can_gc);
        let track = MediaStreamTrack::new(global, id, ty, can_gc);
        this.AddTrack(&track);
        this
    }

    pub(crate) fn get_tracks(&self) -> Ref<[Dom<MediaStreamTrack>]> {
        Ref::map(self.tracks.borrow(), |tracks| &**tracks)
    }

    pub(crate) fn add_track(&self, track: &MediaStreamTrack) {
        self.tracks.borrow_mut().push(Dom::from_ref(track))
    }
}

impl MediaStreamMethods<crate::DomTypeHolder> for MediaStream {
    /// <https://w3c.github.io/mediacapture-main/#dom-mediastream-constructor>
    fn Constructor(
        global: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<MediaStream>> {
        Ok(MediaStream::new_with_proto(&global.global(), proto, can_gc))
    }

    /// <https://w3c.github.io/mediacapture-main/#dom-mediastream-constructor>
    fn Constructor_(
        _: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        stream: &MediaStream,
    ) -> Fallible<DomRoot<MediaStream>> {
        Ok(stream.clone_with_proto(proto, can_gc))
    }

    /// <https://w3c.github.io/mediacapture-main/#dom-mediastream-constructor>
    fn Constructor__(
        global: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        tracks: Vec<DomRoot<MediaStreamTrack>>,
    ) -> Fallible<DomRoot<MediaStream>> {
        let new = MediaStream::new_with_proto(&global.global(), proto, can_gc);
        for track in tracks {
            // this is quadratic, but shouldn't matter much
            // if this becomes a problem we can use a hash map
            new.AddTrack(&track)
        }
        Ok(new)
    }

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
    fn Clone(&self, can_gc: CanGc) -> DomRoot<MediaStream> {
        self.clone_with_proto(None, can_gc)
    }
}

impl MediaStream {
    fn clone_with_proto(&self, proto: Option<HandleObject>, can_gc: CanGc) -> DomRoot<MediaStream> {
        let new = MediaStream::new_with_proto(&self.global(), proto, can_gc);
        for track in &*self.tracks.borrow() {
            new.add_track(track)
        }
        new
    }
}
