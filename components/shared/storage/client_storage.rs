/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::generic_channel::{self, GenericSender};
use base::id::StorageKeyConnectionId;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum ClientStorageThreadMessage {
    NewStorageKeyConnection {
        connection_id: StorageKeyConnectionId,
    },

    StorageKeyConnectionBackendMessage {
        connection_id: StorageKeyConnectionId,
        message: StorageKeyConnectionBackendMessage,
    },

    /// Send a reply when done cleaning up thread resources and then shut it down
    Exit(GenericSender<()>),
}

pub struct ClientStorageProxy {
    generic_sender: GenericSender<ClientStorageThreadMessage>,
}

impl ClientStorageProxy {
    pub fn new(generic_sender: GenericSender<ClientStorageThreadMessage>) -> ClientStorageProxy {
        ClientStorageProxy { generic_sender }
    }

    pub fn send_new_storage_key_connection(&self, connection_id: StorageKeyConnectionId) {
        self.generic_sender
            .send(ClientStorageThreadMessage::NewStorageKeyConnection { connection_id })
            .unwrap();
    }

    pub fn send_storage_key_connection_backend_message(
        &self,
        connection_id: StorageKeyConnectionId,
        message: StorageKeyConnectionBackendMessage,
    ) {
        self.generic_sender
            .send(
                ClientStorageThreadMessage::StorageKeyConnectionBackendMessage {
                    connection_id,
                    message,
                },
            )
            .unwrap();
    }

    pub fn send_exit(&self) {
        let (sender, receiver) = generic_channel::channel().unwrap();
        self.generic_sender
            .send(ClientStorageThreadMessage::Exit(sender))
            .unwrap();
        receiver.recv().unwrap()
    }
}

// Messages towards the backend.

#[derive(Deserialize, Serialize)]
pub enum StorageKeyConnectionBackendMessage {
    Test(GenericSender<i32>),
}
