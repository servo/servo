/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use log::warn;
use openxr::{ApplicationInfo, Entry, ExtensionSet, FormFactor};
use servo_base::generic_channel::{
    self, GenericCallback, GenericReceiver, GenericSender, TryReceiveError,
};

use crate::openxr::{AppInfo, OPENXR_VERSION};

pub struct OpenXRDevicePoller {
    handle: Option<JoinHandle<()>>,
    device_receiver: GenericReceiver<Option<(u64, String)>>,
    shutdown_sender: GenericSender<()>,
    waker: Arc<Mutex<Option<GenericCallback<()>>>>,
}

impl OpenXRDevicePoller {
    pub fn new() -> Self {
        let (device_sender, device_receiver) =
            generic_channel::channel().expect("Failed to create generic channel!");
        let (shutdown_sender, shutdown_receiver) =
            generic_channel::channel().expect("Failed to create generic channel!");
        let waker: Arc<Mutex<Option<GenericCallback<()>>>> = Arc::new(Mutex::new(None));
        let thread_waker = waker.clone();
        let handle = std::thread::spawn(move || {
            let app_info = AppInfo {
                application_name: "XR Device Poll".to_string(),
                engine_name: "Servo".to_string(),
                ..Default::default()
            };
            loop {
                match shutdown_receiver.try_recv_timeout(Duration::from_secs(5)) {
                    Ok(()) => break,
                    Err(TryReceiveError::Empty) => {
                        let device = openxr_device_check(&app_info);
                        let _ = device_sender.send(device);

                        let waker = thread_waker.lock().expect("Lock poisoned").clone();
                        if let Some(waker) = waker {
                            let _ = waker.send(());
                        }
                    },
                    Err(TryReceiveError::ReceiveError(e)) => {
                        warn!("{}", e);
                        break;
                    },
                };
            }
        });
        Self {
            handle: Some(handle),
            device_receiver,
            shutdown_sender,
            waker,
        }
    }

    pub fn try_recv(&self) -> Option<Option<(u64, String)>> {
        match self.device_receiver.try_recv() {
            Ok(device) => Some(device),
            Err(TryReceiveError::Empty) => None,
            Err(TryReceiveError::ReceiveError(error)) => {
                warn!("OpenXR poller channel error: {error}");
                None
            },
        }
    }

    pub fn set_waker(&self, waker: GenericCallback<()>) {
        *self.waker.lock().expect("Lock poisoned") = Some(waker);
    }
}

impl Drop for OpenXRDevicePoller {
    fn drop(&mut self) {
        let _ = self.shutdown_sender.send(());
        if let Some(handle) = self.handle.take() {
            handle
                .join()
                .expect("Failed to join xr device poller handle");
        }
    }
}

fn openxr_device_check(app_info: &AppInfo) -> Option<(u64, String)> {
    let entry = unsafe { Entry::load().ok()? };
    let app_info = ApplicationInfo {
        application_name: &app_info.application_name,
        application_version: app_info.application_version,
        engine_name: &app_info.engine_name,
        engine_version: app_info.engine_version,
        api_version: OPENXR_VERSION,
    };
    let instance = entry
        .create_instance(&app_info, &ExtensionSet::default(), &[])
        .ok()?;
    let system = instance.system(FormFactor::HEAD_MOUNTED_DISPLAY).ok()?;
    let properties = instance.system_properties(system).ok()?;
    Some((properties.system_id.into_raw(), properties.system_name))
}
