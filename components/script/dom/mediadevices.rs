/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use servo_media::streams::capture::{Constrain, ConstrainRange, MediaTrackConstraintSet};
use servo_media::streams::MediaStreamType;
use servo_media::ServoMedia;

use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::MediaDevicesBinding::{
    MediaDevicesMethods, MediaStreamConstraints,
};
use crate::dom::bindings::codegen::UnionTypes::{
    BooleanOrMediaTrackConstraints, ClampedUnsignedLongOrConstrainULongRange as ConstrainULong,
    DoubleOrConstrainDoubleRange as ConstrainDouble,
};
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::DomRoot;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::mediadeviceinfo::MediaDeviceInfo;
use crate::dom::mediastream::MediaStream;
use crate::dom::mediastreamtrack::MediaStreamTrack;
use crate::dom::promise::Promise;
use crate::realms::{AlreadyInRealm, InRealm};
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct MediaDevices {
    eventtarget: EventTarget,
}

impl MediaDevices {
    pub(crate) fn new_inherited() -> MediaDevices {
        MediaDevices {
            eventtarget: EventTarget::new_inherited(),
        }
    }

    pub(crate) fn new(global: &GlobalScope) -> DomRoot<MediaDevices> {
        reflect_dom_object(
            Box::new(MediaDevices::new_inherited()),
            global,
            CanGc::note(),
        )
    }
}

impl MediaDevicesMethods<crate::DomTypeHolder> for MediaDevices {
    /// <https://w3c.github.io/mediacapture-main/#dom-mediadevices-getusermedia>
    #[allow(unsafe_code)]
    fn GetUserMedia(
        &self,
        constraints: &MediaStreamConstraints,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        let p = Promise::new_in_current_realm(comp, can_gc);
        let media = ServoMedia::get();
        let stream = MediaStream::new(&self.global(), can_gc);
        if let Some(constraints) = convert_constraints(&constraints.audio) {
            if let Some(audio) = media.create_audioinput_stream(constraints) {
                let track = MediaStreamTrack::new(&self.global(), audio, MediaStreamType::Audio);
                stream.add_track(&track);
            }
        }
        if let Some(constraints) = convert_constraints(&constraints.video) {
            if let Some(video) = media.create_videoinput_stream(constraints) {
                let track = MediaStreamTrack::new(&self.global(), video, MediaStreamType::Video);
                stream.add_track(&track);
            }
        }

        p.resolve_native(&stream);
        p
    }

    /// <https://w3c.github.io/mediacapture-main/#dom-mediadevices-enumeratedevices>
    fn EnumerateDevices(&self, can_gc: CanGc) -> Rc<Promise> {
        // Step 1.
        let in_realm_proof = AlreadyInRealm::assert();
        let p = Promise::new_in_current_realm(InRealm::Already(&in_realm_proof), can_gc);

        // Step 2.
        // XXX These steps should be run in parallel.
        // XXX Steps 2.1 - 2.4

        // Step 2.5
        let media = ServoMedia::get();
        let device_monitor = media.get_device_monitor();
        let result_list = match device_monitor.enumerate_devices() {
            Ok(devices) => devices
                .iter()
                .map(|device| {
                    // XXX The media backend has no way to group devices yet.
                    MediaDeviceInfo::new(
                        &self.global(),
                        &device.device_id,
                        device.kind.convert(),
                        &device.label,
                        "",
                    )
                })
                .collect(),
            Err(_) => Vec::new(),
        };

        p.resolve_native(&result_list);

        // Step 3.
        p
    }
}

fn convert_constraints(js: &BooleanOrMediaTrackConstraints) -> Option<MediaTrackConstraintSet> {
    match js {
        BooleanOrMediaTrackConstraints::Boolean(false) => None,
        BooleanOrMediaTrackConstraints::Boolean(true) => Some(Default::default()),
        BooleanOrMediaTrackConstraints::MediaTrackConstraints(ref c) => {
            Some(MediaTrackConstraintSet {
                height: c.parent.height.as_ref().and_then(convert_culong),
                width: c.parent.width.as_ref().and_then(convert_culong),
                aspect: c.parent.aspectRatio.as_ref().and_then(convert_cdouble),
                frame_rate: c.parent.frameRate.as_ref().and_then(convert_cdouble),
                sample_rate: c.parent.sampleRate.as_ref().and_then(convert_culong),
            })
        },
    }
}

fn convert_culong(js: &ConstrainULong) -> Option<Constrain<u32>> {
    match js {
        ConstrainULong::ClampedUnsignedLong(val) => Some(Constrain::Value(*val)),
        ConstrainULong::ConstrainULongRange(ref range) => {
            if range.parent.min.is_some() || range.parent.max.is_some() {
                Some(Constrain::Range(ConstrainRange {
                    min: range.parent.min,
                    max: range.parent.max,
                    ideal: range.ideal,
                }))
            } else {
                range.exact.map(Constrain::Value)
            }
        },
    }
}

fn convert_cdouble(js: &ConstrainDouble) -> Option<Constrain<f64>> {
    match js {
        ConstrainDouble::Double(val) => Some(Constrain::Value(**val)),
        ConstrainDouble::ConstrainDoubleRange(ref range) => {
            if range.parent.min.is_some() || range.parent.max.is_some() {
                Some(Constrain::Range(ConstrainRange {
                    min: range.parent.min.map(|x| *x),
                    max: range.parent.max.map(|x| *x),
                    ideal: range.ideal.map(|x| *x),
                }))
            } else {
                range.exact.map(|exact| Constrain::Value(*exact))
            }
        },
    }
}
