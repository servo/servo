/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc;
use serde::{Deserialize, Serialize};
use std::io::Error;
use time;
use time::ProfilerCategory;

#[derive(Debug)]
pub struct IpcReceiver<T> where T: for<'de> Deserialize<'de> + Serialize {
    ipc_receiver: ipc::IpcReceiver<T>
}

impl<T> IpcReceiver<T> where T: for<'de> Deserialize<'de> + Serialize {
    pub fn recv(&self) -> Result<T, bincode::Error> {
        //time::profile(ProfilerCategory::IpcReceiver, None, )
        self.ipc_receiver.recv()
    }

    pub fn try_recv(&self) -> Result<T, bincode::Error> {
        self.ipc_receiver.try_recv()
    }

    pub fn to_opaque(self) -> OpaqueIpcReceiver {
        self.ipc_receiver.to_opaque()
    }
}

pub fn channel<T>() -> Result<(ipc::IpcSender<T>, ipc::IpcReceiver<T>),Error>
    where T: for<'de> Deserialize<'de> + Serialize {
    let (ipc_sender, ipc_receiver) = ipc::channel()?;
    Ok((ipc_sender, ipc_receiver))
}

