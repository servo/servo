/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::MediaDevicesBinding::MediaStreamConstraints;
use crate::dom::bindings::codegen::Bindings::MediaDevicesBinding::{self, MediaDevicesMethods};
use crate::dom::bindings::codegen::UnionTypes::BooleanOrMediaTrackConstraints;
use crate::dom::bindings::codegen::UnionTypes::ClampedUnsignedLongOrConstrainULongRange as ConstrainULong;
use crate::dom::bindings::codegen::UnionTypes::DoubleOrConstrainDoubleRange as ConstrainDouble;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::mediastream::MediaStream;
use crate::dom::promise::Promise;
use dom_struct::dom_struct;
use servo_media::streams::capture::{Constrain, ConstrainRange, MediaTrackConstraintSet};
use servo_media::ServoMedia;
use servo_media_auto::Backend;
use std::rc::Rc;

#[dom_struct]
pub struct MediaDevices {
    eventtarget: EventTarget,
}

impl MediaDevices {
    pub fn new_inherited() -> MediaDevices {
        ServoMedia::init::<Backend>();
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
    #[allow(unsafe_code)]
    fn GetUserMedia(&self, constraints: &MediaStreamConstraints) -> Rc<Promise> {
        let p = unsafe { Promise::new_in_current_compartment(&self.global()) };
        let media = ServoMedia::get().unwrap();
        let mut tracks = vec![];
        if let Some(constraints) = convert_constraints(&constraints.audio) {
            if let Some(audio) = media.create_audioinput_stream(constraints) {
                tracks.push(audio)
            }
        }
        if let Some(constraints) = convert_constraints(&constraints.video) {
            if let Some(video) = media.create_videoinput_stream(constraints) {
                tracks.push(video)
            }
        }
        let stream = MediaStream::new(&self.global(), tracks);
        p.resolve_native(&stream);
        p
    }
}

fn convert_constraints(js: &BooleanOrMediaTrackConstraints) -> Option<MediaTrackConstraintSet> {
    match js {
        BooleanOrMediaTrackConstraints::Boolean(false) => None,
        BooleanOrMediaTrackConstraints::Boolean(true) => Some(Default::default()),
        BooleanOrMediaTrackConstraints::MediaTrackConstraints(ref c) => {
            Some(MediaTrackConstraintSet {
                height: convert_culong(&c.parent.height),
                width: convert_culong(&c.parent.width),
                aspect: convert_cdouble(&c.parent.aspectRatio),
                frame_rate: convert_cdouble(&c.parent.frameRate),
                sample_rate: convert_culong(&c.parent.sampleRate),
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
            } else if let Some(exact) = range.exact {
                Some(Constrain::Value(exact))
            } else {
                // the unspecified case is treated as all three being none
                None
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
            } else if let Some(exact) = range.exact {
                Some(Constrain::Value(*exact))
            } else {
                // the unspecified case is treated as all three being none
                None
            }
        },
    }
}
