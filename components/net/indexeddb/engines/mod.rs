/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::VecDeque;

use ipc_channel::ipc::IpcSender;
use net_traits::indexeddb_thread::{AsyncOperation, IndexedDBTxnMode};
use tokio::sync::oneshot;
use uuid::Uuid;

pub use self::heed::HeedEngine;

mod heed;

pub struct KvsOperation {
    pub sender: IpcSender<Option<Vec<u8>>>,
    pub store_name: Uuid,
    pub operation: AsyncOperation,
}

pub struct KvsTransaction {
    pub mode: IndexedDBTxnMode,
    pub requests: VecDeque<KvsOperation>,
}

pub trait KvsEngine {
    fn create_store(&self, store_name: Uuid, auto_increment: bool);

    fn delete_store(&self, store_name: Uuid);

    #[expect(dead_code)]
    fn close_store(&self, store_name: Uuid);

    fn process_transaction(
        &self,
        transaction: KvsTransaction,
    ) -> oneshot::Receiver<Option<Vec<u8>>>;

    fn has_key_generator(&self, store_name: Uuid) -> bool;
}
