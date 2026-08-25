/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use gstreamer::DeviceMonitor as GstDeviceMonitor;
use gstreamer::prelude::*;
use servo_media_streams::device_monitor::{MediaDeviceInfo, MediaDeviceKind, MediaDeviceMonitor};

pub struct GStreamerDeviceMonitor {
    devices: RefCell<Option<Vec<MediaDeviceInfo>>>,
}

impl GStreamerDeviceMonitor {
    pub fn new() -> Self {
        Self {
            devices: RefCell::new(None),
        }
    }

    fn get_devices(&self) -> Result<Vec<MediaDeviceInfo>, ()> {
        const AUDIO_SOURCE: &str = "Audio/Source";
        const AUDIO_SINK: &str = "Audio/Sink";
        const VIDEO_SOURCE: &str = "Video/Source";
        let device_monitor = GstDeviceMonitor::new();
        let audio_caps = gstreamer_audio::AudioCapsBuilder::new().build();
        device_monitor.add_filter(Some(AUDIO_SOURCE), Some(&audio_caps));
        device_monitor.add_filter(Some(AUDIO_SINK), Some(&audio_caps));
        let video_caps = gstreamer_video::VideoCapsBuilder::new().build();
        device_monitor.add_filter(Some(VIDEO_SOURCE), Some(&video_caps));
        let devices = device_monitor
            .devices()
            .iter()
            .filter_map(|device| {
                let display_name = device.display_name().as_str().to_owned();
                Some(MediaDeviceInfo {
                    device_id: display_name.clone(),
                    kind: match device.device_class().as_str() {
                        AUDIO_SOURCE => MediaDeviceKind::AudioInput,
                        AUDIO_SINK => MediaDeviceKind::AudioOutput,
                        VIDEO_SOURCE => MediaDeviceKind::VideoInput,
                        _ => return None,
                    },
                    label: display_name,
                })
            })
            .collect();
        Ok(devices)
    }
}

impl MediaDeviceMonitor for GStreamerDeviceMonitor {
    fn enumerate_devices(&self) -> Option<Vec<MediaDeviceInfo>> {
        {
            if let Some(ref devices) = *self.devices.borrow() {
                return Some(devices.clone());
            }
        }
        let devices = self.get_devices().ok()?;
        *self.devices.borrow_mut() = Some(devices.clone());
        Some(devices)
    }
}
