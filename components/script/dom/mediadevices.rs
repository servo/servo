/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::MediaDevicesBinding::MediaStreamConstraints;
use crate::dom::bindings::codegen::Bindings::MediaDevicesBinding::{self, MediaDevicesMethods};
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::mediastream::MediaStream;
use crate::dom::promise::Promise;
use dom_struct::dom_struct;
use servo_media::ServoMedia;
use std::rc::Rc;

#[dom_struct]
pub struct MediaDevices {
    eventtarget: EventTarget,
}

impl MediaDevices {
    pub fn new_inherited() -> MediaDevices {
        MediaDevices {
            eventtarget: EventTarget::new_inherited(),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<MediaDevices> {
        reflect_dom_object(
            Box::new(MediaDevices::new_inherited()),
            global,
            MediaDevicesBinding::Wrap,
        )
    }
}

impl MediaDevicesMethods for MediaDevices {
    /// https://w3c.github.io/mediacapture-main/#dom-mediadevices-getusermedia
    fn GetUserMedia(&self, constraints: &MediaStreamConstraints) -> Rc<Promise> {
        let p = Promise::new(&self.global());
        let media = ServoMedia::get().unwrap();
        let mut tracks = vec![];
        if constraints.audio {
            if let Some(audio) = media.create_audioinput_stream() {
                tracks.push(audio)
            }
        }
        if constraints.video {
            if let Some(video) = media.create_videoinput_stream() {
                tracks.push(video)
            }
        }
        let stream = MediaStream::new(&self.global(), tracks);
        p.resolve_native(&stream);
        p
    }
}
