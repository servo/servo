/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::generic_channel::{GenericSend, GenericSender, SendResult};
use base::{IpcSend, IpcSendResult};
use ipc_channel::ipc::{IpcError, IpcSender};
use malloc_size_of::malloc_size_of_is_0;
use serde::{Deserialize, Serialize};

use crate::indexeddb_thread::IndexedDBThreadMsg;
use crate::storage_thread::StorageThreadMsg;

pub mod indexeddb_thread;
pub mod storage_thread;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StorageThreads {
    storage_thread: GenericSender<StorageThreadMsg>,
    idb_thread: IpcSender<IndexedDBThreadMsg>,
}

impl StorageThreads {
    pub fn new(
        s: GenericSender<StorageThreadMsg>,
        i: IpcSender<IndexedDBThreadMsg>,
    ) -> StorageThreads {
        StorageThreads {
            storage_thread: s,
            idb_thread: i,
        }
    }
}

impl IpcSend<IndexedDBThreadMsg> for StorageThreads {
    fn send(&self, msg: IndexedDBThreadMsg) -> IpcSendResult {
        self.idb_thread.send(msg).map_err(IpcError::Bincode)
    }

    fn sender(&self) -> IpcSender<IndexedDBThreadMsg> {
        self.idb_thread.clone()
    }
}

impl GenericSend<StorageThreadMsg> for StorageThreads {
    fn send(&self, msg: StorageThreadMsg) -> SendResult {
        self.storage_thread.send(msg)
    }

    fn sender(&self) -> GenericSender<StorageThreadMsg> {
        self.storage_thread.clone()
    }
}

// Ignore the sub-fields
malloc_size_of_is_0!(StorageThreads);
