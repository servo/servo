/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::{Arc, Mutex, OnceLock, RwLock};

use gstreamer::{
    DeviceMonitor as GstDeviceMonitor, MessageView, bus::BusWatchGuard as GstBusWatchGuard,
    prelude::*,
};
use servo_base::generic_channel::GenericCallback;
use servo_media_streams::device_monitor::{MediaDeviceInfo, MediaDeviceKind, MediaDeviceMonitor};

const AUDIO_SOURCE: &str = "Audio/Source";
const AUDIO_SINK: &str = "Audio/Sink";
const VIDEO_SOURCE: &str = "Video/Source";

static INSTANCE: OnceLock<Arc<GStreamerDeviceMonitor>> = OnceLock::new();

pub struct GStreamerDeviceMonitor {
    monitor: GstDeviceMonitor,
    _watch_guard: GstBusWatchGuard,
    devices: Arc<RwLock<Option<Vec<MediaDeviceInfo>>>>,
    callbacks: Arc<Mutex<Vec<GenericCallback<()>>>>,
}

impl GStreamerDeviceMonitor {
    pub fn get() -> Arc<Self> {
        INSTANCE.get_or_init(|| Arc::new(Self::new())).clone()
    }

    pub fn new() -> Self {
        let monitor = GstDeviceMonitor::new();
        let devices = Arc::new(RwLock::new(None));
        let callbacks = Arc::new(Mutex::new(Vec::<GenericCallback<()>>::new()));

        // add filters
        let audio_caps = gstreamer_audio::AudioCapsBuilder::new().build();
        monitor.add_filter(Some(AUDIO_SOURCE), Some(&audio_caps));
        monitor.add_filter(Some(AUDIO_SINK), Some(&audio_caps));
        let video_caps = gstreamer_video::VideoCapsBuilder::new().build();
        monitor.add_filter(Some(VIDEO_SOURCE), Some(&video_caps));

        // watch changes
        let watch_guard = monitor
            .bus()
            .add_watch({
                let devices = devices.clone();
                let callbacks = callbacks.clone();
                move |_, msg| {
                    if let MessageView::DeviceAdded(_)
                    | MessageView::DeviceRemoved(_)
                    | MessageView::DeviceChanged(_) = msg.view()
                    {
                        // evict cache
                        *devices.write().unwrap() = None;

                        for callback in callbacks.lock().unwrap().iter() {
                            _ = callback.send(());
                        }
                    }
                    glib::ControlFlow::Continue
                }
            })
            .expect("Failed to add watcher");

        // start monitor
        monitor.start().expect("Failed to start monitor");

        Self {
            monitor,
            _watch_guard: watch_guard,
            devices,
            callbacks,
        }
    }

    fn get_devices(&self) -> Result<Vec<MediaDeviceInfo>, ()> {
        let devices = self
            .monitor
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

impl Drop for GStreamerDeviceMonitor {
    fn drop(&mut self) {
        self.monitor.stop();
    }
}

impl MediaDeviceMonitor for GStreamerDeviceMonitor {
    fn enumerate_devices(&self) -> Option<Vec<MediaDeviceInfo>> {
        {
            if let Some(ref devices) = *self.devices.read().unwrap() {
                return Some(devices.clone());
            }
        }
        let devices = self.get_devices().ok()?;
        *self.devices.write().unwrap() = Some(devices.clone());
        Some(devices)
    }

    fn add_devicechange_callback(&self, callback: GenericCallback<()>) {
        self.callbacks.lock().unwrap().push(callback);
    }
}
