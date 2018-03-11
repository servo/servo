/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bincode;
use ipc_channel::ipc;
use serde::{Deserialize, Serialize};
use std::io::Error;
use time;
use time::ProfilerCategory;
use time::ProfilerChan;

pub struct IpcReceiver<T> where T: for<'de> Deserialize<'de> + Serialize {
    ipc_receiver: ipc::IpcReceiver<T>,
    time_profile_chan: ProfilerChan,
}

impl<T> IpcReceiver<T> where T: for<'de> Deserialize<'de> + Serialize {
    pub fn recv(&self) -> Result<T, bincode::Error> {
        time::profile(
            ProfilerCategory::IpcReceiver,
            None,
            self.time_profile_chan.clone(),
            move || self.ipc_receiver.recv(),
        )
    }

    pub fn try_recv(&self) -> Result<T, bincode::Error> {
        self.ipc_receiver.try_recv()
    }

    pub fn to_opaque(self) -> ipc::OpaqueIpcReceiver {
        self.ipc_receiver.to_opaque()
    }
}

pub fn channel<T>(time_profile_chan: ProfilerChan) -> Result<(ipc::IpcSender<T>, IpcReceiver<T>), Error>
    where T: for<'de> Deserialize<'de> + Serialize, {
    let (ipc_sender, ipc_receiver) = ipc::channel()?;
    let profiled_ipc_receiver = IpcReceiver {
        ipc_receiver,
        time_profile_chan,
    };
    Ok((ipc_sender, profiled_ipc_receiver))
}
