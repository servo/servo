/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::sync::{Arc, Mutex};

use dom_struct::dom_struct;
use embedder_traits::AllowOrDeny;
use js::context::JSContext;
use js::realm::CurrentRealm;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::reflect_dom_object;
use servo_base::generic_channel::GenericCallback;
use servo_media::ServoMedia;
use servo_media::streams::MediaStreamType;
use servo_media::streams::capture::{Constrain, ConstrainRange, MediaTrackConstraintSet};
use servo_media::streams::device_monitor::MediaDeviceKind as ServoMediaDeviceKind;
use servo_media::streams::registry::{MediaStreamId, get_stream};

use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::MediaDeviceInfoBinding::MediaDeviceKind;
use crate::dom::bindings::codegen::Bindings::MediaDevicesBinding::{
    MediaDevicesMethods, MediaStreamConstraints,
};
use crate::dom::bindings::codegen::Bindings::PermissionStatusBinding::{
    PermissionName, PermissionState,
};
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::UnionTypes::{
    BooleanOrMediaTrackConstraints, ClampedUnsignedLongOrConstrainULongRange as ConstrainULong,
    DoubleOrConstrainDoubleRange as ConstrainDouble,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::media::mediadeviceinfo::MediaDeviceInfo;
use crate::dom::media::mediastream::MediaStream;
use crate::dom::media::mediastreamtrack::MediaStreamTrack;
use crate::dom::permissions::{
    AsyncPermissionRequest, descriptor_permission_state, record_permission_request_result,
    request_permission_to_use_async,
};
use crate::dom::promise::{Promise, RootedPromise};

#[dom_struct]
pub(crate) struct MediaDevices {
    eventtarget: EventTarget,
    /// Capture tracks and their clones, retained until their source is released.
    capture_tracks: DomRefCell<Vec<Dom<MediaStreamTrack>>>,
    /// Invalidates permission responses from before document cleanup.
    capture_generation: Cell<u64>,
}

/// One capture request shared by the asynchronous permission callbacks.
struct CaptureRequest {
    promise: Option<TrustedPromise>,
    audio: Option<MediaTrackConstraintSet>,
    video: Option<MediaTrackConstraintSet>,
    /// `None` while outstanding; `Some(true)` for a kind that was not requested.
    audio_granted: Option<bool>,
    video_granted: Option<bool>,
    generation: u64,
}

impl MediaDevices {
    pub(crate) fn new_inherited() -> MediaDevices {
        MediaDevices {
            eventtarget: EventTarget::new_inherited(),
            capture_tracks: Default::default(),
            capture_generation: Cell::new(0),
        }
    }

    pub(crate) fn new(cx: &mut JSContext, global: &GlobalScope) -> DomRoot<MediaDevices> {
        reflect_dom_object(cx, Box::new(MediaDevices::new_inherited()), global)
    }

    fn permission_callback(
        &self,
        request: Arc<Mutex<CaptureRequest>>,
        name: PermissionName,
    ) -> GenericCallback<AllowOrDeny> {
        let this = Trusted::new(self);
        let task_source = self
            .global()
            .task_manager()
            .media_element_task_source()
            .to_sendable();
        GenericCallback::new(move |response: Result<AllowOrDeny, _>| {
            let this = this.clone();
            let request = request.clone();
            task_source.queue(task!(getusermedia_permission_response: move |cx| {
                let this = this.root();
                // Do not cache an obsolete answer or restart capture after cleanup.
                if this.reject_obsolete_request(cx, &request) {
                    return;
                }
                let state = match response {
                    Ok(response) => record_permission_request_result(name, &this.global(), response),
                    Err(_) => PermissionState::Denied,
                };
                this.permission_settled(cx, &request, name, state);
            }));
        })
        .expect("Could not create callback in script.")
    }

    fn reject_obsolete_request(&self, cx: &mut JSContext, request: &Mutex<CaptureRequest>) -> bool {
        let mut request = request.lock().unwrap();
        if request.generation == self.capture_generation.get() &&
            self.global().as_window().Document().is_fully_active()
        {
            return false;
        }
        let promise = request.promise.take();
        drop(request);
        if let Some(promise) = promise {
            promise.root(cx).reject_error(cx, Error::InvalidState(None));
        }
        true
    }

    /// Reject on the first refusal; open devices only after every requested grant.
    fn permission_settled(
        &self,
        cx: &mut JSContext,
        request: &Mutex<CaptureRequest>,
        name: PermissionName,
        state: PermissionState,
    ) {
        let mut request = request.lock().unwrap();
        let granted = state == PermissionState::Granted;
        if name == PermissionName::Microphone {
            request.audio_granted = Some(granted);
        } else {
            request.video_granted = Some(granted);
        }
        if !granted {
            let promise = request.promise.take();
            drop(request);
            if let Some(promise) = promise {
                promise
                    .root(cx)
                    .reject_error(cx, Error::NotAllowed(Some("Permission denied".to_string())));
            }
            return;
        }
        if request.audio_granted != Some(true) || request.video_granted != Some(true) {
            return;
        }
        let Some(promise) = request.promise.take() else {
            return;
        };
        let (audio, video) = (request.audio.take(), request.video.take());
        drop(request);
        let promise = promise.root(cx);
        self.start_capture(cx, &promise, audio, video);
    }

    fn start_capture(
        &self,
        cx: &mut JSContext,
        promise: &RootedPromise,
        audio: Option<MediaTrackConstraintSet>,
        video: Option<MediaTrackConstraintSet>,
    ) {
        let global = self.global();
        if !global.as_window().Document().is_fully_active() {
            promise.reject_error(cx, Error::InvalidState(None));
            return;
        }
        let media = ServoMedia::get();
        let mut sources = vec![];
        if let Some(constraints) = audio {
            match media.create_audioinput_stream(constraints) {
                Some(id) => sources.push((id, MediaStreamType::Audio)),
                None => return self.fail_capture(cx, promise, &sources, "No microphone found"),
            }
        }
        if let Some(constraints) = video {
            match media.create_videoinput_stream(constraints) {
                Some(id) => sources.push((id, MediaStreamType::Video)),
                None => return self.fail_capture(cx, promise, &sources, "No camera found"),
            }
        }
        let stream = MediaStream::new(cx, &global);
        for (id, ty) in sources {
            let track = MediaStreamTrack::new_capture(cx, &global, id, ty);
            self.register_capture_track(&track);
            stream.add_track(&track);
        }
        promise.resolve_native(cx, &stream);
    }

    fn fail_capture(
        &self,
        cx: &mut JSContext,
        promise: &RootedPromise,
        opened: &[(MediaStreamId, MediaStreamType)],
        message: &str,
    ) {
        for (id, _) in opened {
            stop_source(*id);
        }
        promise.reject_error(cx, Error::NotFound(Some(message.to_string())));
    }

    pub(crate) fn register_capture_track(&self, track: &MediaStreamTrack) {
        self.capture_tracks.borrow_mut().push(Dom::from_ref(track));
    }

    /// Release a device only when its last live track has stopped.
    pub(crate) fn release_source_if_unused(&self, id: MediaStreamId) {
        if self
            .capture_tracks
            .borrow()
            .iter()
            .any(|track| track.id() == id && !track.is_ended())
        {
            return;
        }
        self.capture_tracks
            .borrow_mut()
            .retain(|track| track.id() != id);
        stop_source(id);
    }

    /// End capture on document unload and window teardown.
    pub(crate) fn stop_all_capture(&self) {
        self.capture_generation
            .set(self.capture_generation.get().wrapping_add(1));
        let tracks: Vec<_> = self
            .capture_tracks
            .borrow_mut()
            .drain(..)
            .map(|track| DomRoot::from_ref(&*track))
            .collect();
        let mut stopped = vec![];
        for track in tracks {
            track.end();
            if !stopped.contains(&track.id()) {
                stopped.push(track.id());
                stop_source(track.id());
            }
        }
        let global = self.global();
        let mut answers = global.permission_state_invocation_results().borrow_mut();
        answers.remove(&PermissionName::Camera);
        answers.remove(&PermissionName::Microphone);
    }
}

fn stop_source(id: MediaStreamId) {
    if let Some(stream) = get_stream(&id) {
        stream.lock().unwrap().stop();
    }
}

impl MediaDevicesMethods<crate::DomTypeHolder> for MediaDevices {
    /// <https://w3c.github.io/mediacapture-main/#dom-mediadevices-getusermedia>
    fn GetUserMedia(
        &self,
        cx: &mut CurrentRealm,
        constraints: &MediaStreamConstraints,
    ) -> RootedPromise {
        let p = Promise::new_in_realm(cx);
        let global = self.global();
        let audio = convert_constraints(&constraints.audio);
        let video = convert_constraints(&constraints.video);

        if audio.is_none() && video.is_none() {
            p.reject_error(
                cx,
                Error::Type(c"getUserMedia() needs audio or video".to_owned()),
            );
            return p;
        }
        if !global.as_window().Document().is_fully_active() {
            p.reject_error(cx, Error::InvalidState(None));
            return p;
        }
        if !global.is_secure_context() {
            p.reject_error(cx, Error::NotAllowed(Some("Permission denied".to_string())));
            return p;
        }

        let wants_audio = audio.is_some();
        let wants_video = video.is_some();
        let request = Arc::new(Mutex::new(CaptureRequest {
            promise: Some(TrustedPromise::from(&p)),
            audio,
            video,
            audio_granted: (!wants_audio).then_some(true),
            video_granted: (!wants_video).then_some(true),
            generation: self.capture_generation.get(),
        }));
        for (wanted, name) in [
            (wants_audio, PermissionName::Microphone),
            (wants_video, PermissionName::Camera),
        ] {
            if !wanted || request.lock().unwrap().promise.is_none() {
                continue;
            }
            let callback = self.permission_callback(request.clone(), name);
            if let AsyncPermissionRequest::Settled(state) =
                request_permission_to_use_async(name, &global, callback)
            {
                self.permission_settled(cx, &request, name, state);
            }
        }
        p
    }

    /// <https://w3c.github.io/mediacapture-main/#dom-mediadevices-enumeratedevices>
    fn EnumerateDevices(&self, cx: &mut JSContext) -> RootedPromise {
        // Step 1.
        let mut realm = CurrentRealm::assert(cx);
        let p = Promise::new_in_realm(&mut realm);

        // Step 2.
        // XXX These steps should be run in parallel.
        // XXX Steps 2.1 - 2.4

        let global = self.global();
        let camera = descriptor_permission_state(PermissionName::Camera, Some(&global)) ==
            PermissionState::Granted;
        let microphone = descriptor_permission_state(PermissionName::Microphone, Some(&global)) ==
            PermissionState::Granted;

        // Do not probe devices before either capture permission has been granted.
        let devices = if camera || microphone {
            let media = ServoMedia::get();
            let device_monitor = media.get_device_monitor();
            Some(device_monitor.enumerate_devices().unwrap_or_default())
        } else {
            None
        };
        let present = |kind: fn(&ServoMediaDeviceKind) -> bool| {
            devices
                .as_ref()
                .is_none_or(|devices| devices.iter().any(|device| kind(&device.kind)))
        };
        let mut result_list = vec![];
        for device in devices.iter().flatten() {
            let exposed = match device.kind {
                ServoMediaDeviceKind::VideoInput => camera,
                ServoMediaDeviceKind::AudioInput | ServoMediaDeviceKind::AudioOutput => microphone,
            };
            if exposed {
                // XXX The media backend has no way to group devices yet.
                result_list.push(MediaDeviceInfo::new(
                    cx,
                    &global,
                    &device.device_id,
                    device.kind.convert(),
                    &device.label,
                    "",
                ));
            }
        }
        if !microphone && present(|kind| matches!(kind, ServoMediaDeviceKind::AudioInput)) {
            result_list.push(MediaDeviceInfo::new(
                cx,
                &global,
                "",
                MediaDeviceKind::Audioinput,
                "",
                "",
            ));
        }
        if !camera && present(|kind| matches!(kind, ServoMediaDeviceKind::VideoInput)) {
            result_list.push(MediaDeviceInfo::new(
                cx,
                &global,
                "",
                MediaDeviceKind::Videoinput,
                "",
                "",
            ));
        }

        p.resolve_native(cx, &result_list);

        // Step 3.
        p
    }
}

fn convert_constraints(js: &BooleanOrMediaTrackConstraints) -> Option<MediaTrackConstraintSet> {
    match js {
        BooleanOrMediaTrackConstraints::Boolean(false) => None,
        BooleanOrMediaTrackConstraints::Boolean(true) => Some(Default::default()),
        BooleanOrMediaTrackConstraints::MediaTrackConstraints(c) => Some(MediaTrackConstraintSet {
            height: c.parent.height.as_ref().and_then(convert_culong),
            width: c.parent.width.as_ref().and_then(convert_culong),
            aspect: c.parent.aspectRatio.as_ref().and_then(convert_cdouble),
            frame_rate: c.parent.frameRate.as_ref().and_then(convert_cdouble),
            sample_rate: c.parent.sampleRate.as_ref().and_then(convert_culong),
        }),
    }
}

// Bare numbers are ideal values, not requirements. Until the backend can select
// ideal values, do not exclude devices that cannot satisfy these preferences.
// https://w3c.github.io/mediacapture-main/#dfn-constrainable-property
fn convert_culong(js: &ConstrainULong) -> Option<Constrain<u32>> {
    match js {
        ConstrainULong::ClampedUnsignedLong(_) => None,
        ConstrainULong::ConstrainULongRange(range) => {
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
        ConstrainDouble::Double(_) => None,
        ConstrainDouble::ConstrainDoubleRange(range) => {
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
