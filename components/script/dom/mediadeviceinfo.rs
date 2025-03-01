/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use servo_media::streams::device_monitor::MediaDeviceKind as ServoMediaDeviceKind;

use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::MediaDeviceInfoBinding::{
    MediaDeviceInfoMethods, MediaDeviceKind,
};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct MediaDeviceInfo {
    reflector_: Reflector,
    device_id: DOMString,
    kind: MediaDeviceKind,
    label: DOMString,
    group_id: DOMString,
}

impl MediaDeviceInfo {
    fn new_inherited(
        device_id: &str,
        kind: MediaDeviceKind,
        label: &str,
        group_id: &str,
    ) -> MediaDeviceInfo {
        MediaDeviceInfo {
            reflector_: Reflector::new(),
            device_id: DOMString::from(device_id),
            kind,
            label: DOMString::from(label),
            group_id: DOMString::from(group_id),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        device_id: &str,
        kind: MediaDeviceKind,
        label: &str,
        group_id: &str,
        can_gc: CanGc,
    ) -> DomRoot<MediaDeviceInfo> {
        reflect_dom_object(
            Box::new(MediaDeviceInfo::new_inherited(
                device_id, kind, label, group_id,
            )),
            global,
            can_gc,
        )
    }
}

impl MediaDeviceInfoMethods<crate::DomTypeHolder> for MediaDeviceInfo {
    /// <https://w3c.github.io/mediacapture-main/#dom-mediadeviceinfo-deviceid>
    fn DeviceId(&self) -> DOMString {
        self.device_id.clone()
    }

    /// <https://w3c.github.io/mediacapture-main/#dom-mediadeviceinfo-kind>
    fn Kind(&self) -> MediaDeviceKind {
        self.kind
    }

    /// <https://w3c.github.io/mediacapture-main/#dom-mediadeviceinfo-label>
    fn Label(&self) -> DOMString {
        self.label.clone()
    }

    /// <https://w3c.github.io/mediacapture-main/#dom-mediadeviceinfo-groupid>
    fn GroupId(&self) -> DOMString {
        self.group_id.clone()
    }
}

impl Convert<MediaDeviceKind> for ServoMediaDeviceKind {
    fn convert(self) -> MediaDeviceKind {
        match self {
            ServoMediaDeviceKind::AudioInput => MediaDeviceKind::Audioinput,
            ServoMediaDeviceKind::AudioOutput => MediaDeviceKind::Audiooutput,
            ServoMediaDeviceKind::VideoInput => MediaDeviceKind::Videoinput,
        }
    }
}
