/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use net_traits::clientstorage::test_msg::ClientStorageTestMsg;

pub struct ClientStorageTestParent {}

#[allow(clippy::new_without_default)]
impl ClientStorageTestParent {
    pub fn new() -> Self {
        ClientStorageTestParent {}
    }

    pub fn recv_message(&self, msg: ClientStorageTestMsg) {
        match msg {
            ClientStorageTestMsg::SyncPing(sender) => {
                self.recv_sync_ping();
                let _ = sender.send(());
            },
        }
    }

    fn recv_sync_ping(&self) {}
}
