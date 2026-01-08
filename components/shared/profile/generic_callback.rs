/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::generic_channel::SendResult;
use serde::{Deserialize, Serialize};

use crate::time::{ProfilerCategory, ProfilerChan};
use crate::time_profile;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenericCallback<T>
where
    T: Serialize + Send + 'static,
{
    callback: base::generic_channel::GenericCallback<T>,
    time_profiler_chan: ProfilerChan,
}

impl<T> GenericCallback<T>
where
    T: for<'de> Deserialize<'de> + Serialize + Send + 'static,
{
    pub fn new<F: FnMut(Result<T, ipc_channel::Error>) + Send + 'static>(
        time_profiler_chan: ProfilerChan,
        callback: F,
    ) -> Result<Self, ipc_channel::Error> {
        Ok(GenericCallback {
            callback: base::generic_channel::GenericCallback::new(callback)?,
            time_profiler_chan,
        })
    }

    pub fn send(&self, value: T) -> SendResult {
        time_profile!(
            ProfilerCategory::IpcReceiver,
            None,
            self.time_profiler_chan.clone(),
            move || self.callback.send(value)
        )
    }
}
