/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::generic_channel::GenericSender;
use base::id::StorageKeyConnectionId;
use storage_traits::client_storage::{ClientStorageProxy, StorageKeyConnectionBackendMessage};

pub struct StorageKeyConnection {
    proxy: ClientStorageProxy,
    connection_id: StorageKeyConnectionId,
}

impl StorageKeyConnection {
    pub fn new(proxy: ClientStorageProxy) -> StorageKeyConnection {
        let connection_id = StorageKeyConnectionId::new();

        proxy.send_new_storage_key_connection(connection_id);

        StorageKeyConnection {
            proxy,
            connection_id,
        }
    }

    pub fn send_test(&self, sender: GenericSender<i32>) {
        self.proxy.send_storage_key_connection_backend_message(
            self.connection_id,
            StorageKeyConnectionBackendMessage::Test(sender),
        );
    }
}
