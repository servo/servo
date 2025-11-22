/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use ipc_channel::ipc::IpcReceiver;
use storage_traits::indexeddb_thread::{
    CreateObjectResult, IndexedDBTransaction, KeyPath, KvsOperation,
};
use tokio::sync::oneshot;

pub use self::sqlite::SqliteEngine;

mod sqlite;

pub trait KvsEngine {
    type Error: std::error::Error;

    fn create_store(
        &self,
        store_name: &str,
        key_path: Option<KeyPath>,
        auto_increment: bool,
    ) -> Result<CreateObjectResult, Self::Error>;

    fn delete_store(&self, store_name: &str) -> Result<(), Self::Error>;

    #[expect(dead_code)]
    fn close_store(&self, store_name: &str) -> Result<(), Self::Error>;

    fn delete_database(self) -> Result<(), Self::Error>;

    fn process_transaction(
        &self,
        transaction: Arc<IndexedDBTransaction>,
        requests: IpcReceiver<KvsOperation>,
    ) -> oneshot::Receiver<()>;

    fn has_key_generator(&self, store_name: &str) -> bool;
    fn key_path(&self, store_name: &str) -> Option<KeyPath>;

    fn create_index(
        &self,
        store_name: &str,
        index_name: String,
        key_path: KeyPath,
        unique: bool,
        multi_entry: bool,
    ) -> Result<CreateObjectResult, Self::Error>;
    fn delete_index(&self, store_name: &str, index_name: String) -> Result<(), Self::Error>;

    fn version(&self) -> Result<u64, Self::Error>;
    fn set_version(&self, version: u64) -> Result<(), Self::Error>;
}
