/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::io::Error;

use ipc_channel::ipc;
use serde::{Deserialize, Serialize};

use crate::time::{ProfilerCategory, ProfilerChan};
use crate::time_profile;

pub struct IpcReceiver<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    ipc_receiver: ipc::IpcReceiver<T>,
    time_profile_chan: ProfilerChan,
}

impl<T> IpcReceiver<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    pub fn recv(&self) -> Result<T, ipc::IpcError> {
        time_profile!(
            ProfilerCategory::IpcReceiver,
            None,
            self.time_profile_chan.clone(),
            move || self.ipc_receiver.recv(),
        )
    }

    pub fn try_recv(&self) -> Result<T, ipc::TryRecvError> {
        self.ipc_receiver.try_recv()
    }

    pub fn to_ipc_receiver(self) -> ipc::IpcReceiver<T> {
        self.ipc_receiver
    }
}

pub fn channel<T>(
    time_profile_chan: ProfilerChan,
) -> Result<(ipc::IpcSender<T>, IpcReceiver<T>), Error>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    let (ipc_sender, ipc_receiver) = ipc::channel()?;
    let profiled_ipc_receiver = IpcReceiver {
        ipc_receiver,
        time_profile_chan,
    };
    Ok((ipc_sender, profiled_ipc_receiver))
}

pub struct IpcBytesReceiver {
    ipc_bytes_receiver: ipc::IpcBytesReceiver,
    time_profile_chan: ProfilerChan,
}

impl IpcBytesReceiver {
    pub fn recv(&self) -> Result<Vec<u8>, ipc::IpcError> {
        time_profile!(
            ProfilerCategory::IpcBytesReceiver,
            None,
            self.time_profile_chan.clone(),
            move || self.ipc_bytes_receiver.recv(),
        )
    }
}

pub fn bytes_channel(
    time_profile_chan: ProfilerChan,
) -> Result<(ipc::IpcBytesSender, IpcBytesReceiver), Error> {
    let (ipc_bytes_sender, ipc_bytes_receiver) = ipc::bytes_channel()?;
    let profiled_ipc_bytes_receiver = IpcBytesReceiver {
        ipc_bytes_receiver,
        time_profile_chan,
    };
    Ok((ipc_bytes_sender, profiled_ipc_bytes_receiver))
}
