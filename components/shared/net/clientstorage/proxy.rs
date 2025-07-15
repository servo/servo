/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::{self, IpcSender};

use super::thread_msg::ClientStorageThreadMsg;

pub struct ClientStorageProxy {
    ipc_sender: IpcSender<ClientStorageThreadMsg>,
}

impl ClientStorageProxy {
    pub fn new(ipc_sender: IpcSender<ClientStorageThreadMsg>) -> ClientStorageProxy {
        ClientStorageProxy { ipc_sender }
    }

    pub fn send_exit(&self) {
        let (sender, receiver) = ipc::channel().unwrap();
        self.ipc_sender
            .send(ClientStorageThreadMsg::Exit(sender))
            .unwrap();
        receiver.recv().unwrap();
    }
}
