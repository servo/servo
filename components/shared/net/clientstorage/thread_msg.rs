/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::{IpcReceiver, IpcSender};
use serde::{Deserialize, Serialize};

use super::routed_msg::ClientStorageRoutedMsg;

#[derive(Debug, Deserialize, Serialize)]
pub enum ClientStorageThreadMsg {
    TestConstructor {
        child_to_parent_receiver: IpcReceiver<ClientStorageRoutedMsg>,
        sender: IpcSender<u64>,
    },

    Routed(ClientStorageRoutedMsg),

    /// Send a reply when done cleaning up thread resources and then shut it down
    Exit(IpcSender<()>),
}
