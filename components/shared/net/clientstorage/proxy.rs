/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::{self, IpcSender};
use log::debug;

use super::routed_msg::ClientStorageRoutedMsg;
use super::thread_msg::ClientStorageThreadMsg;

pub struct ClientStorageProxy {
    ipc_sender: IpcSender<ClientStorageThreadMsg>,
}

impl ClientStorageProxy {
    pub fn new(ipc_sender: IpcSender<ClientStorageThreadMsg>) -> ClientStorageProxy {
        ClientStorageProxy { ipc_sender }
    }

    pub fn send_test_constructor(&self) -> (IpcSender<ClientStorageRoutedMsg>, u64) {
        // Messages the parent receives
        let (child_to_parent_sender, child_to_parent_receiver) = ipc::channel().unwrap();

        let (sender, receiver) = ipc::channel().unwrap();
        self.ipc_sender
            .send(ClientStorageThreadMsg::TestConstructor {
                child_to_parent_receiver,
                sender,
            })
            .unwrap();
        let id = receiver.recv().unwrap();

        (child_to_parent_sender, id)
    }

    pub fn send_exit(&self) {
        let (sender, receiver) = ipc::channel().unwrap();
        self.ipc_sender
            .send(ClientStorageThreadMsg::Exit(sender))
            .unwrap();
        receiver.recv().unwrap();
    }
}

impl Drop for ClientStorageProxy {
    fn drop(&mut self) {
        debug!("Dropping ClientStorageProxy");
    }
}
