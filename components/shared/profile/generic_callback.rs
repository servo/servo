/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use servo_base::generic_channel::{SendError, SendResult};

use crate::generic_channel::GenericReceiver;
use crate::time::ProfilerChan;

#[derive(Debug, Serialize, Deserialize, MallocSizeOf)]
pub struct GenericCallback<T>(servo_base::generic_channel::GenericCallback<T>)
where
    T: Serialize + Send + 'static;

impl<T> GenericCallback<T>
where
    T: for<'de> Deserialize<'de> + Serialize + Send + 'static,
{
    pub fn new<F: FnMut(Result<T, SendError>) + Send + 'static>(
        callback: F,
    ) -> Result<Self, SendError> {
        Ok(GenericCallback(
            servo_base::generic_channel::GenericCallback::new(callback)?,
        ))
    }

    pub fn new_blocking(
        time_profiler_chan: ProfilerChan,
    ) -> Result<(Self, GenericReceiver<T>), SendError> {
        let (callback, receiver) = servo_base::generic_channel::GenericCallback::new_blocking()?;
        Ok((
            GenericCallback(callback),
            GenericReceiver::new(receiver, time_profiler_chan),
        ))
    }

    pub fn send(&self, value: T) -> SendResult {
        self.0.send(value)
    }
}

impl<T> Clone for GenericCallback<T>
where
    T: Serialize + Send + 'static,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
