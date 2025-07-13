/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::{self, IpcSender};

use super::mixed_msg::ClientStorageMixedMsg;
use super::routed_msg::ClientStorageRoutedMsg;
use super::test_msg::ClientStorageTestMsg;

pub struct ClientStorageTestChild {
    ipc_sender: IpcSender<ClientStorageRoutedMsg>,
    id: u64,
}

impl ClientStorageTestChild {
    pub fn new(ipc_sender: IpcSender<ClientStorageRoutedMsg>, id: u64) -> Self {
        ClientStorageTestChild { ipc_sender, id }
    }

    pub fn send_sync_ping(&self) {
        let (sender, receiver) = ipc::channel().unwrap();
        self.send_message(ClientStorageTestMsg::SyncPing(sender));
        receiver.recv().unwrap();
    }

    fn send_message(&self, msg: ClientStorageTestMsg) {
        self.send_mixed_message(ClientStorageMixedMsg::ClientStorageTest(msg));
    }

    fn send_mixed_message(&self, msg: ClientStorageMixedMsg) {
        self.send_routed_message(ClientStorageRoutedMsg {
            id: self.id,
            data: msg,
        });
    }

    fn send_routed_message(&self, msg: ClientStorageRoutedMsg) {
        self.ipc_sender.send(msg).unwrap();
    }
}
