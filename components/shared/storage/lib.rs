/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::generic_channel::{GenericSend, GenericSender, SendResult};
use malloc_size_of::malloc_size_of_is_0;
use serde::{Deserialize, Serialize};

use crate::client_storage::ClientStorageThreadMessage;
use crate::indexeddb::IndexedDBThreadMsg;
use crate::webstorage_thread::WebStorageThreadMsg;

pub mod client_storage;
pub mod indexeddb;
pub mod webstorage_thread;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StorageThreads {
    client_storage_thread: GenericSender<ClientStorageThreadMessage>,
    idb_thread: GenericSender<IndexedDBThreadMsg>,
    web_storage_thread: GenericSender<WebStorageThreadMsg>,
}

impl StorageThreads {
    pub fn new(
        client_storage_thread: GenericSender<ClientStorageThreadMessage>,
        idb_thread: GenericSender<IndexedDBThreadMsg>,
        web_storage_thread: GenericSender<WebStorageThreadMsg>,
    ) -> StorageThreads {
        StorageThreads {
            client_storage_thread,
            idb_thread,
            web_storage_thread,
        }
    }
}

impl GenericSend<ClientStorageThreadMessage> for StorageThreads {
    fn send(&self, msg: ClientStorageThreadMessage) -> SendResult {
        self.client_storage_thread.send(msg)
    }

    fn sender(&self) -> GenericSender<ClientStorageThreadMessage> {
        self.client_storage_thread.clone()
    }
}

impl GenericSend<IndexedDBThreadMsg> for StorageThreads {
    fn send(&self, msg: IndexedDBThreadMsg) -> SendResult {
        self.idb_thread.send(msg)
    }

    fn sender(&self) -> GenericSender<IndexedDBThreadMsg> {
        self.idb_thread.clone()
    }
}

impl GenericSend<WebStorageThreadMsg> for StorageThreads {
    fn send(&self, msg: WebStorageThreadMsg) -> SendResult {
        self.web_storage_thread.send(msg)
    }

    fn sender(&self) -> GenericSender<WebStorageThreadMsg> {
        self.web_storage_thread.clone()
    }
}

// Ignore the sub-fields
malloc_size_of_is_0!(StorageThreads);
