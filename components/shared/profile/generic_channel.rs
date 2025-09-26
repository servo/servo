/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::generic_channel;
use serde::{Deserialize, Serialize};

use crate::time::{ProfilerCategory, ProfilerChan};
use crate::time_profile;

pub struct GenericReceiver<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    receiver: generic_channel::GenericReceiver<T>,
    time_profile_chan: ProfilerChan,
}

impl<T> GenericReceiver<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    pub fn recv(&self) -> Result<T, generic_channel::ReceiveError> {
        time_profile!(
            ProfilerCategory::IpcReceiver,
            None,
            self.time_profile_chan.clone(),
            move || self.receiver.recv(),
        )
    }

    pub fn try_recv(&self) -> Result<T, generic_channel::TryReceiveError> {
        self.receiver.try_recv()
    }

    pub fn into_inner(self) -> generic_channel::GenericReceiver<T> {
        self.receiver
    }
}

pub fn channel<T>(
    time_profile_chan: ProfilerChan,
) -> Option<(generic_channel::GenericSender<T>, GenericReceiver<T>)>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    let (sender, receiver) = generic_channel::channel()?;
    let profiled_receiver = GenericReceiver {
        receiver,
        time_profile_chan,
    };
    Some((sender, profiled_receiver))
}
