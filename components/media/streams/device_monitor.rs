/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use servo_base::generic_channel::GenericCallback;

#[derive(Clone, Copy, Debug)]
pub enum MediaDeviceKind {
    AudioInput,
    AudioOutput,
    VideoInput,
}

#[derive(Clone, Debug)]
pub struct MediaDeviceInfo {
    pub device_id: String,
    pub kind: MediaDeviceKind,
    pub label: String,
}

pub trait MediaDeviceMonitor {
    fn enumerate_devices(&self) -> Option<Vec<MediaDeviceInfo>>;
    fn set_devicechange_callback(&self, callback: Option<GenericCallback<()>>);
}
